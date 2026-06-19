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
use crate::plugin::types::PluginMenuItem;
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
        action: Action::New,
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
static HELP_MENU: &[MenuItem] = &[
    MenuItem {
        label: "Help",
        action: Action::Help,
    },
    MenuItem {
        label: "About",
        action: Action::About,
    },
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
// Resolved menu model (Feature 009) — the composite of built-in + plugin menus
// ---------------------------------------------------------------------------

/// A single resolved, displayable dropdown item.
///
/// Owned `String` label (built-in labels are cloned from the static `&'static str`
/// slices; plugin labels are runtime strings) so built-in and plugin items can
/// live in one list. `action` is a static [`Action`] for built-ins and
/// [`Action::PluginMenuActivated`] for plugin items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedItem {
    pub label: String,
    pub action: Action,
}

/// A single resolved top-level menu as it appears in the bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMenu {
    pub label: String,
    pub items: Vec<ResolvedItem>,
}

/// Build the ordered composite of top-level menus shown in the bar.
///
/// Rules (see `specs/009-menu-bar-activation/data-model.md`):
/// 1. Seed from the six built-in menus, in order.
/// 2. Plugin items whose `menu` name matches a built-in menu are appended to
///    that built-in dropdown (merge on name collision).
/// 3. Remaining plugin items are grouped into new top-level menus inserted
///    immediately before Help (so Help stays rightmost — DOS-faithful).
/// 4. Items within a group are ordered by `position` (ascending) when set,
///    otherwise by stable load order.
///
/// `resolve_menus(&[])` returns exactly the built-in set (parity invariant for
/// FR-011 / SC-003).
pub fn resolve_menus(plugin_items: &[PluginMenuItem]) -> Vec<ResolvedMenu> {
    // 1. Seed from built-ins.
    let mut menus: Vec<ResolvedMenu> = BAR_LABELS
        .iter()
        .zip(ALL_MENUS.iter())
        .map(|(bl, items)| ResolvedMenu {
            label: bl.label.to_string(),
            items: items
                .iter()
                .map(|it| ResolvedItem {
                    label: it.label.to_string(),
                    action: it.action.clone(),
                })
                .collect(),
        })
        .collect();

    // Parity fast-path: no plugin menus → identical to built-ins.
    if plugin_items.is_empty() {
        return menus;
    }

    // Group plugin items by `menu` name, preserving first-appearance order of
    // new menu names.
    let mut groups: Vec<(String, Vec<&PluginMenuItem>)> = Vec::new();
    for pi in plugin_items {
        if let Some(entry) = groups.iter_mut().find(|(name, _)| *name == pi.menu) {
            entry.1.push(pi);
        } else {
            groups.push((pi.menu.clone(), vec![pi]));
        }
    }

    for (name, mut items) in groups {
        // Stable sort: items with `position` first (ascending), then load order.
        items.sort_by(|a, b| match (a.position, b.position) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        let resolved_items: Vec<ResolvedItem> = items
            .iter()
            .map(|pi| ResolvedItem {
                label: pi.item.clone(),
                action: Action::PluginMenuActivated(pi.plugin_id.clone(), pi.item_id.clone()),
            })
            .collect();

        if let Some(existing) = menus.iter_mut().find(|m| m.label == name) {
            // Name collision with a built-in menu → merge items.
            existing.items.extend(resolved_items);
        } else {
            // New plugin menu → insert immediately before Help (current last).
            let insert_at = menus.len().saturating_sub(1);
            menus.insert(
                insert_at,
                ResolvedMenu {
                    label: name,
                    items: resolved_items,
                },
            );
        }
    }

    menus
}

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

    /// Activate the menu bar at the first top-level menu **without** opening a
    /// dropdown (the F10 / DOS-faithful entry path). From here Left/Right move
    /// the highlight and Up/Down open the highlighted menu's dropdown.
    pub fn activate_bar(&mut self) {
        self.state = MenuState::TopActive(0);
    }

    /// Open (or switch to) the top-level menu at index `idx`, showing its
    /// dropdown directly (the Alt+&lt;letter&gt; entry path).
    ///
    /// `idx` is clamped against the resolved menu count.
    /// Transitions: any state → `DropDown { top_idx: idx, item_idx: 0 }`.
    pub fn open_menu(&mut self, idx: usize, menus: &[ResolvedMenu]) {
        if menus.is_empty() {
            return;
        }
        let clamped = idx.min(menus.len().saturating_sub(1));
        self.state = MenuState::DropDown {
            top_idx: clamped,
            item_idx: 0,
        };
    }

    /// Close the active menu/dropdown and return to inactive editing mode.
    pub fn close_menu(&mut self) {
        self.state = MenuState::Inactive;
    }

    /// Number of items in the resolved menu at `idx` (0 if out of range).
    fn item_count(menus: &[ResolvedMenu], idx: usize) -> usize {
        menus.get(idx).map(|m| m.items.len()).unwrap_or(0)
    }

    /// Move the dropdown highlight one item downward (wrapping).
    ///
    /// `Inactive` → no-op. `TopActive` → open the dropdown at item 0 (if the
    /// menu has items). `DropDown` → next item, wrapping.
    pub fn navigate_down(&mut self, menus: &[ResolvedMenu]) {
        match self.state {
            MenuState::Inactive => {}
            MenuState::TopActive(top_idx) => {
                if Self::item_count(menus, top_idx) > 0 {
                    self.state = MenuState::DropDown {
                        top_idx,
                        item_idx: 0,
                    };
                }
            }
            MenuState::DropDown { top_idx, item_idx } => {
                let count = Self::item_count(menus, top_idx);
                if count == 0 {
                    return;
                }
                self.state = MenuState::DropDown {
                    top_idx,
                    item_idx: (item_idx + 1) % count,
                };
            }
        }
    }

    /// Move the dropdown highlight one item upward (wrapping).
    ///
    /// `Inactive` → no-op. `TopActive` → open the dropdown at the last item (if
    /// the menu has items). `DropDown` → previous item, wrapping.
    pub fn navigate_up(&mut self, menus: &[ResolvedMenu]) {
        match self.state {
            MenuState::Inactive => {}
            MenuState::TopActive(top_idx) => {
                let count = Self::item_count(menus, top_idx);
                if count > 0 {
                    self.state = MenuState::DropDown {
                        top_idx,
                        item_idx: count - 1,
                    };
                }
            }
            MenuState::DropDown { top_idx, item_idx } => {
                let count = Self::item_count(menus, top_idx);
                if count == 0 {
                    return;
                }
                let new_idx = if item_idx == 0 {
                    count - 1
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

    /// Move focus to the previous top-level menu (wrapping over the full ring).
    ///
    /// From `TopActive` the highlight moves without opening a dropdown; from
    /// `DropDown` the adjacent menu's dropdown opens (item 0) — DOS-faithful.
    pub fn navigate_left(&mut self, menus: &[ResolvedMenu]) {
        let n = menus.len();
        if n == 0 {
            return;
        }
        match self.state {
            MenuState::Inactive => {}
            MenuState::TopActive(t) => {
                self.state = MenuState::TopActive((t + n - 1) % n);
            }
            MenuState::DropDown { top_idx, .. } => {
                self.state = MenuState::DropDown {
                    top_idx: (top_idx + n - 1) % n,
                    item_idx: 0,
                };
            }
        }
    }

    /// Move focus to the next top-level menu (wrapping over the full ring).
    ///
    /// From `TopActive` the highlight moves without opening a dropdown; from
    /// `DropDown` the adjacent menu's dropdown opens (item 0) — DOS-faithful.
    pub fn navigate_right(&mut self, menus: &[ResolvedMenu]) {
        let n = menus.len();
        if n == 0 {
            return;
        }
        match self.state {
            MenuState::Inactive => {}
            MenuState::TopActive(t) => {
                self.state = MenuState::TopActive((t + 1) % n);
            }
            MenuState::DropDown { top_idx, .. } => {
                self.state = MenuState::DropDown {
                    top_idx: (top_idx + 1) % n,
                    item_idx: 0,
                };
            }
        }
    }

    /// Activate the currently highlighted dropdown item and return its
    /// [`Action`], or `None` if no dropdown item is highlighted (e.g. the bar
    /// is only in `TopActive`). After returning an action the menu is closed.
    pub fn select_item(&mut self, menus: &[ResolvedMenu]) -> Option<Action> {
        if let MenuState::DropDown { top_idx, item_idx } = self.state {
            let action = menus
                .get(top_idx)
                .and_then(|menu| menu.items.get(item_idx))
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
    /// The resolved composite menu list (built-in + plugin) to render. Built by
    /// the caller via [`resolve_menus`]. With no plugin menus this equals the
    /// built-in set and reproduces the pre-feature geometry exactly.
    pub menus: &'a [ResolvedMenu],
}

impl<'a> MenuBarWidget<'a> {
    /// Construct a new [`MenuBarWidget`].
    pub fn new(
        theme: &'static Theme,
        menu_state: &'a MenuBarState,
        toggle_states: &'a [(Action, bool)],
        menus: &'a [ResolvedMenu],
    ) -> Self {
        Self {
            theme,
            menu_state,
            toggle_states,
            menus,
        }
    }
}

/// Compute the 0-based bar column for each top-level label.
///
/// When the resolved set is exactly the built-in menus (no plugin menus), the
/// exact historical columns are reproduced (FR-011 / SC-003 parity). Otherwise
/// labels are laid out sequentially with a 2-space gap.
pub fn bar_label_columns(menus: &[ResolvedMenu]) -> Vec<u16> {
    let is_builtin_only = menus.len() == BAR_LABELS.len()
        && menus
            .iter()
            .zip(BAR_LABELS.iter())
            .all(|(m, b)| m.label == b.label);
    if is_builtin_only {
        return BAR_LABELS.iter().map(|b| b.col).collect();
    }
    let mut cols = Vec::with_capacity(menus.len());
    let mut next = 1u16;
    for m in menus {
        cols.push(next);
        next += m.label.chars().count() as u16 + 2;
    }
    cols
}

/// Geometry of an open dropdown, shared by the renderer and mouse hit-testing
/// so the drawn box and the clickable region can never drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropdownLayout {
    /// 0-based column where the dropdown box starts (clamped to the terminal).
    pub start_col: u16,
    /// Total width of the dropdown box (background fill width).
    pub content_width: u16,
    /// Column offset from the box edge at which item labels begin.
    pub label_offset: u16,
    /// Whether this dropdown reserves a check-mark prefix column.
    pub has_checkable: bool,
}

/// Compute the [`DropdownLayout`] for the menu opened at `drop_col`.
///
/// `term_width` is the full terminal width (used to clamp the box so it never
/// overflows the right edge). This mirrors the geometry the renderer applies.
pub fn dropdown_layout(
    menu: &ResolvedMenu,
    drop_col: u16,
    toggle_states: &[(Action, bool)],
    term_width: u16,
) -> DropdownLayout {
    let has_checkable = menu
        .items
        .iter()
        .any(|item| lookup_checked(toggle_states, &item.action).is_some());

    let content_width: u16 = menu
        .items
        .iter()
        .map(|it| it.label.len() as u16)
        .max()
        .unwrap_or(4)
        + if has_checkable { 6 } else { 4 };

    let label_offset: u16 = if has_checkable { 3 } else { 1 };
    let start_col: u16 = drop_col.min(term_width.saturating_sub(content_width));

    DropdownLayout {
        start_col,
        content_width,
        label_offset,
        has_checkable,
    }
}

/// The result of hit-testing a mouse click against the menu bar and any open
/// dropdown. Coordinates are 0-based terminal cells; the menu bar is row 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuHit {
    /// A top-level menu label on row 0 was clicked.
    TopLevel(usize),
    /// A dropdown item was clicked.
    Item { top_idx: usize, item_idx: usize },
    /// The click landed outside any menu label or open dropdown.
    Outside,
}

/// Hit-test a click at (`col`, `row`) against the resolved menus and current
/// `state`, using the same geometry the widget renders with.
pub fn hit_test_menu(
    menus: &[ResolvedMenu],
    state: &MenuState,
    toggle_states: &[(Action, bool)],
    term_width: u16,
    col: u16,
    row: u16,
) -> MenuHit {
    let columns = bar_label_columns(menus);

    // Row 0: the top-level menu bar.
    if row == 0 {
        for (idx, menu) in menus.iter().enumerate() {
            let start = columns.get(idx).copied().unwrap_or(0);
            let width = menu.label.chars().count() as u16;
            if col >= start && col < start + width {
                return MenuHit::TopLevel(idx);
            }
        }
        return MenuHit::Outside;
    }

    // Rows 1+: an open dropdown, if any.
    if let MenuState::DropDown { top_idx, .. } = state {
        if let Some(menu) = menus.get(*top_idx) {
            let drop_col = columns.get(*top_idx).copied().unwrap_or(0);
            let layout = dropdown_layout(menu, drop_col, toggle_states, term_width);
            let item_count = menu.items.len() as u16;
            // Items render on rows 1..=item_count (drop starts at row 1).
            if row >= 1
                && row <= item_count
                && col >= layout.start_col
                && col < layout.start_col + layout.content_width
            {
                return MenuHit::Item {
                    top_idx: *top_idx,
                    item_idx: (row - 1) as usize,
                };
            }
        }
    }

    MenuHit::Outside
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

        // Resolved top-level columns (built-in parity preserved when no plugins).
        let columns = bar_label_columns(self.menus);

        // Draw each top-level menu label.
        for (menu_idx, menu) in self.menus.iter().enumerate() {
            let col = columns.get(menu_idx).copied().unwrap_or(0);
            let base_x = area.left() + col;
            let style = if active_top == Some(menu_idx) {
                selected_style
            } else {
                bar_style
            };

            for (i, ch) in menu.label.chars().enumerate() {
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

            let menu_items: &[ResolvedItem] = match self.menus.get(top_idx) {
                Some(m) => &m.items,
                None => return,
            };

            // Compute the dropdown column: align with the bar label's start col.
            let drop_col: u16 = columns.get(top_idx).copied().unwrap_or(0);

            // Shared geometry — identical to what mouse hit-testing uses, so the
            // drawn box and the clickable region always agree (T005–T007).
            let menu = match self.menus.get(top_idx) {
                Some(m) => m,
                None => return,
            };
            let DropdownLayout {
                start_col,
                content_width,
                label_offset,
                has_checkable,
            } = dropdown_layout(menu, drop_col, self.toggle_states, area.width);

            // Height of the dropdown (one row per item + 2 for top/bottom border).
            let drop_height: u16 = menu_items.len() as u16 + 2;

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

    // Built-in resolved menus (no plugins) — used by the geometry tests so
    // their assertions stay identical to the pre-feature layout.
    fn builtin_menus() -> Vec<ResolvedMenu> {
        resolve_menus(&[])
    }

    // T009: shared helpers — open a specific menu dropdown for rendering.
    fn view_open() -> MenuBarState {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.open_menu(3, &menus); // View is index 3 in ALL_MENUS / BAR_LABELS
        s
    }

    fn file_open() -> MenuBarState {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.open_menu(0, &menus); // File is index 0
        s
    }

    fn options_open() -> MenuBarState {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.open_menu(4, &menus); // Options is index 4
        s
    }

    // Render a MenuBarWidget into a fresh buffer of the given size.
    fn render_into(
        state: &MenuBarState,
        toggle_states: &[(Action, bool)],
        width: u16,
        height: u16,
    ) -> Buffer {
        let menus = builtin_menus();
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
        let widget = MenuBarWidget::new(theme_by_name("classic"), state, toggle_states, &menus);
        widget.render(buf.area, &mut buf);
        buf
    }

    // Render with an explicit resolved-menu list (for plugin-menu rendering tests).
    fn render_into_with_menus(
        state: &MenuBarState,
        toggle_states: &[(Action, bool)],
        menus: &[ResolvedMenu],
        width: u16,
        height: u16,
    ) -> Buffer {
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
        let widget = MenuBarWidget::new(theme_by_name("classic"), state, toggle_states, menus);
        widget.render(buf.area, &mut buf);
        buf
    }

    // A synthetic plugin menu item for tests.
    fn plugin_item(menu: &str, item: &str, id: &str, plugin: &str) -> PluginMenuItem {
        PluginMenuItem {
            menu: menu.to_string(),
            item: item.to_string(),
            item_id: id.to_string(),
            plugin_id: plugin.to_string(),
            position: None,
        }
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

    // ── Feature 009: resolve_menus model (T002) ─────────────────────────────

    #[test]
    fn test_resolve_menus_empty_matches_builtin() {
        let menus = resolve_menus(&[]);
        let labels: Vec<&str> = menus.iter().map(|m| m.label.as_str()).collect();
        assert_eq!(
            labels,
            ["File", "Edit", "Search", "View", "Options", "Help"]
        );
        // Item labels/actions match the built-in slices exactly.
        for (m, items) in menus.iter().zip(ALL_MENUS.iter()) {
            let got: Vec<&str> = m.items.iter().map(|i| i.label.as_str()).collect();
            let want: Vec<&str> = items.iter().map(|i| i.label).collect();
            assert_eq!(got, want, "items of {} must match built-in", m.label);
        }
    }

    #[test]
    fn test_resolve_menus_inserts_plugin_before_help() {
        let items = vec![plugin_item("Tools", "Word Count", "wc", "word-count")];
        let menus = resolve_menus(&items);
        // "Tools" is at len-2; Help remains last.
        assert_eq!(menus.last().unwrap().label, "Help");
        assert_eq!(menus[menus.len() - 2].label, "Tools");
        assert_eq!(menus[menus.len() - 2].items.len(), 1);
        assert_eq!(menus[menus.len() - 2].items[0].label, "Word Count");
        assert_eq!(
            menus[menus.len() - 2].items[0].action,
            Action::PluginMenuActivated("word-count".into(), "wc".into())
        );
    }

    #[test]
    fn test_resolve_menus_merges_into_builtin_on_name_collision() {
        let items = vec![plugin_item("Edit", "Sort Lines", "sort", "sorter")];
        let menus = resolve_menus(&items);
        // Exactly one "Edit" top-level menu (no duplicate).
        assert_eq!(menus.iter().filter(|m| m.label == "Edit").count(), 1);
        let edit = menus.iter().find(|m| m.label == "Edit").unwrap();
        assert!(
            edit.items.iter().any(|i| i.label == "Sort Lines"),
            "plugin item must be merged into the built-in Edit menu"
        );
        // No "Edit" inserted as a plugin top-level; total count is built-ins.
        assert_eq!(menus.len(), 6);
    }

    #[test]
    fn test_resolve_menus_groups_multiple_plugins_same_menu() {
        let items = vec![
            plugin_item("Tools", "Word Count", "wc", "word-count"),
            plugin_item("Tools", "Line Count", "lc", "line-count"),
        ];
        let menus = resolve_menus(&items);
        let tools: Vec<&ResolvedMenu> = menus.iter().filter(|m| m.label == "Tools").collect();
        assert_eq!(tools.len(), 1, "a single shared Tools menu");
        assert_eq!(tools[0].items.len(), 2);
    }

    #[test]
    fn test_resolve_menus_orders_by_position_then_load_order() {
        let mut a = plugin_item("Tools", "Second", "b", "p");
        a.position = Some(10);
        let mut b = plugin_item("Tools", "First", "a", "p");
        b.position = Some(1);
        let c = plugin_item("Tools", "Loadorder", "c", "p"); // None position → after positioned
        let menus = resolve_menus(&[a, b, c]);
        let tools = menus.iter().find(|m| m.label == "Tools").unwrap();
        let labels: Vec<&str> = tools.items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(labels, ["First", "Second", "Loadorder"]);
    }

    #[test]
    fn test_resolve_menus_widechar_plugin_label_preserved() {
        // FR-014 / remediation M2: UTF-8 wide-character labels survive resolution intact.
        let items = vec![plugin_item("ツール", "文字数", "wc", "jp")];
        let menus = resolve_menus(&items);
        let tools = menus.iter().find(|m| m.label == "ツール").unwrap();
        assert_eq!(tools.items[0].label, "文字数");
    }

    // ── Feature 009: MenuBarState navigation (T005) ─────────────────────────

    #[test]
    fn test_navigate_down_wraps() {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.open_menu(0, &menus); // File, item 0
        let last = menus[0].items.len() - 1;
        for _ in 0..last {
            s.navigate_down(&menus);
        }
        assert_eq!(
            s.state,
            MenuState::DropDown {
                top_idx: 0,
                item_idx: last
            }
        );
        s.navigate_down(&menus); // wrap to 0
        assert_eq!(
            s.state,
            MenuState::DropDown {
                top_idx: 0,
                item_idx: 0
            }
        );
    }

    #[test]
    fn test_navigate_up_wraps() {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.open_menu(0, &menus); // item 0
        s.navigate_up(&menus); // wrap to last
        let last = menus[0].items.len() - 1;
        assert_eq!(
            s.state,
            MenuState::DropDown {
                top_idx: 0,
                item_idx: last
            }
        );
    }

    #[test]
    fn test_top_active_down_opens_dropdown() {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.activate_bar();
        assert_eq!(s.state, MenuState::TopActive(0));
        s.navigate_down(&menus);
        assert_eq!(
            s.state,
            MenuState::DropDown {
                top_idx: 0,
                item_idx: 0
            }
        );
    }

    #[test]
    fn test_top_active_up_opens_last_item() {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.activate_bar();
        s.navigate_up(&menus);
        let last = menus[0].items.len() - 1;
        assert_eq!(
            s.state,
            MenuState::DropDown {
                top_idx: 0,
                item_idx: last
            }
        );
    }

    #[test]
    fn test_activate_bar_enters_top_active() {
        let mut s = MenuBarState::new();
        s.activate_bar();
        assert_eq!(s.state, MenuState::TopActive(0));
        assert!(s.is_active());
    }

    #[test]
    fn test_navigate_left_right_wraps_over_ring() {
        let menus = builtin_menus(); // 6 menus
        let mut s = MenuBarState::new();
        s.activate_bar(); // TopActive(0)
        s.navigate_left(&menus); // wrap to last
        assert_eq!(s.state, MenuState::TopActive(menus.len() - 1));
        s.navigate_right(&menus); // wrap forward to 0
        assert_eq!(s.state, MenuState::TopActive(0));
    }

    #[test]
    fn test_navigate_left_right_opens_adjacent_dropdown() {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.open_menu(1, &menus); // Edit dropdown
        s.navigate_right(&menus);
        assert_eq!(
            s.state,
            MenuState::DropDown {
                top_idx: 2,
                item_idx: 0
            }
        );
        s.navigate_left(&menus);
        assert_eq!(
            s.state,
            MenuState::DropDown {
                top_idx: 1,
                item_idx: 0
            }
        );
    }

    #[test]
    fn test_navigate_left_right_top_active_moves_highlight_only() {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.activate_bar();
        s.navigate_right(&menus);
        assert_eq!(s.state, MenuState::TopActive(1)); // no dropdown
    }

    #[test]
    fn test_select_item_returns_builtin_action_and_closes() {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.open_menu(0, &menus); // File, item 0 "New"
        s.navigate_down(&menus); // Open
        s.navigate_down(&menus); // Save (index 2)
        let action = s.select_item(&menus);
        assert_eq!(action, Some(Action::Save));
        assert_eq!(s.state, MenuState::Inactive);
    }

    #[test]
    fn test_select_item_returns_plugin_activated_action() {
        let items = vec![plugin_item("Tools", "Word Count", "wc", "word-count")];
        let menus = resolve_menus(&items);
        let tools_idx = menus.iter().position(|m| m.label == "Tools").unwrap();
        let mut s = MenuBarState::new();
        s.open_menu(tools_idx, &menus);
        let action = s.select_item(&menus);
        assert_eq!(
            action,
            Some(Action::PluginMenuActivated(
                "word-count".into(),
                "wc".into()
            ))
        );
        assert_eq!(s.state, MenuState::Inactive);
    }

    #[test]
    fn test_select_item_inactive_returns_none() {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        assert_eq!(s.select_item(&menus), None);
    }

    #[test]
    fn test_select_item_top_active_returns_none() {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.activate_bar();
        assert_eq!(s.select_item(&menus), None); // Enter at TopActive is a no-op
        assert_eq!(s.state, MenuState::TopActive(0)); // unchanged
    }

    #[test]
    fn test_open_menu_clamps_to_resolved_len() {
        let menus = builtin_menus();
        let mut s = MenuBarState::new();
        s.open_menu(999, &menus);
        assert_eq!(
            s.state,
            MenuState::DropDown {
                top_idx: menus.len() - 1,
                item_idx: 0
            }
        );
    }

    #[test]
    fn test_empty_plugin_menu_not_openable() {
        // A resolved menu with zero items must be a no-op on navigate_down and
        // select_item must return None (no panic).
        let empty = vec![ResolvedMenu {
            label: "Empty".into(),
            items: vec![],
        }];
        let mut s = MenuBarState::new();
        s.state = MenuState::TopActive(0);
        s.navigate_down(&empty);
        assert_eq!(s.state, MenuState::TopActive(0)); // did not open
        s.state = MenuState::DropDown {
            top_idx: 0,
            item_idx: 0,
        };
        assert_eq!(s.select_item(&empty), None);
    }

    #[test]
    fn test_plugin_menu_renders_between_options_and_help() {
        // Rendering smoke: the Tools label appears in the bar to the left of Help.
        let items = vec![plugin_item("Tools", "Word Count", "wc", "word-count")];
        let menus = resolve_menus(&items);
        let state = MenuBarState::new();
        let buf = render_into_with_menus(&state, &[], &menus, 60, 1);
        let row: String = (0..60)
            .map(|x| buf.get(x, 0).symbol().to_string())
            .collect();
        let tools_at = row.find("Tools").expect("Tools rendered");
        let help_at = row.find("Help").expect("Help rendered");
        assert!(tools_at < help_at, "Tools must render left of Help");
    }

    // ── Feature 011 — mouse hit-testing ──────────────────────────────────────

    #[test]
    fn hit_test_top_level_each_builtin_menu() {
        let menus = resolve_menus(&[]);
        let st = MenuState::Inactive;
        // File col 1, Edit col 7, Search col 13, View col 21, Options 28, Help 37.
        for (col, idx) in [(1u16, 0usize), (7, 1), (13, 2), (21, 3), (28, 4), (37, 5)] {
            assert_eq!(
                hit_test_menu(&menus, &st, &[], 80, col, 0),
                MenuHit::TopLevel(idx),
                "col {col} should hit menu {idx}"
            );
        }
    }

    #[test]
    fn hit_test_top_level_gap_is_outside() {
        let menus = resolve_menus(&[]);
        // Column 5 is between "File" (1-4) and "Edit" (7-10).
        assert_eq!(
            hit_test_menu(&menus, &MenuState::Inactive, &[], 80, 5, 0),
            MenuHit::Outside
        );
    }

    #[test]
    fn hit_test_dropdown_item_rows() {
        let menus = resolve_menus(&[]);
        let st = MenuState::DropDown {
            top_idx: 0,
            item_idx: 0,
        };
        // File dropdown: row 1 = New (item 0), row 2 = Open (item 1).
        assert_eq!(
            hit_test_menu(&menus, &st, &[], 80, 2, 1),
            MenuHit::Item {
                top_idx: 0,
                item_idx: 0
            }
        );
        assert_eq!(
            hit_test_menu(&menus, &st, &[], 80, 2, 2),
            MenuHit::Item {
                top_idx: 0,
                item_idx: 1
            }
        );
        // A row past the last File item is Outside.
        let past = menus[0].items.len() as u16 + 1;
        assert_eq!(
            hit_test_menu(&menus, &st, &[], 80, 2, past),
            MenuHit::Outside
        );
    }

    #[test]
    fn hit_test_dropdown_outside_columns() {
        let menus = resolve_menus(&[]);
        let st = MenuState::DropDown {
            top_idx: 0,
            item_idx: 0,
        };
        // Far-right column is outside the File dropdown box.
        assert_eq!(hit_test_menu(&menus, &st, &[], 80, 70, 1), MenuHit::Outside);
    }

    #[test]
    fn dropdown_layout_reserves_checkable_width() {
        let menus = resolve_menus(&[]);
        // View (index 3) contains the checkable "Soft Wrap (ext)" item.
        let view = &menus[3];
        let plain = dropdown_layout(view, 21, &[], 80);
        let checkable = dropdown_layout(view, 21, &[(Action::ToggleSoftWrap, true)], 80);
        assert!(!plain.has_checkable);
        assert!(checkable.has_checkable);
        assert_eq!(checkable.content_width, plain.content_width + 2);
        assert_eq!(checkable.label_offset, 3);
    }
}
