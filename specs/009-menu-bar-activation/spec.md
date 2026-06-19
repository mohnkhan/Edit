# Feature Specification: Live Menu-Bar Activation

**Feature Branch**: `009-menu-bar-activation`

**Created**: 2026-06-19

**Status**: Draft

**Input**: User description: "Wire live menu-bar keyboard activation of plugin-contributed menu items (follow-up to feature 008, GitHub issue #19). The plugin menu registry, sandboxed dispatch, consent dialog, and Options > Plugins manager already exist and are tested. What is missing: (1) the menu-bar dropdown item-selection event path is not wired for built-in menus either — `MenuBarState::select_item` / `navigate_up` / `navigate_down` exist but are never called from the key event loop; (2) plugin-contributed top-level menus are never rendered. This feature wires keyboard navigation (Up/Down within a dropdown, Left/Right between top-level menus, Enter to activate, Esc to close) for BOTH built-in and plugin menus, and renders plugin-declared top-level menus, dispatching the selected action including `Action::PluginMenuActivated(plugin_id, item_id)`. Existing static-menu geometry and tests must remain intact."

## Clarifications

### Session 2026-06-19

- **Q: Where should plugin-contributed top-level menus appear?** → A: **Between Options and
  Help**, keeping Help rightmost (DOS-faithful, Constitution Principle I). This overrides the
  issue's literal "append after Help" wording and aligns with feature-008 `tasks.md` T025 and
  `ROADMAP.md`.
- **Q: Plugin menu name collides with a built-in menu name (e.g. "Edit")?** → A: **Merge** the
  plugin's items into the matching built-in dropdown; do not create a duplicate top-level entry.
- **Q: Left/Right with a dropdown open?** → A: **Open the adjacent menu's dropdown automatically**
  (DOS EDIT.COM behavior); Left/Right slides between menus keeping a dropdown open.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Activate built-in menu items by keyboard (Priority: P1)

A user opens a pull-down menu (e.g. presses `Alt+F` for File, or `F10`), moves the
highlight to an item with the arrow keys, and presses `Enter` to run it — exactly as in
MS-DOS EDIT.COM. They can move sideways between top-level menus with Left/Right and dismiss
the menu with `Esc` without performing any action.

**Why this priority**: This is the foundational gap. Today a menu can be *opened* but no item
inside it can be *selected* by keyboard — `navigate_up`/`navigate_down`/`select_item` exist but
are never invoked. Without this, every other menu interaction (including plugin menus) is
impossible. It delivers a complete, demonstrable DOS-faithful menu experience on its own.

**Independent Test**: With no plugins installed, open the File menu, arrow down to "Save",
press Enter, and confirm the active buffer is saved; press `Alt+E`, arrow to an item, press
`Esc`, and confirm nothing changed and the menu closed.

**Acceptance Scenarios**:

1. **Given** the editor is in normal editing mode, **When** the user presses `Alt+F` then `Down` then `Enter` on the "New" item, **Then** the New-buffer action runs and the menu closes.
2. **Given** a top-level menu is highlighted (no dropdown open), **When** the user presses `Down`, **Then** that menu's dropdown opens with the first item highlighted.
3. **Given** a dropdown is open with the first item highlighted, **When** the user presses `Up`, **Then** the highlight wraps to the last item of that dropdown.
4. **Given** a dropdown is open, **When** the user presses `Right`, **Then** the adjacent top-level menu to the right opens its dropdown (wrapping from the last menu to the first).
5. **Given** a dropdown is open, **When** the user presses `Esc`, **Then** the menu closes, focus returns to the editor, and no action is performed.
6. **Given** a dropdown is open and an item is highlighted, **When** the user presses `Enter`, **Then** the highlighted item's action is dispatched and the menu closes.
7. **Given** the menu bar is active, **When** the user presses an arrow or `Enter`, **Then** the editor buffer content and cursor position are unchanged by the navigation itself (only the selected action may change them).

---

### User Story 2 - Discover and activate plugin-contributed menus by keyboard (Priority: P2)

A user who has installed and consented to a menu plugin (e.g. the `word-count` reference
plugin, which contributes "Tools > Word Count") sees a new top-level "Tools" menu in the menu
bar, navigates to it with the keyboard, opens it, highlights "Word Count", presses Enter, and
sees the word count reported in the status bar.

**Why this priority**: This is the actual payload of issue #19 — the engine-side plumbing
(registry, sandboxed dispatch, consent, manager) is already complete and tested; the only
missing piece is rendering the plugin menus in the bar and routing selection to
`Action::PluginMenuActivated`. It depends on US1's navigation wiring.

**Independent Test**: Pre-consent the `word-count` fixture; launch the editor; confirm a
"Tools" top-level menu is present; keyboard-navigate to "Word Count", activate it, and confirm
the status bar shows a word count.

**Acceptance Scenarios**:

1. **Given** an active, consented menu plugin contributing `menu="Tools" item="Word Count" item_id="wc"`, **When** the editor renders the menu bar, **Then** a "Tools" top-level menu appears.
2. **Given** the "Tools" menu is open with "Word Count" highlighted, **When** the user presses `Enter`, **Then** `Action::PluginMenuActivated("word-count","wc")` is dispatched and the resulting message appears in the status bar.
3. **Given** two plugins each contribute items to a menu named "Tools", **When** the menu bar renders, **Then** a single "Tools" top-level menu lists the items from both plugins.
4. **Given** a plugin is disabled (in the Plugins manager or by `--no-plugins`), **When** the menu bar renders, **Then** that plugin contributes no menu items.

---

### User Story 3 - Predictable, DOS-faithful navigation semantics (Priority: P3)

Navigation behaves the way a DOS EDIT.COM user expects: arrow keys wrap at the ends, Left/Right
traverse all top-level menus (built-in and plugin) in a single ring, and the menu system never
"swallows" focus in a confusing state.

**Why this priority**: Polish/correctness that makes US1 and US2 feel right and prevents
dead-ends. It is lower priority because the core activation paths (US1/US2) deliver value even
with minimal navigation, but DOS-faithfulness (Constitution Principle I) requires it.

**Independent Test**: Open the rightmost menu, press `Right`, and confirm focus wraps to the
leftmost menu; open the leftmost, press `Left`, confirm it wraps to the rightmost (which is the
last plugin menu or Help, per the resolved placement). Confirm Up/Down wrap within every
dropdown including plugin dropdowns.

**Acceptance Scenarios**:

1. **Given** the rightmost top-level menu's dropdown is open, **When** the user presses `Right`, **Then** focus wraps to the leftmost menu's dropdown.
2. **Given** the leftmost top-level menu's dropdown is open, **When** the user presses `Left`, **Then** focus wraps to the rightmost menu's dropdown.
3. **Given** any dropdown (built-in or plugin) is open, **When** the user presses `Up`/`Down` past an end, **Then** the highlight wraps within that same dropdown.

---

### Edge Cases

- **Modal precedence**: When a modal overlay is active (Find/Replace, Save-As encoding dialog, Plugin Manager, consent prompt), menu-bar navigation keys MUST be handled by the modal, not the menu bar.
- **No plugin menus**: When no active plugin contributes menu items (none installed, all disabled, or `--no-plugins`), the menu bar renders identically to today — same labels, columns, and widths.
- **Plugin menu name collides with a built-in menu** (e.g. a plugin declares `menu="Edit"`): the resolved behavior is to **merge** the plugin's items into the matching built-in dropdown rather than create a duplicate top-level entry [Clarified 2026-06-19].
- **Empty dropdown**: A top-level menu with zero items cannot be opened into a selectable state; pressing Down on it is a no-op (not reachable for built-in menus, but guards against a plugin menu that contributes no items).
- **Toggle items**: Activating an item bound to a toggle action (e.g. "Soft Wrap (ext)") flips the toggle and the check-state indicator updates on the next render.
- **Plugin dispatch failure**: If activating a plugin item results in a sandbox timeout/error, the editor stays responsive and a warning is shown in the status bar (the dispatch layer already disables the offending plugin; this feature only surfaces the message).
- **Activation with no buffer / empty buffer**: Plugin actions that read buffer content receive an empty string; no crash.
- **Terminal too narrow to show all menus**: Top-level labels may be clipped by the existing render path; keyboard navigation still cycles through all menus regardless of visibility.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When the menu bar is active and a dropdown is open, the system MUST route Up/Down keys to move the highlighted item within that dropdown (with wrap-around at both ends) instead of moving the editor cursor.
- **FR-002**: When a top-level menu is highlighted with no dropdown open, the system MUST open that menu's dropdown on Down (first item highlighted) and on Up (last item highlighted).
- **FR-003**: When the menu bar is active, the system MUST route Left/Right keys to move focus between adjacent top-level menus in a single wrapping ring covering all built-in and plugin menus; if a dropdown was open, the adjacent menu opens its dropdown.
- **FR-004**: When a dropdown item is highlighted, the system MUST, on Enter, dispatch that item's associated action and then close the menu.
- **FR-005**: When the menu bar is active, the system MUST, on Esc, close the menu and return focus to the editor without performing any item action.
- **FR-006**: While the menu bar is active, the system MUST NOT allow navigation keys (arrows, Enter, Esc used for menu control) to modify buffer contents or move the editor cursor.
- **FR-007**: The system MUST render plugin-contributed top-level menus in the menu bar, positioned **between the "Options" menu and the "Help" menu** so that "Help" remains the rightmost menu (DOS-faithful). [Clarified 2026-06-19]
- **FR-008**: The system MUST group plugin menu items by their declared menu name so that multiple plugins contributing to the same menu name appear under one shared top-level menu. If a plugin's declared menu name matches a built-in menu name, its items MUST be appended to that built-in dropdown rather than creating a duplicate top-level menu. [Clarified 2026-06-19]
- **FR-009**: When a plugin menu item is activated, the system MUST dispatch `Action::PluginMenuActivated(plugin_id, item_id)` and display the message returned by the plugin dispatch in the status bar.
- **FR-010**: The system MUST only include menu items from active (consented and enabled) plugins; disabled plugins and the `--no-plugins` mode MUST contribute no menu items.
- **FR-011**: When no active plugin contributes menu items, the menu bar's rendered geometry (labels, column positions, widths) MUST be identical to the pre-feature behavior, and all existing menu-bar tests MUST pass unchanged.
- **FR-012**: The menu-bar navigation handling MUST take effect only when no higher-priority modal overlay (Find/Replace, encoding dialog, Plugin Manager, consent prompt) is active.
- **FR-013**: A plugin item whose dispatch fails (timeout/error) MUST NOT crash or hang the editor; a warning MUST be surfaced via the status bar (the existing dispatch layer is responsible for disabling the plugin).
- **FR-014**: All menu labels and plugin-provided menu/item strings rendered in the bar MUST be valid UTF-8 (already guaranteed by the plugin manifest/registry layer) and MUST be displayed using the editor's existing wide-character-aware rendering without column misalignment.
- **FR-015**: The system MUST provide a top-level menu-bar focus state with no dropdown open: pressing `F10` MUST activate the menu bar with the first top-level menu highlighted and no dropdown shown (DOS-faithful), from which Left/Right move the highlight and Down/Up open the highlighted menu's dropdown. Pressing `Alt+<letter>` (e.g. `Alt+F`) MUST open that menu's dropdown directly. This entry state is what makes FR-002 reachable. [Added by analysis remediation H1, 2026-06-19]

### Key Entities *(include if feature involves data)*

- **Top-level menu (composite)**: The ordered list of menus shown in the bar — the fixed built-in set (File, Edit, Search, View, Options, Help) plus zero or more plugin-derived menus inserted at the resolved position. Each has a display label and an ordered list of items.
- **Menu item**: A selectable entry with a display label and an associated action. For built-in items the action is a static `Action`; for plugin items the action is `Action::PluginMenuActivated(plugin_id, item_id)`.
- **Menu focus state**: Which top-level menu is highlighted and, if a dropdown is open, which item within it is highlighted (the existing `MenuBarState` state machine: Inactive / TopActive / DropDown).
- **Plugin menu item**: The registry record (`menu`, `item`, `item_id`, `plugin_id`, `position`) already produced by `PluginRegistry::menu_items()`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can open any built-in menu, move the highlight to any item, and activate it using only the keyboard, with the activated action taking effect — verified for 100% of built-in menu items.
- **SC-002**: A user can activate a plugin-contributed menu item (e.g. "Tools > Word Count") entirely by keyboard and see its result in the status bar.
- **SC-003**: With no plugins active, the menu bar renders identically to the previous release — zero changes to existing menu-bar geometry tests (0 regressions).
- **SC-004**: Menu navigation keystrokes register within the editor's standard interaction-latency budget (≤ 50 ms), so navigation feels instantaneous.
- **SC-005**: Every top-level menu (built-in and plugin) is reachable by Left/Right traversal, and every item within every dropdown is reachable by Up/Down — no dead-ends.
- **SC-006**: A plugin whose menu-action dispatch fails leaves the editor fully responsive, with a status-bar warning, and no buffer data loss.

## Assumptions

- **Placement** *(confirmed 2026-06-19)*: Plugin top-level menus are inserted **between Options and Help**, keeping Help rightmost (Constitution Principle I). This overrides the issue's "append after Help" wording and matches `tasks.md` T025 / `ROADMAP.md`.
- **Name-collision** *(confirmed 2026-06-19)*: A plugin menu whose `menu` name equals a built-in menu name (e.g. "Edit") merges its items into that built-in dropdown rather than creating a duplicate top-level menu.
- **Left/Right semantics** *(confirmed 2026-06-19)*: Left/Right traverse top-level menus; when a dropdown is open, the adjacent menu opens its dropdown automatically (DOS-faithful).
- **Keyboard focus**: This feature concerns keyboard activation only. Mouse-driven menu selection is out of scope for this feature (it may be addressed separately); existing mouse behavior is unchanged.
- **Enter key mapping**: The editor's existing "confirm/newline" key event is the activation key inside an open dropdown; no new physical keybinding is introduced beyond reusing arrows/Enter/Esc while the menu bar is active.
- **Menu-bar entry paths** *(remediation H1)*: `F10` enters the top-level highlight state (`TopActive`, no dropdown); `Alt+<letter>` opens the named menu's dropdown directly (`DropDown`). Today both paths jump straight to a dropdown; this feature adds the `F10`→top-level-highlight path so the FR-002 "highlighted, no dropdown" behavior is reachable and testable. No existing automated test asserts the current F10 behavior, so this change is safe.
- **Reuse of existing engine**: The plugin registry, sandboxed dispatch (`dispatch_menu_action` → `Action::PluginMenuActivated`), consent flow, and Plugins manager from feature 008 are reused unchanged; this feature adds only the navigation wiring and plugin-menu rendering.
- **Item ordering**: Within a shared plugin menu, items appear in plugin load order (the order `PluginRegistry::menu_items()` returns), optionally refined by the existing `position` field if present.
- **No new config or CLI flags** are introduced by this feature; `--no-plugins` already governs whether plugin menus appear.
