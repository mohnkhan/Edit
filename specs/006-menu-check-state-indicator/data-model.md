# Data Model: Menu Check-State Indicator

**Feature**: 006 — Menu Check-State Indicator
**Date**: 2026-06-19

---

## Entities

### `MenuItem` (modified)

Single selectable item in a pull-down menu. **No change to this struct.** Runtime checked state is NOT stored here because `MenuItem` values are `static` constants that cannot hold mutable runtime data.

```
MenuItem
├── label: &'static str   — display text shown in the dropdown
└── action: Action        — editor action dispatched on selection
```

*The checked state is determined at render time by correlating `action` with the `toggle_states` slice passed to `MenuBarWidget`.*

---

### `MenuBarWidget` (modified)

Ratatui widget that renders the menu bar row and any active dropdown. Gains one new field.

```
MenuBarWidget<'a>
├── theme: &'static Theme                — color scheme
├── menu_state: &'a MenuBarState         — open/closed/highlighted state
└── toggle_states: &'a [(Action, bool)]  — runtime check-states for toggleable actions
                                           (NEW — was absent before this feature)
```

**Field semantics for `toggle_states`:**
- A slice entry `(action, true)` means: the item whose `action` matches should render with a `✓ ` prefix.
- A slice entry `(action, false)` means: the item renders with a `  ` (two-space) prefix — still participating in the prefix column for alignment.
- An `action` absent from the slice means: the item has no toggle state and does not participate in the prefix column.
- An empty slice (`&[]`) means no items in any menu are toggleable — the widget behaves identically to pre-feature behavior.

---

### `DropdownRenderContext` (conceptual, not a struct)

A logical grouping of derived values computed per dropdown at render time:

```
DropdownRenderContext (computed, not persisted)
├── menu_items: &'static [MenuItem]   — items for the open dropdown
├── has_checkable: bool               — true if any item.action ∈ toggle_states
├── content_width: u16                — dropdown pixel width
│     when has_checkable = false: max_label_len + 4
│     when has_checkable = true:  max_label_len + 6
├── label_col_offset: u16             — chars from start_col to label start
│     when has_checkable = false: 1
│     when has_checkable = true:  3
└── prefix_for(item): &'static str    — "✓ " | "  " | "" depending on toggle_states
```

---

## State Transitions

```
App::soft_wrap: bool
        │
        │  read at render time
        ▼
toggle_states: &[(Action::ToggleSoftWrap, soft_wrap)]
        │
        │  consumed by
        ▼
MenuBarWidget::render()
        │
        ├── has_checkable=false → no prefix column; existing layout unchanged
        │
        └── has_checkable=true
              ├── checked item  →  "✓ " prefix rendered + label
              └── unchkd item   →  "  " prefix rendered + label
```

---

## Validation Rules

| Rule | Constraint |
|---|---|
| `toggle_states` length | Unconstrained; typically 0–5 entries per rendering frame |
| `Action` in `toggle_states` | Must derive `PartialEq` (already guaranteed by `Action`'s derive) |
| Prefix character | U+2713 `✓` (display width 1, east-asian-width Neutral) |
| Prefix column width | Always 2 terminal cells (`✓ ` or `  `) |
| `content_width` | `u16`; clamped to terminal width via existing `start_col` logic |
| Label rendering | Chars iterated with `.chars().enumerate()`; label truncated at `area.right()` |
