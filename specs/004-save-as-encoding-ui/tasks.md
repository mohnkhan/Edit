# Tasks: Save-As Encoding Selection UI (Feature 004)

**Input**: Design documents from `specs/004-save-as-encoding-ui/`

**Prerequisites**: plan.md ✅ | spec.md ✅ | research.md ✅ | data-model.md ✅ | contracts/ ✅

**Organization**: Tasks are grouped by user story to enable independent implementation
and testing of each story. Foundational tasks (Action enum, dialog widget, menu item)
block all user-story phases and must complete first.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no blocking dependencies)
- **[Story]**: Which user story this task belongs to (US1–US4 from spec.md)
- Exact file paths included in every task description

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the source tree compiles cleanly before any modification and that
no external package changes are needed (no new Cargo dependencies for this feature).

- [X] T001 Run `cargo build` on the unmodified tree and confirm it succeeds — establishes a clean baseline before any edits

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add `Action::SaveAsEncoding`, the `EncodingSelectDialog` widget, and the
File-menu entry. Nothing in any user-story phase can compile or dispatch correctly until
this phase passes `cargo test`.

**⚠️ CRITICAL**: No user-story work begins until this phase passes `cargo test`.

- [X] T002 Add `SaveAsEncoding` variant to the `Action` enum in the `// File operations` block (after `SaveAs`) in `src/input/keymap.rs`
- [X] T003 Add `"F12".to_string() → Action::SaveAsEncoding` entry to `KeybindingMap::default_map()` in `src/input/keymap.rs`
- [X] T004 Add `"SaveAsEncoding" => Some(Action::SaveAsEncoding)` match arm to `action_from_str()` in `src/input/keymap.rs`
- [X] T005 Write unit tests in `src/input/keymap.rs` `#[cfg(test)]` block: `test_f12_maps_to_save_as_encoding` (assert `km.get_action("F12") == Some(&Action::SaveAsEncoding)`) and `test_save_as_encoding_round_trips_action_from_str` (assert `action_from_str("SaveAsEncoding") == Some(Action::SaveAsEncoding)`)
- [X] T006 Add `use crate::encoding::EncodingId;` import and define `pub const ENCODING_OPTIONS: &[(EncodingId, &str)]` with exactly 7 ordered entries — `(Utf8,"UTF-8")`, `(Utf16Le,"UTF-16 LE")`, `(Utf16Be,"UTF-16 BE")`, `(Cp437,"CP437")`, `(Cp850,"CP850")`, `(Iso8859_1,"ISO-8859-1")`, `(Windows1252,"Windows-1252")` — in `src/ui/dialog.rs`
- [X] T007 Add `pub struct EncodingSelectDialog { pub cursor_idx: usize, pub theme: &'static Theme }` and implement `Widget for EncodingSelectDialog` in `src/ui/dialog.rs`: use `centered_rect(40, 11, area)`, `Clear` behind dialog, `Block` with title `"Save As Encoding"` and `Borders::ALL` styled with `theme.menubar_fg/menubar_bg`, 7 encoding rows with `Modifier::REVERSED` on the highlighted row, a blank separator, and a hint line `"  [↑↓] Select  [Enter] Save  [Esc] Cancel  "`; when `dialog_area.width < 40`, truncate each encoding label to `(dialog_area.width.saturating_sub(8))` chars and append `"…"` so text never overflows the clamped area (remediation B1)
- [X] T008 Write unit tests in `src/ui/dialog.rs` `#[cfg(test)]` block: `test_encoding_options_has_seven_entries` (assert len == 7), `test_encoding_options_first_is_utf8` (assert `ENCODING_OPTIONS[0].0 == EncodingId::Utf8`), `test_encoding_options_all_labels_nonempty` (all label strs non-empty), `test_encoding_select_dialog_renders_without_panic` (render on a `ratatui::backend::TestBackend` of size (80, 24); assert no panic), `test_encoding_select_dialog_small_terminal_no_panic` (render on a `TestBackend` of size (25, 8); assert no panic and rendered content fits within (25, 8)) (remediation B1)
- [X] T009 Insert `MenuItem { label: "Save As Encoding...", action: Action::SaveAsEncoding }` into the `FILE_MENU` static slice in `src/ui/menubar.rs`, after the `"Save As"` entry and before `"Exit"`

**Checkpoint**: `cargo test` passes all unit tests (T005 + T008) before any US phase begins.

---

## Phase 3: User Story 1 — Save Active Buffer in Chosen Encoding (Priority: P1) 🎯 MVP

**Goal**: User opens encoding dialog (F12 / File menu), selects an encoding, presses Enter
— file is written to disk in that encoding and status bar confirms the save.

**Independent Test**: Open any named file, press F12, move to "UTF-16 LE", press Enter;
`hexdump -C <file> | head -1` shows `FF FE` as the first two bytes; status bar shows
"Saved as UTF-16 LE".

### Implementation for User Story 1

- [X] T010 [US1] Add `pub pending_encoding_select: Option<usize>` and `pending_save_as_encoding: Option<crate::encoding::EncodingId>` fields to `App` struct in `src/app.rs`; initialize both to `None` in `App::new`
- [X] T011 [US1] Add private helpers `fn encoding_to_idx(enc: crate::encoding::EncodingId) -> usize` (returns index in `ENCODING_OPTIONS` or 0) and `fn label_for_encoding(enc: crate::encoding::EncodingId) -> &'static str` (returns label from `ENCODING_OPTIONS` or `"unknown"`) in `src/app.rs`
- [X] T012 [US1] In `handle_action` in `src/app.rs`, add encoding-dialog intercept guard at the top of the method (before existing match arms): when `self.pending_encoding_select.is_some()`, handle `Action::MoveUp` (decrement with wrap `(idx+N-1)%N`), `Action::MoveDown` (increment with wrap `(idx+1)%N`), `Action::InsertNewline` (call `do_save_as_encoding`, clear field), `Action::MenuClose` (clear field), all other actions (drop silently with `return`)
- [X] T013 [US1] In `handle_action` in `src/app.rs`, add match arm for `Action::SaveAsEncoding`: read `self.active_buf().map(|b| Self::encoding_to_idx(b.encoding)).unwrap_or(0)` and set `self.pending_encoding_select = Some(idx)`
- [X] T014 [US1] Implement `fn do_save_as_encoding(&mut self, enc: crate::encoding::EncodingId)` in `src/app.rs` — **Case A (named buffer)**: save old encoding as `old_enc`, set `buf.encoding = enc`, call `buf.save()`; on `Ok` set `self.status_message = Some(format!("Saved as {}", Self::label_for_encoding(enc)))`; on `Err` revert `buf.encoding = old_enc` and set `self.status_message = Some(format!("Save failed: {}", e))`; add explicit `} else { /* Case B — unnamed buffer; implemented in T020 */ }` stub for the `buf.path.is_none()` branch so the partial implementation is visible at code-review time (remediations A1 canonical error format, C2 partial-impl stub)
- [X] T015 [US1] In `Ui::render` (or equivalent render function) in `src/ui/mod.rs`, add branch after the session-restore dialog check: when `app.pending_encoding_select.is_some()`, construct `crate::ui::dialog::EncodingSelectDialog { cursor_idx: idx, theme: app.theme }` and render it via `frame.render_widget(dialog, frame.size())`
- [X] T016 [US1] Write unit tests in `src/app.rs` `#[cfg(test)]` block: `test_save_as_encoding_action_opens_dialog` (dispatch `SaveAsEncoding` on UTF-8 buf → assert `pending_encoding_select == Some(0)`), `test_dialog_preselects_current_encoding` (UTF-16 LE buf → assert `Some(1)`), `test_dialog_move_down_increments_idx` (Some(1) → MoveDown → Some(2)), `test_dialog_move_down_wraps_at_end` (Some(6) → MoveDown → Some(0)), `test_dialog_move_up_wraps_at_start` (Some(0) → MoveUp → Some(6)), `test_dialog_escape_closes` (Some(3) → MenuClose → None), `test_dialog_other_action_consumed` (Some(2) → MoveLeft → assert still Some(2) AND `self.buffers[0].cursor.grapheme_col` is unchanged) (remediation C1: field is `grapheme_col` not `col` per `src/buffer/mod.rs:99`)

**Checkpoint**: `cargo test` passes; quickstart.md Scenario 1 works end-to-end.

---

## Phase 4: User Story 2 — Cancel Without Saving (Priority: P2)

**Goal**: Pressing Escape in the encoding dialog closes it without writing the file and
without changing the buffer's encoding.

**Independent Test**: Open file, open dialog, press Esc; `md5sum <file>` is unchanged from
before the dialog opened; status bar shows no encoding-change message.

### Implementation for User Story 2

*(Cancel behaviour is fully implemented by the `Action::MenuClose` arm in T012. This phase
adds targeted test coverage for the cancel contract.)*

- [X] T017 [US2] Write unit test `test_cancel_does_not_write_and_leaves_encoding_unchanged` in `src/app.rs` `#[cfg(test)]`: set `pending_encoding_select = Some(3)` on an App backed by a real temp file; dispatch `Action::MenuClose`; assert `pending_encoding_select == None`, `buf.encoding` is unchanged (still UTF-8), and the file's mtime and checksum are unchanged

**Checkpoint**: `cargo test` passes; quickstart.md Scenario 2 works end-to-end.

---

## Phase 5: User Story 3 — Encoding Persists for Subsequent Saves (Priority: P3)

**Goal**: After saving with a new encoding via the dialog, pressing Ctrl+S / F5 again
saves the file in the same encoding — no silent reversion to UTF-8.

**Independent Test**: Save file as UTF-16 LE via dialog; edit one character; press Ctrl+S;
`hexdump -C <file> | head -1` still shows `FF FE`.

### Implementation for User Story 3

*(Encoding persistence is fully implemented by `buf.encoding = enc` in T014's Case A. The
dialog preselect-on-reopen is handled by T013. This phase adds targeted test coverage.)*

- [X] T018 [US3] Write unit test `test_encoding_persists_on_regular_save` in `src/app.rs` `#[cfg(test)]`: call `do_save_as_encoding(EncodingId::Utf16Le)` on an App backed by a real temp file; then dispatch `Action::Save`; read the file bytes and assert `bytes[0..2] == [0xFF, 0xFE]` (UTF-16 LE BOM still present)
- [X] T019 [US3] Write unit test `test_dialog_reopens_with_updated_preselect` in `src/app.rs` `#[cfg(test)]`: after `do_save_as_encoding(EncodingId::Utf16Be)` succeeds, dispatch `Action::SaveAsEncoding` again; assert `pending_encoding_select == Some(2)` (index of UTF-16 BE in ENCODING_OPTIONS)

**Checkpoint**: `cargo test` passes; quickstart.md Scenario 3 works end-to-end.

---

## Phase 6: User Story 4 — Unnamed Buffer Triggers Filename Prompt (Priority: P4)

**Goal**: When the active buffer has no file path, the encoding dialog confirmation
triggers the existing filename-input flow before writing. If the user cancels the filename
prompt, no file is written and the encoding selection is discarded.

**Independent Test**: New buffer (no file arg), press F12, select CP437, Enter; the
filename-input dialog appears; type `/tmp/out.txt`, Enter; `file /tmp/out.txt` reports
ISO-8859 (CP437 compatible); if the filename prompt is Esc'd, no file is created.

### Implementation for User Story 4

- [X] T020 [US4] In `fn do_save_as_encoding` in `src/app.rs`, implement **Case B** (unnamed buffer): when `buf.path.is_none()`, set `self.pending_save_as_encoding = Some(enc)` and call `self.handle_save_as()` to open the filename-input dialog
- [X] T021 [US4] In the `handle_save_as` completion path in `src/app.rs` (where a confirmed path triggers the file write): before `buf.save_as(path)` (or equivalent write call), add `if let Some(enc) = self.pending_save_as_encoding.take() { self.buffers[self.active_idx].encoding = enc; }`; in the cancel path, add `self.pending_save_as_encoding = None;`
- [X] T022 [US4] Write unit tests in `src/app.rs` `#[cfg(test)]`: `test_unnamed_buf_encoding_applied_after_filename_confirm` (set `pending_save_as_encoding = Some(EncodingId::Utf16Le)`; simulate filename confirm with a temp path; assert `buf.encoding == Utf16Le` and `pending_save_as_encoding == None`) and `test_unnamed_buf_encoding_cleared_on_filename_cancel` (set `pending_save_as_encoding = Some(enc)`; simulate cancel; assert field is `None`)

**Checkpoint**: `cargo test` passes; quickstart.md Scenario 4 works end-to-end.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Integration tests, documentation gate, and final CI validation.

- [X] T023 [P] Write integration test `test_save_utf8_file_as_utf16le` in `tests/integration/encoding_select.rs`: create temp UTF-8 file, call `do_save_as_encoding(Utf16Le)` via test App, read bytes, assert `bytes[0..2] == [0xFF, 0xFE]`, decode content and compare to original
- [X] T024 [P] Write integration test `test_save_utf8_file_as_utf16be` in `tests/integration/encoding_select.rs`: same pattern for UTF-16 BE; assert BOM `[0xFE, 0xFF]`
- [X] T025 [P] Write integration test `test_cancel_leaves_file_unchanged` in `tests/integration/encoding_select.rs`: record pre-dialog checksum; dispatch `Action::SaveAsEncoding` then `Action::MenuClose`; assert checksum unchanged
- [X] T026 [P] Write integration test `test_encoding_persists_on_regular_save` in `tests/integration/encoding_select.rs`: save as UTF-16 LE via dialog; dispatch `Action::Save`; assert file bytes start with `[0xFF, 0xFE]`
- [X] T027 [P] Write integration test `test_io_error_reverts_encoding` in `tests/integration/encoding_select.rs`: create read-only temp file; call `do_save_as_encoding(Utf16Be)`; assert `buf.encoding` is still `Utf8` and `status_message` contains "Save failed"
- [X] T028 [P] Write integration test `test_new_buffer_pending_encoding_held` in `tests/integration/encoding_select.rs`: new App; set `pending_save_as_encoding = Some(Cp437)`; confirm a filename; assert `buf.encoding == Cp437` and `pending_save_as_encoding == None`
- [X] T029 Add `[[test]]` entry `name = "encoding_select"`, `path = "tests/integration/encoding_select.rs"` to `Cargo.toml`
- [X] T030 [P] Update `CHANGELOG.md` — add feature 004 entry under `[Unreleased]`: "Save As Encoding dialog (F12 / File › Save As Encoding...); UTF-8, UTF-16 LE/BE, CP437, CP850, ISO-8859-1, Windows-1252; encoding persists for subsequent saves (FR-001–FR-013)"
- [X] T031 [P] Update `docs/STATUS.md` — add F004 user story rows (US1–US4) with status Complete
- [X] T032 [P] Update `docs/CAPABILITIES.md` — add F12 to the keybindings table with description "Save As Encoding dialog"; add "Save As Encoding..." to the File menu items table
- [X] T033 [P] Update `man/edit.1` — add `F12` entry in the KEYBINDINGS section and add "Save As Encoding..." to the File menu description in the MENUS section (remediation D1: removed unnecessary sequential constraint)
- [X] T034 Write `tests/smoke/encoding_select.exp` expect script: launch editor on a temp UTF-8 file; send `F12`; assert dialog title "Save As Encoding" appears in terminal output; send `↓↓`; send `Enter`; assert file first two bytes are `FE FF` (UTF-16 BE BOM) using `hexdump`; exit with status 0 (remediation I1: Constitution V smoke test for F12/dialog/confirm flow)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — **BLOCKS all US phases**
- **Phase 3 (US1)**: Depends on Phase 2 completion; T010→T011→T012→T013→T014→T015→T016 (sequential within US1)
- **Phase 4 (US2)**: Depends on Phase 3 (T017 validates T012's cancel path)
- **Phase 5 (US3)**: Depends on Phase 3 (T018/T019 validate T014's persistence)
- **Phase 6 (US4)**: Depends on Phase 3 (T020–T022 extend T014 Case B)
- **Phase 7 (Polish)**: Depends on Phases 3–6 completion

### User Story Dependencies

- **US1 (P1)**: Requires Phase 2 only. Core story.
- **US2 (P2)**: Requires US1 (cancel path is in T012, already implemented in US1).
- **US3 (P3)**: Requires US1 (persistence is in T014, already implemented in US1).
- **US4 (P4)**: Requires US1 (Case B of `do_save_as_encoding` + handle_save_as composition).

### Within Each Phase

- T002 → T003 → T004 must be sequential (each adds to the same enum/map/function).
- T006 → T007 must be sequential (T007 uses the const from T006).
- T008 depends on T006 + T007.
- T009 depends on T002 (needs `Action::SaveAsEncoding` in scope).
- T010 → T011 → T012 → T013 → T014 → T015 → T016 are sequential (each builds on previous).
- T020 → T021 → T022 are sequential.
- T023–T028 and T030–T032 are all independent [P].
- T029 depends on T023–T028 (test file must exist before registering in Cargo.toml).
- T033 and T034 are independent of T030–T032 and can run in parallel [P].

### Parallel Opportunities

- T002–T004 must be sequential; T006–T008 can start in parallel with T002–T005 (different files)
- T009 can start once T002 is done (different file from T006–T008)
- T023–T028 (integration tests) are all independent [P]
- T030–T032 (docs) are all independent [P]

---

## Parallel Example: Foundational Phase

```text
Sequential (enum/keymap additions depend on each prior step):
T002 → T003 → T004 → T005   (src/input/keymap.rs)

Parallel with above (different file):
T006 → T007 → T008          (src/ui/dialog.rs)

Sequential (depends on T002 for Action variant in scope):
T009                         (src/ui/menubar.rs — after T002 done)
```

## Parallel Example: Polish Phase

```text
Parallel (all independent):
T023  encoding_select.rs (UTF-16 LE round-trip)
T024  encoding_select.rs (UTF-16 BE round-trip)
T025  encoding_select.rs (cancel unchanged)
T026  encoding_select.rs (persistence on Ctrl+S)
T027  encoding_select.rs (IO error reverts encoding)
T028  encoding_select.rs (new-buffer pending encoding)
T030  CHANGELOG.md
T031  docs/STATUS.md
T032  docs/CAPABILITIES.md
T033  man/edit.1
T034  tests/smoke/encoding_select.exp

Sequential (Cargo.toml registration after all test files written):
T029  Cargo.toml [[test]] entry
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002–T009) — all unit tests green
3. Complete Phase 3: User Story 1 (T010–T016)
4. **STOP and VALIDATE**: Run quickstart.md Scenario 1 manually
5. US1 is independently shippable as the core value proposition

### Incremental Delivery

1. Setup + Foundational → action dispatches, dialog widget renders, menu item appears
2. US1 → full save flow; F12 → dialog → encoding select → file write → status bar
3. US2 → cancel path tested explicitly (already implemented; adds test coverage)
4. US3 → persistence tested explicitly (already implemented; adds test coverage)
5. US4 → unnamed-buffer filename-prompt composition working
6. Polish → integration tests, docs, CI gate

---

## Notes

- [P] tasks operate on different files with no dependency on incomplete tasks in the same phase
- [Story] labels map each task to a specific user story for traceability
- `cargo test` must stay green after every task — no "will fix later" broken builds
- `cargo clippy -- -D warnings` must be clean after every task
- `buf.encoding` is updated **only** after a successful `buf.save()` — never on IO error
- `pending_save_as_encoding` must be cleared in **both** the confirm and cancel paths of the filename prompt to prevent stale state on subsequent encodings-dialog invocations
- Session.toml does not currently record per-buffer encoding; this is a known follow-up (ROADMAP)
