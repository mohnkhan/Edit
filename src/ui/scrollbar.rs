//! Feature 021: a thin wrapper over ratatui's [`Scrollbar`] so every scrollable
//! view draws a consistent bar with one shared rule: **draw nothing when the
//! content fits** (`content_len <= viewport_len`). The caller reserves the edge
//! the bar occupies (rightmost column for vertical, bottom row for horizontal)
//! so the bar never hides content.

use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::Style,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};

use crate::ui::theme::Theme;

/// Draw a vertical scrollbar along the right edge of `area`.
///
/// `content_len` is the total scrollable extent, `viewport_len` the visible
/// extent, and `position` the current scroll offset (top of the viewport).
/// No-op when the content fits, or when the area is too small to draw into.
pub fn render_vertical(
    buf: &mut TuiBuffer,
    area: Rect,
    content_len: usize,
    viewport_len: usize,
    position: usize,
    theme: &Theme,
) {
    if content_len <= viewport_len || area.width == 0 || area.height == 0 {
        return;
    }
    let mut state = ScrollbarState::new(content_len)
        .viewport_content_length(viewport_len)
        .position(position.min(content_len));
    let bar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"))
        .track_symbol(Some("░"))
        .thumb_symbol("█")
        .style(Style::default().fg(theme.menubar_fg).bg(theme.menubar_bg));
    StatefulWidget::render(bar, area, buf, &mut state);
}

/// Draw a horizontal scrollbar along the bottom edge of `area`.
///
/// Same semantics as [`render_vertical`] but for the horizontal axis.
pub fn render_horizontal(
    buf: &mut TuiBuffer,
    area: Rect,
    content_len: usize,
    viewport_len: usize,
    position: usize,
    theme: &Theme,
) {
    if content_len <= viewport_len || area.width == 0 || area.height == 0 {
        return;
    }
    let mut state = ScrollbarState::new(content_len)
        .viewport_content_length(viewport_len)
        .position(position.min(content_len));
    let bar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
        .begin_symbol(Some("◄"))
        .end_symbol(Some("►"))
        .track_symbol(Some("░"))
        .thumb_symbol("█")
        .style(Style::default().fg(theme.menubar_fg).bg(theme.menubar_bg));
    StatefulWidget::render(bar, area, buf, &mut state);
}

/// Whether a scrollbar would be drawn for the given extents (content overflows
/// the viewport). Lets callers decide to reserve the edge only when needed.
pub fn is_needed(content_len: usize, viewport_len: usize) -> bool {
    content_len > viewport_len
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::theme_by_name;
    use ratatui::buffer::Buffer;

    fn count_non_space(buf: &Buffer) -> usize {
        buf.content().iter().filter(|c| c.symbol() != " ").count()
    }

    #[test]
    fn draws_nothing_when_content_fits() {
        let theme = theme_by_name("classic");
        let area = Rect::new(0, 0, 10, 6);
        let mut buf = Buffer::empty(area);
        render_vertical(&mut buf, area, 6, 6, 0, theme);
        assert_eq!(count_non_space(&buf), 0, "no bar when content fits");
        render_vertical(&mut buf, area, 3, 6, 0, theme);
        assert_eq!(count_non_space(&buf), 0, "no bar when content < viewport");
    }

    #[test]
    fn draws_bar_when_content_overflows() {
        let theme = theme_by_name("classic");
        let area = Rect::new(0, 0, 10, 6);
        let mut buf = Buffer::empty(area);
        render_vertical(&mut buf, area, 100, 6, 0, theme);
        assert!(count_non_space(&buf) > 0, "a bar is drawn on overflow");
    }

    #[test]
    fn horizontal_overflow_draws_and_fit_does_not() {
        let theme = theme_by_name("classic");
        let area = Rect::new(0, 0, 12, 4);
        let mut buf = Buffer::empty(area);
        render_horizontal(&mut buf, area, 4, 12, 0, theme);
        assert_eq!(count_non_space(&buf), 0, "no h-bar when content fits");
        let mut buf2 = Buffer::empty(area);
        render_horizontal(&mut buf2, area, 200, 12, 0, theme);
        assert!(count_non_space(&buf2) > 0, "h-bar drawn on overflow");
    }

    #[test]
    fn tiny_or_zero_area_does_not_panic() {
        let theme = theme_by_name("classic");
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        render_vertical(&mut buf, Rect::new(0, 0, 1, 1), 100, 1, 50, theme);
        render_horizontal(&mut buf, Rect::new(0, 0, 1, 1), 100, 1, 50, theme);
        // Zero-area rects are no-ops.
        render_vertical(&mut buf, Rect::new(0, 0, 0, 0), 100, 1, 50, theme);
    }

    #[test]
    fn position_past_end_is_clamped_without_panic() {
        let theme = theme_by_name("classic");
        let area = Rect::new(0, 0, 6, 6);
        let mut buf = Buffer::empty(area);
        render_vertical(&mut buf, area, 10, 6, 9999, theme);
        assert!(count_non_space(&buf) > 0);
    }

    #[test]
    fn is_needed_matches_overflow() {
        assert!(!is_needed(6, 6));
        assert!(!is_needed(3, 6));
        assert!(is_needed(7, 6));
    }
}
