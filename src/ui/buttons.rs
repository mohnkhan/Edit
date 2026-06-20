//! Feature 016: a reusable row of boxed, focusable dialog buttons.
//!
//! One geometry source ([`button_rects`]) is shared by the renderer
//! ([`render_buttons`]) and mouse hit-testing ([`hit_test_buttons`]) so a click
//! always lands on the button that was drawn. Buttons are 3-row boxes laid out
//! in a horizontally-centered row on the bottom interior of a dialog; the
//! focused button is drawn inverted with a `▶` marker.

use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Widget},
};
use unicode_width::UnicodeWidthStr;

use crate::ui::theme::Theme;

/// Horizontal padding inside each button box (one space each side).
const PAD: u16 = 1;

/// Width of a button box for `label`: borders (2) + padding (2) + label width.
fn button_width(label: &str) -> u16 {
    2 + 2 * PAD + UnicodeWidthStr::width(label) as u16
}

/// Compute the rectangles for a centered row of boxed buttons along the bottom
/// interior rows of `area`. Each rect is 3 rows tall. Buttons that would overflow
/// the dialog width are dropped from the row (the caller keeps them
/// keyboard-reachable). Never panics on a tiny `area`.
pub fn button_rects(area: Rect, labels: &[&str]) -> Vec<Rect> {
    if area.width < 4 || area.height < 3 || labels.is_empty() {
        return Vec::new();
    }
    let inner_w = area.width.saturating_sub(2); // leave the dialog border
                                                // Greedily keep buttons that fit (label order preserved).
    let mut widths: Vec<u16> = Vec::new();
    let mut total = 0u16;
    for (i, l) in labels.iter().enumerate() {
        let w = button_width(l);
        let gap = if i == 0 { 0 } else { 1 };
        if total + gap + w > inner_w {
            break;
        }
        total += gap + w;
        widths.push(w);
    }
    if widths.is_empty() {
        return Vec::new();
    }
    // 3-row boxes sit just above the dialog's bottom border.
    let y = area.y + area.height.saturating_sub(4);
    let start_x = area.x + 1 + (inner_w.saturating_sub(total)) / 2;
    let mut x = start_x;
    let mut rects = Vec::with_capacity(widths.len());
    for (i, w) in widths.iter().enumerate() {
        if i > 0 {
            x += 1; // gap
        }
        rects.push(Rect::new(x, y, *w, 3));
        x += *w;
    }
    rects
}

/// Index of the button whose box contains `(col, row)`, or `None`.
pub fn hit_test_buttons(rects: &[Rect], col: u16, row: u16) -> Option<usize> {
    rects
        .iter()
        .position(|r| col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height)
}

/// Next focus index (wraps). `n` must be > 0.
pub fn next(focus: usize, n: usize) -> usize {
    if n == 0 {
        0
    } else {
        (focus + 1) % n
    }
}

/// Previous focus index (wraps). `n` must be > 0.
pub fn prev(focus: usize, n: usize) -> usize {
    if n == 0 {
        0
    } else {
        (focus + n - 1) % n
    }
}

/// Draw the boxed buttons. `focused` is the index rendered distinctly.
pub fn render_buttons(
    buf: &mut TuiBuffer,
    rects: &[Rect],
    labels: &[&str],
    focused: usize,
    theme: &Theme,
) {
    let normal = Style::default().fg(theme.menubar_fg).bg(theme.menubar_bg);
    let focus_style = Style::default()
        .fg(theme.menubar_bg)
        .bg(theme.menu_selected_bg)
        .add_modifier(Modifier::BOLD);
    for (i, r) in rects.iter().enumerate() {
        let is_focus = i == focused;
        let style = if is_focus { focus_style } else { normal };
        Block::default()
            .borders(Borders::ALL)
            .style(style)
            .render(*r, buf);
        // Label centered on the middle row, with a focus marker.
        let label = labels.get(i).copied().unwrap_or("");
        let inner_w = r.width.saturating_sub(2) as usize;
        let text = if is_focus {
            format!("▶{}", label)
        } else {
            label.to_string()
        };
        let tw = UnicodeWidthStr::width(text.as_str());
        let pad_left = inner_w.saturating_sub(tw) / 2;
        let start = r.x + 1 + pad_left as u16;
        let my = r.y + 1;
        for (j, g) in text.chars().enumerate() {
            let cx = start + j as u16;
            if cx >= r.x + r.width - 1 {
                break;
            }
            buf.get_mut(cx, my).set_char(g).set_style(style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::theme_by_name;
    use ratatui::buffer::Buffer;

    #[test]
    fn rects_centered_with_correct_widths_and_gaps() {
        let area = Rect::new(0, 0, 60, 10);
        let labels = ["Save", "Discard", "Cancel"];
        let rects = button_rects(area, &labels);
        assert_eq!(rects.len(), 3);
        // widths = label width + 4
        assert_eq!(rects[0].width, 8); // "Save"=4 +4
        assert_eq!(rects[1].width, 11); // "Discard"=7 +4
        assert_eq!(rects[2].width, 10); // "Cancel"=6 +4
                                        // 1-col gaps, ascending x
        assert_eq!(rects[1].x, rects[0].x + rects[0].width + 1);
        assert_eq!(rects[2].x, rects[1].x + rects[1].width + 1);
        // all 3 rows tall, same y
        assert!(rects.iter().all(|r| r.height == 3 && r.y == rects[0].y));
    }

    #[test]
    fn tiny_area_does_not_panic_and_drops_overflow() {
        assert!(button_rects(Rect::new(0, 0, 3, 2), &["X"]).is_empty());
        // Only the first button fits in a narrow dialog.
        let rects = button_rects(Rect::new(0, 0, 12, 6), &["OK", "Cancel"]);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].width, button_width("OK"));
    }

    #[test]
    fn hit_test_maps_inside_and_outside() {
        let area = Rect::new(0, 0, 60, 10);
        let rects = button_rects(area, &["Save", "Discard", "Cancel"]);
        let r = rects[1];
        assert_eq!(hit_test_buttons(&rects, r.x + 1, r.y + 1), Some(1));
        assert_eq!(hit_test_buttons(&rects, 0, 0), None);
    }

    #[test]
    fn focus_wraps() {
        assert_eq!(next(2, 3), 0);
        assert_eq!(prev(0, 3), 2);
        assert_eq!(next(0, 1), 0);
    }

    #[test]
    fn render_marks_focused_button() {
        let area = Rect::new(0, 0, 40, 8);
        let labels = ["Yes", "No"];
        let rects = button_rects(area, &labels);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 8));
        render_buttons(&mut buf, &rects, &labels, 1, theme_by_name("classic"));
        // The focused (No) button row contains the ▶ marker somewhere.
        let r = rects[1];
        let row: String = (r.x..r.x + r.width)
            .map(|x| buf.get(x, r.y + 1).symbol().to_string())
            .collect();
        assert!(
            row.contains('▶'),
            "focused button shows the marker: {row:?}"
        );
    }
}
