# Phase 0 Research: Centralize Editor UI State

All "unknowns" here are codebase-internal facts (this is a refactor of existing code, not new tech), so
research = grounding the design in what `src/app.rs` and `src/ui/` actually do today. Findings below are
verified against the current source.

## R1 — What state actually represents "an open overlay"?

**Decision**: Fold these into one `Modal` enum (one open at a time): `pending_context_menu`,
`pending_session_restore`, `pending_save_prompt` (bool), `pending_external_change`,
`pending_revert_confirm` (usize), `pending_close_confirm` (usize), `pending_find_replace`,
`pending_goto_line` (+ `pending_goto_line_caret`), `pending_encoding_select` (usize), `file_browser`,
`pending_help` (+ `help_scroll`), `pending_plugin_consent` (Vec), `pending_plugin_manager`
(+ `plugin_manager_cursor`).

**Keep separate (NOT in `Modal`)**:
- `menu_bar: MenuBarState` — the menu can be active while editing; it is a distinct *layer*, not a
  mutually-exclusive foreground modal. Already an enum (`MenuState`), already the source of truth.
- `drag_anchor`, `scrollbar_drag` — transient pointer-interaction state, orthogonal to "which overlay
  is open" (low read-count: 2 and 4).
- `dialog_focus`, `dialog_focus_init` — adjunct focus state shared by whichever button/interactive
  dialog is open; kept as-is to bound the diff (deferred per spec Assumptions).
- `pending_save_as_encoding` — *flow* state carried across the Save-As → filename steps, not a
  concurrently-open overlay. Stays a field.

**Rationale**: The spec's exclusivity invariant (FR-001) applies to *foreground overlays*. Menu and
pointer-drag are concurrent-by-design with editing, so forcing them into the same enum would m:isrepresent
reality and complicate the precedence. **Alternatives considered**: (a) one mega-enum including menu —
rejected, menu legitimately coexists with editing; (b) keeping `dialog_focus` inside each variant —
rejected for this PR to limit churn (recorded as deferral).

**Verified**: `menu_active` has 0 reads (dead); `handle_action` checks the flags in a fixed precedence
(context menu → button dialogs → interactive dialogs → file browser → help → plugin consent → plugin
manager → menu); `handle_mouse_event` has a consolidated OR-guard mirroring the same set.

## R2 — How is overlay precedence currently encoded, and where does it drift?

**Decision**: Declare one ordered layer precedence (top→bottom):
`Modal (foreground) > MenuState::DropDown > MenuState::TopActive > TabBar > Editor`.
Render iterates bottom→top; mouse iterates top→bottom and dispatches to the first layer whose rect
contains the cell.

**Rationale**: Today the order lives in three places — `handle_action` (key), `handle_mouse_event`
(mouse), `Ui::render` (paint) — and they drift. Bug 033 (`render tab bar before menu bar so dropdowns
overlay it`) fixed paint order; bug 038 then had to bolt `!dropdown_open &&` onto the tab-bar hit-test
because mouse order still disagreed. One shared precedence removes the reconciliation entirely (FR-004,
FR-005). **Alternatives considered**: keep three orderings but add an assertion they match — rejected,
doesn't remove the drift, just detects it.

**Verified**: `repro_menu_click_over_tabs` (app.rs ~6958) and
`first_dropdown_item_clickable_with_tab_bar_open` (~6580) exist precisely as regression patches for this
drift; they become consequences of the shared precedence.

## R3 — Which geometry is computed twice (paint vs hit-test)?

**Decision**: Most overlays already share a `rect()`+`hit_test()` helper between render and mouse
(`tab_hit_regions`, `button_rects`/`hit_test_buttons`, `hit_test_menu`, `FileBrowser::hit_test`,
`contextmenu::hit_test`, `find_replace_field_rects`, `scrollbar_regions`, `editor_panes`). The two
exceptions recompute inline: **Go-to-Line** (mouse ~app.rs:4424 vs render ui/mod.rs:403–407) and the
**Find/Replace field** render path (hit-test already uses `find_replace_field_rects`, render does not).

**Action**: Add `goto_line_rect()` (single source) and route the Find/Replace *render* through
`find_replace_field_rects` so paint and hit-test share one function (FR-006).

**Rationale**: This is the class behind bug 014 (`sync terminal_size with the rendered frame for mouse
hit-testing`) — geometry computed from a stale/independent source. **Alternatives considered**: a full
retained-widget tree storing every rect after paint — rejected as out of scope (deferred); the targeted
helper extraction gets the FR-006 guarantee at a fraction of the cost.

## R4 — Is the cursor-bounds invariant safe across overlay-close paths?

**Decision**: No new clamp needed. `clamp_all_cursors()` (app.rs ~2530) runs before each paint and
`EditorRope::line_slice` (rope.rs:69) returns `""` out-of-range in release. Requirement: ensure the new
`close_modal()` path does not bypass the pre-render clamp (it cannot — clamp is in the render entry).

**Rationale**: Bug 034 already centralized this. The refactor must preserve it (FR-011), not redo it.

## R5 — Migration safety: how do we keep behavior identical?

**Decision**: Keep the signatures of existing helpers (`open_button_dialog`, `interactive_dialog`,
`*_rect`) stable; switch only their *bodies* to read `self.modal`. Provide accessors
(`modal()`, `modal_is_open()`, `close_modal()`, and `*_mut()` for inner payloads) so call-sites and
tests change mechanically (field read → accessor) with no behavior change.

**Rationale**: The 33 integration files + 87 inline tests assert behavior; FR-009 forbids changing
assertions. Stable helper signatures + thin accessors make the diff mechanical and let the test suite be
the proof of equivalence. **Alternatives considered**: big-bang rewrite of dispatch — rejected, would
force test-assertion churn and lose the safety net.

## Open questions

None. All resolved against current source. No `NEEDS CLARIFICATION` remain.
