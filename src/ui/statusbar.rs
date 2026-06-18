//! Task T029: Status bar widget.
//!
//! Renders the single-row status bar at the bottom of the screen, showing:
//! filename (or `[No Name]`), modification / read-only flags, cursor position,
//! and the active encoding.

#![allow(dead_code, unused_variables, unused_imports)]

use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::buffer::Buffer;
use crate::ui::theme::Theme;

// ---------------------------------------------------------------------------
// StatusBar
// ---------------------------------------------------------------------------

/// Widget that renders the bottom status bar.
pub struct StatusBar<'a> {
    /// The active buffer (supplies filename, flags, cursor, encoding).
    pub buffer: &'a Buffer,
    /// The active color theme.
    pub theme: &'static Theme,
    /// 0-based index of the active buffer (shown as N+1 in the UI).
    pub buffer_idx: usize,
    /// Total number of open buffers.
    pub total_buffers: usize,
}

impl<'a> StatusBar<'a> {
    /// Construct a new [`StatusBar`].
    pub fn new(
        buffer: &'a Buffer,
        theme: &'static Theme,
        buffer_idx: usize,
        total_buffers: usize,
    ) -> Self {
        Self {
            buffer,
            theme,
            buffer_idx,
            total_buffers,
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn filename(&self) -> String {
        match &self.buffer.path {
            None => "[No Name]".to_string(),
            Some(p) => {
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "[No Name]".to_string());
                // When multiple buffers are open, include the parent directory
                // to help distinguish files with the same basename.
                if self.total_buffers > 1 {
                    if let Some(parent) = p.parent() {
                        let parent_str = parent
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        if !parent_str.is_empty() {
                            return format!("{}/{}", parent_str, name);
                        }
                    }
                }
                name
            }
        }
    }

    /// Format the buffer position indicator, e.g. `[2/3]` when 3 buffers are open.
    fn buffer_indicator(&self) -> String {
        if self.total_buffers > 1 {
            format!(" [{}/{}]", self.buffer_idx + 1, self.total_buffers)
        } else {
            String::new()
        }
    }

    fn flags(&self) -> &'static str {
        if self.buffer.readonly {
            " [Read Only]"
        } else if self.buffer.modified {
            " [Modified]"
        } else {
            ""
        }
    }

    fn position(&self) -> String {
        let cur = self.buffer.cursor;
        // Display 1-based row:col.
        format!("{}:{}", cur.line + 1, cur.grapheme_col + 1)
    }

    fn encoding(&self) -> &'static str {
        use crate::encoding::EncodingId;
        match self.buffer.encoding {
            EncodingId::Utf8 => "UTF-8",
            EncodingId::Cp437 => "CP437",
            EncodingId::Cp850 => "CP850",
            EncodingId::Iso8859_1 => "ISO-8859-1",
            EncodingId::Windows1252 => "WIN-1252",
        }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let style = Style::default()
            .fg(self.theme.status_fg)
            .bg(self.theme.status_bg);

        // Build left section: filename + buffer indicator + flags
        let filename = self.filename();
        let indicator = self.buffer_indicator();
        let flags = self.flags();
        let left = format!(" {}{}{}", filename, indicator, flags);

        // Build right section: position + encoding
        let position = self.position();
        let encoding = self.encoding();
        let right = format!("{}  {}  ", position, encoding);

        let width = area.width as usize;

        // Fill row with background color first.
        let y = area.top();
        for x in area.left()..area.right() {
            buf.get_mut(x, y).set_style(style).set_char(' ');
        }

        // Write left section.
        let left_chars: Vec<char> = left.chars().collect();
        for (i, ch) in left_chars.iter().enumerate() {
            let x = area.left() + i as u16;
            if x >= area.right() {
                break;
            }
            buf.get_mut(x, y).set_style(style).set_char(*ch);
        }

        // Write right section (right-aligned).
        let right_chars: Vec<char> = right.chars().collect();
        let right_len = right_chars.len();
        if right_len <= width {
            let start_col = width - right_len;
            for (i, ch) in right_chars.iter().enumerate() {
                let x = area.left() + (start_col + i) as u16;
                if x >= area.right() {
                    break;
                }
                buf.get_mut(x, y).set_style(style).set_char(*ch);
            }
        }
    }
}
