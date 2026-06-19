# Phase 1 Data Model: Live Menu-Bar Activation

This feature is UI-state, not persisted data. The "model" here is the in-memory menu structure
and the navigation state machine.

## Entities

### ResolvedMenu (new)

A single top-level menu as it appears in the bar, after merging built-ins with plugin items.

| Field | Type | Notes |
|---|---|---|
| `label` | `String` | Display label (e.g. "File", "Tools"). Owned because plugin labels are runtime strings. |
| `items` | `Vec<ResolvedItem>` | Ordered dropdown items. Non-empty for built-ins; may be empty only for a malformed plugin menu (guarded). |

### ResolvedItem (new)

A selectable dropdown entry with its dispatch action.

| Field | Type | Notes |
|---|---|---|
| `label` | `String` | Display label (e.g. "Save", "Word Count"). |
| `action` | `Action` | Static `Action` for built-ins; `Action::PluginMenuActivated(plugin_id, item_id)` for plugin items. |

**Why owned `String` (not `&'static str` like `MenuItem`)**: built-in labels are static but plugin
labels are owned at runtime; a single owned type lets built-in and plugin items live in one list.
Built-in labels are cloned from the existing `&'static str` slices when resolving.

### MenuItem (existing, unchanged)

`{ label: &'static str, action: Action }` — the static built-in dropdown entry. Source for
built-in `ResolvedItem`s.

### PluginMenuItem (existing, unchanged — `src/plugin/types.rs`)

`{ menu: String, item: String, item_id: String, plugin_id: String, position: Option<u32> }`.
Produced by `PluginRegistry::menu_items()` for active menu plugins. Source for plugin
`ResolvedItem`s and any plugin-only `ResolvedMenu`s.

### MenuState (existing, unchanged — `src/ui/menubar.rs`)

State machine enum:
- `Inactive` — menu bar closed (normal editing).
- `TopActive(usize)` — a top-level menu is highlighted, no dropdown open. `usize` indexes the
  **resolved** menu list.
- `DropDown { top_idx, item_idx }` — dropdown open; both indices reference the **resolved** list.

### MenuBarState (existing — methods change signature)

Holds `state: MenuState`. Methods are refactored to operate on a `&[ResolvedMenu]` argument:
`open_menu`, `navigate_up`, `navigate_down`, `navigate_left` (new), `navigate_right` (new),
`select_item`, plus new `activate_bar()` (sets `TopActive(0)`, no model needed) and unchanged
`close_menu`, `is_active`. The `open_menu` doc-comment must be updated: it is the Alt+letter
direct-dropdown entry, not the only entry path (F10 uses `activate_bar`).

## Derivation rule: `resolve_menus(plugin_items: &[PluginMenuItem]) -> Vec<ResolvedMenu>`

1. Seed the list from the six built-in menus (File, Edit, Search, View, Options, Help) in order,
   converting each `MenuItem` to a `ResolvedItem`.
2. Partition plugin items by `menu` name into:
   - **collisions** — name matches a built-in menu → append their `ResolvedItem`s
     (`PluginMenuActivated(plugin_id, item_id)`) to that built-in's `items`.
   - **new menus** — name matches no built-in → group into new `ResolvedMenu`s.
3. Within each group, order plugin items by `position` (ascending) when set, else by
   `registry().menu_items()` load order (stable).
4. Insert the new plugin menus, in first-appearance order, **immediately before the Help menu**
   (i.e. after Options) so Help remains last.
5. `resolve_menus(&[])` MUST equal the built-in list with identical labels/order/items (parity
   invariant for FR-011 / SC-003).

## State transitions (with resolved menu list `M`)

| From | Event | To |
|---|---|---|
| `Inactive` | `activate_bar()` (F10) | `TopActive(0)` — bar highlighted, no dropdown (DOS-faithful entry; remediation H1) |
| `Inactive` | `open_menu(i)` (e.g. Alt+F) | `DropDown { top_idx: clamp(i, M.len()), item_idx: 0 }` |
| `TopActive(t)` | `navigate_down` | `DropDown { t, 0 }` |
| `TopActive(t)` | `navigate_up` | `DropDown { t, last(t) }` |
| `TopActive(t)` | `navigate_left/right` | `TopActive((t∓1) mod M.len())` |
| `DropDown{t,i}` | `navigate_down` | `DropDown{t, (i+1) mod len(t)}` |
| `DropDown{t,i}` | `navigate_up` | `DropDown{t, (i+len(t)-1) mod len(t)}` |
| `DropDown{t,i}` | `navigate_right` | `DropDown{(t+1) mod M.len(), 0}` |
| `DropDown{t,i}` | `navigate_left` | `DropDown{(t+M.len()-1) mod M.len(), 0}` |
| `DropDown{t,i}` | `select_item` | returns `M[t].items[i].action`; → `Inactive` |
| any active | `close_menu` (Esc) | `Inactive` (no action) |

`len(t)` = number of items in `M[t]`; `last(t) = len(t).saturating_sub(1)`. All indices are
clamped/guarded so an empty or out-of-range menu cannot panic (returns no action / no-op).

## Validation rules

- Resolved indices are always clamped to the current `M` bounds before use.
- A `ResolvedMenu` with zero items is non-openable: `navigate_down`/`up` from `TopActive` on it
  is a no-op; `select_item` returns `None`.
- Plugin labels are already UTF-8-validated by the manifest/registry layer; no revalidation here,
  but rendering uses the existing wide-character-aware path (FR-014).
