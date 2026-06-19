# Phase 0 Research: Live Menu-Bar Activation

All Technical Context items were resolvable from the existing codebase (no external unknowns).
This document records the design decisions and the alternatives weighed.

## R1. How to keep rendered menus and navigated menus in sync

**Decision**: Introduce one resolved menu model (`Vec<ResolvedMenu>`) built by
`resolve_menus(&[PluginMenuItem])`, consumed by both `MenuBarWidget::render` and the `App`
navigation/selection path.

**Rationale**: Today `MenuBarWidget` renders from the static `ALL_MENUS` and `MenuBarState`
navigates against `ALL_MENUS` too — they happen to agree because both are static. Adding plugin
menus makes the menu list dynamic; if render and navigation computed it independently they could
disagree (off-by-one selection, activating the wrong item). A single builder used by both removes
that whole class of bug.

**Alternatives considered**:
- *Two parallel computations (render builds its own, nav builds its own)* — rejected: duplicate
  logic, drift risk, exactly the bug we want to avoid.
- *Store the resolved model as cached `App` state, rebuilt on plugin change* — rejected for v1:
  the menu set is tiny (≤ ~11 menus) and rebuilding per frame/selection is negligible; caching
  adds invalidation complexity for no measurable gain. Can be added later if profiling shows need.

## R2. Where plugin menus go and how name collisions resolve

**Decision**: Insert plugin-only menus **between Options and Help** (Help stays rightmost). When
a plugin's `menu` name equals a built-in menu name, **merge** its items onto that built-in
dropdown instead of creating a duplicate top-level entry. (Both confirmed in spec Clarifications.)

**Rationale**: Constitution Principle I (DOS-faithful UI) — EDIT.COM always renders Help last.
Merging avoids a confusing duplicate top-level label and matches user mental model ("my plugin
adds an item under Edit").

**Alternatives considered**:
- *Append after Help* (issue #19 literal wording) — rejected by clarification: breaks Help-last
  convention.
- *Always separate top-level menu even on name collision* — rejected: duplicate labels are
  confusing; merge is cleaner.

## R3. Item ordering within a merged/plugin menu

**Decision**: Built-in items first (for merged built-in menus), then plugin items in
`registry().menu_items()` order; if the optional `position` field is set, use it as a stable sort
key, otherwise preserve load order.

**Rationale**: Deterministic and predictable; load order is already stable. `position` exists on
`PluginMenuItem` and is honored if provided, future-proofing ordering without new config.

**Alternatives considered**: alphabetical sort (rejected — surprising, non-DOS); ignore
`position` entirely (rejected — wastes an existing, cheap signal).

## R4. Refactoring `MenuBarState` without breaking its API contract

**Decision**: Change `open_menu`, `navigate_up`, `navigate_down`, `select_item` to accept the
resolved menu slice, and add `navigate_left(&menus)` / `navigate_right(&menus)`. `select_item`
returns the resolved `Action` (a static `Action` for built-ins, `Action::PluginMenuActivated`
for plugin items) and closes the menu.

**Rationale**: The state machine must know item counts (for wrap math) and the action to return.
Passing the model keeps the state struct a pure index holder (no embedded menu data), which is
the smallest correct change. All existing call sites are in `app.rs` (handle_action) and tests,
which this feature updates anyway.

**Alternatives considered**:
- *Keep `ALL_MENUS` hardcoded and special-case plugin menus in the widget only* — rejected:
  navigation/selection would not reach plugin items at all (the core requirement).
- *Make `select_item` return `(top_idx, item_idx)` and resolve the action in `app.rs`* — viable,
  but splits the lookup across two places; returning the `Action` keeps resolution in one spot.

## R5. Left/Right semantics (dropdown follow)

**Decision**: When a dropdown is open, Left/Right move to the adjacent top-level menu **and open
its dropdown** (item 0 highlighted), wrapping at both ends across the full composite ring. When
only a top-level menu is highlighted (TopActive, no dropdown), Left/Right move the highlight
without opening. (Confirmed in spec Clarifications.)

**Rationale**: Matches EDIT.COM. Wrapping over the *composite* ring means plugin menus are part
of normal Left/Right traversal (FR-003, SC-005).

**Alternatives considered**: Left/Right closes dropdown and only highlights (rejected by
clarification — less fluid).

## R6. Event-loop integration point and modal precedence

**Decision**: Add a `menu_bar.is_active()` guard block in `App::handle_action`, placed **after**
the existing `pending_*` modal guards (session restore, save prompt, external change, encoding
select, plugin consent, plugin manager) and **before** the normal action match. Inside it, route:
`MoveUp→navigate_up`, `MoveDown→navigate_down`, `MoveLeft→navigate_left`, `MoveRight→navigate_right`,
`InsertNewline→select_item` (then dispatch the returned action), `MenuClose→close_menu`. Other
actions while a menu is open are consumed (no-op) to avoid leaking navigation into the buffer
(FR-006).

**Rationale**: Mirrors the proven dialog pattern already in `handle_action` (e.g.
`pending_encoding_select` block at `src/app.rs:512`). Placement after modal guards satisfies
FR-012 (modals win). Re-dispatching the selected action after `select_item` closed the menu means
the recursive `handle_action` call won't re-enter the guard (depth 1, terminates).

**Alternatives considered**: handling menu keys in `src/input/dispatch_key` before producing an
Action (rejected — `dispatch_key` is stateless and has no access to `App`/menu state; the keymap
must stay context-free).

## R7. Preserving existing geometry tests (regression safety)

**Decision**: `resolve_menus(&[])` returns the six built-in menus unchanged; the renderer computes
top-level label columns from the resolved list but, for the built-in-only case, MUST produce the
exact current columns (File@1, Edit@7, Search@13, View@21, Options@28, Help@37) and dropdown
widths. Add an explicit `test_resolve_menus_empty_matches_builtin` and keep all existing
`test_*` geometry tests untouched.

**Rationale**: FR-011 / SC-003 require zero visual regression with no plugins. The existing tests
(`test_checkmark_shown_when_toggle_true`, `test_non_toggle_menu_unaffected`, etc.) are the guard;
they must pass without modification.

**Alternatives considered**: rewriting the static `BAR_LABELS` table into pure dynamic layout and
updating the geometry tests (rejected — needlessly perturbs a tested, working layout and weakens
the regression guard).

## Summary of resolved unknowns

| Item | Resolution |
|---|---|
| Sync of render vs navigation | Single `resolve_menus` model (R1) |
| Placement / collisions | Between Options & Help; merge by name (R2) |
| Item ordering | built-in then plugin load order, honor `position` (R3) |
| State refactor | nav/select take the model; add left/right (R4) |
| Left/Right behavior | dropdown-follow with wrap over composite ring (R5) |
| Event wiring | `is_active()` guard after modals in `handle_action` (R6) |
| Regression safety | empty-resolve parity + keep existing tests (R7) |

No `NEEDS CLARIFICATION` remain.
