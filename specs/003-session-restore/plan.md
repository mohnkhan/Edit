# Implementation Plan: Session Restore

**Branch**: `003-session-restore` | **Date**: 2026-06-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/003-session-restore/spec.md`

## Summary

On a user-initiated clean exit, serialize the editor's open buffer paths, per-buffer
cursor positions, split layout, and active buffer index to
`$XDG_STATE_HOME/edit/session.toml` (human-readable TOML, schema version 1). On the next
startup with no explicit file arguments and no `--no-session` flag, load and validate the
session file, show an in-TUI restore-prompt dialog, and — on confirmation — reopen all
recorded paths, seek cursors to their saved positions, and restore the split layout.
Missing files are skipped with status-bar warnings; a corrupt session file is treated as
absent and silently overwritten on the next clean exit.

All serde, TOML, XDG directory, and CLI infrastructure is already present in the project
(`serde`, `toml`, `dirs`, `clap` are all Cargo dependencies). The feature adds one new
module (`src/session/`) and integrates at the existing clean-exit and startup hooks.

## Technical Context

**Language/Version**: Rust stable, edition 2021; MSRV 1.74.0

**Primary Dependencies**:
- `serde 1` + `derive` feature — already in Cargo.toml; used for session TOML model
- `toml 0.8` — already in Cargo.toml; session file serialization / deserialization
- `dirs 5` — already in Cargo.toml; `dirs::state_dir()` resolves `$XDG_STATE_HOME`
- `clap 4` — already in Cargo.toml; `--no-session` flag
- `ratatui 0.26` — existing TUI framework; session restore dialog is an overlay widget
- No new Cargo dependencies required

**Storage**:
- Session file: `$XDG_STATE_HOME/edit/session.toml`
  - Fallback if `dirs::state_dir()` returns `None`: `$HOME/.local/state/edit/session.toml`
  - Same directory family as existing logs and crash reports
- File is written atomically (write to `.session.toml.tmp`, rename) to avoid partial writes

**Testing**:
- Unit: `cargo test` — session module unit tests (serialize/deserialize round-trip, corrupt
  file handling, missing-path filtering)
- Integration: `cargo test --test session` — new `tests/integration/session.rs`;
  spawns temp editor state, verifies save/load round-trip and degraded-restore scenarios
- Smoke: extend existing expect scripts in `tests/smoke/` to cover the restore prompt flow

**Target Platform**: Same as project — Linux x86_64/ARM64, FreeBSD, macOS; no new
platform-specific code paths

**Project Type**: CLI terminal application (existing); no structural change

**Performance Goals**:
- Session write on exit: ≤ 50 ms (simple TOML serialisation of ≤ 100 paths)
- Session read + prompt render on startup: ≤ 200 ms additional startup overhead

**Constraints**:
- Crash exits MUST NOT write the session file (hook only into clean-exit paths)
- Session file MUST be written atomically (tmp-rename) to prevent corruption from kill -9
- `--no-session` suppresses prompt in all invocations, including interactive
- Restore prompt MUST render inside the TUI (not as a pre-TUI terminal prompt), per the
  DOS-faithful UI constitution principle

**Scale/Scope**: Single-user; session records at most all open buffers (no hard cap)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Gate | Status | Notes |
|---|---|---|---|
| I. DOS-Faithful UI | Restore prompt rendered as a TUI dialog overlay inside the existing ratatui frame, matching the blue-background style | ✅ PASS | Dialog uses theme colors; keyboard-driven (Y/N/Enter/Escape) |
| II. UTF-8 First | Session file paths stored as UTF-8 strings; no raw-byte path handling | ✅ PASS | `PathBuf::to_string_lossy()` for serialization; paths re-validated on restore |
| III. Portable Build | `dirs::state_dir()` is cross-platform; session module has no OS-specific code | ✅ PASS | Same XDG helper pattern already used throughout |
| IV. Minimal Footprint | No new Cargo dependencies; TOML + serde already present | ✅ PASS | session.toml adds <1 KB to runtime; no binary size impact |
| V. Test-Gated | Session unit tests + new integration test file required before merge | ✅ PASS | TDD: write tests first for session module, then dialog integration |
| VI. YAGNI | Session restore is explicitly called out in ROADMAP.md issue #6 and accepted spec | ✅ PASS | No speculative features added |
| VII. Security | File paths from session.toml are validated for path traversal before opening; corrupt TOML silently ignored (no panic) | ✅ PASS | Re-use existing `security::sanitize` path helpers |

**No violations.**

## Project Structure

### Documentation (this feature)

```text
specs/003-session-restore/
├── plan.md              # This file
├── research.md          # Phase 0 decisions
├── data-model.md        # Phase 1 entity model
├── quickstart.md        # Phase 1 validation guide
├── contracts/
│   └── session-toml.md  # TOML schema contract
└── checklists/
    └── requirements.md  # Spec quality checklist
```

### Source Code

```text
src/
├── session/
│   └── mod.rs           # NEW: SessionData, save_session(), load_session(), session_path()
├── app.rs               # MODIFY: add pending_session_restore dialog state; call session::save
│                        #   on clean exit; handle RestoreSession / DeclineRestore actions
├── main.rs              # MODIFY: add --no-session CLI flag; load session data before App::new;
│                        #   pass Option<SessionData> to App::new
├── config/
│   └── schema.rs        # MODIFY: add no_session: bool field
├── input/
│   └── mod.rs           # MODIFY: add Action::RestoreSession, Action::DeclineRestore (or
│                        #   re-use InsertChar dispatch — see Phase 0 decision)
└── ui/
    └── mod.rs           # MODIFY: render SessionRestoreDialog overlay (similar to save-prompt)

tests/integration/
└── session.rs           # NEW: integration tests — save/load round-trip, missing files,
                         #   corrupt TOML, --no-session, crash-exit guard

man/edit.1               # MODIFY: document --no-session flag

CHANGELOG.md             # MODIFY: feature 003 entry
docs/STATUS.md           # MODIFY: F003 user stories
```

## Implementation Phases

### Phase 1 — Session Module (Foundation)

**Goal**: `src/session/mod.rs` — pure data model + read/write; no TUI dependency.

**Files**:
- `src/session/mod.rs` — new

**Tasks**:
- Define `BufferEntry { path: String, cursor_line: u32, cursor_col: u32 }` with
  `#[derive(Serialize, Deserialize)]`
- Define `SplitLayoutKind` enum (`None`, `Horizontal`, `Vertical`) with serde string
  representation (`"none"`, `"horizontal"`, `"vertical"`)
- Define `SessionData { version: u32, active_buffer: usize, split_layout: SplitLayoutKind,
  active_pane: u32, buffers: Vec<BufferEntry> }` with serde derive
- Implement `session_path() -> PathBuf` — uses `dirs::state_dir()`, falls back to
  `$HOME/.local/state`
- Implement `save_session(data: &SessionData) -> io::Result<()>` — atomic write via
  `.session.toml.tmp` → rename; creates parent dir if needed
- Implement `load_session() -> Option<SessionData>` — reads and parses TOML; returns
  `None` on missing file, IO error, TOML parse error, or unknown schema version; logs
  warnings for each failure mode

**Tests** (unit, in `src/session/mod.rs` `#[cfg(test)]`):
- `test_round_trip_single_buffer` — write then read, assert identical data
- `test_round_trip_split_vertical` — verify `SplitLayoutKind::Vertical` survives serde
- `test_corrupt_toml_returns_none` — write garbage bytes, assert `load_session()` → None
- `test_unknown_version_returns_none` — write version=99, assert → None
- `test_missing_file_returns_none` — load from nonexistent path → None
- `test_atomic_write` — verify tmp file is renamed (no leftover .tmp)

---

### Phase 2 — CLI Flag and Config Integration

**Goal**: Wire `--no-session` through CLI → Config → startup guard.

**Files**:
- `src/main.rs`
- `src/config/schema.rs`

**Tasks**:
- Add `no_session: bool = false` field to `Config` in `config/schema.rs`
- Add `--no-session` arg to `build_cli()` in `main.rs` (matches `--no-autosave` pattern)
- Add `merge_cli_flags` branch: `if matches.get_flag("no-session") { config.no_session = true; }`
- In `main()`, after config is loaded: if `!config.no_session && files.is_empty()` →
  call `session::load_session()` to get `Option<SessionData>`; pass it to `App::new`

---

### Phase 3 — App Integration: Dialog State and Session Restore Logic

**Goal**: `App` shows the restore dialog and executes the restore on confirmation.

**Files**:
- `src/app.rs`
- `src/input/mod.rs` (possibly — see below)

**Tasks**:

Add to `App` struct:
```
pending_session_restore: Option<SessionData>,  // Some = dialog showing; None = not showing
```

Modify `App::new` signature: `pub fn new(config: Config, files: Vec<PathBuf>, default_encoding: EncodingId, session: Option<SessionData>) -> Self`

In `App::new` body: `pending_session_restore: session`

In `handle_action` when `self.pending_session_restore.is_some()`:
- Y / Enter → `self.do_restore_session()`, clear `pending_session_restore`
- N / Escape → clear `pending_session_restore`, leave blank buffer intact

Implement `do_restore_session(&mut self)`:
- For each `BufferEntry` in `session.buffers`:
  - Validate path via `security::sanitize::validate_path` (reject path traversal)
  - `Buffer::open(path, default_encoding)` — on error, skip + append to `status_messages`
  - On success: seek cursor to `(cursor_line - 1, cursor_col - 1)` (convert 1-based to 0-based)
  - Apply syntax highlighting if enabled
- If all entries failed: keep the existing blank buffer; set status message
- Else replace `self.buffers` with the restored buffers; set `self.active_idx` from session;
  restore `self.split_mode` from session; set `self.active_idx` from session `active_buffer`

In clean-exit paths, call `session::save_session` **before** setting `self.running = false`:
- `handle_quit` (no-modified-buffer fast path)
- `prompt_save_and_quit` (save succeeded)
- `prompt_discard_and_quit`

Session data to write = snapshot of current `App` state:
- `buffers`: filter to entries where `buf.path.is_some()` and `buf.path` is not a new-file
  stub (i.e. the path exists on disk — or if it was just saved, it exists now)
- `cursor_line`/`cursor_col`: from `buf.cursor.line + 1` / `buf.cursor.grapheme_col + 1`
- `active_buffer`: `self.active_idx`
- `split_layout`: derived from `self.split_mode`
- `active_pane`: 0 for left/only pane, 1 for right pane when split and `active_idx > 0`

---

### Phase 4 — TUI Dialog Rendering

**Goal**: The session restore dialog renders as a themed overlay in the TUI frame.

**Files**:
- `src/ui/mod.rs`

**Tasks**:
- Add a `SessionRestoreDialog` branch in `Ui::render`: when `app.pending_session_restore.is_some()`,
  draw a centered overlay (same pattern as the existing save-prompt dialog):
  - Title: "Restore Session"
  - Body: "Restore previous session? [Y/n]"
  - Colors from `app.theme.menubar_fg` / `app.theme.menubar_bg`
  - Dimensions: 50×5 (fixed; clamped to terminal size)

---

### Phase 5 — Tests

**Goal**: Integration test suite for session feature.

**Files**:
- `tests/integration/session.rs` — new
- `Cargo.toml` — add `[[test]]` entry

**Tasks**:
- `test_save_then_load_round_trip` — create a `SessionData`, call `save_session`, call
  `load_session`, assert equal
- `test_restore_missing_file_skipped` — record a non-existent path, verify restore skips it
  (requires calling `do_restore_session` logic directly or via a test harness shim)
- `test_no_session_flag_skips_load` — `config.no_session = true` → `load_session` never
  called (assert `session` arg to `App::new` is `None`)
- `test_corrupt_session_file_not_shown` — write corrupt TOML to session path, verify
  `load_session` returns None and no panic
- `test_explicit_files_bypass_restore` — when `files.is_empty()` is false, session not loaded

---

### Phase 6 — Documentation Gate

**Goal**: Update CHANGELOG, STATUS, CAPABILITIES, man page.

**Files**:
- `CHANGELOG.md` — feature 003 entry
- `docs/STATUS.md` — F003 user story rows
- `docs/CAPABILITIES.md` — new CLI flag (`--no-session`) entry
- `man/edit.1` — document `--no-session` in the OPTIONS section

---

## Deferred Items

None. The full scope of FR-001–FR-012 is addressed in Phases 1–6.

## Complexity Tracking

No Constitution Check violations. No complexity justification required.
