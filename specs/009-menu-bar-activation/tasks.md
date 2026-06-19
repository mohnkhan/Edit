# Tasks: Live Menu-Bar Activation (Feature 009)

**Input**: Design documents from `specs/009-menu-bar-activation/`

**Prerequisites**: plan.md ✅ | spec.md ✅ | research.md ✅ | data-model.md ✅ | contracts/ ✅ | quickstart.md ✅

**Reuse note**: The feature-008 plugin engine is consumed unchanged — `PluginRegistry::menu_items()`,
`PluginHost::dispatch_menu_action()`, `Action::PluginMenuActivated`, consent flow, Plugins manager.
**No new `Action` variants, CLI flags, or config keys** are introduced.

**Organization**: Tasks grouped by user story (US1–US3 from spec.md). Per Constitution Principle V
(TDD), test tasks are ordered **before** the implementation they cover within each phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on incomplete tasks in same phase)
- **[Story]**: Maps to user story from spec.md (US1–US3)

---

## Phase 1: Setup

- [x] T001 Confirm the working branch is `009-menu-bar-activation` (branched from `origin/master`) and that `cargo build` succeeds on a clean checkout before any edits. No new dependencies are added in this feature.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The shared resolved-menu model that both rendering and navigation depend on. **No
user-story work begins until this phase passes `cargo test` and `cargo clippy -- -D warnings`.**

**⚠️ CRITICAL**: US1, US2, US3 all consume `resolve_menus()` and the model-aware `MenuBarState`.

- [x] T002 **Tests first** — add unit tests in `src/ui/menubar.rs` (`#[cfg(test)]`) for the model builder, written before T003/T004:
  - `test_resolve_menus_empty_matches_builtin` — `resolve_menus(&[])` yields exactly the 6 built-in menus (File, Edit, Search, View, Options, Help) with identical labels, order, and item labels/actions.
  - `test_resolve_menus_inserts_plugin_before_help` — one plugin item `menu="Tools"` produces a "Tools" `ResolvedMenu` at index `len-1` (immediately before Help, after Options).
  - `test_resolve_menus_merges_into_builtin_on_name_collision` — a plugin item with `menu="Edit"` appends a `ResolvedItem` to the Edit menu's items; the resolved list still has exactly one "Edit" and no extra top-level entry.
  - `test_resolve_menus_groups_multiple_plugins_same_menu` — two plugins both `menu="Tools"` produce a single "Tools" menu containing both items.
  - `test_resolve_menus_orders_by_position_then_load_order` — items with `position` set sort ascending; unset preserve load order.
  - `test_resolve_menus_widechar_plugin_label_preserved` — a plugin item with a multibyte/wide-character `menu`/`item` label resolves with the label intact (FR-014, Constitution II). [Remediation M2]

- [x] T003 Add the resolved-menu model to `src/ui/menubar.rs`: `pub struct ResolvedMenu { pub label: String, pub items: Vec<ResolvedItem> }` and `pub struct ResolvedItem { pub label: String, pub action: Action }` (derive `Debug, Clone, PartialEq`). Convert the existing static `MenuItem`/`ALL_MENUS` built-ins into `ResolvedItem`s when building.

- [x] T004 Implement `pub fn resolve_menus(plugin_items: &[crate::plugin::PluginMenuItem]) -> Vec<ResolvedMenu>` in `src/ui/menubar.rs` per `data-model.md`: seed from the 6 built-ins (in order), merge name-collision plugin items into the matching built-in dropdown, group remaining plugin items into new `ResolvedMenu`s inserted immediately before Help, ordering items by `position` then load order. `resolve_menus(&[])` MUST be byte-equal to the built-in set (parity invariant). Make T002 pass.

**Checkpoint**: `cargo test menubar` green; `cargo clippy -- -D warnings` clean; all pre-existing menu-bar geometry tests still pass unchanged.

---

## Phase 3: User Story 1 — Activate built-in menu items by keyboard (Priority: P1) 🎯 MVP

**Goal**: Open a built-in menu, navigate with arrows, activate with Enter, dismiss with Esc — by keyboard only.

**Independent Test**: With no plugins, open File, arrow to Save, Enter → buffer saved; open Edit, Esc → nothing changed, menu closed.

- [x] T005 [US1] **Tests first** — unit tests in `src/ui/menubar.rs` for model-aware `MenuBarState` navigation (written before T006/T007), using `resolve_menus(&[])`:
  - `test_navigate_down_wraps` / `test_navigate_up_wraps` over the resolved list.
  - `test_top_active_down_opens_first_item` / `test_top_active_up_opens_last_item`.
  - `test_navigate_left_right_wraps_over_ring` — Left from index 0 → last menu; Right from last → 0.
  - `test_navigate_left_right_opens_adjacent_dropdown` — from `DropDown`, Left/Right yields `DropDown` on the adjacent menu at item 0.
  - `test_navigate_left_right_top_active_moves_highlight_only` — from `TopActive`, Left/Right stays `TopActive` (no dropdown).
  - `test_activate_bar_enters_top_active` — `activate_bar()` from `Inactive` yields `TopActive(0)` (no dropdown). [Remediation H1]
  - `test_top_active_down_opens_dropdown` — from `TopActive(t)`, `navigate_down` opens `DropDown{t,0}` (makes FR-002 reachable). [Remediation H1]
  - `test_select_item_returns_builtin_action_and_closes` — `DropDown` on File→Save returns `Action::Save` and state becomes `Inactive`.
  - `test_select_item_returns_plugin_activated_action` — `DropDown` on a resolved plugin item returns `Action::PluginMenuActivated(plugin_id,item_id)` and closes. [Remediation M3]
  - `test_select_item_inactive_returns_none` and `test_select_item_top_active_returns_none` (Enter at `TopActive` is a no-op).
  - `test_open_menu_clamps_to_resolved_len`.

- [x] T006 [US1] Refactor `MenuBarState` methods in `src/ui/menubar.rs` to take `menus: &[ResolvedMenu]`: update `open_menu`, `navigate_up`, `navigate_down`, `select_item` to index the passed slice instead of `ALL_MENUS`; `select_item` returns the resolved `Action` and closes. Add `pub fn activate_bar(&mut self)` setting `state = TopActive(0)` (the F10 entry path; no model needed). Update the `open_menu` doc-comment to describe it as the Alt+letter direct-dropdown entry (not the only entry path). Keep `close_menu`/`is_active` as-is. All index math clamped/guarded (empty menu → no-op / `None`). [Covers remediation H1, L3]

- [x] T007 [US1] Add `pub fn navigate_left(&mut self, menus: &[ResolvedMenu])` and `pub fn navigate_right(&mut self, menus: &[ResolvedMenu])` to `MenuBarState` in `src/ui/menubar.rs`: ring traversal over all top-level menus; from `DropDown` open the adjacent menu's dropdown (item 0); from `TopActive` move highlight only; wrap at both ends. Make T005 pass.

- [x] T008 [US1] **Tests first** — integration tests in `tests/integration/menu_activation.rs` (new file). Integration tests are per-file in this repo, so **add a `[[test]] name="menu_activation" path="tests/integration/menu_activation.rs"` block to `Cargo.toml`** alongside the existing `encoding_roundtrip`/`file_io`/`recovery` entries. [Remediation M5] Tests:
  - `test_keyboard_open_navigate_activate_builtin_save` — build an `App` on a temp file; `handle_action(MenuFile)`, then `MoveDown` to Save, then `InsertNewline`; assert the file was written and `menu_bar` is `Inactive`.
  - `test_f10_enters_top_active_then_down_opens_dropdown` — `handle_action(Menu)` (F10) leaves the menu bar in `TopActive(0)` (no dropdown); a subsequent `MoveDown` opens File's dropdown. [Remediation H1]
  - `test_escape_closes_without_action` — open a menu, `MenuClose`; assert `Inactive` and buffer unchanged.
  - `test_navigation_does_not_mutate_buffer` — open a menu; send `MoveDown`/`MoveUp`/`MoveLeft`/`MoveRight`; assert buffer text and cursor (grapheme_col/line) unchanged.

- [x] T009 [US1] Wire the menu-active guard in `App::handle_action` (`src/app.rs`): after the existing `pending_*` modal guards and before the normal action match, add `if self.menu_bar.is_active() { ... }` that builds `let menus = resolve_menus(self.plugin_host.registry().menu_items().as_slice());` and routes: `MoveUp→navigate_up(&menus)`, `MoveDown→navigate_down(&menus)`, `MoveLeft→navigate_left(&menus)`, `MoveRight→navigate_right(&menus)`, `MenuClose→close_menu()`, `InsertNewline→` `if let Some(a)=select_item(&menus) { return self.handle_action(a); }`; any other action while active is consumed (return Ok). Change the `Action::Menu` (F10) arm to call `self.menu_bar.activate_bar()` (top-level highlight, no dropdown — remediation H1); keep `Action::MenuFile..MenuOpen` arms calling `open_menu` (Alt+letter opens dropdown directly). Make T008 pass. **Note**: the guard recomputes `menus` on every call, so plugins enabled/disabled mid-session are reflected (supports T014).

**Checkpoint**: US1 fully functional for built-in menus by keyboard; `cargo test` green; existing geometry tests unchanged.

---

## Phase 4: User Story 2 — Discover & activate plugin menus by keyboard (Priority: P2)

**Goal**: Plugin top-level menus render in the bar and their items activate via `Action::PluginMenuActivated`.

**Independent Test**: Pre-consent `word-count`; "Tools" menu appears; navigate to Word Count, Enter → status bar shows count.

- [x] T010 [US2] Render the resolved menus in `MenuBarWidget`: in `src/ui/mod.rs` build `let menus = resolve_menus(app.plugin_host.registry().menu_items().as_slice());` and pass it into the widget; in `src/ui/menubar.rs` change `MenuBarWidget` to render top-level labels and dropdowns from the provided `&[ResolvedMenu]` instead of static `ALL_MENUS`. The built-in-only case MUST reproduce the exact current top-level label columns as defined by the existing `BarLabel.col` table (0-based within bar content after the leading space: File@1, Edit@7, Search@13, View@21, Options@28, Help@37) and dropdown widths — verified by the unchanged geometry tests. [L2 wording]

- [x] T011 [US2] **Tests first** — integration tests in `tests/integration/menu_activation.rs`:
  - `test_plugin_menu_renders_between_options_and_help` — load the `word-count` fixture (pre-consented); assert `resolve_menus(...)` places "Tools" at index `len-2` (Help last).
  - `test_plugin_menu_keyboard_activation_sets_status` — open the Tools menu, navigate to "Word Count", `InsertNewline`; assert `app.status_message` contains the word count for a known buffer (e.g. 5 words → contains "5").
  - `test_no_plugins_menu_bar_unchanged` — with no active plugin menu items, `resolve_menus(...)` equals the built-in set (mirrors T002 at the app layer).
  - `test_disabled_plugin_contributes_no_menu` — a disabled plugin contributes no `ResolvedMenu`/items.
  - `test_no_plugins_flag_yields_no_plugin_menus` — an `App` constructed with `no_plugins=true` resolves to exactly the built-in menus (explicit `--no-plugins` coverage). [Remediation M4, FR-010]

- [x] T012 [US2] Confirm plugin-item selection routes correctly end-to-end: a `ResolvedItem` from a plugin carries `Action::PluginMenuActivated(plugin_id, item_id)`; the T009 guard's `select_item` path dispatches it through the existing `Action::PluginMenuActivated` arm in `src/app.rs` (which calls `dispatch_menu_action` and sets `status_message`). Add any missing glue only; do NOT modify `src/plugin/*`. Make T011 pass.

**Checkpoint**: Plugin menus visible and keyboard-activatable; `--no-plugins` and disabled plugins show no plugin menus.

---

## Phase 5: User Story 3 — Predictable DOS-faithful navigation semantics (Priority: P3)

**Goal**: Wrap-around everywhere; Left/Right span the full composite ring including plugin menus; no dead-ends.

**Independent Test**: Open rightmost menu, Right → wraps to leftmost; Up/Down wrap within plugin dropdowns too.

- [x] T013 [US3] **Tests** in `tests/integration/menu_activation.rs` (and/or `src/ui/menubar.rs` units) covering the full ring **with a plugin menu present**:
  - `test_left_right_ring_includes_plugin_menu` — with "Tools" inserted, Right from Options lands on Tools, Right again lands on Help, Right again wraps to File.
  - `test_updown_wrap_in_plugin_dropdown` — Up/Down wrap within the Tools dropdown.
  - `test_modal_precedence_over_menu` — when `pending_plugin_manager` (or another modal) is set, `MoveDown` is handled by the modal, not the menu bar (menu guard does not fire).
  - `test_empty_plugin_menu_not_openable` — a plugin menu resolved with zero items is a no-op on `navigate_down` from `TopActive` and `select_item` returns `None` (no panic).
  - `test_plugin_menu_dispatch_failure_surfaces_warning` — using a failing/looping menu-plugin fixture (e.g. `fs-violation` or a timeout fixture), open its menu and `InsertNewline`; assert the editor stays responsive, `status_message` holds a warning, the buffer is intact, and the plugin is disabled by the dispatch layer. [Remediation M1, FR-013 / SC-006]

- [x] T014 [US3] Fix any wrap/ring/precedence edge cases surfaced by T013 in `src/ui/menubar.rs` / `src/app.rs` (e.g. ensure the guard recomputes `menus` each call so a plugin enabled/disabled mid-session is reflected; ensure modal guards precede the menu guard). Make T013 pass.

**Checkpoint**: All navigation edge cases pass; `cargo test` fully green.

---

## Phase 6: Polish & Docs Gate

- [x] T015 [P] Write smoke test `tests/smoke/plugin_menu_activate.exp`: launch `./edit` on a temp buffer with the `word-count` fixture pre-consented (seed `plugins.toml` with `allowed=true`); drive keys to open the Tools menu and select Word Count; assert the status line shows a word count; exit cleanly. Wire into `make smoke` if the smoke harness enumerates by glob (verify it runs).

- [x] T016 [P] Update `CHANGELOG.md` — feature 009 entry: "Live menu-bar keyboard activation: arrow-key navigation within and between pull-down menus (built-in and plugin), Enter to activate, Esc to close; plugin-contributed top-level menus now render (between Options and Help) and activate via the keyboard. Completes the deferred portion of the Plugin API (issue #19)."

- [x] T017 [P] Update `docs/STATUS.md` — change F008-US3 from "Partial" to "Complete"; add an F009 row (or note) for menu-bar activation; remove/replace the "Known Limitations" line about deferred plugin menu activation.

- [x] T018 [P] Update `docs/CAPABILITIES.md` — document keyboard menu navigation (arrows/Enter/Esc) and plugin-provided top-level menus as a user-visible capability.

- [x] T019 [P] Update `man/edit.1` — note keyboard menu navigation keys and that plugin menus appear between Options and Help.

- [x] T020 [P] Update `ROADMAP.md` — change "Plugin top-level menu activation (follow-up to feature 008)" / issue #19 from "Deferred" to "Complete as of 2026-06-19 (feature 009)".

- [x] T021 [P] Mark feature-008 `specs/008-plugin-api/tasks.md` T025 as resolved by feature 009 (cross-reference), since the menu-bar rendering/activation it described is now implemented here.

- [x] T022 Verified the gate: `cargo fmt --check` clean; `cargo clippy --lib -- -D warnings` clean; `cargo test` = 321 lib + all integration suites green (incl. 12 new `menu_activation` tests); `cargo build --release` OK (no new deps → static link unaffected). Smoke: `plugin_menu_activate.exp` and all others PASS in a proper tty; the two failures observed in the headless sandbox (`menu_nav`, `encoding_select`) are environment artifacts (0×0 pty size + a stray `session.toml`), proven non-regressions by re-running with a pty size and clean state. No pre-existing menu-bar geometry test was modified (all 33 menubar unit tests pass unchanged).

- [ ] T023 Close GitHub issue #19 with a comment referencing the feature-009 PR (do this at PR-merge time, not before).

---

## Dependencies & Execution Order

- **Phase 1 (T001)** → no deps.
- **Phase 2 (T002–T004)** → Phase 1; **BLOCKS all user stories** (shared model). T002 before T003/T004 (TDD). T003 before T004.
- **Phase 3 / US1 (T005–T009)** → Phase 2. T005 before T006/T007; T008 before T009; T006/T007 before T009.
- **Phase 4 / US2 (T010–T012)** → Phase 2 + US1 navigation (T009). T011 before T012; T010 before T011.
- **Phase 5 / US3 (T013–T014)** → US1 + US2 (needs plugin menus present for ring tests). T013 before T014.
- **Phase 6 (T015–T023)** → Phases 3–5. T015–T021 are [P] (different files); T022 then T023 sequential and last.

### Parallel opportunities

- All Phase 6 docs tasks (T016–T021) are independent and parallelizable.
- T015 (smoke test) parallel with the docs tasks.
- Within Phase 2/3 the test task and impl task are sequential (TDD), not parallel.

---

## Implementation Strategy

### MVP (User Story 1)

1. Phase 1 (T001) → 2. Phase 2 (T002–T004, model + parity) → 3. Phase 3 (T005–T009, built-in keyboard activation) → 4. STOP & validate quickstart Scenario 1 + Scenario 3 (no-plugin parity).

### Incremental delivery

Setup+Foundational → US1 built-in activation (MVP, shippable on its own) → US2 plugin menus →
US3 navigation polish → Phase 6 smoke + docs gate.

---

## Notes

- TDD: each phase lists its test task(s) before implementation (Constitution V). `cargo test` +
  `cargo clippy -- -D warnings` MUST stay green after every task.
- **Regression guard**: pre-existing menu-bar geometry tests in `src/ui/menubar.rs` MUST pass
  unchanged. The no-plugin parity test (T002) plus those tests are the FR-011 / SC-003 gate.
- No changes to `src/plugin/*` or `src/input/keymap.rs` (no new `Action`). Reuse only.
- Docs gate (Constitution / CLAUDE.md): CHANGELOG.md + docs/STATUS.md required; CAPABILITIES.md
  because keybinding/menu behavior is user-visible.
- **SC-004 (≤50 ms latency)** [L1]: no dedicated perf task. Menu resolution is an in-memory build
  over ≤ ~11 menus per keystroke — negligible; it is covered by the existing `make perf-check`
  keystroke-latency budget (T022). No new perf harness is warranted.
