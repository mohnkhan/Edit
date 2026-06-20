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

// ── Feature 024: interactive hit-testing / mapping (pure) ──────────────────

/// Which part of the track a click landed on, relative to the thumb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitZone {
    /// Before the thumb (above / left) — page toward the start.
    Above,
    /// On the thumb — begin a drag.
    Thumb,
    /// After the thumb (below / right) — page toward the end.
    Below,
}

/// Thumb `(start, len)` within a track of `track_len` cells for the given
/// content/viewport/position. Mirrors the rendered bar: full track when content
/// fits, minimum length 1, start clamped so the thumb stays on the track.
pub fn thumb_span(track_len: usize, content: usize, viewport: usize, pos: usize) -> (usize, usize) {
    if track_len == 0 {
        return (0, 0);
    }
    if content <= viewport || viewport == 0 {
        return (0, track_len); // nothing to scroll → full track
    }
    let len = (track_len * viewport).div_ceil(content).clamp(1, track_len);
    let max_off = content - viewport;
    let travel = track_len - len;
    let start = (travel * pos.min(max_off) + max_off / 2)
        .checked_div(max_off)
        .unwrap_or(0);
    (start.min(travel), len)
}

/// Classify a 0-based `click` position along a `track_len` track.
pub fn hit_zone(
    track_len: usize,
    content: usize,
    viewport: usize,
    pos: usize,
    click: usize,
) -> HitZone {
    let (start, len) = thumb_span(track_len, content, viewport, pos);
    if click < start {
        HitZone::Above
    } else if click >= start + len {
        HitZone::Below
    } else {
        HitZone::Thumb
    }
}

/// Map a 0-based `click` position along the track to a scroll offset in
/// `[0, content-viewport]` (proportional; centers the thumb on the cursor).
pub fn pos_to_offset(track_len: usize, content: usize, viewport: usize, click: usize) -> usize {
    if content <= viewport || track_len == 0 {
        return 0;
    }
    let max_off = content - viewport;
    let (_, len) = thumb_span(track_len, content, viewport, 0);
    let travel = track_len.saturating_sub(len);
    if travel == 0 {
        return 0;
    }
    let c = click.min(track_len - 1);
    let centered = c.saturating_sub(len / 2);
    ((max_off * centered.min(travel)) + travel / 2) / travel
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

    // ── Feature 024: interactive math ──────────────────────────────────────

    #[test]
    fn thumb_fills_track_when_content_fits() {
        assert_eq!(thumb_span(10, 5, 10, 0), (0, 10));
        assert_eq!(thumb_span(10, 10, 10, 0), (0, 10));
    }

    #[test]
    fn thumb_min_len_and_bounds_and_monotonic() {
        let track = 10;
        let (s0, l) = thumb_span(track, 100, 10, 0);
        assert!(l >= 1 && l <= track);
        assert_eq!(s0, 0, "at top the thumb starts at 0");
        let (s_mid, _) = thumb_span(track, 100, 10, 45);
        let (s_max, lm) = thumb_span(track, 100, 10, 90);
        assert!(
            s0 <= s_mid && s_mid <= s_max,
            "thumb start monotonic in pos"
        );
        assert!(
            s_max + lm <= track,
            "thumb stays on the track at the bottom"
        );
    }

    #[test]
    fn pos_to_offset_endpoints_and_clamp() {
        let track = 12;
        assert_eq!(pos_to_offset(track, 100, 10, 0), 0, "top → 0");
        let max = pos_to_offset(track, 100, 10, track - 1);
        assert_eq!(max, 90, "bottom → content-viewport");
        // Monotonic, clamped.
        let a = pos_to_offset(track, 100, 10, 3);
        let b = pos_to_offset(track, 100, 10, 8);
        assert!(a <= b);
        assert!(pos_to_offset(track, 100, 10, 999) <= 90);
        // Content fits → always 0.
        assert_eq!(pos_to_offset(track, 5, 10, 7), 0);
    }

    #[test]
    fn hit_zone_classifies_above_thumb_below() {
        // Tall content: thumb is short and near the top at pos 0.
        let track = 10;
        let (start, len) = thumb_span(track, 100, 10, 0);
        assert_eq!(hit_zone(track, 100, 10, 0, start), HitZone::Thumb);
        if start + len < track {
            assert_eq!(hit_zone(track, 100, 10, 0, start + len), HitZone::Below);
        }
        // Scrolled to bottom: a click at row 0 is above the thumb.
        assert_eq!(hit_zone(track, 100, 10, 90, 0), HitZone::Above);
    }
}
