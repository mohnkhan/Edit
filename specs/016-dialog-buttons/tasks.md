---
description: "Task list for feature 016: Focusable dialog buttons (borders, tab order, mouse)"
---

# Tasks: Focusable dialog buttons

**Input**: Design documents from `specs/016-dialog-buttons/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/dialog-buttons.md

**Tests**: INCLUDED — Constitution Principle V; TDD mandatory.

**Organization**: by user story (US1 mouse, US2 boxed+focus render, US3 tab order). The shared component
underpins all three; dialogs are integrated uniformly.

## Path Conventions

Single Rust project. Primary files: `src/ui/buttons.rs` (new), `src/ui/mod.rs`, `src/app.rs`,
`src/input/keymap.rs`.

---

## Phase 1: Setup

- [x] T001 Create `src/ui/buttons.rs`, declare `pub mod buttons;` in `src/ui/mod.rs`, and create the
  integration test file `tests/integration/dialog_buttons.rs` registered as a `[[test]]` in `Cargo.toml`.

---

## Phase 2: Foundational (shared boxed-button component)

- [x] T002 [P] Unit tests in `src/ui/buttons.rs`: `button_rects` centers a row of 3-row boxes with
  correct widths (label+4) and gaps; total width clamps without panic on a tiny `area`; wide-char label
  width is correct. `hit_test_buttons` returns the right index inside a box and `None` outside. `next`/
  `prev` wrap.
- [x] T003 Implement `button_rects(area, labels) -> Vec<Rect>`, `hit_test_buttons(rects, col, row) ->
  Option<usize>`, `next`/`prev`, and `render_buttons(buf, rects, labels, focused, theme)` (boxed; focused
  inverted + `▶` marker) in `src/ui/buttons.rs`.
- [x] T004 Add `dialog_focus: usize` to `App` (init 0); add keymap `"BackTab" -> Action::FocusPrevField`
  and the `Action::FocusPrevField` variant + string parse arm (mirror of `FocusNextField`); add inert
  no-op arm for `FocusPrevField` in the main action match.

**Checkpoint**: `cargo test --lib buttons` passes; crate builds.

---

## Phase 3: User Story 2 — Boxed buttons with focus (render) (Priority: P1)

- [x] T005 [US2] In `src/app.rs` add `dialog_button_labels(&self) -> Vec<&'static str>`,
  `dialog_default_focus(&self) -> usize`, `dialog_supports_outside_cancel(&self) -> bool`, and
  `activate_dialog_button(&mut self, idx)` covering the in-scope dialogs (save prompt, session restore,
  revert, external change, plugin consent, Help/About, encoding select, plugin manager), each mapping to
  the existing handler. Set `dialog_focus = dialog_default_focus()` wherever each dialog is opened.
- [x] T006 [US2] Render a boxed button row (via `buttons::render_buttons`) in each in-scope dialog's
  overlay in `src/ui/mod.rs`, growing each dialog's height by ~3 rows to fit. List dialogs draw the row
  below their list.
- [x] T007 [P] [US2] Unit test (TestBackend) asserting a representative dialog (e.g. save prompt) renders
  3 boxed buttons with exactly one focused-styled.

**Checkpoint**: US2 — dialogs show boxed buttons, one focused.

---

## Phase 4: User Story 3 — Tab order + keyboard activation (Priority: P1)

- [x] T008 [US3] In each in-scope modal guard in `handle_action`: `FocusNextField`→`dialog_focus =
  buttons::next(dialog_focus, n)`, `FocusPrevField`→`prev`, `InsertNewline`/`InsertChar(' ')`→
  `activate_dialog_button(self.dialog_focus)` (exclude Space in plugin-manager). Keep existing letter
  shortcuts, list Up/Down, and Esc.
- [x] T009 [P] [US3] Integration tests in `tests/integration/dialog_buttons.rs`: open the unsaved-changes
  prompt, `Tab` cycles focus through Save/Discard/Cancel and wraps; `Enter` on the focused button runs
  that action; the S/D/C letter shortcuts still work; default focus is Cancel.

**Checkpoint**: US3 — every in-scope dialog button is Tab-reachable and Enter-activatable.

---

## Phase 5: User Story 1 — Mouse activation (Priority: P1)

- [x] T010 [US1] In `App::handle_mouse_event`, replace the "ignore all modal mouse" early-return: when an
  in-scope dialog is open, compute `button_rects` for the current frame and `hit_test_buttons`; a hit →
  `activate_dialog_button(i)`; outside the dialog box → cancel if `dialog_supports_outside_cancel`, else
  inert; inside-not-on-button → inert. Leave file-browser/menu paths unchanged.
- [x] T011 [P] [US1] Integration tests: a click on the Discard button (at its drawn rect) discards; a
  click on Cancel cancels; a click outside cancels a cancelable dialog; a click inside-not-on-a-button is
  inert.

**Checkpoint**: US1 — dialogs are mouse-navigable.

---

## Phase 6: Polish & Cross-Cutting

- [x] T012 File the deferral: GitHub `follow-up` issue for boxed buttons on the Find/Replace dialog and
  the file browser (both already navigable), and add a `ROADMAP.md` row referencing it.
- [x] T013 [P] Update `CHANGELOG.md` (feature 016) and `docs/STATUS.md` (F016 rows) and
  `docs/CAPABILITIES.md` (dialogs: boxed buttons, Tab/Shift+Tab, Enter/Space, mouse click, outside-cancel).
- [x] T014 [P] Add `Learnings.md` note if anything surprising surfaces (e.g. Tab disambiguation).
- [x] T015 Run the full gate: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`;
  live-verify in tmux per `quickstart.md` (click + Tab a confirm dialog; mouse-pick an encoding).

---

## Dependencies & Execution Order

- Setup (T001) → Foundational (T002–T004) blocks all stories.
- US2 render (T005–T007) → US3 keyboard (T008–T009) → US1 mouse (T010–T011) (mouse reuses the same
  button labels/activation + geometry).
- Polish (T012–T015) last; T012 (deferral) MUST be filed before merge.

## Parallel Opportunities

- T002 (unit), T007 (render unit), T009/T011 (integration), T013 (docs).

## Implementation Strategy

- **MVP = Foundational + US2 + US3** for the confirm/dismiss dialogs: visible boxed buttons + keyboard
  tab/activate. Then US1 (mouse) and the list dialogs. Deferrals filed (T012). Test-gated per phase.
