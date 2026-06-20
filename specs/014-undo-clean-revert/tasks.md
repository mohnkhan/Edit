---
description: "Task list for feature 014: Undo-to-clean state and Revert to saved"
---

# Tasks: Undo-to-clean state and Revert to saved

**Input**: Design documents from `specs/014-undo-clean-revert/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/clean-state-and-revert.md

**Tests**: INCLUDED — Constitution Principle V (Test-Gated Merges); TDD mandatory.

**Organization**: by user story (US1 undo-to-clean, US2 no-false-clean, US3 Revert).

## Path Conventions

Single Rust project. Primary files: `src/buffer/undo.rs`, `src/buffer/mod.rs`, `src/app.rs`,
`src/input/keymap.rs`, `src/ui/menubar.rs`.

---

## Phase 1: Setup

- [x] T001 Create integration test file `tests/integration/undo_clean_revert.rs` (scaffold mirroring
  `tests/integration/menu_mnemonics.rs`) and register a `[[test]]` target in `Cargo.toml`.

---

## Phase 2: Foundational (saved-point marker in the undo history)

- [x] T002 [P] Unit tests in `src/buffer/undo.rs` for the saved marker: `mark_saved` + `is_at_saved`
  true at the marked cursor; false after an edit; true again after undo back to it; true after redo back
  to it; **and the divergent case**: save → undo → push → `is_at_saved()` is false (marker invalidated,
  no false-clean).
- [x] T003 Add `saved: Option<usize>` to `UndoStack` (init `None` in `new`); implement
  `mark_saved(&mut self)` and `is_at_saved(&self) -> bool` in `src/buffer/undo.rs`.
- [x] T004 In `UndoStack::push`, before `truncate_redo`, invalidate the marker when it is in the branch
  being discarded: `if let Some(s) = self.saved { if s > self.cursor { self.saved = None; } }`.

**Checkpoint**: `cargo test --lib undo` passes (T002 green).

---

## Phase 3: User Story 1 — Undo back to saved clears Modified (Priority: P1)

- [x] T005 [P] [US1] Unit test in `src/buffer/mod.rs`: a freshly `open`ed buffer is not modified and
  `undo_stack.is_at_saved()`; `new_empty()` buffer likewise clean.
- [x] T006 [US1] In `Buffer::open` call `undo_stack.mark_saved()` after construction; in `new_empty()`
  call `mark_saved()` on its fresh stack. Add `Buffer::refresh_modified(&mut self)` =
  `self.modified = !self.undo_stack.is_at_saved()`.
- [x] T007 [US1] In `App::apply_history_cursor` (`src/app.rs`), replace `buf.modified = true` with
  `buf.refresh_modified()` so undo/redo clears Modified when content returns to the saved baseline.
- [x] T008 [US1] After each successful save path in `src/app.rs` (`handle_save_action`, `do_save_as`,
  `do_save_as_encoding`), call `self.buffers[idx].undo_stack.mark_saved()` (keeping `modified = false`).
- [x] T009 [P] [US1] Integration test in `tests/integration/undo_clean_revert.rs`: open a temp file,
  edit (modified true), undo (modified false), redo (modified true); save after edits then undo back to
  the save point → modified false; undo before save point → modified true.

**Checkpoint**: US1 verifiable — Modified flag tracks the saved baseline through undo/redo.

---

## Phase 4: User Story 2 — No false clean after divergent edits (Priority: P1)

- [x] T010 [P] [US2] Integration test in `tests/integration/undo_clean_revert.rs`: save; edit A; edit B;
  undo once; divergent edit C → buffer modified and cannot be made clean by undo/redo; also save → edit →
  undo (clean) → different edit → modified. Assert content-vs-clean correctness (no false clean).
- [x] T011 [US2] Verify (and if needed harden) the `push` invalidation from T004 against the integration
  scenarios; no code beyond T004 expected, but confirm via the new tests.

**Checkpoint**: US2 verifiable — never falsely clean.

---

## Phase 5: User Story 3 — Revert to saved (Priority: P2)

- [x] T012 [US3] Add `Action::Revert` to `src/input/keymap.rs` (enum + string parse arm; no default
  keybinding) and a `File ▸ Revert` item `{ "Revert", Action::Revert, mnemonic 'r' }` before `Exit` in
  `src/ui/menubar.rs`.
- [x] T013 [US3] Add `pending_revert_confirm: Option<usize>` field to `App` (init `None`) and a confirm
  modal: intercept in `handle_action` (Enter/`Y` → reload + clear; Esc/`N` → clear), placed with the
  other modal guards; render a simple confirm dialog.
- [x] T014 [US3] Implement `Action::Revert` handling in `src/app.rs`: no path → status
  "Nothing to revert (never saved)"; path + clean → `reload_from_disk(active_idx)`; path + modified →
  set `pending_revert_confirm`. Reuse `reload_from_disk` (feature 007) for the actual reload.
- [x] T015 [P] [US3] Integration test in `tests/integration/undo_clean_revert.rs`: file-backed buffer +
  edits → Revert + confirm → content equals disk and clean; Revert + cancel → unchanged; clean buffer →
  Revert reloads with no harm; pathless buffer → Revert is a no-op with a notice.
- [x] T016 [US3] Unit/integration test that a reload failure (e.g. deleted file) leaves the buffer and
  its Modified state unchanged and surfaces a notice (reuses `reload_from_disk` error path).

**Checkpoint**: US3 verifiable — Revert restores disk content with confirmation/guards.

---

## Phase 6: Polish & Cross-Cutting

- [x] T017 [P] Update `CHANGELOG.md` (`[Unreleased]` → feature 014: undo-to-clean Modified tracking +
  File ▸ Revert).
- [x] T018 [P] Update `docs/STATUS.md` (F014 user-story rows) and `docs/CAPABILITIES.md` (File menu:
  add Revert; note Modified clears on undo-to-saved).
- [x] T019 Run the full gate: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`;
  then live-verify in tmux per `quickstart.md` (edit→undo clears `[Modified]`; File ▸ Revert reloads).

---

## Dependencies & Execution Order

- Setup (T001) → Foundational (T002–T004) blocks all stories.
- US1 (T005–T009) depends on Foundational. US2 (T010–T011) depends on Foundational + the push
  invalidation. US3 (T012–T016) depends on Foundational (clean reload yields clean buffer) but is
  otherwise independent of US1/US2.
- Polish (T017–T019) last.

## Parallel Opportunities

- T002, T005 (unit tests in different files); T009, T010, T015 (integration scenarios); T017, T018 (docs).

## Implementation Strategy

- **MVP = Foundational + US1 + US2** (P1): correct Modified tracking with no false-clean.
- **Increment = US3** (P2): File ▸ Revert.
- Each phase green before the next (test-gated).
