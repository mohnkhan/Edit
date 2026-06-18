# Tasks: Session Restore (Feature 003)

**Input**: Design documents from `specs/003-session-restore/`

**Prerequisites**: plan.md ✅ | spec.md ✅ | research.md ✅ | data-model.md ✅ | contracts/ ✅

**Remediation applied**: I1 (active_idx clamp) · I2 (Traversal vs Io distinction) · I3 (corrupt TOML status-bar warning) · A1 (InsertNewline key) · C1 (T036 partial-restore test) · C2 (T037 crash-exit test) · D1 (build_session_data helper) · B1 (T027 absorbed into T012)

**Organization**: Tasks are grouped by user story to enable independent implementation
and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no blocking dependencies)
- **[Story]**: Which user story this task belongs to (US1–US4 from spec.md)
- Exact file paths included in every task description

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the new session module directory and stub; register it in the module
tree so subsequent phases can compile incrementally.

- [X] T001 Create `src/session/mod.rs` with empty module stub and `#![allow(dead_code)]` attribute
- [X] T002 Add `mod session;` declaration to `src/main.rs` and `pub mod session;` in `src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement the complete session data model plus read/write logic in
`src/session/mod.rs`. Nothing in any user story can compile until this phase is done.

**⚠️ CRITICAL**: No user story work begins until this phase passes `cargo test session`.

- [X] T003 Define `BufferEntry { path: String, cursor_line: u32, cursor_col: u32 }` with `#[derive(Debug, Clone, Serialize, Deserialize)]` in `src/session/mod.rs`
- [X] T004 Define `SplitLayoutKind` enum (`None`, `Horizontal`, `Vertical`) with serde string representation (`"none"`, `"horizontal"`, `"vertical"`) in `src/session/mod.rs`
- [X] T005 Define `SessionData { version: u32, active_buffer: usize, split_layout: SplitLayoutKind, active_pane: u32, buffers: Vec<BufferEntry> }` with `#[derive(Debug, Clone, Serialize, Deserialize)]` in `src/session/mod.rs`
- [X] T006 Implement `session_path() -> PathBuf` in `src/session/mod.rs` — use `dirs::state_dir()`, push `"edit/session.toml"`, fall back to `$HOME/.local/state/edit/session.toml` when `state_dir()` returns `None`
- [X] T007 Implement `save_session(data: &SessionData) -> io::Result<()>` in `src/session/mod.rs` — serialize via `toml::to_string_pretty`, create parent dir with `fs::create_dir_all`, write to `.session.toml.tmp`, then `fs::rename` to final path (atomic write)
- [X] T008 Implement `load_session() -> Result<Option<SessionData>, String>` in `src/session/mod.rs` — three outcomes: `Ok(None)` = file absent (no prompt, no warning); `Ok(Some(data))` = loaded successfully; `Err(msg)` = file existed but corrupt/invalid (caller shows status-bar warning). Return `Err` on TOML parse failure, `version != 1`, or `active_buffer >= buffers.len()`; log `warn!` in each error case with the reason string. Return `Ok(None)` on `NotFound` IO error only.
- [X] T009 Write unit tests in `src/session/mod.rs` `#[cfg(test)]` block: `test_round_trip_single_buffer`, `test_round_trip_split_vertical`, `test_corrupt_toml_returns_err` (assert `Err(_)`), `test_unknown_version_returns_err` (assert `Err(_)`), `test_missing_file_returns_ok_none` (assert `Ok(None)`), `test_atomic_write_no_leftover_tmp`

**Checkpoint**: `cargo test session` passes all 6 unit tests before any US phase begins.

---

## Phase 3: User Story 1 — Restore Previous Session on Startup (Priority: P1) 🎯 MVP

**Goal**: On a clean exit, write session.toml. On next no-arg startup, show a TUI restore
dialog, and on Y/Enter re-open all files at their saved cursor positions and split layout.

**Independent Test**: Open two files in split view, navigate cursors, quit, relaunch,
press Y — both files reopen at exact saved positions with vertical split restored.

### Implementation for User Story 1

- [X] T010 [US1] Add `pending_session_restore: Option<crate::session::SessionData>` field to `App` struct in `src/app.rs`; initialize to `None` in `App::new`
- [X] T011 [US1] Update `App::new` signature to `pub fn new(config: Config, files: Vec<PathBuf>, default_encoding: EncodingId, session: Option<crate::session::SessionData>, session_warning: Option<String>) -> Self` in `src/app.rs`; store `session` in `pending_session_restore`; if `session_warning.is_some()` and `session.is_none()`, set `self.status_message = session_warning` so a corrupt-session notice is visible on startup (remediation for I3)
- [X] T012 [US1] In `main()` in `src/main.rs`: after collecting `files`, add `// Session restore: only when no explicit file arguments`; when `files.is_empty()`, call `crate::session::load_session()` and match the result — `Ok(Some(data))` → `(Some(data), None)`, `Err(msg)` → `(None, Some(msg))`, `Ok(None)` → `(None, None)` — bind to `(session, session_warning)`; otherwise `let (session, session_warning) = (None, None)`; pass both as the fourth and fifth args to `App::new`. (The `!config.no_session` guard is added in T026 once `Config::no_session` exists.)
- [X] T013 [US1] In `handle_action` in `src/app.rs`: when `self.pending_session_restore.is_some()`, intercept `Action::InsertChar('Y' | 'y')` **and** `Action::InsertNewline` (the variant generated by the Enter key) to call `self.do_restore_session()` then clear `pending_session_restore`; intercept `Action::InsertChar('N' | 'n')`, `Action::Quit`, and `Action::MenuClose` (Escape) to clear `pending_session_restore` without restoring (blank buffer stays); drop all other actions silently while dialog is active (remediation for A1)
- [X] T014 [US1] Implement `fn do_restore_session(&mut self)` in `src/app.rs` — happy path: iterate `SessionData.buffers`, call `Buffer::open(path, self.default_encoding)` for each, seek cursor to `(entry.cursor_line.saturating_sub(1), entry.cursor_col.saturating_sub(1))` via `buf.cursor`, apply syntax highlighting if `config.highlight`, collect successes into `new_buffers`; on completion replace `self.buffers`, set `self.split_mode` from `session.split_layout`, then **clamp** `self.active_idx = session.active_buffer.min(new_buffers.len().saturating_sub(1))` to handle the case where the active buffer was in the skipped-file list (prevents out-of-bounds panic — remediation for I1)
- [X] T015 [US1] Add `default_encoding: EncodingId` field to `App` struct in `src/app.rs`; populate from the existing `default_encoding` parameter in `App::new` so `do_restore_session` can access it
- [X] T016 [US1] Render session restore dialog overlay in `src/ui/mod.rs` `Ui::render`: when `app.pending_session_restore.is_some()`, draw a centered 50×5 `Block` titled `"Restore Session"` with body `"Restore previous session? [Y/n]"` using `app.theme.menubar_fg` / `app.theme.menubar_bg` colors (follow the existing save-prompt overlay pattern)
- [X] T017 [US1] Add `fn build_session_data(&self) -> Option<crate::session::SessionData>` in `src/app.rs` — extracts and encapsulates the snapshot logic shared by T018/T019/T020: filters `self.buffers` to entries where `buf.path.is_some() && buf.path.as_ref().map(|p| p.exists()).unwrap_or(false)`, converts cursor coords to 1-based, maps `split_mode` to `SplitLayoutKind`, derives `active_pane` from `split_mode + active_idx`, returns `None` if the resulting buffer list is empty (no saveable session). Then call this helper in `fn handle_quit` in `src/app.rs` — in the no-modified-buffer fast path (`self.running = false`): `if let Some(data) = self.build_session_data() { if let Err(e) = crate::session::save_session(&data) { log::warn!("session save failed: {}", e); } }` (remediation for D1)
- [X] T018 [US1] Call `crate::session::save_session(...)` in `fn prompt_save_and_quit` in `src/app.rs` — after `self.running = false`: call `self.build_session_data()` (extracted in T017) and save; log any IO error as `warn!`
- [X] T019 [US1] Call `crate::session::save_session(...)` in `fn prompt_discard_and_quit` in `src/app.rs` — after `self.running = false`: call `self.build_session_data()` (extracted in T017) and save; log any IO error as `warn!`

**Checkpoint**: `cargo test` passes; manual scenario 1 from `quickstart.md` works end-to-end.

---

## Phase 4: User Story 2 — Handle Missing Files Gracefully (Priority: P2)

**Goal**: When one or more recorded files are missing or unreadable, skip them with a
status-bar warning; restore remaining files normally; fall back to blank buffer if all fail.

**Independent Test**: Record session with two files, delete one, confirm restore — surviving
file opens correctly and status bar shows warning for the deleted file.

### Implementation for User Story 2

- [X] T020 [US2] In `do_restore_session` in `src/app.rs`: before calling `Buffer::open`, call `crate::security::sanitize::validate_path(path)`; pattern-match the result — `Err(PathError::Traversal)` → log `warn!("session: path traversal rejected: {:?}", path)` + push `format!("session: path rejected: {}", path.display())` to `warnings` + skip; `Err(PathError::Io(_))` → fall through normally to T021's missing-file handler (do NOT emit a separate security warning for Io errors, which are normal missing-file cases); `Ok(canonical)` → proceed to `Buffer::open(canonical, ...)` (remediation for I2)
- [X] T021 [US2] In `do_restore_session` in `src/app.rs`: when `Buffer::open` returns `Err` or path does not exist, skip that entry and push a warning string `format!("session: {} not found", path.display())` to a local `warnings: Vec<String>`; after the loop set `self.status_message` to the first warning (subsequent ones logged via `warn!`)
- [X] T022 [US2] In `do_restore_session` in `src/app.rs`: after the loop, if `new_buffers.is_empty()`, keep the existing blank buffer and set `self.status_message = Some("session: no files could be restored".to_string())`; else replace buffers normally (T014 happy path already handles this — integrate the empty-check into T014's logic)

**Checkpoint**: `cargo test` passes; quickstart.md scenario 2 (missing file) works.

---

## Phase 5: User Story 3 — Suppress Session Prompt via Flag (Priority: P3)

**Goal**: `--no-session` on the command line skips the restore prompt entirely; editor
opens a blank buffer regardless of session file state.

**Independent Test**: Create a session file, launch with `--no-session`, verify no restore
dialog and blank buffer opens immediately.

### Implementation for User Story 3

- [X] T023 [P] [US3] Add `no_session: bool` field (default `false`) to `Config` struct in `src/config/schema.rs` with `#[serde(default)]`
- [X] T024 [P] [US3] Add `--no-session` arg to `build_cli()` in `src/main.rs` with `ArgAction::SetTrue` and help text `"Skip session restore prompt on startup"`
- [X] T025 [US3] Add `if matches.get_flag("no-session") { config.no_session = true; }` to `merge_cli_flags()` in `src/config/mod.rs`
- [X] T026 [US3] In `main()` in `src/main.rs`: extend the session-load condition added in T012 from `files.is_empty()` to `files.is_empty() && !config.no_session` — update the comment to `// Session restore: only when no explicit file arguments and --no-session not set`; this makes `--no-session` produce `(None, None)` regardless of session file state (depends on T023 adding `Config::no_session`)

**Checkpoint**: `./target/debug/edit --no-session` opens blank buffer without prompt even when session.toml exists.

---

## Phase 6: User Story 4 — File Arguments Bypass Session Restore (Priority: P4)

**Goal**: Explicit file arguments on the CLI open those files directly; no restore prompt
appears regardless of session file state.

**Note**: US4 is fully implemented by the `files.is_empty() && !config.no_session` guard
added in T012. No separate implementation task is required; the inline comment in T012
(`// Session restore: only when no explicit file arguments and --no-session not set`) serves
as the code-level documentation. Verify in T012's checkpoint. *(T027 absorbed into T012
per analysis finding B1.)*

**Checkpoint**: `./target/debug/edit /tmp/any_file.txt` opens file directly; no prompt.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Integration tests, documentation gate, and final CI validation.

- [X] T028 [P] Write integration test `tests/integration/session.rs` — test `test_save_then_load_round_trip`: create a `SessionData`, call `save_session` to a temp dir, call `load_session`, assert `Ok(Some(data))` with equal content; test `test_corrupt_session_file_returns_err`: write garbage bytes to session path, assert `load_session` returns `Err(_)`; test `test_unknown_version_returns_err`: write TOML with `version = 99`, assert `Err(_)`; test `test_absent_file_returns_ok_none`: call `load_session` with no file present, assert `Ok(None)`
- [X] T029 [P] Add integration test `test_no_session_flag` to `tests/integration/session.rs`: verify that the `files.is_empty() && !config.no_session` branch in `main()` yields `(None, None)` when `config.no_session = true` even if a valid session.toml exists
- [X] T030 [P] Add integration test `test_explicit_files_bypass` to `tests/integration/session.rs`: verify that the guard yields `(None, None)` when `files` is non-empty, regardless of session file state
- [X] T036 [P] Add integration test `test_partial_restore_skips_missing` to `tests/integration/session.rs`: write a `SessionData` with two paths — one to a real temp file and one to a non-existent path — call `do_restore_session` via a test shim or by inspecting post-restore `App` state; assert `self.buffers.len() == 1` (surviving file) and `self.status_message` contains the missing filename; assert `self.active_idx == 0` (clamped, not 1) (remediation for C1 / SC-003)
- [X] T037 [P] Add integration test `test_crash_exit_does_not_write_session` to `tests/integration/session.rs`: write a known session.toml, then simulate a panic-exit by calling `std::panic::catch_unwind` around a forced panic — verify the session file on disk is unchanged (same content as before, timestamp not updated); this covers FR-002 / SC-005 (remediation for C2)
- [X] T031 Add `[[test]] name = "session" path = "tests/integration/session.rs"` to `Cargo.toml`
- [X] T032 Update `CHANGELOG.md` — add feature 003 entry under `[Unreleased]` listing all FR-001–FR-012 additions, --no-session flag, and session.toml format
- [X] T033 [P] Update `docs/STATUS.md` — add F003 user story rows (US1–US4) with status Complete
- [X] T034 [P] Update `docs/CAPABILITIES.md` — add `--no-session` to the CLI flags table
- [X] T035 Update `man/edit.1` — add `--no-session` entry in the OPTIONS section with description matching `--help` output

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — **BLOCKS all US phases**
- **Phase 3 (US1)**: Depends on Phase 2 completion
- **Phase 4 (US2)**: Depends on Phase 3 (extends `do_restore_session`)
- **Phase 5 (US3)**: Depends on Phase 2; can run in parallel with Phase 3 (different files: config/schema.rs, main.rs CLI arg)
- **Phase 6 (US4)**: No implementation task; US4 is implemented by the guard in T012. Checkpoint only.
- **Phase 7 (Polish)**: Depends on Phases 3–6 completion

### User Story Dependencies

- **US1 (P1)**: Requires Foundational (Phase 2). Core story — all others build on it.
- **US2 (P2)**: Requires US1 (extends `do_restore_session` from T014).
- **US3 (P3)**: Requires Phase 2 only; CLI/config tasks (T023, T024) are parallel with US1.
- **US4 (P4)**: Fully implemented by the guard in T012 (absorbed T027 per analysis finding B1).

### Within Each Phase

- Session module tasks T003–T008 must execute in order (each builds on the previous type/function).
- T009 (unit tests) runs after T003–T008 and must pass before US phases begin.
- Within US1: T011 (App::new sig) before T012 (main.rs call); T014 before T017–T019.
- T020–T022 (US2) extend T014 — must be done after T014 is merged.

### Parallel Opportunities

- T023 (Config field) and T024 (CLI arg) are independent of US1 tasks and can run in parallel
- T028, T029, T030, T036, T037 (integration tests) are all independent of each other [P]
- T033, T034 (docs) are independent of each other [P]

---

## Parallel Example: Foundational Phase

```text
Sequential (each task depends on the previous type being defined):
T003 → T004 → T005 → T006 → T007 → T008 → T009
```

## Parallel Example: Polish Phase

```text
Parallel (all independent files):
T028  tests/integration/session.rs (core round-trip + corrupt/absent)
T029  tests/integration/session.rs (no-session flag test)
T030  tests/integration/session.rs (explicit files test)
T036  tests/integration/session.rs (partial restore / missing file — C1)
T037  tests/integration/session.rs (crash-exit does not write session — C2)
T033  docs/STATUS.md
T034  docs/CAPABILITIES.md

Sequential (each depends on parallel tasks completing):
T031  Cargo.toml [[test]] registration
T032  CHANGELOG.md
T035  man/edit.1
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T009) — all unit tests green
3. Complete Phase 3: User Story 1 (T010–T019)
4. **STOP and VALIDATE**: Run quickstart.md scenario 1 manually
5. US1 is independently shippable as the core value proposition

### Incremental Delivery

1. Setup + Foundational → session module working, all unit tests pass
2. US1 → full restore flow working; clean exit writes session; prompt + restore on relaunch
3. US2 → missing-file degradation working (no regressions on US1)
4. US3 → `--no-session` working
5. US4 → file-args bypass confirmed (US4 is effectively free from T012's guard)
6. Polish → integration tests, docs, CI gate

---

## Notes

- [P] tasks operate on different files and have no dependency on incomplete tasks in the same phase
- [Story] labels map each task to a specific user story for traceability
- `cargo test` must stay green after every task — no "will fix later" broken builds
- `cargo clippy -- -D warnings` must be clean after every task
- Session write failures (disk full, permissions) log `warn!` only — never show an error dialog to the user on exit
- Cursor coordinates stored 1-based in TOML, converted to 0-based (`saturating_sub(1)`) in `do_restore_session`
