//! Tasks T041–T047: DOS-style menu bar with interactive pull-down menus.
//!
//! This module provides:
//! - [`MenuState`] — the state machine driving menu open/close/navigation.
//! - [`MenuBarState`] — the public controller object stored in [`App`].
//! - [`MenuBarWidget`] — the ratatui [`Widget`] that renders the bar and any
//!   active dropdown.
//!
//! Non-interactive label-only rendering (T030) has been merged into this
//! module; T041–T047 add the full pull-down behaviour.

#![allow(dead_code, unused_variables)]

use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Clear, Widget},
};

use crate::input::keymap::Action;
use crate::ui::theme::Theme;

// ---------------------------------------------------------------------------
// MenuItem — a single item inside a pull-down menu
// ---------------------------------------------------------------------------

/// A single selectable item in a pull-down menu.
pub struct MenuItem {
    /// The text shown in the dropdown (e.g. `"New"`, `"Open…"`).
    pub label: &'static str,
    /// The editor action fired when this item is chosen.
    pub action: Action,
}

// ---------------------------------------------------------------------------
// Static menu definitions (T042–T047)
// ---------------------------------------------------------------------------

/// File menu items.
static FILE_MENU: &[MenuItem] = &[
    MenuItem {
        label: "New",
        action: Action::Noop,
    },
    MenuItem {
        label: "Open",
        action: Action::Open,
    },
    MenuItem {
        label: "Save",
        action: Action::Save,
    },
    MenuItem {
        label: "Save As",
        action: Action::SaveAs,
    },
    MenuItem {
        label: "Save As Encoding...",
        action: Action::SaveAsEncoding,
    },
    MenuItem {
        label: "Exit",
        action: Action::Quit,
    },
];

/// Edit menu items.
static EDIT_MENU: &[MenuItem] = &[
    MenuItem {
        label: "Undo",
        action: Action::Undo,
    },
    MenuItem {
        label: "Redo",
        action: Action::Redo,
    },
    MenuItem {
        label: "Cut",
        action: Action::Cut,
    },
    MenuItem {
        label: "Copy",
        action: Action::Copy,
    },
    MenuItem {
        label: "Paste",
        action: Action::Paste,
    },
    MenuItem {
        label: "Select All",
        action: Action::SelectAll,
    },
];

/// Search menu items.
static SEARCH_MENU: &[MenuItem] = &[
    MenuItem {
        label: "Find",
        action: Action::Find,
    },
    MenuItem {
        label: "Find Next",
        action: Action::FindNext,
    },
    MenuItem {
        label: "Find Prev",
        action: Action::FindPrev,
    },
    MenuItem {
        label: "Find Replace",
        action: Action::FindReplace,
    },
];

/// View menu items.
static VIEW_MENU: &[MenuItem] = &[
    MenuItem {
        label: "Split View",
        action: Action::SplitView,
    },
    MenuItem {
        label: "Next Buffer",
        action: Action::NextBuffer,
    },
    MenuItem {
        label: "Prev Buffer",
        action: Action::PrevBuffer,
    },
    MenuItem {
        label: "Toggle Line Nos",
        action: Action::ToggleLineNumbers,
    },
    MenuItem {
        label: "Soft Wrap (ext)",
        action: Action::ToggleSoftWrap,
    },
];

/// Options menu items.
static OPTIONS_MENU: &[MenuItem] = &[
    MenuItem {
        label: "Toggle Highlight",
        action: Action::ToggleHighlight,
    },
    MenuItem {
        label: "Plugins...",
        action: Action::OpenPluginManager,
    },
];

/// Help menu items.
static HELP_MENU: &[MenuItem] = &[MenuItem {
    label: "Help",
    action: Action::Help,
}];

/// All six menus in display order. Index matches `MenuBarState::open_menu(idx)`.
static ALL_MENUS: &[&[MenuItem]] = &[
    FILE_MENU,
    EDIT_MENU,
    SEARCH_MENU,
    VIEW_MENU,
    OPTIONS_MENU,
    HELP_MENU,
];

// ---------------------------------------------------------------------------
// Top-level bar label definitions (T030 layout preserved)
// ---------------------------------------------------------------------------

/// A top-level menu label with its column position in the bar.
struct BarLabel {
    label: &'static str,
    /// 0-based column within the bar content (after the leading space).
    col: u16,
}

/// The ordered list of top-level menu labels and their column positions.
///
/// Positions match the EDIT.COM layout (0-based within the bar).
static BAR_LABELS: &[BarLabel] = &[
    BarLabel {
        label: "File",
        col: 1,
    },
    BarLabel {
        label: "Edit",
        col: 7,
    },
    BarLabel {
        label: "Search",
        col: 13,
    },
    BarLabel {
        label: "View",
        col: 21,
    },
    BarLabel {
        label: "Options",
        col: 28,
    },
    BarLabel {
        label: "Help",
        col: 37,
    },
];

// ---------------------------------------------------------------------------
// MenuState — T041
// ---------------------------------------------------------------------------

/// State machine that tracks whether the menu bar is active and which item is
/// highlighted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuState {
    /// Menu bar is closed; normal editing mode.
    Inactive,
    /// Menu bar is focused at the top level; `top_idx` is the highlighted menu.
    TopActive(usize),
    /// A dropdown is open for `top_idx`; `item_idx` is the highlighted item.
    DropDown { top_idx: usize, item_idx: usize },
}

// ---------------------------------------------------------------------------
// MenuBarState — T041
// ---------------------------------------------------------------------------

/// Controller for the menu bar state machine.
///
/// Stored as a field on [`App`] and mutated in response to key/mouse events.
pub struct MenuBarState {
    /// Current phase of the menu state machine.
    pub state: MenuState,
}

impl MenuBarState {
    /// Construct an idle [`MenuBarState`].
    pub fn new() -> Self {
        Self {
            state: MenuState::Inactive,
        }
    }

    /// Open (or switch to) the top-level menu at index `idx`.
    ///
    /// Transitions: any state → `DropDown { top_idx: idx, item_idx: 0 }`.
    pub fn open_menu(&mut self, idx: usize) {
        let clamped = idx.min(ALL_MENUS.len().saturating_sub(1));
        self.state = MenuState::DropDown {
            top_idx: clamped,
            item_idx: 0,
        };
    }

    /// Close the active menu/dropdown and return to inactive editing mode.
    pub fn close_menu(&mut self) {
        self.state = MenuState::Inactive;
    }

    /// Move the dropdown highlight one item downward (wrapping).
    ///
    /// If the menu is `Inactive` this is a no-op.
    /// If it is `TopActive`, transitions to `DropDown` at item 0.
    pub fn navigate_down(&mut self) {
        match self.state {
            MenuState::Inactive => {}
            MenuState::TopActive(top_idx) => {
                self.state = MenuState::DropDown {
                    top_idx,
                    item_idx: 0,
                };
            }
            MenuState::DropDown { top_idx, item_idx } => {
                let item_count = ALL_MENUS.get(top_idx).map(|m| m.len()).unwrap_or(1);
                self.state = MenuState::DropDown {
                    top_idx,
                    item_idx: (item_idx + 1) % item_count,
                };
            }
        }
    }

    /// Move the dropdown highlight one item upward (wrapping).
    ///
    /// If the menu is `Inactive` this is a no-op.
    /// If it is `TopActive`, transitions to `DropDown` at the last item.
    pub fn navigate_up(&mut self) {
        match self.state {
            MenuState::Inactive => {}
            MenuState::TopActive(top_idx) => {
                let item_count = ALL_MENUS.get(top_idx).map(|m| m.len()).unwrap_or(1);
                self.state = MenuState::DropDown {
                    top_idx,
                    item_idx: item_count.saturating_sub(1),
                };
            }
            MenuState::DropDown { top_idx, item_idx } => {
                let item_count = ALL_MENUS.get(top_idx).map(|m| m.len()).unwrap_or(1);
                let new_idx = if item_idx == 0 {
                    item_count.saturating_sub(1)
                } else {
                    item_idx - 1
                };
                self.state = MenuState::DropDown {
                    top_idx,
                    item_idx: new_idx,
                };
            }
        }
    }

    /// Activate the currently highlighted item and return the associated
    /// [`Action`], or `None` if no dropdown item is highlighted.
    ///
    /// After returning an action the menu is closed.
    pub fn select_item(&mut self) -> Option<Action> {
        if let MenuState::DropDown { top_idx, item_idx } = self.state {
            let action = ALL_MENUS
                .get(top_idx)
                .and_then(|menu| menu.get(item_idx))
                .map(|item| item.action.clone());
            self.state = MenuState::Inactive;
            action
        } else {
            None
        }
    }

    /// Return `true` when any menu/dropdown is active.
    pub fn is_active(&self) -> bool {
        self.state != MenuState::Inactive
    }
}

impl Default for MenuBarState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// MenuBarWidget — T030 + T041–T047
// ---------------------------------------------------------------------------

/// Widget that renders the top menu bar row and any active pull-down dropdown.
pub struct MenuBarWidget<'a> {
    /// The active color theme.
    pub theme: &'static Theme,
    /// Current menu state (drives dropdown rendering).
    pub menu_state: &'a MenuBarState,
    /// Runtime check-states for toggleable actions.
    ///
    /// Each entry `(action, checked)` controls whether the dropdown item
    /// whose `action` matches renders with a `✓ ` prefix (`checked = true`)
    /// or a `  ` filler (`checked = false`). An empty slice produces
    /// identical rendering to pre-feature behavior.
    pub toggle_states: &'a [(Action, bool)],
}

impl<'a> MenuBarWidget<'a> {
    /// Construct a new [`MenuBarWidget`].
    pub fn new(
        theme: &'static Theme,
        menu_state: &'a MenuBarState,
        toggle_states: &'a [(Action, bool)],
    ) -> Self {
        Self {
            theme,
            menu_state,
            toggle_states,
        }
    }
}

/// Return `Some(true)` if `action` is in `toggle_states` and checked,
/// `Some(false)` if present but unchecked, or `None` if absent.
fn lookup_checked(toggle_states: &[(Action, bool)], action: &Action) -> Option<bool> {
    toggle_states
        .iter()
        .find(|(a, _)| a == action)
        .map(|(_, checked)| *checked)
}

impl<'a> Widget for MenuBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let bar_style = Style::default()
            .fg(self.theme.menubar_fg)
            .bg(self.theme.menubar_bg);

        let selected_style = Style::default()
            .fg(self.theme.menubar_bg) // invert fg/bg for selected top label
            .bg(self.theme.menu_selected_bg);

        let y = area.top();

        // Fill the entire row with the menubar background.
        for x in area.left()..area.right() {
            buf.get_mut(x, y).set_style(bar_style).set_char(' ');
        }

        // Determine which top-level index (if any) should appear selected.
        let active_top: Option<usize> = match &self.menu_state.state {
            MenuState::Inactive => None,
            MenuState::TopActive(idx) => Some(*idx),
            MenuState::DropDown { top_idx, .. } => Some(*top_idx),
        };

        // Draw each top-level menu label.
        for (menu_idx, bar_label) in BAR_LABELS.iter().enumerate() {
            let base_x = area.left() + bar_label.col;
            let style = if active_top == Some(menu_idx) {
                selected_style
            } else {
                bar_style
            };

            for (i, ch) in bar_label.label.chars().enumerate() {
                let x = base_x + i as u16;
                if x >= area.right() {
                    break;
                }
                buf.get_mut(x, y).set_style(style).set_char(ch);
            }
        }

        // ── Render dropdown (T047) ─────────────────────────────────────────

        if let MenuState::DropDown { top_idx, item_idx } = &self.menu_state.state {
            let top_idx = *top_idx;
            let item_idx = *item_idx;

            let menu_items = match ALL_MENUS.get(top_idx) {
                Some(m) => m,
                None => return,
            };

            // Compute the dropdown column: align with the bar label's start col,
            // clamping so the dropdown doesn't run off the right edge.
            let drop_col: u16 = BAR_LABELS.get(top_idx).map(|bl| bl.col).unwrap_or(0);

            // T005: Is this a checkable-aware dropdown?
            // True when at least one item's action appears in toggle_states.
            let has_checkable = menu_items
                .iter()
                .any(|item| lookup_checked(self.toggle_states, &item.action).is_some());

            // T006: Dropdown width — add 2 extra chars when checkable to
            // reserve the prefix column (✓ or space + space).
            let content_width: u16 = menu_items
                .iter()
                .map(|it| it.label.len() as u16)
                .max()
                .unwrap_or(4)
                + if has_checkable { 6 } else { 4 };

            // T007: Label starts 1 col from dropdown edge normally,
            // or 3 cols when checkable (1 border space + 2-char prefix).
            let label_offset: u16 = if has_checkable { 3 } else { 1 };

            // Height of the dropdown (one row per item + 2 for top/bottom border).
            let drop_height: u16 = menu_items.len() as u16 + 2;

            // Clamp dropdown horizontally so it doesn't overflow the terminal.
            let start_col: u16 = drop_col.min(area.width.saturating_sub(content_width));

            // The dropdown starts on row 1 (row 0 is the menu bar itself),
            // but only if there are rows below the bar area.
            // We draw into the global buffer directly; area is the full frame
            // passed by the caller so rows below the bar are accessible.
            let drop_y_start = y + 1;

            let dropdown_style = Style::default()
                .fg(self.theme.menubar_fg)
                .bg(self.theme.menubar_bg);

            let selected_item_style = Style::default()
                .fg(self.theme.menubar_bg)
                .bg(self.theme.menu_selected_bg);

            // Draw each dropdown row.
            for (row_offset, item) in menu_items.iter().enumerate() {
                let row_y = drop_y_start + row_offset as u16;

                // Stop drawing if we've gone past the buffer height.
                // (buf height is the full frame height)
                // We use the area rect's bottom as the safe limit since
                // MenuBarWidget only gets the menubar area (1 row).
                // To draw below the menubar area we need to write directly
                // to the buffer without the area boundary check.
                // ratatui's Buffer::get_mut does not enforce Rect bounds,
                // so we guard manually.
                let item_style = if row_offset == item_idx {
                    selected_item_style
                } else {
                    dropdown_style
                };

                // Fill entire dropdown row background.
                for col_offset in 0..content_width {
                    let cx = area.left() + start_col + col_offset;
                    // Guard: don't write past the buffer dimensions.
                    // We can't easily query buf.area here, so we use the
                    // terminal width as a proxy (area.right() for the bar
                    // is the full terminal width when the layout fills the frame).
                    if cx >= area.right() {
                        break;
                    }
                    buf.get_mut(cx, row_y).set_style(item_style).set_char(' ');
                }

                // T007: Write the 2-char prefix column for checkable menus.
                // The background fill already placed spaces everywhere; we only
                // need to overwrite position 1 (start_col+1) with '✓' if checked.
                if has_checkable {
                    let cx = area.left() + start_col + 1;
                    if cx < area.right() {
                        let prefix_char = match lookup_checked(self.toggle_states, &item.action) {
                            Some(true) => '✓',
                            _ => ' ',
                        };
                        buf.get_mut(cx, row_y)
                            .set_style(item_style)
                            .set_char(prefix_char);
                    }
                    // start_col+2 stays ' ' from the background fill above.
                }

                // Write label text starting at label_offset from dropdown edge.
                let label_x = area.left() + start_col + label_offset;
                for (i, ch) in item.label.chars().enumerate() {
                    let cx = label_x + i as u16;
                    if cx >= area.right() {
                        break;
                    }
                    buf.get_mut(cx, row_y).set_style(item_style).set_char(ch);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests — T009–T015
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::theme_by_name;
    use ratatui::{buffer::Buffer, layout::Rect};

    // T009: shared helpers — open a specific menu dropdown for rendering.
    fn view_open() -> MenuBarState {
        let mut s = MenuBarState::new();
        s.open_menu(3); // View is index 3 in ALL_MENUS / BAR_LABELS
        s
    }

    fn file_open() -> MenuBarState {
        let mut s = MenuBarState::new();
        s.open_menu(0); // File is index 0
        s
    }

    fn options_open() -> MenuBarState {
        let mut s = MenuBarState::new();
        s.open_menu(4); // Options is index 4
        s
    }

    // Render a MenuBarWidget into a fresh buffer of the given size.
    fn render_into(
        state: &MenuBarState,
        toggle_states: &[(Action, bool)],
        width: u16,
        height: u16,
    ) -> Buffer {
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
        let widget = MenuBarWidget::new(theme_by_name("classic"), state, toggle_states);
        widget.render(buf.area, &mut buf);
        buf
    }

    // T010: checkmark shown when toggle true
    //
    // View dropdown geometry (checkable, toggle_states = [(ToggleSoftWrap, true)]):
    //   drop_col = 21, content_width = 15+6 = 21
    //   start_col = min(21, 40-21) = min(21,19) = 19
    //   "Soft Wrap (ext)" is item index 4 → row_y = drop_y_start+4 = 1+4 = 5
    //   prefix col = 0+19+1 = 20
    #[test]
    fn test_checkmark_shown_when_toggle_true() {
        let state = view_open();
        let buf = render_into(&state, &[(Action::ToggleSoftWrap, true)], 40, 8);
        assert_eq!(
            buf.get(20, 5).symbol(),
            "✓",
            "Soft Wrap item (col 20, row 5) must show '✓' when soft_wrap=true"
        );
    }

    // T011: no checkmark when toggle false
    //
    // Same geometry as T010; toggle = false → prefix = ' '.
    #[test]
    fn test_no_checkmark_when_toggle_false() {
        let state = view_open();
        let buf = render_into(&state, &[(Action::ToggleSoftWrap, false)], 40, 8);
        assert_eq!(
            buf.get(20, 5).symbol(),
            " ",
            "Soft Wrap item prefix (col 20, row 5) must be ' ' when soft_wrap=false"
        );
    }

    // T012: non-toggle menu unaffected
    //
    // File dropdown geometry (no matching toggle → not checkable):
    //   drop_col = 1, content_width = 19+4 = 23 (not 25)
    //   start_col = min(1, 40-23) = 1
    //   label_offset = 1 → "New" starts at col 1+1 = 2 (not 1+3 = 4)
    //   Item 0 "New" → row_y = 1
    #[test]
    fn test_non_toggle_menu_unaffected() {
        let state = file_open();
        let buf = render_into(&state, &[(Action::ToggleSoftWrap, true)], 40, 10);

        // (a) No '✓' anywhere in the dropdown area
        for row in 1u16..10 {
            for col in 0u16..40 {
                assert_ne!(
                    buf.get(col, row).symbol(),
                    "✓",
                    "unexpected '✓' at ({col},{row}) — File menu must not be checkable-aware"
                );
            }
        }

        // (b) content_width = 23 (not 25): "New" label starts at col 2, not col 4
        assert_eq!(
            buf.get(2, 1).symbol(),
            "N",
            "'N' of 'New' must be at col 2 (label_offset=1, non-checkable)"
        );
        assert_ne!(
            buf.get(4, 1).symbol(),
            "N",
            "col 4 must NOT be 'N' — that would indicate wrong checkable expansion"
        );
    }

    // T013: label alignment consistent across all items in a checkable menu
    //
    // View dropdown, ToggleSoftWrap=true:
    //   start_col=19, label_offset=3, label_x=22 for ALL five items.
    //   Items 0-3 (not in toggle_states) → prefix at col 20 = ' '.
    //   Item 4 (ToggleSoftWrap=true) → prefix at col 20 = '✓'.
    #[test]
    fn test_label_alignment_in_checkable_menu() {
        let state = view_open();
        let buf = render_into(&state, &[(Action::ToggleSoftWrap, true)], 40, 8);

        // All five View items must start their label at col 22.
        let expected_first_chars = [('S', 1u16), ('N', 2), ('P', 3), ('T', 4), ('S', 5)];
        for (ch, row_y) in expected_first_chars {
            assert_eq!(
                buf.get(22, row_y).symbol(),
                ch.to_string(),
                "label first char '{ch}' must be at col 22 (row {row_y})"
            );
        }

        // Items 0-3 (not in toggle_states) get ' ' at the prefix column (col 20).
        for row_y in 1u16..5 {
            assert_eq!(
                buf.get(20, row_y).symbol(),
                " ",
                "non-toggle item at row {row_y} must have ' ' prefix at col 20 (FR-003)"
            );
        }
    }

    // T013b: second action also shows checkmark (FR-007 generality proof)
    //
    // Options dropdown, ToggleHighlight=true:
    //   drop_col=28, content_width=16+6=22
    //   start_col = min(28, 60-22) = min(28,38) = 28
    //   "Toggle Highlight" is item 0 → row_y = 1
    //   prefix col = 0+28+1 = 29
    #[test]
    fn test_second_action_also_shows_checkmark() {
        let state = options_open();
        let buf = render_into(&state, &[(Action::ToggleHighlight, true)], 60, 4);
        assert_eq!(
            buf.get(29, 1).symbol(),
            "✓",
            "ToggleHighlight=true must show '✓' at col 29 (FR-007: mechanism is action-agnostic)"
        );
    }

    // T014: empty toggle_states — no regression in existing behavior
    //
    // View dropdown, toggle_states = &[]:
    //   has_checkable=false → content_width=15+4=19
    //   start_col = min(21, 40-19) = 21
    //   label_offset = 1 → "Split View" 'S' at col 22 (same as checkable case here,
    //   but no checkmark anywhere and no width expansion)
    #[test]
    fn test_empty_toggle_states_no_regression() {
        let state = view_open();
        let buf = render_into(&state, &[], 40, 8);

        // No '✓' anywhere in the buffer
        for row in 0u16..8 {
            for col in 0u16..40 {
                assert_ne!(
                    buf.get(col, row).symbol(),
                    "✓",
                    "no '✓' expected anywhere with empty toggle_states (col {col}, row {row})"
                );
            }
        }

        // Label alignment still works (start_col=21, label_offset=1, label_x=22)
        assert_eq!(
            buf.get(22, 1).symbol(),
            "S",
            "'S' of 'Split View' must be at col 22 with non-checkable layout"
        );
    }

    // T015: config-persisted initial state renders correctly (US3)
    //
    // Simulate config-loaded App::soft_wrap=true: pass toggle_states with
    // ToggleSoftWrap=true without any prior in-session toggle call.
    // The widget must reflect the persisted state on first render.
    #[test]
    fn test_initial_soft_wrap_state_from_config() {
        let state = view_open();
        // toggle_states derived from config-loaded soft_wrap=true
        let buf = render_into(&state, &[(Action::ToggleSoftWrap, true)], 40, 8);
        assert_eq!(
            buf.get(20, 5).symbol(),
            "✓",
            "config-persisted soft_wrap=true must show '✓' immediately on first render (US3)"
        );
    }
}
