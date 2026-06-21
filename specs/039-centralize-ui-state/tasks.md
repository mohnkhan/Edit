# Tasks: Centralize Editor UI State

**Feature**: `039-centralize-ui-state` | **Branch**: `039-centralize-ui-state`
**Input**: [plan.md](./plan.md), [spec.md](./spec.md), [data-model.md](./data-model.md),
[contracts/internal-api.md](./contracts/internal-api.md), [quickstart.md](./quickstart.md)

**Field count note**: "~14 modal flags" in the spec = 13 folded overlay fields **+** the dead
`menu_active` flag removed; sub-state fields (`*_caret`, `help_scroll`, `*_cursor`) fold into their
owning variant. See [data-model.md](./data-model.md) for the exact field→variant table.

**Overriding constraint (applies to every task)**: This is a **behavior-preserving** refactor. No
existing test's asserted behavior may change. The ONLY permitted test edits are mechanical
field→accessor renames (e.g. `app.pending_find_replace.is_some()` → `app.find_replace().is_some()` or
`matches!(app.modal(), Modal::FindReplace(_))`). If an assertion's expected value must change to make a
test pass, STOP — the change altered behavior and is a defect (FR-009, FR-010, SC-001).

**Story → phase map**: US1 = Phase 1 (Modal enum), US2 = Phase 2 (layer precedence),
US3 = Phase 3 (geometry hygiene). Each phase is independently shippable and test-gated.

---

## Phase 1: Setup

- [ ] T001 Confirm clean baseline: run `make tmpfs-setup` then `make check` on branch
  `039-centralize-ui-state` and record the passing test count (baseline for "unchanged suite") in the
  PR notes. No code changes.

---

## Phase 2: Foundational (blocking prerequisites)

- [ ] T002 Define `enum Modal` in `src/app.rs` near the existing `HelpScreen`/`ButtonDialog` enums,
  with all variants and payloads per [data-model.md](./data-model.md) (`None`, `ContextMenu`,
  `SessionRestore`, `SavePrompt`, `ExternalChange`, `RevertConfirm(usize)`, `CloseConfirm(usize)`,
  `FindReplace`, `GotoLine{digits,caret}`, `EncodingSelect{row}`, `FileBrowser`, `Help{screen,scroll}`,
  `PluginConsent(Vec<PluginMeta>)`, `PluginManager{cursor}`). Derive nothing it can't (payloads hold
  non-Clone types) — keep it owned/moved.
- [ ] T003 Add `modal: Modal` field to `pub struct App` (init `Modal::None` in the constructor) and the
  accessor surface from [contracts/internal-api.md](./contracts/internal-api.md): `modal()`,
  `modal_is_open()`, `close_modal()`, plus `find_replace()/_mut()`, `file_browser()/_mut()`,
  `context_menu()`. Do NOT remove the old fields yet (compiles alongside; removed in T010).

**Checkpoint**: `Modal` and accessors exist and compile; behavior unchanged (old fields still drive
everything). `make check` green.

---

## Phase 3: User Story 1 — Only one overlay can ever be open (Priority: P1)

**Goal**: Replace the ~14 modal flags with `Modal`; all three orderings derive from one `match`. Two
overlays open at once becomes unrepresentable.

**Independent test**: Existing per-dialog inline + integration tests (open/close each overlay via key
and mouse) pass unchanged via the new accessors.

- [ ] T004 [US1] Migrate **key dispatch** `handle_action` (`src/app.rs` ~918–1687): replace the
  17-branch `if self.pending_X.is_some()` precedence chain with one `match self.modal { … }`, preserving
  the exact existing precedence/early-return semantics. Setting/clearing overlays goes through
  `self.modal = Modal::X(..)` / `self.close_modal()`.
- [ ] T005 [US1] Migrate **mouse dispatch** `handle_mouse_event` (`src/app.rs` ~4210–4681): replace the
  consolidated OR-guard (~4588–4602) with `if self.modal_is_open()`, and convert the per-overlay
  hit-test branches to arms keyed off `self.modal`. Keep the existing per-widget hit-test helpers
  (`button_rects`, `hit_test_buttons`, `find_replace_field_rects`, `FileBrowser::hit_test`,
  `contextmenu::hit_test`, `encoding_row_hit`) — only the selection of which runs changes.
- [ ] T006 [US1] Migrate **render** `Ui::render` (`src/ui/mod.rs` ~225–493): replace the cascade of
  `if pending_X.is_some()` overlay draws with a `match app.modal()`. Paint order of overlays unchanged
  for now (precedence list comes in US2).
- [ ] T007 [US1] Switch the bodies of `open_button_dialog()` (~3625), `interactive_dialog()` (~3875),
  `button_dialog_rect()` (~3820), `interactive_dialog_rect()` (~3980) to read `self.modal`. Keep their
  signatures and return types identical so dependent code/tests are untouched (contract C-3).
- [ ] T008 [US1] Update overlay readers outside `app.rs` to the accessors: `src/ui/menubar.rs`,
  `src/ui/tabbar.rs`, `src/ui/contextmenu.rs`, `src/ui/dialog.rs`, `src/ui/file_browser.rs`,
  `src/session/mod.rs`, `src/watcher/mod.rs`, `src/plugin/mod.rs` (read-mostly; mechanical).
- [ ] T009 [US1] Mechanically update tests reading removed fields to accessors:
  `tests/integration/*.rs` and the inline `#[test]` fns in `src/app.rs`
  (`app.pending_X.is_some()` → `app.X()`/`matches!(app.modal(), Modal::X(_))`). Assertions' expected
  values MUST NOT change (FR-009).
- [ ] T010 [US1] Delete the now-unused fields from `pub struct App` and their constructor inits:
  the 13 folded `pending_*`/`file_browser`/`help_scroll`/`plugin_manager_cursor`/`pending_goto_line_caret`
  fields **and** the dead `menu_active` flag (FR-007). Confirm no remaining references.

**Checkpoint (US1 / MVP)**: `make check` + `make smoke` green with no assertion changes. State type now
forbids two open overlays (SC-003). This phase alone is a shippable improvement.

---

## Phase 4: User Story 2 — Clicks and paint agree on stacking order (Priority: P1)

**Goal**: One declared layer precedence consumed by both render and mouse; delete the tab-bar/dropdown
special-case.

**Independent test**: New generic invariant test (T013) + existing `repro_menu_click_over_tabs` /
`first_dropdown_item_clickable_with_tab_bar_open` pass.

- [ ] T011 [US2] Define `enum Layer` and `App::active_layers()` (bottom→top) in `src/app.rs` per
  [data-model.md](./data-model.md), deriving active layers from
  `(self.modal, self.menu_bar.state, tab_bar_visible())`.
- [ ] T012 [US2] **(TDD — write before T013-impl)** Add a generic invariant test in `src/app.rs`: for
  each active layer in a representative state (2+ buffers, dropdown open over tab bar, a modal open), a
  press inside that layer's rect dispatches to it and never to a lower layer (SC-005, Principle V).
  Initially it may fail against the old `!dropdown_open` special-case path; T013 makes it pass via the
  shared precedence. Keep the existing two regression tests
  (`repro_menu_click_over_tabs`, `first_dropdown_item_clickable_with_tab_bar_open`) as-is — they should
  still pass, now as consequences.
- [ ] T013 [US2] Route paint and hit-test through the precedence: `Ui::render` iterates
  `active_layers()` ascending; `handle_mouse_event` iterates it reversed and dispatches to the first
  layer whose rect contains the cell. **Remove** the `!dropdown_open &&` tab-bar guard
  (`src/app.rs` ~4615) — precedence now handles it (FR-005, L-3). Confirm T012 now passes.

**Checkpoint (US2)**: `make check` + `make smoke` green; z-order is single-sourced (SC-004).

---

## Phase 5: User Story 3 — Clicks land where things are drawn (Priority: P2)

**Goal**: Eliminate the last duplicated geometry; standardize active-buffer access.

**Independent test**: Existing field-caret-click tests pass; manual non-default-size check from
quickstart.

- [ ] T014 [US3] Add `App::goto_line_rect()` in `src/app.rs` and use it from BOTH the Go-to-Line render
  path (`src/ui/mod.rs` ~403–407) and the mouse hit-test (`src/app.rs` ~4424), removing the duplicated
  inline math (FR-006, G-1).
- [ ] T015 [US3] Route the Find/Replace field **render** through `crate::ui::find_replace_field_rects`
  so render and hit-test share one source (G-2). No geometry value changes.
- [ ] T016 [US3] Standardize active-buffer access in `src/app.rs`: replace direct
  `self.buffers[self.active_idx]` with `active_buffer()`/`active_buffer_mut()` (FR-008). Leave explicit
  indexing where a specific non-active buffer is intended (`Modal::RevertConfirm`/`CloseConfirm`
  indices, loops over all buffers).
- [ ] T016a [US3] Preserve the cursor-bounds invariant (FR-011): verify every `close_modal()` and
  active-buffer transition still flows through the pre-render `clamp_all_cursors()` (no new path
  bypasses it), and confirm the existing `render_with_stale_cursor_line_is_clamped_not_panicking`
  inline test still passes unchanged. No new clamp logic — this is a verification task guarding the
  invariant across the refactor's new close paths.

**Checkpoint (US3)**: `make check` + `make smoke` green.

---

## Phase 6: Polish & Cross-Cutting (verification, docs, ship)

- [ ] T017 Run `make ci-local` (fmt --check → clippy -D warnings → test → smoke → perf-check) and fix
  any fmt/clippy fallout from the rewrites (e.g. collapse redundant matches) WITHOUT changing behavior.
  Confirm test count matches the T001 baseline (B-4, SC-002).
- [ ] T018 Manual validation per [quickstart.md](./quickstart.md) on a non-80×24 terminal: every overlay
  open/close, dropdown-over-tabs first-item click, field caret clicks, rapid top-row clicks — confirm
  identical behavior and no panic (SC-006; guards bugs 014/033/038).
- [ ] T019 Docs gate: update `CHANGELOG.md` (feature 039 entry) and `docs/STATUS.md` (architecture
  note: centralized UI state). Do NOT touch `docs/CAPABILITIES.md` — no user-visible capability changed
  (FR-010). Per CLAUDE.md docs-gate this is a feature PR, so docs updates are required (no `[no-docs]`).
- [ ] T020 Open PR targeting `master` (title `feat(039): centralize editor UI state (Modal enum +
  single layer precedence)`), strip any AI-attribution footer, ensure CI green, then merge.

---

## Dependencies & Execution Order

- **Setup (T001)** → **Foundational (T002–T003)** → **US1 (T004–T010)** → **US2 (T011–T013)** →
  **US3 (T014–T016)** → **Polish (T017–T020)**.
- US1 is the MVP and a hard prerequisite for US2/US3 (they operate on the `Modal`/accessor surface).
- Within US1: T004/T005/T006 touch the three dispatchers and can be done in sequence (all in
  `app.rs`/`ui/mod.rs`, so not `[P]`); T008 (other-module readers) is `[P]` relative to T004–T007 once
  accessors exist. T009 follows the code migration. T010 (field deletion) is last in the phase — it is
  the compile-enforced proof that nothing reads the old fields.

## Parallel Opportunities

- T008 `[P]` — the `src/ui/*`, `src/session`, `src/watcher`, `src/plugin` reader updates are
  independent files and can be edited together once accessors (T003) exist.
- Most other tasks are serialized because they edit the same large files (`app.rs`, `ui/mod.rs`).

## MVP Scope

**US1 (Phase 3)** alone — the `Modal` enum replacing the flag soup — delivers the highest-leverage fix
(illegal two-overlay states become unrepresentable) and is independently shippable and test-gated.
US2 and US3 build on it within the same PR.

## Implementation Strategy

Migrate behind the existing test suite as the safety net: add `Modal` + accessors alongside the old
fields (T002–T003), move each consumer over (T004–T009), then delete the old fields last (T010) so the
compiler proves the cut-over is complete. Ship all three phases as ordered commits in one PR.
