//! Tasks T031+: Modal dialog widgets.
//!
//! Provides overlay dialogs rendered as centered bordered boxes:
//! - [`SavePromptDialog`]  — "Save changes?" with Save / Discard / Cancel.
//! - [`ErrorDialog`]       — Simple error message.
//! - [`SaveErrorDialog`]   — "Cannot save" with Retry / Cancel.

#![allow(dead_code, unused_variables, unused_imports)]

use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::encoding::EncodingId;
use crate::ui::theme::Theme;

// ---------------------------------------------------------------------------
// ENCODING_OPTIONS — T006
// ---------------------------------------------------------------------------

/// Ordered list of all supported output encodings shown in the Save As Encoding dialog.
/// Tuple: (EncodingId, display label).
pub const ENCODING_OPTIONS: &[(EncodingId, &str)] = &[
    (EncodingId::Utf8, "UTF-8"),
    (EncodingId::Utf16Le, "UTF-16 LE"),
    (EncodingId::Utf16Be, "UTF-16 BE"),
    (EncodingId::Cp437, "CP437"),
    (EncodingId::Cp850, "CP850"),
    (EncodingId::Iso8859_1, "ISO-8859-1"),
    (EncodingId::Windows1252, "Windows-1252"),
];

// ---------------------------------------------------------------------------
// EncodingSelectDialog — T007
// ---------------------------------------------------------------------------

/// Modal listbox dialog for selecting the output encoding when saving a file.
///
/// Layout (40 cols × 11 rows):
/// ```text
/// ┌─ Save As Encoding ──────────────────┐
/// │  UTF-8                              │  <- highlighted with REVERSED
/// │  UTF-16 LE                          │
/// │  UTF-16 BE                          │
/// │  CP437                              │
/// │  CP850                              │
/// │  ISO-8859-1                         │
/// │  Windows-1252                       │
/// │                                     │
/// │  [↑↓] Select  [Enter] Save  [Esc] Cancel │
/// └─────────────────────────────────────┘
/// ```
pub struct EncodingSelectDialog {
    /// Index into [`ENCODING_OPTIONS`] of the currently highlighted row.
    pub cursor_idx: usize,
    /// Active color theme.
    pub theme: &'static Theme,
}

impl Widget for EncodingSelectDialog {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        let dialog_area = centered_rect(40, 11, area);

        Clear.render(dialog_area, buf);

        let dialog_style = Style::default()
            .fg(self.theme.menubar_fg)
            .bg(self.theme.menubar_bg);
        let selected_style = dialog_style.add_modifier(Modifier::REVERSED);

        // Maximum label chars before truncation (when terminal is smaller than 40 cols).
        let max_label_chars = dialog_area.width.saturating_sub(8) as usize;

        let mut lines: Vec<Line> = Vec::with_capacity(ENCODING_OPTIONS.len() + 2);

        for (i, (_, label)) in ENCODING_OPTIONS.iter().enumerate() {
            let style = if i == self.cursor_idx {
                selected_style
            } else {
                dialog_style
            };

            let display = if dialog_area.width < 40 && max_label_chars > 0 {
                let char_count = label.chars().count();
                if char_count > max_label_chars && max_label_chars >= 1 {
                    let truncated: String = label.chars().take(max_label_chars - 1).collect();
                    format!("{}…", truncated)
                } else {
                    label.to_string()
                }
            } else {
                label.to_string()
            };

            lines.push(Line::from(Span::styled(format!("  {}", display), style)));
        }

        // Blank separator row.
        lines.push(Line::from(Span::raw("")));
        // Hint row.
        lines.push(Line::from(Span::styled(
            "  [↑↓] Select  [Enter] Save  [Esc] Cancel",
            dialog_style,
        )));

        let paragraph = Paragraph::new(lines).style(dialog_style).block(
            Block::default()
                .title("Save As Encoding")
                .borders(Borders::ALL)
                .style(dialog_style),
        );

        paragraph.render(dialog_area, buf);
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Compute a centered [`Rect`] of size `(w, h)` within `area`.
fn centered_rect(w: u16, h: u16, area: Rect) -> Rect {
    let x = area.left() + area.width.saturating_sub(w) / 2;
    let y = area.top() + area.height.saturating_sub(h) / 2;
    Rect {
        x,
        y,
        width: w.min(area.width),
        height: h.min(area.height),
    }
}

// ---------------------------------------------------------------------------
// Tests — T008
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::theme_by_name;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_encoding_options_has_seven_entries() {
        assert_eq!(ENCODING_OPTIONS.len(), 7);
    }

    #[test]
    fn test_encoding_options_first_is_utf8() {
        assert_eq!(ENCODING_OPTIONS[0].0, EncodingId::Utf8);
    }

    #[test]
    fn test_encoding_options_all_labels_nonempty() {
        for (_, label) in ENCODING_OPTIONS {
            assert!(!label.is_empty(), "label must not be empty");
        }
    }

    #[test]
    fn test_encoding_select_dialog_renders_without_panic() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let dialog = EncodingSelectDialog {
                    cursor_idx: 0,
                    theme: theme_by_name("classic"),
                };
                frame.render_widget(dialog, frame.size());
            })
            .unwrap();
    }

    #[test]
    fn test_encoding_select_dialog_small_terminal_no_panic() {
        let backend = TestBackend::new(25, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let dialog = EncodingSelectDialog {
                    cursor_idx: 2,
                    theme: theme_by_name("classic"),
                };
                frame.render_widget(dialog, frame.size());
            })
            .unwrap();
        // Verify the rendered buffer fits within 25×8
        let buf = terminal.backend().buffer().clone();
        assert_eq!(buf.area.width, 25);
        assert_eq!(buf.area.height, 8);
    }
}

// ---------------------------------------------------------------------------
// SavePromptResponse
// ---------------------------------------------------------------------------

/// The user's response to a save-prompt dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavePromptResponse {
    /// Write the buffer to disk before closing.
    Save,
    /// Discard unsaved changes and close.
    Discard,
    /// Dismiss the dialog and return to the editor.
    Cancel,
}

// ---------------------------------------------------------------------------
// SavePromptDialog
// ---------------------------------------------------------------------------

/// Modal dialog: "Save changes to <filename>?"
///
/// Key bindings (interpreted by the caller):
/// - `S` / `s` → [`SavePromptResponse::Save`]
/// - `D` / `d` → [`SavePromptResponse::Discard`]
/// - `C` / `c` / `Esc` → [`SavePromptResponse::Cancel`]
pub struct SavePromptDialog<'a> {
    /// The filename shown in the dialog title.
    pub filename: &'a str,
    /// The active color theme.
    pub theme: &'static Theme,
}

impl<'a> SavePromptDialog<'a> {
    /// Construct a new [`SavePromptDialog`].
    pub fn new(filename: &'a str, theme: &'static Theme) -> Self {
        Self { filename, theme }
    }
}

impl<'a> Widget for SavePromptDialog<'a> {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        // Dialog dimensions: enough for the title + message + key hints.
        let dialog_w: u16 = 52.min(area.width);
        let dialog_h: u16 = 5.min(area.height);
        let dialog_area = centered_rect(dialog_w, dialog_h, area);

        // Clear the area behind the dialog.
        Clear.render(dialog_area, buf);

        let dialog_style = Style::default()
            .fg(self.theme.menubar_fg)
            .bg(self.theme.menubar_bg);

        let title = format!("Save — {}", self.filename);
        let body = "Save changes to this file?";
        let hint = "  [S]ave   [D]iscard   [C]ancel  ";

        let text = vec![
            Line::from(Span::raw(body)),
            Line::from(Span::raw("")),
            Line::from(Span::raw(hint)),
        ];

        let paragraph = Paragraph::new(text).style(dialog_style).block(
            Block::default()
                .title(title.as_str())
                .borders(Borders::ALL)
                .style(dialog_style),
        );

        paragraph.render(dialog_area, buf);
    }
}

// ---------------------------------------------------------------------------
// ErrorDialog
// ---------------------------------------------------------------------------

/// Modal dialog that shows a plain error message.
pub struct ErrorDialog {
    /// The error message to display.
    pub message: String,
    /// The active color theme.
    pub theme: &'static Theme,
}

impl ErrorDialog {
    /// Construct a new [`ErrorDialog`].
    pub fn new(message: impl Into<String>, theme: &'static Theme) -> Self {
        Self {
            message: message.into(),
            theme,
        }
    }
}

impl Widget for ErrorDialog {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        let msg_len = self.message.len() as u16 + 4; // padding
        let dialog_w: u16 = msg_len.max(30).min(area.width);
        let dialog_h: u16 = 4.min(area.height);
        let dialog_area = centered_rect(dialog_w, dialog_h, area);

        Clear.render(dialog_area, buf);

        let dialog_style = Style::default().fg(Color::Red).bg(self.theme.menubar_bg);

        let text = vec![
            Line::from(Span::raw(self.message.as_str())),
            Line::from(Span::raw("")),
            Line::from(Span::raw("  Press any key to continue  ")),
        ];

        let paragraph = Paragraph::new(text).style(dialog_style).block(
            Block::default()
                .title("Error")
                .borders(Borders::ALL)
                .style(dialog_style),
        );

        paragraph.render(dialog_area, buf);
    }
}

// ---------------------------------------------------------------------------
// SaveErrorResponse
// ---------------------------------------------------------------------------

/// The user's response to a save-error dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveErrorResponse {
    /// Attempt to save again.
    Retry,
    /// Give up saving.
    Cancel,
}

// ---------------------------------------------------------------------------
// SaveErrorDialog
// ---------------------------------------------------------------------------

/// Modal dialog: "Cannot save: <error>. Retry / Cancel"
pub struct SaveErrorDialog {
    /// The error string returned from the save attempt.
    pub error: String,
    /// The active color theme.
    pub theme: &'static Theme,
}

impl SaveErrorDialog {
    /// Construct a new [`SaveErrorDialog`].
    pub fn new(error: impl Into<String>, theme: &'static Theme) -> Self {
        Self {
            error: error.into(),
            theme,
        }
    }
}

impl Widget for SaveErrorDialog {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        let err_len = self.error.len() as u16 + 4;
        let dialog_w: u16 = err_len.max(40).min(area.width);
        let dialog_h: u16 = 5.min(area.height);
        let dialog_area = centered_rect(dialog_w, dialog_h, area);

        Clear.render(dialog_area, buf);

        let dialog_style = Style::default().fg(Color::Red).bg(self.theme.menubar_bg);

        let error_line = format!("Cannot save: {}", self.error);
        let text = vec![
            Line::from(Span::raw(error_line)),
            Line::from(Span::raw("")),
            Line::from(Span::raw("  [R]etry   [C]ancel  ")),
        ];

        let paragraph = Paragraph::new(text).style(dialog_style).block(
            Block::default()
                .title("Save Error")
                .borders(Borders::ALL)
                .style(dialog_style),
        );

        paragraph.render(dialog_area, buf);
    }
}

// ---------------------------------------------------------------------------
// FindDialog — T054
// ---------------------------------------------------------------------------

/// Modal find dialog.
///
/// Displays the current search query plus toggles for regex and case-sensitive
/// modes.  Full text-input handling is wired in the event layer; this widget
/// is responsible only for rendering.
///
/// Layout:
/// ```text
/// ┌─ Find ───────────────────────────────────┐
/// │  Find: [<query>]  [Regex: Y/N]  [Case: Y/N]  │
/// └──────────────────────────────────────────┘
/// ```
pub struct FindDialog {
    /// The current search query being typed.
    pub query: String,
    /// Whether regex mode is enabled.
    pub regex_mode: bool,
    /// Whether the search is case-sensitive.
    pub case_sensitive: bool,
}

impl FindDialog {
    /// Construct a new [`FindDialog`].
    pub fn new(query: impl Into<String>, regex_mode: bool, case_sensitive: bool) -> Self {
        Self {
            query: query.into(),
            regex_mode,
            case_sensitive,
        }
    }
}

impl Widget for FindDialog {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        // Enough width for the full content line.
        let dialog_w: u16 = 56.min(area.width);
        let dialog_h: u16 = 4.min(area.height);
        let dialog_area = centered_rect(dialog_w, dialog_h, area);

        Clear.render(dialog_area, buf);

        let dialog_style = Style::default().fg(Color::White).bg(Color::DarkGray);

        let regex_flag = if self.regex_mode { "Y" } else { "N" };
        let case_flag = if self.case_sensitive { "Y" } else { "N" };

        let content = format!(
            "Find: [{}]  [Regex: {}]  [Case: {}]",
            self.query, regex_flag, case_flag
        );

        let text = vec![
            Line::from(Span::raw(content)),
            Line::from(Span::raw("")),
            Line::from(Span::raw("  Enter: find next   Esc: cancel  ")),
        ];

        let paragraph = Paragraph::new(text).style(dialog_style).block(
            Block::default()
                .title("Find")
                .borders(Borders::ALL)
                .style(dialog_style),
        );

        paragraph.render(dialog_area, buf);
    }
}

// ---------------------------------------------------------------------------
// ReplaceDialog — T056
// ---------------------------------------------------------------------------

/// Modal find-and-replace dialog.
///
/// Displays both query and replacement input fields.
///
/// Layout:
/// ```text
/// ┌─ Find & Replace ──────────────────────────────┐
/// │  Find:    [<query>]                            │
/// │  Replace: [<replacement>]  [Regex: Y/N]  [Case: Y/N] │
/// │                                                │
/// │    [A]ll   Enter: replace next   Esc: cancel   │
/// └────────────────────────────────────────────────┘
/// ```
pub struct ReplaceDialog {
    /// The current search query.
    pub query: String,
    /// The replacement string.
    pub replacement: String,
    /// Whether regex mode is enabled.
    pub regex_mode: bool,
    /// Whether the search is case-sensitive.
    pub case_sensitive: bool,
}

impl ReplaceDialog {
    /// Construct a new [`ReplaceDialog`].
    pub fn new(
        query: impl Into<String>,
        replacement: impl Into<String>,
        regex_mode: bool,
        case_sensitive: bool,
    ) -> Self {
        Self {
            query: query.into(),
            replacement: replacement.into(),
            regex_mode,
            case_sensitive,
        }
    }
}

impl Widget for ReplaceDialog {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        let dialog_w: u16 = 60.min(area.width);
        let dialog_h: u16 = 7.min(area.height);
        let dialog_area = centered_rect(dialog_w, dialog_h, area);

        Clear.render(dialog_area, buf);

        let dialog_style = Style::default().fg(Color::White).bg(Color::DarkGray);

        let regex_flag = if self.regex_mode { "Y" } else { "N" };
        let case_flag = if self.case_sensitive { "Y" } else { "N" };

        let find_line = format!("Find:    [{}]", self.query);
        let replace_line = format!(
            "Replace: [{}]  [Regex: {}]  [Case: {}]",
            self.replacement, regex_flag, case_flag
        );

        let text = vec![
            Line::from(Span::raw(find_line)),
            Line::from(Span::raw(replace_line)),
            Line::from(Span::raw("")),
            Line::from(Span::raw("  [A]ll   Enter: replace next   Esc: cancel  ")),
        ];

        let paragraph = Paragraph::new(text).style(dialog_style).block(
            Block::default()
                .title("Find & Replace")
                .borders(Borders::ALL)
                .style(dialog_style),
        );

        paragraph.render(dialog_area, buf);
    }
}

// ---------------------------------------------------------------------------
// RecoveryDialog — T064 / US5
// ---------------------------------------------------------------------------

/// Modal dialog shown when a stale recovery file is detected for the buffer's path.
///
/// Key bindings (interpreted by the caller):
/// - `Y` / `y` / `Enter` → accept recovery (load the auto-saved content)
/// - `N` / `n` / `Esc`   → discard recovery (open the file from disk)
pub struct RecoveryDialog {
    /// Unix epoch seconds from the recovery file's `timestamp` field.
    pub timestamp: u64,
    /// The path of the file that the recovery belongs to.
    pub path: String,
    /// The active color theme.
    pub theme: &'static Theme,
}

impl RecoveryDialog {
    /// Construct a new [`RecoveryDialog`].
    pub fn new(timestamp: u64, path: impl Into<String>, theme: &'static Theme) -> Self {
        Self {
            timestamp,
            path: path.into(),
            theme,
        }
    }
}

impl Widget for RecoveryDialog {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        let dialog_w: u16 = 62.min(area.width);
        let dialog_h: u16 = 6.min(area.height);
        let dialog_area = centered_rect(dialog_w, dialog_h, area);

        Clear.render(dialog_area, buf);

        let dialog_style = Style::default().fg(Color::Yellow).bg(self.theme.menubar_bg);

        let ts_str = format!("unix:{}", self.timestamp);
        let msg1 = format!("Recovery file found from {}.", ts_str);
        let path_display = if self.path.len() > 50 {
            format!("...{}", &self.path[self.path.len() - 47..])
        } else {
            self.path.clone()
        };
        let msg2 = format!("File: {}", path_display);
        let hint = "  [Y]es — restore   [N]o — open from disk  ";

        let text = vec![
            Line::from(Span::raw(msg1)),
            Line::from(Span::raw(msg2)),
            Line::from(Span::raw("")),
            Line::from(Span::raw(hint)),
        ];

        let paragraph = Paragraph::new(text).style(dialog_style).block(
            Block::default()
                .title("Recover unsaved changes?")
                .borders(Borders::ALL)
                .style(dialog_style),
        );

        paragraph.render(dialog_area, buf);
    }
}

// ---------------------------------------------------------------------------
// LocaleWarningDialog
// ---------------------------------------------------------------------------

/// Modal dialog: warns the user that the current locale is not UTF-8.
///
/// Displayed at startup when the resolved locale does not contain "UTF-8",
/// since the editor requires a UTF-8 locale for correct Unicode display.
///
/// Key binding (interpreted by the caller):
/// - any key → dismiss the dialog and continue
pub struct LocaleWarningDialog {
    /// The detected locale string (e.g. "en_US.ISO-8859-1").
    pub detected_locale: String,
    /// The active color theme.
    pub theme: &'static Theme,
}

impl LocaleWarningDialog {
    /// Construct a new [`LocaleWarningDialog`].
    pub fn new(detected_locale: impl Into<String>, theme: &'static Theme) -> Self {
        Self {
            detected_locale: detected_locale.into(),
            theme,
        }
    }
}

impl Widget for LocaleWarningDialog {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        // The message line is the widest part; compute the required width.
        let msg = format!("Warning: locale {} is not UTF-8.", self.detected_locale);
        let msg_len = msg.len() as u16 + 4; // 2-char padding each side
        let dialog_w: u16 = msg_len.max(50).min(area.width);
        let dialog_h: u16 = 6.min(area.height);
        let dialog_area = centered_rect(dialog_w, dialog_h, area);

        Clear.render(dialog_area, buf);

        let dialog_style = Style::default().fg(Color::Yellow).bg(self.theme.menubar_bg);

        let text = vec![
            Line::from(Span::raw(msg)),
            Line::from(Span::raw("Unicode display may be incorrect.")),
            Line::from(Span::raw("")),
            Line::from(Span::raw("  Press any key to continue  ")),
        ];

        let paragraph = Paragraph::new(text).style(dialog_style).block(
            Block::default()
                .title("Locale Warning")
                .borders(Borders::ALL)
                .style(dialog_style),
        );

        paragraph.render(dialog_area, buf);
    }
}
