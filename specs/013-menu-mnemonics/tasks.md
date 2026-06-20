---
description: "Task list for feature 013: DOS-style menu mnemonic accelerators"
---

# Tasks: DOS-style menu mnemonic accelerators

**Input**: Design documents from `specs/013-menu-mnemonics/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/menu-mnemonics.md

**Tests**: INCLUDED — Constitution Principle V (Test-Gated Merges) is NON-NEGOTIABLE and TDD is
mandatory. Every behavior task is preceded by its test task.

**Organization**: Grouped by user story (US1–US4) so each is an independently testable increment.

## Path Conventions

Single Rust project: `src/` and `tests/` at repository root. Primary files:
`src/ui/menubar.rs`, `src/app.rs`, `src/input/mod.rs`.

---

## Phase 1: Setup

- [ ] T001 Create the integration test file `tests/integration/menu_mnemonics.rs` with module
  scaffolding (imports of `edit::app::App`, `edit::ui::menubar`, `edit::input`) and register it in
  `tests/integration/` (mirror the harness used by `tests/integration/menu_activation.rs`).
- [ ] T002 Create the smoke test stub `tests/smoke/menu_mnemonics.exp` (expect script skeleton based
  on `tests/smoke/file_browser.exp`) — body filled in T026.

---

## Phase 2: Foundational (blocking prerequisites for ALL user stories)

**Purpose**: the shared mnemonic data model + assignment + derivation helpers. No user story can be
implemented or tested until these exist.

- [ ] T003 [P] Unit tests in `src/ui/menubar.rs` (`#[cfg(test)]`) for `auto_mnemonic(label, &mut used)`:
  returns first free alphanumeric lowercase letter; skips used letters; returns `None` when all
  candidate letters are taken or label has no letters; is deterministic; UTF-8 safe (e.g. accented /
  wide label).
- [ ] T004 [P] Unit tests in `src/ui/menubar.rs` for `underline_col(label, mnemonic)`: returns the
  display-column of the FIRST char whose lowercase equals `mnemonic`; `None` when `mnemonic` is `None`
  or absent; correct column for a label containing a leading wide character (no split).
- [ ] T005 [P] Unit tests in `src/ui/menubar.rs` for built-in authored mnemonics: every built-in item
  has `Some` mnemonic; mnemonics are unique within each menu (case-insensitive); they equal the R4
  table (New→n, Open→o, Save→s, Save As→a, Save As Encoding→e, Exit→x, Edit/Search/View/Options/Help
  sets).
- [ ] T006 Add `mnemonic: Option<char>` to the static `MenuItem` struct and populate every entry in
  `FILE_MENU`/`EDIT_MENU`/`SEARCH_MENU`/`VIEW_MENU`/`OPTIONS_MENU`/`HELP_MENU` per the R4 table in
  `src/ui/menubar.rs` (canonical lowercase).
- [ ] T007 Add `mnemonic: Option<char>` to `ResolvedItem` and `ResolvedMenu` in `src/ui/menubar.rs`.
- [ ] T008 Implement `auto_mnemonic(label: &str, used: &mut std::collections::HashSet<char>) ->
  Option<char>` in `src/ui/menubar.rs` per data-model.md.
- [ ] T009 Implement `underline_col(label: &str, mnemonic: Option<char>) -> Option<u16>` in
  `src/ui/menubar.rs` using the existing `grapheme_width`/char-width logic.
- [ ] T010 Update `resolve_menus` in `src/ui/menubar.rs` to populate mnemonics: built-in top-level =
  first letter (f/e/s/v/o/h); built-in items copy `MenuItem.mnemonic`; plugin items merged into a
  built-in menu auto-assign seeded with that menu's existing item letters; new plugin top-level menus
  auto-assign seeded with all top-level letters, items from a fresh per-menu set.
- [ ] T011 Update all existing `ResolvedItem`/`ResolvedMenu` literal constructions (tests + non-test
  code, e.g. the empty-plugin-menu test, `render_into_with_menus` helpers) to include the new
  `mnemonic` field so the crate compiles.

**Checkpoint**: `cargo test --lib menubar` passes (T003–T005 green); crate builds.

---

## Phase 3: User Story 1 — See which key activates each menu and item (Priority: P1)

**Goal**: every top-level menu title and dropdown item renders exactly one underlined accelerator.

**Independent test**: render the bar and a dropdown into a ratatui `Buffer`; assert the accelerator
cell carries `Modifier::UNDERLINED` and no other cell of that label does.

- [ ] T012 [P] [US1] Unit test in `src/ui/menubar.rs`: rendering the bar underlines exactly the
  first letter of each top-level title (F/E/S/V/O/H) and nothing else on the bar row.
- [ ] T013 [P] [US1] Unit test in `src/ui/menubar.rs`: with the File dropdown open, each item row has
  `UNDERLINED` on exactly its accelerator column (e.g. "New"→col of N) and not elsewhere; verify a
  multi-word item (Save As → underline on A) and that the check-mark column/label alignment from
  feature 006 is unchanged.
- [ ] T014 [US1] In `MenuBarWidget::render` (`src/ui/menubar.rs`), add `Modifier::UNDERLINED` to the
  accelerator cell when drawing each top-level label, using `underline_col` + the menu's `mnemonic`.
- [ ] T015 [US1] In `MenuBarWidget::render`, add `Modifier::UNDERLINED` to the accelerator cell when
  drawing each dropdown item label, using `underline_col` + the item's `mnemonic`; preserve selected/
  unselected styles and the check-mark prefix.

**Checkpoint**: US1 independently verifiable — underlines visible; all prior tests green.

---

## Phase 4: User Story 2 — Activate a dropdown item by its accelerator letter (Priority: P1)

**Goal**: with a dropdown open, pressing an item's letter runs its action and closes the menu; a
non-matching letter is an inert no-op that keeps the menu open.

**Independent test**: open File via `App`, send `InsertChar('n')`, assert a new buffer was created and
the menu closed; send a non-matching letter, assert the menu stays open and the buffer is unchanged.

- [ ] T016 [P] [US2] Unit test in `src/ui/menubar.rs` for `select_item_by_mnemonic(menus, ch)`:
  returns the matching item's `Action` and sets state `Inactive`; case-insensitive; returns `None`
  and leaves state unchanged when no item matches; `None` when not in `DropDown`.
- [ ] T017 [US2] Implement `MenuBarState::select_item_by_mnemonic(&mut self, menus, ch) ->
  Option<Action>` in `src/ui/menubar.rs` per data-model.md.
- [ ] T018 [US2] In the menu-active keyboard intercept in `src/app.rs` (`handle_action`), add an
  `Action::InsertChar(c)` arm: when state is `DropDown`, call `select_item_by_mnemonic`; if `Some`,
  dispatch the returned action (`return self.handle_action(act)`); otherwise consume (no-op, menu
  stays open). Must not fall through to buffer editing (FR-007/FR-011).
- [ ] T019 [P] [US2] Integration test in `tests/integration/menu_mnemonics.rs`: open each built-in
  menu via the existing open path, press each item's accelerator, assert the corresponding action ran
  (e.g. File→`n` adds a buffer; View→`w` toggles soft wrap) and the menu closed; that a
  non-accelerator letter leaves the menu open with the buffer untouched; and (FR-011 no-regression)
  that with NO menu active, `InsertChar('x')` inserts `x` into the buffer as before.

**Checkpoint**: US2 independently verifiable — letter activation works end-to-end.

---

## Phase 5: User Story 3 — Open top-level by Alt+letter / bare Alt (Priority: P2)

**Goal**: the underlined top-level letter opens its menu when the bar is active; `Alt+letter` opens it
from editing and equals the shown letter; tapping `Alt` alone activates the bar (best-effort).

**Independent test**: `F10` then `e` opens Edit; `Alt+V` opens View; a synthesized lone-`Alt`
`KeyEvent` dispatches to `Action::Menu`.

- [ ] T020 [P] [US3] Unit test in `src/ui/menubar.rs` for `open_menu_by_mnemonic(menus, ch)`: from
  `TopActive`, a matching top-level letter switches to that menu's `DropDown{item_idx:0}` and returns
  `true`; non-match returns `false` and leaves state unchanged.
- [ ] T021 [US3] Implement `MenuBarState::open_menu_by_mnemonic(&mut self, menus, ch) -> bool` and
  extend the `src/app.rs` `InsertChar(c)` intercept so that in `TopActive` it calls this (consume
  regardless of match).
- [ ] T022 [P] [US3] Unit test in `src/input/mod.rs`: `dispatch_key` maps a `Press` of
  `KeyCode::Modifier(ModifierKeyCode::LeftAlt)` and `RightAlt` to `Action::Menu`; a `Release` of the
  same yields `None`; assert `Alt+F` still maps to `Action::MenuFile` (no regression).
- [ ] T022a [P] [US3] Unit test in `src/ui/menubar.rs` (FR-005/SC-002 consistency): for each built-in
  top-level menu, its resolved `mnemonic` equals the letter of its `Alt+<letter>` opener (File→f,
  Edit→e, Search→s, View→v, Options→o, Help→h) — the underlined letter and the opening key never drift.
- [ ] T023 [US3] In `src/input/mod.rs` `dispatch_key`, add the lone-`Alt` → `Action::Menu` mapping
  (before the char fallback), guarded to `KeyEventKind::Press`/`Repeat` only.
- [ ] T024 [US3] In `src/app.rs` terminal init/teardown, if `supports_keyboard_enhancement()` is
  `Ok(true)` push `PushKeyboardEnhancementFlags(REPORT_ALL_KEYS_AS_ESCAPE_CODES)` after
  `EnterAlternateScreen`, and `PopKeyboardEnhancementFlags` before `LeaveAlternateScreen`; ignore
  errors so unsupported terminals are unaffected (graceful degradation, R2).

**Checkpoint**: US3 verifiable — Alt+letter consistent with underline; lone-Alt mapping unit-tested.

---

## Phase 6: User Story 4 — Plugin menu items get accelerators automatically (Priority: P3)

**Goal**: plugin items and plugin top-level menus receive unique auto-assigned accelerators that do
not collide with built-ins, and are letter-activatable.

**Independent test**: resolve menus with a synthetic plugin contributing two items into a built-in
menu and a new top-level menu; assert each has a unique `Some` mnemonic distinct from built-ins, and
letter activation runs the plugin action.

- [ ] T025 [P] [US4] Unit tests in `src/ui/menubar.rs`: (a) two plugin items merged into Edit get
  unique mnemonics that avoid Edit's built-in letters; (b) a new plugin top-level menu gets a letter
  not in {f,e,s,v,o,h}; (c) a plugin item whose every letter is already used resolves to
  `mnemonic == None` (FR-006); (d) wide-character plugin label assigns a sensible letter or `None`
  without panic.
- [ ] T026 [US4] Integration test in `tests/integration/menu_mnemonics.rs`: build `App` resolved
  menus including a synthetic plugin menu; open it and activate a plugin item by its assigned letter;
  assert the plugin `Action::PluginMenuActivated(..)` path is taken (status message set / no crash).

**Checkpoint**: US4 verifiable — plugin mnemonics unique and activatable.

---

## Phase 7: Polish & Cross-Cutting

- [ ] T027 Fill in `tests/smoke/menu_mnemonics.exp`: launch editor, `F10`, open File, press an item
  letter, then `Ctrl+Q`; assert clean exit 0 (mirror `tests/smoke/file_browser.exp` structure).
- [ ] T028 [P] Update `CHANGELOG.md` under `[Unreleased]` with a `feature 013` entry (Added:
  underlined accelerators on all menus/items; letter activation in dropdowns; bare-Alt activates bar
  best-effort).
- [ ] T029 [P] Update `docs/STATUS.md` with feature-013 user-story rows (US1–US4, Complete).
- [ ] T030 [P] Update `docs/CAPABILITIES.md`: Menu keybindings section — note underlined accelerators,
  letter-to-activate inside an open menu, and bare-Alt (terminal-permitting) bar activation.
- [ ] T031 Run the full local gate: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, `expect tests/smoke/menu_mnemonics.exp`; fix any failures. Then live-verify in tmux
  per `quickstart.md` (underlines visible; `n` opens a new buffer from the File menu).

---

## Dependencies & Execution Order

- **Setup (T001–T002)** → no deps.
- **Foundational (T003–T011)** → blocks all user stories. T006/T007 before T010/T011; T008/T009 before
  T010 and before the render tasks. Tests T003–T005 are written first (TDD) and pass after T006–T010.
- **US1 (T012–T015)** depends on Foundational (T009 underline helper, T007/T010 mnemonic fields).
- **US2 (T016–T019)** depends on Foundational; independent of US1 (logic vs. render) but both ship P1.
- **US3 (T020–T024)** depends on Foundational + US2's `InsertChar` intercept (T018) for the TopActive
  extension (T021).
- **US4 (T025–T026)** depends on Foundational (T010 assignment); independent of US1–US3 rendering.
- **Polish (T027–T031)** after all stories.

## Parallel Opportunities

- T003, T004, T005 (distinct test fns, same file — author together) in Foundational.
- T012 & T013 (US1 render tests), T016 (US2 unit), T020 (US3 unit), T022 (input unit), T025 (US4 unit)
  are independent test authoring tasks.
- T028, T029, T030 (three different docs files) run in parallel.

## Implementation Strategy

- **MVP = Foundational + US1 + US2** (P1): visible underlined accelerators and working letter
  activation inside open menus — the core of the request, fully testable.
- **Increment 2 = US3** (P2): Alt+letter/underline consistency + bare-Alt convenience.
- **Increment 3 = US4** (P3): plugin auto-mnemonics.
- Each phase ends green before the next begins (test-gated).
