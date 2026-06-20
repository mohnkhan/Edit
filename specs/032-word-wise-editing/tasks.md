---
description: "Task list for feature 032 — word-wise navigation, selection, and deletion"
---

# Tasks: Word-wise navigation, selection, and deletion

**Input**: Design documents from `specs/032-word-wise-editing/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/behavior.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V. Tests first.

**Organization**: Setup → Foundational (`next_word_pos` + actions/bindings) → US1 move → US2 select →
US3 delete → Polish.

## Format: `[ID] [P?] [Story?] Description`

---

## Phase 1: Setup

- [X] T001 Confirm a clean baseline build on branch `032-word-wise-editing` (`make tmpfs-setup`, `make`).
- [X] T002 Register a new integration test target `word_editing` in `Cargo.toml`; create `tests/integration/word_editing.rs`.

---

## Phase 2: Foundational — boundary helper + actions/bindings (blocking)

- [X] T003 [P] Unit tests in `src/app.rs`: `next_word_pos` over `"foo  bar_baz, café"` and multibyte — mid-word and in-whitespace targets, at end-of-line (→ next line col 0), at column 0 (→ prev line end), at buffer start/end (no-op).
- [X] T004 In `src/app.rs`, add `next_word_pos(&self, dir) -> (usize, usize)` using `grapheme_class` per the algorithm (right = current run + following whitespace; left = preceding whitespace + token run; line-crossing; clamp). (FR-001, FR-002)
- [X] T005 [P] Unit tests in `src/input/mod.rs`: `default_map` maps Ctrl+Left→MoveWordLeft, Ctrl+Right→MoveWordRight, Ctrl+Shift+Left→SelectWordLeft, Ctrl+Shift+Right→SelectWordRight, Ctrl+Backspace→DeleteWordLeft, Ctrl+Delete→DeleteWordRight; existing F-keys/Ctrl bindings unchanged.
- [X] T006 In `src/input/keymap.rs`, add the six `Action` variants, their `action_from_str` arms, and the six `default_map` bindings (no conflict with existing). (FR-007)

**Checkpoint**: helper + bindings proven; `make check` green.

---

## Phase 3: US1 — move by word (Priority: P1) 🎯

- [X] T007 [P] [US1] Unit test in `src/app.rs`: `move_word(dir)` moves to `next_word_pos` and clears any selection; line-crossing works; buffer ends no-op.
- [X] T008 [US1] In `src/app.rs`, add `move_word(dir)` (clear selection + `set_cursor_lc(next_word_pos)`) and dispatch `MoveWordLeft`/`MoveWordRight` in `handle_action`. (FR-001, FR-002, FR-003)
- [X] T009 [P] [US1] Integration test in `tests/integration/word_editing.rs`: from a known position, `MoveWordRight`/`MoveWordLeft` land the cursor at the expected `(line, gcol)` incl. across a line boundary.

**Checkpoint**: word movement works; `make check` green.

---

## Phase 4: US2 — select by word (Priority: P2)

- [X] T010 [P] [US2] Unit test in `src/app.rs`: `move_word_selecting(dir)` builds the expected selection (anchor preserved across multiple steps); `selection_text()` returns the spanned words.
- [X] T011 [US2] In `src/app.rs`, add `move_word_selecting(dir)` (anchor + `set_cursor_lc(next_word_pos)` + `update_selection_to_cursor`) and dispatch `SelectWordLeft`/`SelectWordRight`. (FR-004)
- [X] T012 [P] [US2] Integration test in `tests/integration/word_editing.rs`: `SelectWordRight` ×2 selects two words; Copy (via `selection_text`) returns exactly them.

**Checkpoint**: word selection works; `make check` green.

---

## Phase 5: US3 — delete by word (Priority: P1)

- [X] T013 [P] [US3] Unit tests in `src/app.rs`: `delete_word(dir)` removes the expected range in one undo step (undo restores text + cursor); deletes an active selection instead when present; no-op at buffer ends; read-only → no change + message.
- [X] T014 [US3] In `src/app.rs`, add `delete_word(dir)` (read-only guard; selection→`delete_selection`; else span cursor↔`next_word_pos` and `delete_selection`) and dispatch `DeleteWordLeft`/`DeleteWordRight`. (FR-005, FR-006)
- [X] T015 [P] [US3] Integration test in `tests/integration/word_editing.rs`: `DeleteWordLeft`/`DeleteWordRight` remove the expected token; `Undo` restores it.

**Checkpoint**: word deletion works; `make check` green.

---

## Phase 6: Polish & cross-cutting

- [X] T016 [P] Update `CHANGELOG.md` (feature 032 under `[Unreleased]`), `docs/STATUS.md`, `docs/CAPABILITIES.md` (the six word-wise keybindings in the Editing/Selection tables).
- [X] T017 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check); fix findings; note the pre-existing F12/Ctrl+O PTY smoke failure is not a regression.
- [X] T018 Run the `specs/032-word-wise-editing/quickstart.md` manual walkthrough.

---

## Dependencies & Execution Order

- Setup → none. Foundational (`next_word_pos` + actions/bindings) blocks US1/US2/US3. US1 is the MVP;
  US2/US3 reuse the helper and existing selection/delete paths and are independent. Polish after.

### Parallel opportunities

- All `[P]` unit/integration tests; T016 docs `[P]`.

## Implementation Strategy

TDD per story (Constitution V). No new crates (IV). Reuse `grapheme_class`, `move_cursor`/
`move_cursor_selecting` patterns, and the char-safe `delete_selection` (one undo step). Branch
`032-word-wise-editing`, PR to `master`, merge via GitHub. No AI attribution.
