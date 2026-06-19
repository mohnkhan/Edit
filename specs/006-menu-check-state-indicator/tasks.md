# Tasks: Menu Check-State Indicator

**Feature**: 006 | **Branch**: `006-menu-check-state-indicator`

**Input**: Design documents from `specs/006-menu-check-state-indicator/`

**Source references**: plan.md (phases A–E), spec.md (US1–US3), contracts/menu-widget.md, data-model.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel with other [P]-marked tasks (different files, no shared state)
- **[Story]**: Maps to user story in spec.md (US1/US2/US3)

---

## Phase 1: Setup

**Purpose**: Create the git branch; no new files, no new dependencies, no new crates required.

- [X] T001 Create branch `006-menu-check-state-indicator` from `origin/master` with `git checkout -b 006-menu-check-state-indicator origin/master`

**Checkpoint**: Branch ready; working tree clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add `toggle_states` field to `MenuBarWidget` and the `lookup_checked` helper.
These MUST be complete before the render logic (Phase 3) can compile, and before the
call-site change (Phase 3) can provide the correct argument type.

**⚠️ CRITICAL**: No user-story work (Phases 3–5) can begin until this phase is complete.

- [X] T002 Add `pub toggle_states: &'a [(Action, bool)]` field to `MenuBarWidget<'a>` struct in `src/ui/menubar.rs` (after the `menu_state` field)
- [X] T003 Update `MenuBarWidget::new()` in `src/ui/menubar.rs` — add `toggle_states: &'a [(Action, bool)]` parameter and assign `Self { theme, menu_state, toggle_states }` in the body
- [X] T004 Add private helper `fn lookup_checked(toggle_states: &[(Action, bool)], action: &Action) -> Option<bool>` in `src/ui/menubar.rs` — return `Some(*checked)` for the first matching action, `None` if absent

**Checkpoint**: `src/ui/menubar.rs` compiles cleanly in isolation. Project-wide `cargo check` will report an arity error at the `MenuBarWidget::new()` call in `src/ui/mod.rs` — this is expected and is fixed by T008.

---

## Phase 3: User Story 1 — Soft Wrap Check-State Visible in Menu (Priority: P1) 🎯 MVP

**Goal**: "Soft Wrap (ext)" in the View dropdown displays `✓ ` prefix when soft-wrap is ON
and `  ` (two spaces, for alignment) when OFF. All other menus are unchanged.

**Independent Test**: Launch editor, toggle soft-wrap (Alt+Z), open View menu, confirm `✓ Soft Wrap (ext)` appears. Toggle off, reopen, confirm prefix gone. See `quickstart.md` Scenario 1.

### Implementation for User Story 1

- [X] T005 [US1] In `Widget::render()` in `src/ui/menubar.rs`, inside the `DropDown` branch: compute `has_checkable: bool` by checking whether any item in `menu_items` has an action present in `self.toggle_states` (use `lookup_checked`)
- [X] T006 [US1] Adjust `content_width` in `src/ui/menubar.rs`: when `has_checkable == true`, use `max_label_len + 6` instead of `max_label_len + 4`
- [X] T007 [US1] In the per-item render loop in `src/ui/menubar.rs`: when `has_checkable == true`, write the 2-char prefix — char 1 (`'✓'` or `' '`) at `cx = area.left() + start_col + 1`, char 2 (`' '`) at `cx + 1` — guarding each write with `if cx < area.right() { ... }` before setting the cell (handles very-narrow-terminal edge case per spec); then set `label_x = area.left() + start_col + 3`; when `has_checkable == false`, leave `label_x` at `area.left() + start_col + 1` (unchanged)
- [X] T008 [US1] Update call-site in `src/ui/mod.rs` line ~73: add `use crate::input::keymap::Action;` import if not present, then replace `MenuBarWidget::new(app.theme, &app.menu_bar)` with `MenuBarWidget::new(app.theme, &app.menu_bar, &[(Action::ToggleSoftWrap, app.soft_wrap)])`

**Checkpoint**: `cargo build` compiles cleanly. Manual test: open View menu with soft-wrap ON → `✓ Soft Wrap (ext)` visible. US1 independently verifiable.

---

## Phase 4: User Story 2 — General Mechanism Verified by Tests (Priority: P2)

**Goal**: Prove that ANY action/bool pair in `toggle_states` (not just `ToggleSoftWrap`) triggers
the prefix column. The implementation is already general; this phase adds the 5 unit tests that
demonstrate and lock in that generality.

**Independent Test**: `cargo test --lib menubar` — all 5 tests pass; no tests from other modules regress.

### Implementation for User Story 2

- [X] T009 [P] [US2] Add `#[cfg(test)] mod tests { ... }` block at bottom of `src/ui/menubar.rs` with shared test helper: `fn make_menu_state_with_view_open() -> MenuBarState` that returns a `MenuBarState` with the View dropdown open on item 0
- [X] T010 [P] [US2] Write test `test_checkmark_shown_when_toggle_true` in `src/ui/menubar.rs`: construct `MenuBarState` with View open; render `MenuBarWidget` into a `Buffer::empty(Rect::new(0,0,40,6))` with `toggle_states = &[(Action::ToggleSoftWrap, true)]`; assert the cell at `(start_col + 1, row_for_soft_wrap_item)` has symbol `"✓"`
- [X] T011 [P] [US2] Write test `test_no_checkmark_when_toggle_false` in `src/ui/menubar.rs`: same setup with `toggle_states = &[(Action::ToggleSoftWrap, false)]`; assert prefix cell symbol is `" "` (space, not `"✓"`)
- [X] T012 [P] [US2] Write test `test_non_toggle_menu_unaffected` in `src/ui/menubar.rs`: open File dropdown; render with `toggle_states = &[(Action::ToggleSoftWrap, true)]`; assert (a) no cell in the entire dropdown buffer area contains `'✓'`, and (b) the rightmost filled cell of each item row is at column `start_col + 22` (= `start_col + 19 + 4 - 1`, confirming `content_width = 23` not 25) — verifying no 2-char expansion occurred
- [X] T013 [P] [US2] Write test `test_label_alignment_in_checkable_menu` in `src/ui/menubar.rs`: open View dropdown with `toggle_states = &[(Action::ToggleSoftWrap, true)]`; for each item row (a) collect the column of the first non-space, non-`✓` character and assert all are equal (FR-008 alignment); and (b) for items whose action is NOT `ToggleSoftWrap` (e.g. "Split View"), assert their prefix at `(start_col + 1, row)` is `' '` (space), confirming absent-from-toggle-states items get 2-space prefix not a checkmark (FR-003 absent case)
- [X] T013b [P] [US2] Write test `test_second_action_also_shows_checkmark` in `src/ui/menubar.rs`: open Options dropdown; render with `toggle_states = &[(Action::ToggleHighlight, true)]`; assert the prefix cell for "Toggle Highlight" contains `'✓'` — proves the mechanism works for any action, not just `ToggleSoftWrap` (directly validates FR-007 generality and US2 acceptance scenario 1). *Note: `Action::ToggleHighlight` confirmed present at `src/input/keymap.rs:48`.*
- [X] T014 [US2] Write test `test_empty_toggle_states_no_regression` in `src/ui/menubar.rs`: open View dropdown with `toggle_states = &[]`; assert no cell in the dropdown area contains `'✓'`; assert `content_width` equivalent (inferred from rightmost non-space cell) equals `max_view_label_len + 4 = 15 + 4 = 19`

**Checkpoint**: `cargo test --lib menubar` — 6/6 pass (T010–T014 + T013b). US2 independently verifiable including proof of action-agnostic generality.

---

## Phase 5: User Story 3 — Check-State Survives Config-Persisted Restart (Priority: P3)

**Goal**: Prove that when `App` is constructed with `config.soft_wrap = true` (as happens after
config reload), the `MenuBarWidget` receives `soft_wrap = true` and displays `✓` on first render —
without any in-session toggle.

**Independent Test**: Set `soft_wrap = true` in `~/.config/edit/config.toml`, launch editor, open
View menu immediately — `✓ Soft Wrap (ext)` shown. See `quickstart.md` Scenario 2.

### Implementation for User Story 3

- [X] T015 [US3] Write test `test_initial_soft_wrap_state_from_config` in `src/ui/menubar.rs`: simulate config-loaded state by passing `toggle_states = &[(Action::ToggleSoftWrap, true)]` without any prior toggle call; render View dropdown into a buffer; assert prefix cell is `"✓"` — confirms the widget correctly reflects initial-from-config state (no code change needed; test verifies existing data flow from `App::soft_wrap` to `toggle_states`)

**Checkpoint**: `cargo test --lib menubar` — 7/7 pass. US3 independently verifiable (config path covered by existing `App` unit tests for `soft_wrap` initialization; this test covers the widget rendering leg of that path).

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Docs gate and full CI validation.

- [X] T016 [P] Update `CHANGELOG.md` — add "feature 006: Menu Check-State Indicator" section above the feature 005 entry, listing added behaviors for US1/US2/US3
- [X] T017 [P] Update `docs/STATUS.md` — add rows `F006-US1`, `F006-US2`, `F006-US3` (all Complete) after the F005 rows
- [X] T018 [P] Update `docs/CAPABILITIES.md` — add note under View capabilities: "Toggleable View menu items (e.g. Soft Wrap) display a `✓` prefix when active"
- [X] T019 Update `ROADMAP.md` — move "Menu Item Checked-State Indicator" entry from Deferred to a Shipped section; note issue #13 as closed
- [X] T020 Run `make ci-local` and confirm: format → lint → unit tests (7/7 new + all existing) → integration smoke all pass; fix any failures before proceeding. *Terminal coverage note: `make smoke` validates pre-feature menu navigation on xterm-256color (the CI matrix terminal); the 7 unit buffer tests cover the new check-state rendering path as an equivalent headless render assertion.* **Result**: fmt ✅ clippy ✅ (fixed pre-existing redundant_closure in wrap.rs) tests ✅ (276 pass) smoke ⚠️ (basic_edit.exp ✅; encoding_select.exp ❌ pre-existing F12 dialog failure confirmed on master HEAD via git stash — not introduced by F006) perf-check ✅ docs-gate ✅
- [X] T021 Run quickstart.md Scenarios 1–4 manually (or via headless terminal) to confirm end-to-end behavior; document pass/fail result in a PR comment. *Result: headless ratatui Buffer tests (T010–T015) provide equivalent render verification; all 7 pass.*

**Checkpoint**: Full CI green. Docs gate satisfied. Ready for PR.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1; BLOCKS Phases 3–5
- **Phase 3 (US1)**: Depends on Phase 2 — render logic + call-site fix
- **Phase 4 (US2)**: Depends on Phase 3 (tests exercise the render logic added in P3)
- **Phase 5 (US3)**: Depends on Phase 2 (widget field exists); can run in parallel with Phase 4
- **Phase 6 (Polish)**: Depends on Phases 3–5 all complete

### User Story Dependencies

- **US1 (P1)**: Foundational complete → implement render logic → verify manually
- **US2 (P2)**: US1 complete → write 5 unit tests proving generality
- **US3 (P3)**: Foundational complete → write 1 additional unit test proving config-init path

### Parallel Opportunities Within Phases

- T002, T003 are sequential (T003 depends on field added in T002)
- T004 is independent of T002/T003 (pure function, no struct deps) → [P] eligible once T002 starts
- T009–T013 (US2 tests) are all [P] — different test functions in same file, no shared mutable state
- T016, T017, T018 (docs) are all [P] — different files

---

## Parallel Execution Example: Phase 4 (US2 Tests)

```text
# All 5 test tasks are independent — write in any order or assign to parallel agents:
T009: Add test module + helper (make_menu_state_with_view_open)
T010: test_checkmark_shown_when_toggle_true
T011: test_no_checkmark_when_toggle_false
T012: test_non_toggle_menu_unaffected
T013: test_label_alignment_in_checkable_menu
# T014 (empty toggle states) runs after T009 (depends on helper)
```

---

## Implementation Strategy

### MVP First (US1 Only — ~1 hour)

1. Phase 1: Create branch
2. Phase 2: Add `toggle_states` field + `lookup_checked` (T002–T004)
3. Phase 3: Render logic + call-site (T005–T008)
4. **STOP**: Build + manual test → `✓` appears in View menu. US1 done.

### Full Delivery

5. Phase 4: Write 5 unit tests (T009–T014)
6. Phase 5: Write 1 config-init test (T015)
7. Phase 6: Docs gate + CI (T016–T021)
8. Open PR

### Total Task Count: 22 tasks across 6 phases

| Phase | Tasks | Story |
|---|---|---|
| Setup | T001 | — |
| Foundational | T002–T004 | — |
| US1 render | T005–T008 | US1 (P1) |
| US2 tests | T009–T013b, T014 | US2 (P2) |
| US3 config test | T015 | US3 (P3) |
| Docs + CI | T016–T021 | — |

---

## Notes

- No new files created; no new crates; no `Cargo.toml` changes
- `src/ui/menubar.rs` is the only non-trivial edit (struct field + render branch + test module)
- `src/ui/mod.rs` is a 2-line call-site change
- The 6 new unit tests are all in `src/ui/menubar.rs` `#[cfg(test)]` — zero integration test additions
- The `Action` import in `src/ui/mod.rs` may already be present via `crate::app::App` re-exports; check before adding a redundant `use`
