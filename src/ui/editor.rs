//! Task T028: Editor area widget.
//!
//! Renders the main editing area: file content with optional line-number gutter,
//! horizontal scrolling (no soft-wrap, matching EDIT.COM behaviour), and cursor
//! highlight.  T076 adds syntax-highlight span overlay.

#![allow(dead_code, unused_variables, unused_imports)]

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::buffer::Buffer;
use crate::highlight::Span;
use crate::ui::theme::Theme;

// ---------------------------------------------------------------------------
// EditorWidget
// ---------------------------------------------------------------------------

/// Widget that renders the main editing area for one [`Buffer`].
pub struct EditorWidget<'a> {
    /// The buffer whose contents are rendered.
    pub buffer: &'a Buffer,
    /// The active color theme.
    pub theme: &'static Theme,
    /// Whether to show the line-number gutter.
    pub show_line_numbers: bool,
}

impl<'a> EditorWidget<'a> {
    /// Construct a new [`EditorWidget`].
    pub fn new(buffer: &'a Buffer, theme: &'static Theme, show_line_numbers: bool) -> Self {
        Self {
            buffer,
            theme,
            show_line_numbers,
        }
    }

    /// Width of the gutter (including the `|` separator) when line numbers are shown.
    ///
    /// Format: `NNN|` — always 4 columns.
    const GUTTER_WIDTH: u16 = 4;
}

impl<'a> Widget for EditorWidget<'a> {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let normal_style = Style::default()
            .fg(self.theme.foreground)
            .bg(self.theme.background);
        let cursor_style = Style::default()
            .fg(self.theme.background)  // invert: use background as fg …
            .bg(self.theme.foreground); // … and foreground as bg

        // Fill the entire area with the background color first.
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf.get_mut(x, y).set_style(normal_style).set_char(' ');
            }
        }

        let gutter_cols = if self.show_line_numbers {
            Self::GUTTER_WIDTH
        } else {
            0
        };

        let content_width = area.width.saturating_sub(gutter_cols) as usize;
        let content_x_start = area.left() + gutter_cols;

        let (scroll_line, scroll_vcol) = self.buffer.scroll_offset;
        let cursor = self.buffer.cursor;

        let visible_rows = area.height as usize;
        let total_lines = self.buffer.rope.line_count();

        for row_idx in 0..visible_rows {
            let file_line = scroll_line + row_idx;
            let screen_y = area.top() + row_idx as u16;

            // ── Gutter ──────────────────────────────────────────────────────
            if self.show_line_numbers {
                let gutter_style = Style::default()
                    .fg(Color::DarkGray)
                    .bg(self.theme.background);
                let gutter_text = if file_line < total_lines {
                    // 1-based line numbers, capped at 3 digits.
                    format!("{:3}|", file_line + 1)
                } else {
                    "   |".to_string()
                };
                for (i, ch) in gutter_text.chars().enumerate() {
                    let gx = area.left() + i as u16;
                    if gx >= content_x_start {
                        break;
                    }
                    buf.get_mut(gx, screen_y)
                        .set_style(gutter_style)
                        .set_char(ch);
                }
            }

            // ── Content line ─────────────────────────────────────────────────
            if file_line >= total_lines {
                // Beyond EOF — already filled with background above.
                continue;
            }

            let line_str = self.buffer.rope.line_slice(file_line);

            // Compute syntax highlight spans for this line (T076).
            let hl_spans: Vec<Span> = self
                .buffer
                .syntax
                .as_ref()
                .map(|h| h.highlight(&line_str))
                .unwrap_or_default();

            /// Look up the highlight style for a byte offset, falling back to
            /// `normal_style` when no span covers that offset.
            #[inline]
            fn span_style_at(spans: &[Span], byte_off: usize, normal: Style) -> Style {
                // Spans are sorted and non-overlapping; binary search for the
                // last span whose start <= byte_off, then check containment.
                match spans.binary_search_by_key(&byte_off, |s| s.start) {
                    Ok(i) => {
                        // Exact match on start.
                        if byte_off < spans[i].end { spans[i].style } else { normal }
                    }
                    Err(0) => normal,
                    Err(i) => {
                        let s = &spans[i - 1];
                        if byte_off < s.end { s.style } else { normal }
                    }
                }
            }

            // Walk grapheme clusters, skipping those before scroll_vcol and
            // collecting those that fit in content_width.
            let mut visual_col: usize = 0;
            let mut screen_col: usize = 0; // position within the content area
            let mut byte_off: usize = 0;   // byte offset within line_str

            for grapheme in line_str.graphemes(true) {
                let gw = UnicodeWidthStr::width(grapheme);
                let gbytes = grapheme.len();

                // Skip graphemes that are entirely before the horizontal scroll.
                if visual_col + gw <= scroll_vcol {
                    visual_col += gw;
                    byte_off += gbytes;
                    continue;
                }

                // Partially visible grapheme at the left edge — treat as space.
                if visual_col < scroll_vcol {
                    // The grapheme straddles the scroll boundary; fill its
                    // visible portion with spaces.
                    let visible_part = (visual_col + gw).saturating_sub(scroll_vcol);
                    for _ in 0..visible_part {
                        if screen_col >= content_width {
                            break;
                        }
                        let px = content_x_start + screen_col as u16;
                        buf.get_mut(px, screen_y)
                            .set_style(normal_style)
                            .set_char(' ');
                        screen_col += 1;
                    }
                    visual_col += gw;
                    byte_off += gbytes;
                    continue;
                }

                // Grapheme fits entirely within the viewport.
                if screen_col + gw > content_width {
                    break; // no room
                }

                // Determine whether this grapheme is under the cursor.
                let is_cursor = file_line == cursor.line
                    && visual_col == cursor.visual_col;

                // Pick the base style: cursor overrides syntax highlight.
                let base_style = if is_cursor {
                    cursor_style
                } else {
                    span_style_at(&hl_spans, byte_off, normal_style)
                };

                // Preserve the background from normal_style so highlights
                // don't accidentally turn the background black.
                let style = if is_cursor {
                    base_style
                } else {
                    base_style.bg(self.theme.background)
                };

                // Write the grapheme. For wide chars (gw == 2) ratatui will
                // automatically place a space in the second cell.
                let px = content_x_start + screen_col as u16;
                buf.get_mut(px, screen_y)
                    .set_style(style)
                    .set_symbol(grapheme);

                screen_col += gw;
                visual_col += gw;
                byte_off += gbytes;
            }

            // If the cursor is at the end of the line (past all graphemes) and
            // this is the cursor line, highlight that position.
            let cursor_past_eol = file_line == cursor.line
                && cursor.visual_col >= visual_col;

            if cursor_past_eol {
                let cursor_screen_col =
                    cursor.visual_col.saturating_sub(scroll_vcol);
                if cursor_screen_col < content_width {
                    let px = content_x_start + cursor_screen_col as u16;
                    buf.get_mut(px, screen_y)
                        .set_style(cursor_style)
                        .set_char(' ');
                }
            }
        }
    }
}
