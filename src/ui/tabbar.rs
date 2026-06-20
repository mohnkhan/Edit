//! Feature 027: the buffer tab bar.
//!
//! One geometry source ([`tab_hit_regions`]) is shared by the renderer
//! ([`render_tab_bar`]) and mouse hit-testing so a click always lands on the tab
//! / `[x]` that was drawn. Shown only when 2+ buffers are open (the caller
//! decides). Tabs that overflow the width scroll so the **active** tab stays
//! visible; labels truncate by display width (UTF-8-correct).

use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::{Modifier, Style},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::buffer::Buffer;
use crate::ui::file_browser::{grapheme_width, truncate_to_width};
use crate::ui::theme::Theme;

/// Max display columns for a tab's name before truncation.
const MAX_NAME: u16 = 16;

/// A drawn, clickable tab region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabRegion {
    /// Index into `buffers`.
    pub idx: usize,
    /// Clickable region that switches to the buffer (the label).
    pub label_rect: Rect,
    /// Clickable `[x]` close region.
    pub close_rect: Rect,
}

/// Display name for a buffer's tab (file name, or a placeholder when unsaved).
pub fn tab_name(buf: &Buffer) -> String {
    buf.path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "[No Name]".to_string())
}

/// Display width of a string (sum of grapheme widths).
fn width_of(s: &str) -> u16 {
    s.graphemes(true).map(grapheme_width).sum()
}

/// The label text for tab `i` (leading space + truncated name + modified marker
/// + trailing space). The `[x]` is drawn separately, immediately after.
fn label_text(buf: &Buffer) -> String {
    let name = truncate_to_width(&tab_name(buf), MAX_NAME);
    let marker = if buf.modified { "●" } else { " " };
    format!(" {name}{marker} ")
}

/// Compute the visible tab regions within `area` (a single row). Lays tabs out
/// left→right; if they overflow, scrolls the start so the `active` tab is fully
/// visible. Each tab is `<label>[x]` followed by a one-column gap.
pub fn tab_hit_regions(area: Rect, buffers: &[Buffer], active: usize) -> Vec<TabRegion> {
    if area.width == 0 || area.height == 0 || buffers.is_empty() {
        return Vec::new();
    }
    // Per-tab widths: label + 1 (close glyph). A 1-col gap separates tabs.
    let labels: Vec<String> = buffers.iter().map(label_text).collect();
    let tab_w: Vec<u16> = labels.iter().map(|l| width_of(l) + 1).collect();

    // Choose the first visible tab so the active tab fits (scroll from the right).
    let avail = area.width;
    let mut first = 0usize;
    loop {
        let mut used = 0u16;
        let mut active_fits = false;
        for (i, w) in tab_w.iter().enumerate().skip(first) {
            let next = used + w + if i > first { 1 } else { 0 };
            if next > avail {
                break;
            }
            used = next;
            if i == active {
                active_fits = true;
            }
        }
        if active_fits || first >= active || first + 1 >= buffers.len() {
            break;
        }
        first += 1;
    }

    let mut regions = Vec::new();
    let mut x = area.x;
    for (i, l) in labels.iter().enumerate().skip(first) {
        let lw = width_of(l);
        let total = lw + 1; // label + close glyph
        let gap = if i > first { 1 } else { 0 };
        if x + gap + total > area.x + area.width {
            break;
        }
        x += gap;
        let label_rect = Rect::new(x, area.y, lw, 1);
        let close_rect = Rect::new(x + lw, area.y, 1, 1);
        regions.push(TabRegion {
            idx: i,
            label_rect,
            close_rect,
        });
        x += total;
    }
    regions
}

/// Render the tab bar into `area` using the same geometry as [`tab_hit_regions`].
pub fn render_tab_bar(
    buf: &mut TuiBuffer,
    area: Rect,
    buffers: &[Buffer],
    active: usize,
    theme: &Theme,
) {
    let base = Style::default().fg(theme.menubar_fg).bg(theme.menubar_bg);
    let active_style = Style::default()
        .fg(theme.menubar_bg)
        .bg(theme.menu_selected_bg)
        .add_modifier(Modifier::BOLD);
    // Clear the row.
    for cx in area.x..area.x + area.width {
        buf.get_mut(cx, area.y).set_symbol(" ").set_style(base);
    }
    let regions = tab_hit_regions(area, buffers, active);
    for r in &regions {
        let style = if r.idx == active { active_style } else { base };
        let label = label_text(&buffers[r.idx]);
        // Draw the label.
        let mut cx = r.label_rect.x;
        for g in label.graphemes(true) {
            if cx >= r.label_rect.x + r.label_rect.width {
                break;
            }
            buf.get_mut(cx, area.y).set_symbol(g).set_style(style);
            cx += grapheme_width(g);
        }
        // Draw the close glyph.
        buf.get_mut(r.close_rect.x, area.y)
            .set_symbol("✕")
            .set_style(style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::theme_by_name;
    use ratatui::buffer::Buffer as TuiBuf;

    fn buf(name: &str, modified: bool) -> Buffer {
        let mut b = Buffer::new_empty();
        if !name.is_empty() {
            b.path = Some(std::path::PathBuf::from(name));
        }
        b.modified = modified;
        b
    }

    #[test]
    fn regions_have_label_and_close_per_visible_tab() {
        let bufs = vec![buf("a.rs", false), buf("b.rs", true)];
        let area = Rect::new(0, 1, 80, 1);
        let r = tab_hit_regions(area, &bufs, 0);
        assert_eq!(r.len(), 2);
        for tr in &r {
            assert!(tr.label_rect.width >= 1);
            assert_eq!(tr.close_rect.width, 1);
            assert_eq!(tr.close_rect.x, tr.label_rect.x + tr.label_rect.width);
        }
        assert!(
            r[1].label_rect.x > r[0].close_rect.x,
            "tabs ordered with a gap"
        );
    }

    #[test]
    fn overflow_keeps_active_visible() {
        // Many long-named buffers in a narrow row; the active (last) must appear.
        let bufs: Vec<Buffer> = (0..20)
            .map(|i| buf(&format!("longfilename_{i:02}.rs"), false))
            .collect();
        let area = Rect::new(0, 1, 30, 1);
        let active = 19;
        let r = tab_hit_regions(area, &bufs, active);
        assert!(r.iter().any(|tr| tr.idx == active), "active tab is visible");
        // All visible tabs stay within the row.
        for tr in &r {
            assert!(tr.close_rect.x < area.x + area.width);
        }
    }

    #[test]
    fn tiny_width_does_not_panic() {
        let bufs = vec![buf("a", false), buf("b", false)];
        for w in [0u16, 1, 2, 3] {
            let _ = tab_hit_regions(Rect::new(0, 1, w, 1), &bufs, 0);
        }
    }

    #[test]
    fn render_shows_names_and_active_marker() {
        let bufs = vec![buf("alpha.rs", false), buf("beta.rs", true)];
        let area = Rect::new(0, 1, 80, 1);
        let mut tb = TuiBuf::empty(Rect::new(0, 0, 80, 2));
        render_tab_bar(&mut tb, area, &bufs, 0, theme_by_name("classic"));
        let row: String = (0..80).map(|x| tb.get(x, 1).symbol().to_string()).collect();
        assert!(row.contains("alpha.rs"));
        assert!(row.contains("beta.rs"));
        assert!(row.contains('●'), "modified marker shown");
        assert!(row.contains('✕'), "close glyph shown");
    }
}
