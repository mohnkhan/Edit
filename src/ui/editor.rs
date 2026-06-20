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
    /// Enable soft-wrap visual rendering (Feature 005).
    pub soft_wrap: bool,
    /// Pre-computed visual sub-line start byte offsets per logical line.
    /// None when soft_wrap is false.
    pub wrap_starts: Option<&'a [Vec<u32>]>,
    /// Active search match ranges (char indices) to highlight (Feature 015).
    pub match_ranges: &'a [crate::search::CharRange],
    /// Index into `match_ranges` of the current match (rendered distinctly).
    pub active_match: Option<usize>,
}

impl<'a> EditorWidget<'a> {
    /// Construct a new [`EditorWidget`].
    pub fn new(
        buffer: &'a Buffer,
        theme: &'static Theme,
        show_line_numbers: bool,
        soft_wrap: bool,
        wrap_starts: Option<&'a [Vec<u32>]>,
    ) -> Self {
        Self {
            buffer,
            theme,
            show_line_numbers,
            soft_wrap,
            wrap_starts,
            match_ranges: &[],
            active_match: None,
        }
    }

    /// Attach search match highlighting (Feature 015).
    pub fn with_matches(
        mut self,
        match_ranges: &'a [crate::search::CharRange],
        active_match: Option<usize>,
    ) -> Self {
        self.match_ranges = match_ranges;
        self.active_match = active_match;
        self
    }

    /// Index of the match range containing char index `c`, if any (Feature 015).
    fn match_at(&self, c: usize) -> Option<usize> {
        self.match_ranges
            .iter()
            .position(|r| c >= r.start && c < r.end)
    }

    /// Whether `(line, gcol)` falls within the buffer's selection (Feature 017).
    /// Direction-independent: the anchor/active are ordered first.
    fn selected(&self, line: usize, gcol: usize) -> bool {
        let sel = match &self.buffer.selection {
            Some(s) => s,
            None => return false,
        };
        let a = (sel.anchor.line, sel.anchor.grapheme_col);
        let b = (sel.active.line, sel.active.grapheme_col);
        let (start, end) = if a <= b { (a, b) } else { (b, a) };
        if start == end {
            return false;
        }
        let p = (line, gcol);
        p >= start && p < end
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
            .fg(self.theme.background) // invert: use background as fg …
            .bg(self.theme.foreground); // … and foreground as bg

        // Fill the entire area with the background color first.
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf.get_mut(x, y).set_style(normal_style).set_char(' ');
            }
        }

        /// Look up the highlight style for a byte offset, falling back to
        /// `normal_style` when no span covers that offset.
        #[inline]
        fn span_style_at(spans: &[Span], byte_off: usize, normal: Style) -> Style {
            // Spans are sorted and non-overlapping; binary search for the
            // last span whose start <= byte_off, then check containment.
            match spans.binary_search_by_key(&byte_off, |s| s.start) {
                Ok(i) => {
                    // Exact match on start.
                    if byte_off < spans[i].end {
                        spans[i].style
                    } else {
                        normal
                    }
                }
                Err(0) => normal,
                Err(i) => {
                    let s = &spans[i - 1];
                    if byte_off < s.end {
                        s.style
                    } else {
                        normal
                    }
                }
            }
        }

        // ── Soft-wrap rendering branch (Feature 005) ─────────────────────────
        if self.soft_wrap {
            if let Some(wrap_starts) = self.wrap_starts {
                let gutter_cols = if self.show_line_numbers {
                    Self::GUTTER_WIDTH
                } else {
                    0
                };
                let content_width = area.width.saturating_sub(gutter_cols) as usize;
                let content_x_start = area.left() + gutter_cols;
                let scroll_visual_row = self.buffer.scroll_offset.0;
                let cursor = self.buffer.cursor;
                let visible_rows = area.height as usize;
                let total_lines = self.buffer.rope.line_count();

                let mut global_visual_row: usize = 0;
                let mut screen_row: usize = 0;

                'wrap_outer: for (logical_line, starts) in wrap_starts.iter().enumerate() {
                    if logical_line >= total_lines {
                        break;
                    }
                    let line_str = self.buffer.rope.line_slice(logical_line);

                    // Pre-compute cursor byte offset for this logical line.
                    let cursor_byte: Option<usize> = if cursor.line == logical_line {
                        let b: usize = line_str
                            .graphemes(true)
                            .take(cursor.grapheme_col)
                            .map(|g| g.len())
                            .sum();
                        Some(b)
                    } else {
                        None
                    };

                    let hl_spans: Vec<Span> = self
                        .buffer
                        .syntax
                        .as_ref()
                        .map(|h| h.highlight(&line_str))
                        .unwrap_or_default();

                    for (seg_idx, &seg_start_u32) in starts.iter().enumerate() {
                        let seg_start = seg_start_u32 as usize;
                        let seg_end = if seg_idx + 1 < starts.len() {
                            starts[seg_idx + 1] as usize
                        } else {
                            line_str.len()
                        };

                        if global_visual_row < scroll_visual_row {
                            global_visual_row += 1;
                            continue;
                        }
                        if screen_row >= visible_rows {
                            break 'wrap_outer;
                        }

                        let screen_y = area.top() + screen_row as u16;
                        screen_row += 1;

                        // Gutter.
                        if self.show_line_numbers {
                            let gs = Style::default()
                                .fg(Color::DarkGray)
                                .bg(self.theme.background);
                            let gt = if seg_idx == 0 {
                                format!("{:3}|", logical_line + 1)
                            } else {
                                "   |".to_string()
                            };
                            for (i, ch) in gt.chars().enumerate() {
                                let gx = area.left() + i as u16;
                                if gx >= content_x_start {
                                    break;
                                }
                                buf.get_mut(gx, screen_y).set_style(gs).set_char(ch);
                            }
                        }

                        // Continuation marker '»'.
                        let text_offset = if seg_idx > 0 {
                            if content_width > 0 {
                                buf.get_mut(content_x_start, screen_y)
                                    .set_style(normal_style)
                                    .set_symbol("»");
                            }
                            1usize
                        } else {
                            0usize
                        };

                        // Walk graphemes in [seg_start, seg_end).
                        // Feature 028: clamp to the current line length so a stale
                        // wrap cache (offsets computed for different content — e.g.
                        // right after a session restore or buffer switch) can never
                        // slice out of bounds. Renders truncated/blank, never panics.
                        let line_len = line_str.len();
                        let s = seg_start.min(line_len);
                        let e = seg_end.min(line_len).max(s);
                        let seg_str = &line_str[s..e];
                        let mut screen_col = text_offset;
                        let mut byte_in_seg: usize = 0;

                        for grapheme in seg_str.graphemes(true) {
                            let gw = UnicodeWidthStr::width(grapheme);
                            let gbytes = grapheme.len();
                            let abs_byte = seg_start + byte_in_seg;

                            if screen_col + gw > content_width {
                                break;
                            }

                            let is_cursor = cursor_byte == Some(abs_byte);
                            let base_style = if is_cursor {
                                cursor_style
                            } else {
                                span_style_at(&hl_spans, abs_byte, normal_style)
                            };
                            let style = if is_cursor {
                                base_style
                            } else {
                                base_style.bg(self.theme.background)
                            };

                            let px = content_x_start + screen_col as u16;
                            buf.get_mut(px, screen_y)
                                .set_style(style)
                                .set_symbol(grapheme);
                            screen_col += gw;
                            byte_in_seg += gbytes;
                        }

                        // Cursor past end of line (last segment only).
                        if seg_idx + 1 >= starts.len() {
                            if let Some(cb) = cursor_byte {
                                if cb >= line_str.len() && screen_col < content_width {
                                    let px = content_x_start + screen_col as u16;
                                    buf.get_mut(px, screen_y)
                                        .set_style(cursor_style)
                                        .set_char(' ');
                                }
                            }
                        }

                        global_visual_row += 1;
                    }
                }
                return; // soft-wrap render complete
            }
        }
        // ── End soft-wrap branch ──────────────────────────────────────────────

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

            // Feature 015: char index of the first char on this line, for
            // mapping match ranges (char indices) onto rendered cells.
            let line_start_char = self.buffer.rope.line_to_char(file_line);

            // Walk grapheme clusters, skipping those before scroll_vcol and
            // collecting those that fit in content_width.
            let mut visual_col: usize = 0;
            let mut screen_col: usize = 0; // position within the content area
            let mut byte_off: usize = 0; // byte offset within line_str
            let mut char_off: usize = 0; // char offset within line_str
            let mut gcol: usize = 0; // grapheme column within line (Feature 017)

            for grapheme in line_str.graphemes(true) {
                let gw = UnicodeWidthStr::width(grapheme);
                let gbytes = grapheme.len();

                // Skip graphemes that are entirely before the horizontal scroll.
                if visual_col + gw <= scroll_vcol {
                    visual_col += gw;
                    byte_off += gbytes;
                    char_off += grapheme.chars().count();
                    gcol += 1;
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
                    char_off += grapheme.chars().count();
                    gcol += 1;
                    continue;
                }

                // Grapheme fits entirely within the viewport.
                if screen_col + gw > content_width {
                    break; // no room
                }

                // Determine whether this grapheme is under the cursor.
                let is_cursor = file_line == cursor.line && visual_col == cursor.visual_col;

                // Pick the base style: cursor overrides syntax highlight.
                let base_style = if is_cursor {
                    cursor_style
                } else {
                    span_style_at(&hl_spans, byte_off, normal_style)
                };

                // Preserve the background from normal_style so highlights
                // don't accidentally turn the background black.
                let mut style = if is_cursor {
                    base_style
                } else {
                    base_style.bg(self.theme.background)
                };

                // Feature 015: overlay a search-match background (current match
                // distinct from other matches). Cursor cell keeps its own style.
                if !is_cursor {
                    let abs_char = line_start_char + char_off;
                    if let Some(mi) = self.match_at(abs_char) {
                        let is_active = self.active_match == Some(mi);
                        style = if is_active {
                            style
                                .bg(ratatui::style::Color::LightYellow)
                                .fg(ratatui::style::Color::Black)
                                .add_modifier(ratatui::style::Modifier::BOLD)
                        } else {
                            style
                                .bg(ratatui::style::Color::Yellow)
                                .fg(ratatui::style::Color::Black)
                        };
                    }
                }

                // Feature 017: selection highlight (reverse video), distinct from
                // the search-match colors. The cursor cell keeps its own style.
                if !is_cursor && self.selected(file_line, gcol) {
                    style = style.add_modifier(ratatui::style::Modifier::REVERSED);
                }

                // Write the grapheme. For wide chars (gw == 2) ratatui will
                // automatically place a space in the second cell.
                let px = content_x_start + screen_col as u16;
                buf.get_mut(px, screen_y)
                    .set_style(style)
                    .set_symbol(grapheme);

                screen_col += gw;
                visual_col += gw;
                byte_off += gbytes;
                char_off += grapheme.chars().count();
                gcol += 1;
            }

            // If the cursor is at the end of the line (past all graphemes) and
            // this is the cursor line, highlight that position.
            let cursor_past_eol = file_line == cursor.line && cursor.visual_col >= visual_col;

            if cursor_past_eol {
                let cursor_screen_col = cursor.visual_col.saturating_sub(scroll_vcol);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::rope::EditorRope;
    use crate::ui::theme::theme_by_name;
    use ratatui::layout::Rect;

    fn render_with(buf: &Buffer, soft_wrap: bool, wrap_starts: Option<&[Vec<u32>]>) {
        let area = Rect::new(0, 0, 40, 10);
        let mut tb = TuiBuffer::empty(area);
        let w = EditorWidget::new(buf, theme_by_name("classic"), false, soft_wrap, wrap_starts);
        w.render(area, &mut tb); // must not panic
    }

    // T003 (Feature 028): a stale/oversized wrap cache must never make the
    // soft-wrap renderer slice out of bounds — the headline session-restore crash.
    #[test]
    fn render_with_stale_oversized_wrap_cache_does_not_panic() {
        let mut buf = Buffer::new_empty();
        // Two short lines + one empty line (the crash repro was an empty line).
        buf.rope = EditorRope::from_str("ab\n\ncd\n");
        // Wrap cache built for DIFFERENT, much longer content: segment offsets far
        // exceed each line's length (e.g. 227 into a 0-length line).
        let stale: Vec<Vec<u32>> = vec![vec![0, 227], vec![0, 99], vec![0, 5], vec![0]];
        render_with(&buf, true, Some(&stale));
    }

    #[test]
    fn render_empty_buffer_softwrap_does_not_panic() {
        let buf = Buffer::new_empty();
        let stale: Vec<Vec<u32>> = vec![vec![0, 50]];
        render_with(&buf, true, Some(&stale));
    }

    #[test]
    fn render_nonwrap_does_not_panic() {
        let mut buf = Buffer::new_empty();
        buf.rope = EditorRope::from_str("hello\nworld\n");
        render_with(&buf, false, None);
    }
}
