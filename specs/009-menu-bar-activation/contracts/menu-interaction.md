# Contract: Menu-Bar Keyboard Interaction & Dispatch

This editor is a TUI application; its external contract is the **keyboard interaction model** of
the menu bar and the **action dispatch** it produces. This document is the testable surface.

## C1. Activation keys (while menu bar is active)

Precondition: no higher-priority modal overlay is active (Find/Replace, encoding dialog, Plugin
Manager, consent prompt). When such a modal is active, these keys belong to the modal (FR-012).

| Key (Action) | Menu state | Effect |
|---|---|---|
| `Up` (`MoveUp`) | `DropDown` | Highlight previous item, wrap to last at top. |
| `Up` (`MoveUp`) | `TopActive` | Open dropdown at last item. |
| `Down` (`MoveDown`) | `DropDown` | Highlight next item, wrap to first at bottom. |
| `Down` (`MoveDown`) | `TopActive` | Open dropdown at first item. |
| `Left` (`MoveLeft`) | `DropDown` | Move to previous top-level menu, open its dropdown (wrap). |
| `Left` (`MoveLeft`) | `TopActive` | Highlight previous top-level menu (wrap), no dropdown. |
| `Right` (`MoveRight`) | `DropDown` | Move to next top-level menu, open its dropdown (wrap). |
| `Right` (`MoveRight`) | `TopActive` | Highlight next top-level menu (wrap), no dropdown. |
| `Enter` (`InsertNewline`) | `DropDown` | Dispatch highlighted item's action; close menu. |
| `Esc` (`MenuClose`) | any active | Close menu; perform no action. |
| any other action | any active | Consumed (no buffer mutation, no cursor move). |

Postcondition (FR-006): after any navigation key, buffer content and editor cursor position are
unchanged. Only `Enter` may change them — via the dispatched action, not the navigation.

## C2. Open keys (while menu bar is inactive)

Unchanged from current behavior; listed for completeness:

| Key | Effect |
|---|---|
| `Alt+F/E/S/V/O/H` | Open File/Edit/Search/View/Options/Help dropdown directly (`DropDown`, item 0). |
| `F10` | Activate the menu bar at the first menu, **no dropdown open** (`TopActive(0)`); Left/Right then move the highlight, Down/Up open it. (Remediation H1.) |

`Alt+<letter>` maps to existing `Action::MenuFile..MenuHelp` → `open_menu(idx)` (clamped against the
resolved menu count). `F10` maps to `Action::Menu` → new `activate_bar()`.

## C3. Built-in item dispatch

Selecting a built-in dropdown item dispatches its static `Action` exactly as if that action were
triggered directly. Examples:

| Menu > Item | Dispatched Action | Observable result |
|---|---|---|
| File > New | (new-buffer action) | A new empty buffer is active. |
| File > Save | `Save` | Active buffer written to disk. |
| View > Soft Wrap (ext) | `ToggleSoftWrap` | Soft-wrap toggles; check-state indicator flips next render. |
| Options > Plugins… | `OpenPluginManager` | Plugin Manager overlay opens. |

## C4. Plugin item dispatch

Selecting a plugin dropdown item MUST dispatch
`Action::PluginMenuActivated(plugin_id, item_id)`, which routes to the existing
`PluginHost::dispatch_menu_action(plugin_id, item_id, buffer_content)`:

- On success: the returned message string is shown in the status bar.
- On sandbox timeout/error: the editor stays responsive; a warning is shown in the status bar; the
  offending plugin is disabled by the existing dispatch layer (no change here).

Example (reference `word-count` plugin): `Tools > Word Count` →
`PluginMenuActivated("word-count","wc")` → status bar shows e.g. `"Word count: 5"`.

## C5. Rendering contract

- Plugin top-level menus render **between Options and Help**; Help is always the rightmost menu.
- A plugin whose `menu` name equals a built-in name contributes its items to that built-in
  dropdown (no duplicate top-level label).
- **No-plugin parity**: with zero active plugin menu items, the menu bar renders byte-identically
  to the pre-feature layout (same labels, columns, widths). This is the FR-011 / SC-003 guard.
- Disabled plugins and `--no-plugins` contribute nothing (FR-010).

## C6. Test obligations (TDD)

Unit (in `src/ui/menubar.rs`):
- `test_resolve_menus_empty_matches_builtin` — `resolve_menus(&[])` equals built-in set.
- `test_resolve_menus_inserts_plugin_before_help` — a new plugin menu lands at index `len-1`
  (immediately before Help).
- `test_resolve_menus_merges_into_builtin_on_name_collision` — plugin `menu="Edit"` appends to
  Edit's items; no duplicate top-level "Edit".
- `test_navigate_down_wraps`, `test_navigate_up_wraps` (over resolved list).
- `test_navigate_left_right_wraps_over_full_ring` (includes a plugin menu).
- `test_navigate_left_right_opens_adjacent_dropdown`.
- `test_navigate_left_right_top_active_moves_highlight_only` (from `TopActive`, no dropdown).
- `test_activate_bar_enters_top_active` and `test_top_active_down_opens_dropdown` (remediation H1).
- `test_select_item_returns_builtin_action_and_closes`.
- `test_select_item_returns_plugin_activated_action` (remediation M3).
- `test_resolve_menus_widechar_plugin_label_preserved` — a plugin menu/item with multibyte/wide
  characters resolves with the label intact (UTF-8, FR-014; remediation M2).

Integration (`tests/integration/menu_activation.rs`):
- `test_keyboard_open_navigate_activate_builtin_save` — Alt+F, Down×n to Save, Enter → file saved.
- `test_escape_closes_without_action` — open menu, Esc → inactive, buffer unchanged.
- `test_navigation_does_not_mutate_buffer` — arrows while menu open leave buffer + cursor intact.
- `test_plugin_menu_keyboard_activation_sets_status` — consented `word-count` fixture; navigate to
  Tools > Word Count; Enter → status bar contains the count.
- `test_no_plugins_menu_bar_unchanged` — with no plugin menu items, resolved menus == built-ins.
- `test_no_plugins_flag_yields_no_plugin_menus` — an `App` built with `no_plugins=true` resolves to
  exactly the built-in menus (remediation M4, FR-010).
- `test_plugin_menu_dispatch_failure_surfaces_warning` — activating a failing/looping plugin item
  via the menu leaves the editor responsive, sets a status-bar warning, and keeps the buffer intact
  (remediation M1, FR-013 / SC-006).

Smoke (`tests/smoke/plugin_menu_activate.exp`):
- Launch on a temp buffer with the `word-count` fixture pre-consented; drive keys to open Tools,
  select Word Count; assert the status line shows a count; exit cleanly.
