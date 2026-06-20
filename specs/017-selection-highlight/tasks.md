---
description: "Task list for feature 017: Visible text selection (highlight, Shift-select, mouse-drag)"
---

# Tasks: Visible text selection

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/selection.md
**Tests**: INCLUDED (Principle V, TDD). **Stories**: US1 highlight, US2 shift-select, US3 mouse-drag.

## Phase 1: Setup
- [x] T001 Create `tests/integration/selection.rs` and register a `[[test]]` in `Cargo.toml`.

## Phase 2: Foundational (selection range helper)
- [x] T002 [P] Unit test in `src/buffer/mod.rs` (or `src/app.rs`) for `selection_ordered`: orders
  anchor/active (handles reverse); empty when equal.
- [x] T003 Add a helper to order a selection into `((line,gcol) start, end)` reused by render + tests.

## Phase 3: US1 — Highlight (Priority: P1)
- [x] T004 [US1] In `src/ui/editor.rs`, overlay `Modifier::REVERSED` on cells whose `(line, gcol)` is in
  the ordered selection range, in BOTH the plain and soft-wrap render paths (after syntax/match styles,
  before the cursor cell). Read `buffer.selection`.
- [x] T005 [P] [US1] Unit test (TestBackend): after `select_all`, rendered buffer cells carry the
  reverse modifier; with no selection, none do; a 1-char selection highlights exactly one cell.

## Phase 4: US2 — Shift-select + copy (Priority: P1)
- [x] T006 [US2] Add `Action::Select{Left,Right,Up,Down,LineStart,LineEnd}` + keymap bindings
  `Shift+Left/Right/Up/Down/Home/End` in `src/input/keymap.rs`; main-match arms calling the handlers.
- [x] T007 [US2] Implement `move_cursor_selecting(dir)` and `select_line_start/end()` in `src/app.rs`
  (anchor on first shifted move; set active; clear if empty). Ensure plain `move_cursor` and edits clear
  the selection.
- [x] T008 [P] [US2] Integration test in `tests/integration/selection.rs`: Shift+Right ×3 selects 3
  chars; `Copy` yields those 3; a plain Move clears the selection; typing replaces a selection.

## Phase 5: US3 — Mouse drag (Priority: P2)
- [x] T009 [US3] In `src/app.rs::handle_mouse_event`, on left Press in the editor set cursor + anchor
  (selection None); on Drag set cursor + `selection = Some({anchor, active})`; single click (no drag)
  clears selection. Leave menu/dialog/file-browser paths unchanged.
- [x] T010 [P] [US3] Integration test: a Press then Drag across cells produces a selection covering that
  range; a Press+release at one cell clears the selection.

## Phase 6: Polish
- [x] T011 [P] Update `CHANGELOG.md` (feature 017), `docs/STATUS.md` (F017 rows), `docs/CAPABILITIES.md`
  (Selection keys + mouse drag).
- [x] T012 Run the gate: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; live
  tmux check (Ctrl+A highlights; Shift+Right selects).

## Dependencies
- Setup → Foundational → US1 → US2 → US3 → Polish. US1 (render) is independent of US2/US3 input.

## Strategy
- MVP = Foundational + US1 + US2 (visible selection + keyboard select + copy). Then US3 (mouse drag).
