//! Feature 030 (US3 / #55): the editor right-click context menu.
//!
//! A small fixed popup (Cut / Copy / Paste / Select All) anchored at the click,
//! clamped on-screen. The geometry helper [`menu_rect`] is shared by the renderer
//! and the hit-test so a click always lands on the drawn item. Modelled on the
//! menubar dropdown + the boxed-button widgets; no new dependency.

use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Widget},
};

use crate::input::keymap::Action;
use crate::ui::theme::Theme;

/// The fixed context-menu items and the action each fires.
pub const ITEMS: &[(&str, Action)] = &[
    ("Cut", Action::Cut),
    ("Copy", Action::Copy),
    ("Paste", Action::Paste),
    ("Select All", Action::SelectAll),
];

/// Open editor context menu: an anchor (click position) and the focused index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextMenu {
    pub anchor: (u16, u16),
    pub focus: usize,
}

impl ContextMenu {
    /// Open at the given click position with the first item focused.
    pub fn new(col: u16, row: u16) -> Self {
        ContextMenu {
            anchor: (col, row),
            focus: 0,
        }
    }

    /// Move focus to the next item (wraps).
    pub fn focus_next(&mut self) {
        self.focus = (self.focus + 1) % ITEMS.len();
    }

    /// Move focus to the previous item (wraps).
    pub fn focus_prev(&mut self) {
        self.focus = (self.focus + ITEMS.len() - 1) % ITEMS.len();
    }
}

/// Inner content width (widest label) and the box dimensions.
fn box_size() -> (u16, u16) {
    let label_w = ITEMS.iter().map(|(l, _)| l.len()).max().unwrap_or(4) as u16;
    let w = label_w + 4; // 2 borders + 1 leading + 1 trailing space
    let h = ITEMS.len() as u16 + 2; // items + top/bottom border
    (w, h)
}

/// The popup rect for `menu`, anchored at the click and clamped to stay fully
/// within `area`.
pub fn menu_rect(menu: &ContextMenu, area: Rect) -> Rect {
    let (w, h) = box_size();
    let w = w.min(area.width.max(1));
    let h = h.min(area.height.max(1));
    let max_x = (area.x + area.width).saturating_sub(w);
    let max_y = (area.y + area.height).saturating_sub(h);
    let x = menu.anchor.0.min(max_x).max(area.x);
    let y = menu.anchor.1.min(max_y).max(area.y);
    Rect::new(x, y, w, h)
}

/// Map a click at `(col, row)` to an item index, or `None` if outside the menu.
pub fn hit_test(rect: Rect, col: u16, row: u16) -> Option<usize> {
    let inside =
        col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height;
    if !inside {
        return None;
    }
    let first = rect.y + 1; // inside the top border
    if row < first {
        return None;
    }
    let idx = (row - first) as usize;
    if idx < ITEMS.len() {
        Some(idx)
    } else {
        None
    }
}

/// Render the context menu overlay into `area`.
pub fn render(buf: &mut TuiBuffer, area: Rect, menu: &ContextMenu, theme: &Theme) {
    let rect = menu_rect(menu, area);
    let base = Style::default().fg(theme.menubar_fg).bg(theme.menubar_bg);
    let selected = base.add_modifier(Modifier::REVERSED);

    Clear.render(rect, buf);
    Block::default()
        .borders(Borders::ALL)
        .style(base)
        .render(rect, buf);

    let inner_w = rect.width.saturating_sub(2) as usize;
    for (i, (label, _)) in ITEMS.iter().enumerate() {
        let y = rect.y + 1 + i as u16;
        if y + 1 >= rect.y + rect.height {
            break; // clamped height
        }
        let style = if i == menu.focus { selected } else { base };
        // " label" padded to the inner width.
        let text = format!(" {label:<width$}", width = inner_w.saturating_sub(1));
        for (ci, ch) in text.chars().enumerate() {
            let cx = rect.x + 1 + ci as u16;
            if cx + 1 >= rect.x + rect.width {
                break;
            }
            buf.get_mut(cx, y).set_char(ch).set_style(style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::theme_by_name;

    #[test]
    fn rect_clamps_on_screen() {
        let area = Rect::new(0, 0, 80, 24);
        // Anchor near the bottom-right corner — the box must stay fully on-screen.
        let m = ContextMenu::new(79, 23);
        let r = menu_rect(&m, area);
        assert!(r.x + r.width <= area.x + area.width);
        assert!(r.y + r.height <= area.y + area.height);
    }

    #[test]
    fn rect_no_panic_tiny_terminal() {
        for (w, h) in [(0u16, 0u16), (1, 1), (3, 2), (10, 3)] {
            let area = Rect::new(0, 0, w, h);
            let _ = menu_rect(&ContextMenu::new(0, 0), area);
        }
    }

    #[test]
    fn hit_test_maps_items() {
        let area = Rect::new(0, 0, 80, 24);
        let m = ContextMenu::new(10, 5);
        let r = menu_rect(&m, area);
        assert_eq!(hit_test(r, r.x + 1, r.y), None, "top border");
        assert_eq!(hit_test(r, r.x + 1, r.y + 1), Some(0));
        assert_eq!(hit_test(r, r.x + 1, r.y + 4), Some(3));
        assert_eq!(
            hit_test(r, r.x + 1, r.y + 5),
            None,
            "past last item / bottom border"
        );
        assert_eq!(hit_test(r, r.x + 200, r.y + 1), None, "outside");
    }

    #[test]
    fn focus_wraps() {
        let mut m = ContextMenu::new(0, 0);
        m.focus_prev();
        assert_eq!(m.focus, ITEMS.len() - 1);
        m.focus_next();
        assert_eq!(m.focus, 0);
    }

    #[test]
    fn render_shows_items_no_panic() {
        let area = Rect::new(0, 0, 80, 24);
        let m = ContextMenu::new(5, 5);
        let mut buf = TuiBuffer::empty(area);
        render(&mut buf, area, &m, theme_by_name("classic"));
        let r = menu_rect(&m, area);
        let row: String = (r.x..r.x + r.width)
            .map(|x| buf.get(x, r.y + 1).symbol().to_string())
            .collect();
        assert!(row.contains("Cut"));
    }
}
