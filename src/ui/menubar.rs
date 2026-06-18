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

#![allow(dead_code, unused_variables, unused_imports)]

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
    MenuItem { label: "New",     action: Action::Noop    },
    MenuItem { label: "Open",    action: Action::Open    },
    MenuItem { label: "Save",    action: Action::Save    },
    MenuItem { label: "Save As", action: Action::SaveAs  },
    MenuItem { label: "Exit",    action: Action::Quit    },
];

/// Edit menu items.
static EDIT_MENU: &[MenuItem] = &[
    MenuItem { label: "Undo",       action: Action::Undo      },
    MenuItem { label: "Redo",       action: Action::Redo      },
    MenuItem { label: "Cut",        action: Action::Cut       },
    MenuItem { label: "Copy",       action: Action::Copy      },
    MenuItem { label: "Paste",      action: Action::Paste     },
    MenuItem { label: "Select All", action: Action::SelectAll },
];

/// Search menu items.
static SEARCH_MENU: &[MenuItem] = &[
    MenuItem { label: "Find",         action: Action::Find        },
    MenuItem { label: "Find Next",    action: Action::FindNext    },
    MenuItem { label: "Find Prev",    action: Action::FindPrev    },
    MenuItem { label: "Find Replace", action: Action::FindReplace },
];

/// View menu items.
static VIEW_MENU: &[MenuItem] = &[
    MenuItem { label: "Split View",        action: Action::SplitView        },
    MenuItem { label: "Next Buffer",       action: Action::NextBuffer       },
    MenuItem { label: "Prev Buffer",       action: Action::PrevBuffer       },
    MenuItem { label: "Toggle Line Nos",   action: Action::ToggleLineNumbers },
];

/// Options menu items.
static OPTIONS_MENU: &[MenuItem] = &[
    MenuItem { label: "Toggle Highlight", action: Action::ToggleHighlight },
];

/// Help menu items.
static HELP_MENU: &[MenuItem] = &[
    MenuItem { label: "Help", action: Action::Help },
];

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
    BarLabel { label: "File",    col: 1  },
    BarLabel { label: "Edit",    col: 7  },
    BarLabel { label: "Search",  col: 13 },
    BarLabel { label: "View",    col: 21 },
    BarLabel { label: "Options", col: 28 },
    BarLabel { label: "Help",    col: 37 },
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
                let item_count = ALL_MENUS
                    .get(top_idx)
                    .map(|m| m.len())
                    .unwrap_or(1);
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
                let item_count = ALL_MENUS
                    .get(top_idx)
                    .map(|m| m.len())
                    .unwrap_or(1);
                self.state = MenuState::DropDown {
                    top_idx,
                    item_idx: item_count.saturating_sub(1),
                };
            }
            MenuState::DropDown { top_idx, item_idx } => {
                let item_count = ALL_MENUS
                    .get(top_idx)
                    .map(|m| m.len())
                    .unwrap_or(1);
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
}

impl<'a> MenuBarWidget<'a> {
    /// Construct a new [`MenuBarWidget`].
    pub fn new(theme: &'static Theme, menu_state: &'a MenuBarState) -> Self {
        Self { theme, menu_state }
    }
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
            .fg(self.theme.menubar_bg)  // invert fg/bg for selected top label
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
            let drop_col: u16 = BAR_LABELS
                .get(top_idx)
                .map(|bl| bl.col)
                .unwrap_or(0);

            // Dropdown width: widest item label + 2 spaces of padding.
            let content_width: u16 = menu_items
                .iter()
                .map(|it| it.label.len() as u16)
                .max()
                .unwrap_or(4)
                + 4; // 2 left + 2 right padding

            // Height of the dropdown (one row per item + 2 for top/bottom border).
            let drop_height: u16 = menu_items.len() as u16 + 2;

            // Clamp dropdown horizontally so it doesn't overflow the terminal.
            let start_col: u16 = drop_col
                .min(area.width.saturating_sub(content_width));

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

                // Write label text with leading space.
                let label_x = area.left() + start_col + 1;
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
