# Contract: MenuBarWidget

**Feature**: 006 — Menu Check-State Indicator
**File**: `src/ui/menubar.rs`
**Date**: 2026-06-19

---

## Public API Changes

### `MenuBarWidget<'a>` struct

**Before (feature 005 state):**
```rust
pub struct MenuBarWidget<'a> {
    pub theme: &'static Theme,
    pub menu_state: &'a MenuBarState,
}

impl<'a> MenuBarWidget<'a> {
    pub fn new(theme: &'static Theme, menu_state: &'a MenuBarState) -> Self
}
```

**After (feature 006):**
```rust
pub struct MenuBarWidget<'a> {
    pub theme: &'static Theme,
    pub menu_state: &'a MenuBarState,
    pub toggle_states: &'a [(Action, bool)],  // NEW
}

impl<'a> MenuBarWidget<'a> {
    pub fn new(
        theme: &'static Theme,
        menu_state: &'a MenuBarState,
        toggle_states: &'a [(Action, bool)],  // NEW
    ) -> Self
}
```

---

## Terminology Glossary

| Term | Canonical? | Maps to |
|------|-----------|---------|
| **check state** | ✅ Canonical (spec FR-001) | The `bool` value associated with an action in `toggle_states` |
| **toggle state** | Synonym | Same concept; field name `toggle_states` uses this term |
| **checked / unchecked** | Synonym | `true` / `false` value of the check state |
| **checkable-aware menu** | Derived term | A dropdown containing ≥ 1 item whose action appears in `toggle_states` |

When reading code and docs: `toggle_states` (code) ↔ "check state" (spec) — both refer to the same runtime boolean mapping from `Action` to on/off.

---

## Behavioral Contract

### Preconditions
- `toggle_states` may be empty (`&[]`); this is valid and produces identical behavior to pre-feature rendering.
- `toggle_states` entries must have unique actions (undefined behavior if two entries share the same `Action` variant; callers must not pass duplicates).
- `Action` variants in `toggle_states` are matched via `PartialEq`; the derived `PartialEq` implementation is used.

### Postconditions / Render Invariants

1. **Non-checkable menu (no items match `toggle_states`)**: Dropdown renders identically to pre-feature behavior. `content_width` is `max_label_len + 4`. Label starts at `start_col + 1`. No prefix column.

2. **Checkable menu (≥ 1 item action appears in `toggle_states`)**: Dropdown reserves a 2-char prefix column.
   - `content_width` = `max_label_len + 6`.
   - Every item gets a 2-char prefix at `start_col + 1`.
   - Label text starts at `start_col + 3`.
   - Checked items (`Some(true)` in `toggle_states`): prefix = `'✓'` + `' '`.
   - Unchecked items (`Some(false)` in `toggle_states`): prefix = `' '` + `' '`.
   - Items not in `toggle_states` but in a checkable menu: prefix = `' '` + `' '` (treated as unchecked for alignment).

3. **The `✓` character**: U+2713, rendered via `buf.get_mut(cx, row_y).set_style(item_style).set_char('✓')` — same `item_style` (normal or selected/inverted-video) applied to the rest of that dropdown row. Display width = 1 terminal cell. The checkmark does NOT use a distinct color.

4. **Width overflow**: existing `start_col` clamping (`drop_col.min(area.width.saturating_sub(content_width))`) applies unchanged. Labels are truncated at `area.right()` as before.

5. **No open dropdown**: If `MenuState` is not `DropDown { .. }`, this function has no effect on the buffer beyond the menu bar row. Unchanged from pre-feature behavior.

---

## Call-Site Contract

### `src/ui/mod.rs` — line ~73

**Before:**
```rust
let menubar = MenuBarWidget::new(app.theme, &app.menu_bar);
```

**After:**
```rust
let toggle_states: &[(Action, bool)] = &[(Action::ToggleSoftWrap, app.soft_wrap)];
let menubar = MenuBarWidget::new(app.theme, &app.menu_bar, toggle_states);
```

The `Action` import used in `src/ui/mod.rs` must include `Action` from `crate::input::keymap`. Verify existing imports before adding.

---

## Unit Test Expectations

Tests live in `src/ui/menubar.rs` `#[cfg(test)]` module (7 tests total).

| Test name | What it asserts |
|---|---|
| `test_checkmark_shown_when_toggle_true` | Render View dropdown with `soft_wrap = true`; cell at prefix column contains `'✓'` |
| `test_no_checkmark_when_toggle_false` | Render View dropdown with `soft_wrap = false`; prefix column contains `' '` |
| `test_non_toggle_menu_unaffected` | Render File dropdown with toggle_states for soft_wrap; no `'✓'`, content_width = 23 (not 25) |
| `test_label_alignment_in_checkable_menu` | All items in View dropdown start label at same column; absent-from-toggle-states items get `' '` prefix (not a checkmark) |
| `test_empty_toggle_states_no_regression` | `toggle_states = &[]`; full dropdown renders identically to pre-feature (content_width = max_len + 4) |
| `test_second_action_also_shows_checkmark` | Render Options dropdown with `toggle_states = &[(Action::ToggleHighlight, true)]`; prefix cell = `'✓'` — proves action-agnostic generality (FR-007) |
| `test_initial_soft_wrap_state_from_config` | `toggle_states = &[(Action::ToggleSoftWrap, true)]` with no prior toggle call; prefix cell = `'✓'` — proves config-persisted state renders correctly (US3) |
