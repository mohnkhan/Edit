# Implementation Plan: Save-As Encoding Selection UI

**Branch**: `004-save-as-encoding-ui` | **Date**: 2026-06-19 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/004-save-as-encoding-ui/spec.md`

## Summary

Add a modal TUI listbox dialog that lets the user select an output encoding (UTF-8, UTF-16 LE,
UTF-16 BE, CP437, CP850, ISO-8859-1, Windows-1252) when saving a file. Triggered via a new
"Save As Encoding..." File menu item and the F12 key. On confirmation the active buffer is
written to disk using the selected encoding (via the existing `encode()` + atomic-write
pipeline from feature 002); the buffer's encoding is updated so subsequent saves use the
same encoding. On cancel the file and encoding are unchanged.

All required infrastructure (encoding registry, encode/decode pipeline, ratatui dialog
pattern, atomic-write, App state machine) already exists. This feature adds one new
`Action` variant, one new `App` field, one new dialog widget, and the wiring between them.

## Technical Context

**Language/Version**: Rust stable, edition 2021; MSRV 1.74.0

**Primary Dependencies** (all already in `Cargo.toml`):
- `ratatui 0.26` — dialog overlay widget (`EncodingSelectDialog`)
- `encoding_rs` / `crate::encoding` — `EncodingId`, `encode()` (feature 002 infrastructure)
- No new Cargo dependencies required

**Storage**:
- No new storage. The active buffer's `encoding: EncodingId` field is updated in-memory;
  the file is written via the existing `Buffer::save()` atomic-write path.

**Testing**:
- Unit: `cargo test` — `EncodingSelectDialog` widget render tests (in `src/ui/dialog.rs`)
- Integration: `cargo test --test encoding_select` — new `tests/integration/encoding_select.rs`;
  covers save round-trip, cancel idempotency, encoding persistence, new-buffer filename flow
- Smoke: extend existing expect scripts in `tests/smoke/` to cover F12 dialog open + confirm

**Target Platform**: Same as project — Linux x86_64/ARM64, FreeBSD, macOS; no new
platform-specific code paths.

**Project Type**: CLI terminal application (existing); no structural change.

**Performance Goals**:
- Dialog open: single frame cycle (same tick as F12 keypress) — no perceptible delay
- Save on confirm: bounded by existing `encode()` pipeline (≤ 50 ms for files ≤ 10 MB)

**Constraints**:
- New buffer (no path): encoding-select must compose with the existing `handle_save_as`
  filename-prompt flow — no duplicate dialog logic
- `buf.encoding` is updated **only** after a successful `buf.save()` — never on failure
- All actions other than Up/Down/Enter/Esc are silently consumed while the dialog is open
- F12 MUST NOT conflict with any existing binding (confirmed unbound as of master)

**Scale/Scope**: Single-buffer operation; no multi-buffer or batch encoding changes in scope.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Gate | Status | Notes |
|---|---|---|---|
| I. DOS-Faithful UI | Dialog is a TUI overlay rendered inside the ratatui frame, styled with DOS theme colors, keyboard-driven (F12 / ↑↓ / Enter / Esc), and added to the File pull-down menu | ✅ PASS | F12 is consistent with the F-key binding row; listbox matches DOS modal dialog UX |
| II. UTF-8 First | The dialog surfaces all `EncodingId` variants; CP437/CP850/ISO-8859-1/Windows-1252 saves are explicit and user-confirmed; internal buffer stays UTF-8 until `encode()` converts for write | ✅ PASS | Legacy encoding writes are always user-initiated — satisfies "explicit and user-confirmed" |
| III. Portable Build | No OS-specific code; `ratatui`, `encoding_rs`, and Rust standard library are cross-platform | ✅ PASS | |
| IV. Minimal Footprint | No new Cargo dependencies; net code addition is ~250 lines across 5 files | ✅ PASS | |
| V. Test-Gated | Unit tests for dialog widget; integration tests for save/cancel/persist scenarios; smoke test extension | ✅ PASS | TDD: write tests before implementation of each component |
| VI. YAGNI | Feature is the exact scope described in ROADMAP.md #9 — no extra encodings, no batch operations, no encoding autodetect-on-save | ✅ PASS | |
| VII. Security | File write uses existing atomic tmp-rename path; no new path handling; no new CLI parsing surface | ✅ PASS | `buf.save()` already sanitizes via OS file API; no traversal risk in encoding select |

**No violations.**

## Project Structure

### Documentation (this feature)

```text
specs/004-save-as-encoding-ui/
├── plan.md                          # This file
├── research.md                      # Phase 0 decisions
├── data-model.md                    # Phase 1 entity model
├── quickstart.md                    # Phase 1 validation guide
├── contracts/
│   └── encoding-select-ui.md        # UI contract: dialog layout, keys, menu, status bar
└── checklists/
    └── requirements.md              # Spec quality checklist
```

### Source Code

```text
src/
├── input/
│   └── keymap.rs        # MODIFY: add Action::SaveAsEncoding; bind F12; add to action_from_str
├── ui/
│   ├── dialog.rs        # MODIFY: add ENCODING_OPTIONS const + EncodingSelectDialog widget
│   └── menubar.rs       # MODIFY: add "Save As Encoding..." MenuItem to FILE_MENU
└── app.rs               # MODIFY: add pending_encoding_select + pending_save_as_encoding fields;
                         #   dialog state transitions; do_save_as_encoding(); handle_action branches

tests/integration/
└── encoding_select.rs   # NEW: integration tests

Cargo.toml               # MODIFY: add [[test]] entry for encoding_select
CHANGELOG.md             # MODIFY: feature 004 entry
docs/STATUS.md           # MODIFY: F004 user story rows
docs/CAPABILITIES.md     # MODIFY: F12 keybinding + "Save As Encoding..." menu item entry
man/edit.1               # MODIFY: document F12 in KEYBINDINGS + "Save As Encoding..." in MENUS
```

## Implementation Phases

### Phase 1 — Action & Keymap (Foundation)

**Goal**: Wire `Action::SaveAsEncoding` into the input pipeline so it can be fired by F12
and from the menu.

**Files**: `src/input/keymap.rs`

**Tasks**:
- Add `SaveAsEncoding` variant to the `Action` enum in the `// File operations` block,
  after `SaveAs`
- Add `"F12".to_string() → Action::SaveAsEncoding` to `KeybindingMap::default_map()`
- Add `"SaveAsEncoding" => Some(Action::SaveAsEncoding)` to `action_from_str()`

**Tests** (unit, in `src/input/keymap.rs` `#[cfg(test)]`):
- `test_f12_maps_to_save_as_encoding` — `km.get_action("F12") == Some(&Action::SaveAsEncoding)`
- `test_save_as_encoding_round_trips_action_from_str` — `action_from_str("SaveAsEncoding") == Some(Action::SaveAsEncoding)`

**Checkpoint**: `cargo test` passes.

---

### Phase 2 — Dialog Widget (Foundation)

**Goal**: Implement the `EncodingSelectDialog` ratatui widget and the `ENCODING_OPTIONS`
constant in `src/ui/dialog.rs`.

**Files**: `src/ui/dialog.rs`

**Tasks**:
- Add import: `use crate::encoding::EncodingId;` at the top of `src/ui/dialog.rs`
- Add `pub const ENCODING_OPTIONS: &[(EncodingId, &str)] = &[...]` with 7 entries ordered:
  `(Utf8, "UTF-8")`, `(Utf16Le, "UTF-16 LE")`, `(Utf16Be, "UTF-16 BE")`,
  `(Cp437, "CP437")`, `(Cp850, "CP850")`, `(Iso8859_1, "ISO-8859-1")`,
  `(Windows1252, "Windows-1252")`
- Add `pub struct EncodingSelectDialog { pub cursor_idx: usize, pub theme: &'static Theme }`
- Implement `Widget for EncodingSelectDialog`:
  - Compute `centered_rect(40, 11, area)` — reuse existing `centered_rect` helper
  - `Clear.render(dialog_area, buf)` — clear behind dialog
  - Build `dialog_style = Style::default().fg(theme.menubar_fg).bg(theme.menubar_bg)`
  - Build content: 7 rows of `format!("  {:<24}", label)` for each encoding; apply
    `Modifier::REVERSED` on `dialog_style` for the row where `idx == cursor_idx`
  - Add blank separator line after the 7 rows
  - Add hint line: `"  [↑↓] Select  [Enter] Save  [Esc] Cancel  "`
  - Render as `Paragraph::new(lines).style(dialog_style).block(Block::default().title("Save As Encoding").borders(Borders::ALL).style(dialog_style))`

**Tests** (unit, in `src/ui/dialog.rs` `#[cfg(test)]`):
- `test_encoding_options_has_seven_entries` — `assert_eq!(ENCODING_OPTIONS.len(), 7)`
- `test_encoding_options_first_is_utf8` — `assert_eq!(ENCODING_OPTIONS[0].0, EncodingId::Utf8)`
- `test_encoding_options_all_labels_nonempty` — all label strings are non-empty
- `test_encoding_select_dialog_renders_without_panic` — construct `EncodingSelectDialog`
  with `cursor_idx = 0` and a test theme; call `Widget::render` on a `ratatui::backend::TestBackend`
  buffer of size (80, 24); assert no panic

**Checkpoint**: `cargo test` passes including new unit tests.

---

### Phase 3 — File Menu Item (Foundation)

**Goal**: Expose "Save As Encoding..." in the File pull-down menu.

**Files**: `src/ui/menubar.rs`

**Tasks**:
- In the `FILE_MENU` static slice, after the `Save As` entry and before `Exit`, insert:
  ```rust
  MenuItem { label: "Save As Encoding...", action: Action::SaveAsEncoding }
  ```
- Verify `Action::SaveAsEncoding` is in scope (it is, via `use crate::input::keymap::Action`)

**Tests**: Covered by existing smoke tests; no new unit test required.

**Checkpoint**: `cargo build` succeeds; `cargo clippy -- -D warnings` clean.

---

### Phase 4 — App State Machine (Core Logic)

**Goal**: Wire the dialog open/close/confirm/cancel lifecycle into `App`; implement
`do_save_as_encoding`.

**Files**: `src/app.rs`

**Tasks**:

1. Add two new fields to `App` struct:
   ```rust
   /// `Some(idx)` while the encoding-select dialog is open; `None` otherwise.
   pub pending_encoding_select: Option<usize>,
   /// Encoding held while the Save-As filename prompt is open for an unnamed buffer.
   pending_save_as_encoding: Option<crate::encoding::EncodingId>,
   ```
   Initialize both to `None` in `App::new`.

2. Add private helper `fn encoding_to_idx(enc: crate::encoding::EncodingId) -> usize`:
   - Iterate `crate::ui::dialog::ENCODING_OPTIONS`; return the first matching index;
     default to 0 if not found.

3. Add private helper `fn label_for_encoding(enc: crate::encoding::EncodingId) -> &'static str`:
   - Return the label string from `ENCODING_OPTIONS` for the given `EncodingId`;
     default to `"unknown"` if not found.

4. In `handle_action`, add a **guard block at the top** (before any existing match arm) —
   when `self.pending_encoding_select.is_some()`:
   ```rust
   if let Some(idx) = self.pending_encoding_select {
       let n = crate::ui::dialog::ENCODING_OPTIONS.len();
       match action {
           Action::MoveUp => {
               self.pending_encoding_select = Some((idx + n - 1) % n);
           }
           Action::MoveDown => {
               self.pending_encoding_select = Some((idx + 1) % n);
           }
           Action::InsertNewline => {
               let enc = crate::ui::dialog::ENCODING_OPTIONS[idx].0;
               self.pending_encoding_select = None;
               self.do_save_as_encoding(enc);
           }
           Action::MenuClose => {
               self.pending_encoding_select = None;
           }
           _ => {} // consume silently
       }
       return;
   }
   ```

5. In main `handle_action` match, add arm for `Action::SaveAsEncoding`:
   ```rust
   Action::SaveAsEncoding => {
       if let Some(buf) = self.active_buf() {
           let idx = Self::encoding_to_idx(buf.encoding);
           self.pending_encoding_select = Some(idx);
       }
   }
   ```

6. Implement `fn do_save_as_encoding(&mut self, enc: crate::encoding::EncodingId)`:
   - Guard: if `self.buffers.is_empty()`, return early
   - Get active buffer index `idx = self.active_idx`
   - **Case A — buffer has a path**:
     ```rust
     let old_enc = self.buffers[idx].encoding;
     self.buffers[idx].encoding = enc;
     match self.buffers[idx].save() {
         Ok(()) => {
             let label = Self::label_for_encoding(enc);
             self.status_message = Some(format!("Saved as {}", label));
         }
         Err(e) => {
             self.buffers[idx].encoding = old_enc;
             self.status_message = Some(format!("Save failed: {}", e));
         }
     }
     ```
   - **Case B — buffer has no path**:
     ```rust
     self.pending_save_as_encoding = Some(enc);
     self.handle_save_as();
     ```

**Tests** (unit, in `src/app.rs` `#[cfg(test)]`):
- `test_save_as_encoding_action_opens_dialog` — dispatch `Action::SaveAsEncoding` on App
  with a UTF-8 buffer; assert `pending_encoding_select == Some(0)`
- `test_dialog_preselects_current_encoding` — dispatch `Action::SaveAsEncoding` on App
  with a UTF-16 LE buffer; assert `pending_encoding_select == Some(1)`
- `test_dialog_move_down_increments_idx` — set `pending_encoding_select = Some(1)`;
  dispatch `Action::MoveDown`; assert `Some(2)`
- `test_dialog_move_down_wraps_at_end` — set `pending_encoding_select = Some(6)`;
  dispatch `Action::MoveDown`; assert `Some(0)`
- `test_dialog_move_up_wraps_at_start` — set `pending_encoding_select = Some(0)`;
  dispatch `Action::MoveUp`; assert `Some(6)`
- `test_dialog_escape_closes` — set `pending_encoding_select = Some(3)`;
  dispatch `Action::MenuClose`; assert `None`
- `test_dialog_other_action_consumed` — set `pending_encoding_select = Some(2)`;
  dispatch `Action::MoveLeft`; assert `pending_encoding_select == Some(2)` (unchanged)
  AND `self.buffers[0].cursor.col` is unchanged

**Checkpoint**: `cargo test` passes all new unit tests.

---

### Phase 5 — Filename Prompt Composition for Unnamed Buffers

**Goal**: Apply `pending_save_as_encoding` when the filename prompt completes for an
unnamed buffer triggered by the encoding dialog.

**Files**: `src/app.rs`

**Tasks**:
- Locate the confirmation path in `handle_save_as` (or whichever method writes the file
  after the user enters a path in the Open/Save-As dialog)
- After the path is confirmed and before `buf.save_as(path)` (or equivalent write call):
  ```rust
  if let Some(enc) = self.pending_save_as_encoding.take() {
      self.buffers[self.active_idx].encoding = enc;
  }
  ```
- In the cancel path of the same filename-prompt flow, also clear the pending encoding:
  ```rust
  self.pending_save_as_encoding = None;
  ```

**Tests** (unit):
- `test_unnamed_buf_encoding_applied_after_filename_confirm` — set
  `pending_save_as_encoding = Some(EncodingId::Utf16Le)`; simulate filename confirm with a
  temp path; assert `buf.encoding == EncodingId::Utf16Le` and `pending_save_as_encoding == None`
- `test_unnamed_buf_encoding_cleared_on_filename_cancel` — set
  `pending_save_as_encoding = Some(enc)`; simulate filename cancel; assert
  `pending_save_as_encoding == None`

**Checkpoint**: `cargo test` passes.

---

### Phase 6 — TUI Rendering

**Goal**: Render `EncodingSelectDialog` overlay when the dialog is open.

**Files**: `src/ui/mod.rs`

**Tasks**:
- Add import: `use crate::ui::dialog::{EncodingSelectDialog, ENCODING_OPTIONS};` (or bring
  in scope as needed)
- In `Ui::render` (or equivalent render method), after the session-restore dialog check:
  ```rust
  if let Some(idx) = app.pending_encoding_select {
      let dialog = EncodingSelectDialog { cursor_idx: idx, theme: app.theme };
      frame.render_widget(dialog, frame.size());
  }
  ```
  (Use `frame.render_widget` following the pattern of all other dialog renders in this file)

**Tests**: Covered by Phase 2 widget unit test and Phase 7 smoke test.

**Checkpoint**: `cargo build` succeeds; `cargo clippy -- -D warnings` clean.

---

### Phase 7 — Integration Tests & Documentation Gate

**Goal**: Full integration test suite and mandatory docs update.

**Files**:
- `tests/integration/encoding_select.rs` — new
- `Cargo.toml` — add `[[test]]` entry
- `CHANGELOG.md`, `docs/STATUS.md`, `docs/CAPABILITIES.md`, `man/edit.1`

**Integration Test Tasks**:
- `test_save_utf8_file_as_utf16le` — create temp UTF-8 file; build App; call
  `do_save_as_encoding(EncodingId::Utf16Le)` directly; read file bytes; assert first 2
  bytes == `[0xFF, 0xFE]`; decode and compare content
- `test_save_utf8_file_as_utf16be` — same for UTF-16 BE; BOM = `[0xFE, 0xFF]`
- `test_cancel_leaves_file_unchanged` — create temp file; open App; dispatch
  `Action::SaveAsEncoding`; dispatch `Action::MenuClose`; assert file checksum unchanged
- `test_encoding_persists_on_regular_save` — save as UTF-16 LE via `do_save_as_encoding`;
  edit one char; dispatch `Action::Save`; assert file bytes start with `[0xFF, 0xFE]`
- `test_io_error_reverts_encoding` — create read-only temp file; call
  `do_save_as_encoding(EncodingId::Utf16Be)`; assert `buf.encoding` is still `Utf8`
  (original); assert `status_message` contains "Save failed"
- `test_new_buffer_pending_encoding_held` — new App (no file arg); set
  `pending_save_as_encoding = Some(EncodingId::Cp437)`; confirm a filename; assert
  `buf.encoding == Cp437` and `pending_save_as_encoding == None`
- Add `[[test]] name = "encoding_select" path = "tests/integration/encoding_select.rs"` to
  `Cargo.toml`

**Documentation Tasks**:
- `CHANGELOG.md` — add feature 004 entry under `[Unreleased]`
- `docs/STATUS.md` — add F004 user story rows (US1–US4) with status Complete
- `docs/CAPABILITIES.md` — add F12 to the keybindings table; add "Save As Encoding..." to
  the File menu items table
- `man/edit.1` — add F12 in the KEYBINDINGS section; add "Save As Encoding..." to the File
  menu description in the MENUS section

**Checkpoint**: `cargo test --test encoding_select` passes all integration tests;
`make ci-local` green.

---

## Deferred Items

None. The full scope of FR-001–FR-013 is addressed in Phases 1–7.

Items explicitly out of scope (no spec, no issue):
- Encoding auto-detection on save
- Batch encoding change across multiple open buffers
- Per-buffer encoding persistence in `session.toml` (separate follow-up)

## Complexity Tracking

No Constitution Check violations. No complexity justification required.
