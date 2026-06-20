//! Task T032: User interface subsystem root.
//!
//! Exports all UI sub-modules and provides the top-level [`Ui::render`]
//! function that composes the full terminal frame from application state.

#![allow(dead_code, unused_variables, unused_imports)]

pub mod buttons;
pub mod dialog;
pub mod editor;
pub mod file_browser;
pub mod menubar;
pub mod plugin_manager;
pub mod statusbar;
pub mod theme;
pub mod wrap;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::{App, HelpScreen};
use crate::ui::{
    editor::EditorWidget,
    menubar::{resolve_menus, MenuBarWidget},
    statusbar::StatusBar,
};

// ---------------------------------------------------------------------------
// SplitMode — T067
// ---------------------------------------------------------------------------

/// How the editor area is divided between buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitMode {
    /// Single-panel view — only the active buffer is shown.
    #[default]
    Single,
    /// Two equal panels side-by-side.
    /// - Left panel always shows `buffers[0]`.
    /// - Right panel shows `buffers[active_idx.max(1)]` (or 0 when there is only one buffer).
    Vertical,
}

// ---------------------------------------------------------------------------
// Ui
// ---------------------------------------------------------------------------

/// Stateless renderer that composes the full terminal frame.
pub struct Ui;

impl Ui {
    /// Render the complete editor UI into `frame`.
    ///
    /// Layout (top → bottom):
    /// - Row 0: menu bar
    /// - Rows 1..height-1: editor area
    /// - Last row: status bar
    ///
    /// If a dialog is active it is drawn as an overlay on top.
    pub fn render(frame: &mut Frame, app: &App) {
        let size = frame.size();

        // ── Layout ──────────────────────────────────────────────────────────
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // menu bar
                Constraint::Min(1),    // editor
                Constraint::Length(1), // status bar
            ])
            .split(size);

        let menubar_area = chunks[0];
        let editor_area = chunks[1];
        let statusbar_area = chunks[2];

        // ── Editor area ───────────────────────────────────────────────────────
        let buf = app.active_buffer();
        let show_line_numbers = app.config.line_numbers;

        match app.split_mode {
            SplitMode::Single => {
                let wrap_starts = app.wrap_cache.as_ref().map(|c| c.visual_starts.as_slice());
                let editor_widget = EditorWidget::new(
                    buf,
                    app.theme,
                    show_line_numbers,
                    app.soft_wrap,
                    wrap_starts,
                )
                .with_matches(&app.search_state.matches, app.search_state.active_match);
                frame.render_widget(editor_widget, editor_area);
            }
            SplitMode::Vertical => {
                let half_width = editor_area.width / 2;
                let left_area =
                    Rect::new(editor_area.x, editor_area.y, half_width, editor_area.height);
                let right_area = Rect::new(
                    editor_area.x + half_width,
                    editor_area.y,
                    editor_area.width - half_width,
                    editor_area.height,
                );
                frame.render_widget(
                    EditorWidget::new(
                        &app.buffers[0],
                        app.theme,
                        show_line_numbers,
                        app.soft_wrap,
                        app.wrap_cache.as_ref().map(|c| c.visual_starts.as_slice()),
                    ),
                    left_area,
                );
                let right_buf_idx = if app.buffers.len() > 1 {
                    app.active_idx.max(1)
                } else {
                    0
                };
                frame.render_widget(
                    EditorWidget::new(
                        &app.buffers[right_buf_idx],
                        app.theme,
                        show_line_numbers,
                        app.soft_wrap,
                        app.wrap_cache.as_ref().map(|c| c.visual_starts.as_slice()),
                    ),
                    right_area,
                );
            }
        }

        // ── Status bar ────────────────────────────────────────────────────────
        // Feature 007 watcher notice takes precedence; otherwise show the
        // transient action message (search result, save confirmation, and —
        // Feature 009 — plugin menu-action results). FR-009.
        let status_notice = app
            .watcher_notice
            .as_deref()
            .or(app.status_message.as_deref());
        let status_bar = StatusBar::new(
            buf,
            app.theme,
            app.active_idx,
            app.buffers.len(),
            app.soft_wrap,
            status_notice,
        );
        frame.render_widget(status_bar, statusbar_area);

        // ── Menu bar ──────────────────────────────────────────────────────────
        // Rendered AFTER the editor/status bar so an open dropdown overlays the
        // editor content (the dropdown extends into the editor rows), but BEFORE
        // the modal dialog overlays so dialogs stay on top.
        use crate::input::keymap::Action;
        let toggle_states: &[(Action, bool)] = &[(Action::ToggleSoftWrap, app.soft_wrap)];
        // Feature 009: render the composite menu list (built-in + active plugin menus).
        let menus = resolve_menus(&app.plugin_host.registry.menu_items());
        let menubar = MenuBarWidget::new(app.theme, &app.menu_bar, toggle_states, &menus);
        frame.render_widget(menubar, menubar_area);

        // ── Dialogs (overlaid) ────────────────────────────────────────────────
        // Feature 016 — confirm/dismiss dialogs with boxed, focusable buttons.
        // One shared geometry (App::button_dialog_render → buttons::button_rects)
        // drives both this render and mouse hit-testing. Covers session restore,
        // unsaved-changes save prompt, external change, revert, and plugin consent.
        if let Some((rect, title, body, labels, focus)) = app.button_dialog_render() {
            let base = ratatui::style::Style::default()
                .fg(app.theme.menubar_fg)
                .bg(app.theme.menubar_bg);
            frame.render_widget(ratatui::widgets::Clear, rect);
            let block = ratatui::widgets::Block::default()
                .title(title)
                .borders(ratatui::widgets::Borders::ALL)
                .style(base);
            let inner = block.inner(rect);
            frame.render_widget(block, rect);
            // Body lines at the top of the interior (above the button row).
            let body_area = ratatui::layout::Rect::new(
                inner.x,
                inner.y,
                inner.width,
                inner.height.saturating_sub(3),
            );
            frame.render_widget(
                ratatui::widgets::Paragraph::new(body.join("\n"))
                    .style(base)
                    .wrap(ratatui::widgets::Wrap { trim: true }),
                body_area,
            );
            // Boxed buttons in the bottom interior rows.
            let rects = crate::ui::buttons::button_rects(rect, &labels);
            crate::ui::buttons::render_buttons(
                frame.buffer_mut(),
                &rects,
                &labels,
                focus,
                app.theme,
            );
            return;
        }

        // Feature 015 — interactive Find / Replace dialog overlay.
        if let Some(ref d) = app.pending_find_replace {
            use crate::ui::dialog::{DialogField, DialogMode};
            use crate::ui::file_browser::truncate_to_width;
            let base = ratatui::style::Style::default()
                .fg(app.theme.menubar_fg)
                .bg(app.theme.menubar_bg);
            let is_replace = d.mode == DialogMode::Replace;

            let count = match (d.query.is_empty(), app.search_state.active_match) {
                (true, _) => String::new(),
                (false, Some(i)) => format!("{}/{}", i + 1, app.search_state.matches.len()),
                (false, None) => {
                    if app.search_state.matches.is_empty() {
                        "not found".to_string()
                    } else {
                        format!("{} matches", app.search_state.matches.len())
                    }
                }
            };
            let opt = |on: bool, label: &str| format!("[{}] {}", if on { 'x' } else { ' ' }, label);
            let opts = format!(
                "{}  {}  {}  {}",
                opt(d.case_sensitive, "Case(Alt+C)"),
                opt(d.wrap, "Wrap(Alt+A)"),
                opt(d.regex, "Regex(Alt+R)"),
                opt(d.whole_word, "Word(Alt+W)"),
            );
            let hint = if is_replace {
                "Enter replace · Ctrl+A all · Tab field · F3/F2 next/prev · Esc close"
            } else {
                "Enter find · F3/F2 next/prev · Esc close"
            };
            let title = if is_replace { " Replace " } else { " Find " };

            // Feature 019: each field is a labeled, bordered 3-row input box,
            // matching the file-browser input box from feature 018. Layout:
            // (label row + 3-row box) per field, then an options row and a hint row.
            let n_fields: u16 = if is_replace { 2 } else { 1 };
            let content_h = n_fields * 4 + 2; // fields + options + hint
            let dw = 70u16.min(size.width.max(1));
            let dh = (content_h + 2).min(size.height.max(1)); // +2 outer borders
            let dx = size.x + size.width.saturating_sub(dw) / 2;
            // Place near the top so it doesn't hide the current match.
            let dy = size.y + 1;
            let dialog_area = ratatui::layout::Rect::new(dx, dy, dw, dh);

            frame.render_widget(ratatui::widgets::Clear, dialog_area);
            frame.render_widget(
                ratatui::widgets::Block::default()
                    .title(title)
                    .borders(ratatui::widgets::Borders::ALL)
                    .style(base),
                dialog_area,
            );

            let inner_x = dx + 1;
            let inner_w = dw.saturating_sub(2);
            let bottom = dy + dh.saturating_sub(1); // first bottom-border row
            let mut row = dy + 1;

            render_find_field(
                frame,
                base,
                inner_x,
                inner_w,
                &mut row,
                bottom,
                "Find what:",
                &count,
                &d.query,
                d.focus == DialogField::Query,
                d.caret,
            );
            if is_replace {
                render_find_field(
                    frame,
                    base,
                    inner_x,
                    inner_w,
                    &mut row,
                    bottom,
                    "Replace with:",
                    "",
                    &d.replacement,
                    d.focus == DialogField::Replacement,
                    d.caret,
                );
            }
            // Options row.
            if row < bottom {
                frame.render_widget(
                    ratatui::widgets::Paragraph::new(truncate_to_width(&opts, inner_w)).style(base),
                    ratatui::layout::Rect::new(inner_x, row, inner_w, 1),
                );
                row += 1;
            }
            // Hint row.
            if row < bottom {
                frame.render_widget(
                    ratatui::widgets::Paragraph::new(truncate_to_width(hint, inner_w)).style(base),
                    ratatui::layout::Rect::new(inner_x, row, inner_w, 1),
                );
            }
        }

        // T015 — Encoding select dialog overlay.
        if let Some(cursor_idx) = app.pending_encoding_select {
            use crate::ui::dialog::EncodingSelectDialog;
            let dialog = EncodingSelectDialog {
                cursor_idx,
                theme: app.theme,
            };
            frame.render_widget(dialog, size);
        }

        // Feature 012 — File browser overlay (Open / Save As).
        if let Some(ref browser) = app.file_browser {
            use crate::ui::file_browser::FileBrowserWidget;
            let widget = FileBrowserWidget {
                browser,
                theme: app.theme,
            };
            frame.render_widget(widget, size);
        }

        // Feature 011 — Help / About overlay.
        if let Some(screen) = app.pending_help {
            render_help_overlay(frame, app, screen, size);
        }

        // Feature 008 — Plugin manager dialog.
        if app.pending_plugin_manager {
            let body = crate::ui::plugin_manager::manager_body(
                &app.plugin_host,
                app.plugin_manager_cursor,
            );
            let dh = (crate::ui::plugin_manager::line_count(&body) + 2).min(size.height);
            let dw = 70u16.min(size.width);
            let dialog = ratatui::widgets::Paragraph::new(body)
                .style(
                    ratatui::style::Style::default()
                        .fg(app.theme.menubar_fg)
                        .bg(app.theme.menubar_bg),
                )
                .block(
                    ratatui::widgets::Block::default()
                        .title("Plugins")
                        .borders(ratatui::widgets::Borders::ALL),
                );
            let dx = size.x + size.width.saturating_sub(dw) / 2;
            let dy = size.y + size.height.saturating_sub(dh) / 2;
            let dialog_area = ratatui::layout::Rect::new(dx, dy, dw, dh);
            frame.render_widget(ratatui::widgets::Clear, dialog_area);
            frame.render_widget(dialog, dialog_area);
        }
    }
}

/// Feature 019: build the in-box display string for a Find/Replace field.
/// Embeds the caret glyph `▏` at grapheme index `caret` (only when `focused`),
/// then right-anchors the result to `inner_w` display columns so the caret and
/// trailing text stay visible when the value is wider than the box.
fn field_box_text(text: &str, focused: bool, caret: usize, inner_w: u16) -> String {
    use crate::ui::file_browser::grapheme_width;
    use unicode_segmentation::UnicodeSegmentation;

    let mut s = String::new();
    let mut len = 0usize;
    for (i, g) in text.graphemes(true).enumerate() {
        if focused && i == caret {
            s.push('▏');
        }
        s.push_str(g);
        len = i + 1;
    }
    if focused && caret >= len {
        s.push('▏');
    }

    let total: u16 = s.graphemes(true).map(grapheme_width).sum();
    if total <= inner_w {
        return s;
    }
    // Keep the tail (caret + latest chars) visible.
    let mut acc = 0u16;
    let mut tail = String::new();
    for g in s.graphemes(true).rev() {
        let w = grapheme_width(g);
        if acc + w > inner_w {
            break;
        }
        acc += w;
        tail.insert_str(0, g);
    }
    tail
}

/// Feature 019: render one labeled, bordered Find/Replace input box starting at
/// `*row`, advancing `*row` past the box. Clamps to `bottom` so a short terminal
/// degrades gracefully instead of drawing outside the dialog.
#[allow(clippy::too_many_arguments)]
fn render_find_field(
    frame: &mut Frame,
    base: ratatui::style::Style,
    inner_x: u16,
    inner_w: u16,
    row: &mut u16,
    bottom: u16,
    label: &str,
    extra: &str,
    text: &str,
    focused: bool,
    caret: usize,
) {
    use crate::ui::file_browser::truncate_to_width;
    use ratatui::layout::Rect;
    use ratatui::style::Modifier;
    use ratatui::widgets::{Block, Borders, Paragraph};

    if *row >= bottom {
        return;
    }
    // Label row (label left; optional `extra`, e.g. the match count, after it).
    let label_line = if extra.is_empty() {
        label.to_string()
    } else {
        format!("{}  {}", label, extra)
    };
    frame.render_widget(
        Paragraph::new(truncate_to_width(&label_line, inner_w))
            .style(base.add_modifier(Modifier::BOLD)),
        Rect::new(inner_x, *row, inner_w, 1),
    );
    *row += 1;

    // 3-row bordered box, clamped to the remaining height.
    let box_h = 3u16.min(bottom.saturating_sub(*row));
    if box_h == 0 {
        return;
    }
    let box_rect = Rect::new(inner_x, *row, inner_w, box_h);
    frame.render_widget(Block::default().borders(Borders::ALL).style(base), box_rect);
    if box_h >= 2 {
        let box_inner_w = inner_w.saturating_sub(2);
        let shown = field_box_text(text, focused, caret, box_inner_w);
        frame.render_widget(
            Paragraph::new(shown).style(base),
            Rect::new(inner_x + 1, *row + 1, box_inner_w, 1),
        );
    }
    *row += box_h;
}

/// Render the Help / About overlay (Feature 011), centred over the editor.
/// Feature 018: the Help cheat sheet as grouped (key, action) rows.
const HELP_SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "File",
        &[
            ("Ctrl+N", "New buffer"),
            ("Ctrl+O", "Open (file browser)"),
            ("Ctrl+S", "Save"),
            ("F12", "Save As Encoding"),
            ("(menu) Revert", "Reload last saved"),
            ("Ctrl+Q", "Quit"),
        ],
    ),
    (
        "Edit",
        &[
            ("Ctrl+Z", "Undo"),
            ("Ctrl+Y", "Redo"),
            ("Ctrl+X", "Cut selection"),
            ("Ctrl+C", "Copy selection"),
            ("Ctrl+V", "Paste"),
        ],
    ),
    (
        "Selection",
        &[
            ("Ctrl+A", "Select all"),
            ("Shift+Arrows", "Extend selection"),
            ("Shift+Home/End", "Select to line start/end"),
            ("Mouse drag", "Select a range"),
        ],
    ),
    (
        "Search",
        &[
            ("Ctrl+F", "Find"),
            ("F3 / F2", "Find next / previous"),
            ("Ctrl+H", "Find & Replace"),
        ],
    ),
    (
        "View",
        &[
            ("Alt+Z", "Toggle soft-wrap"),
            ("Arrows / PgUp / PgDn", "Move / page"),
            ("Home / End", "Line start / end"),
        ],
    ),
    (
        "Menus",
        &[
            ("F10 / Alt", "Activate menu bar"),
            ("Alt+<letter>", "Open a menu (underlined key)"),
            ("Arrows / Enter", "Navigate / select"),
            ("Esc", "Close menu"),
        ],
    ),
    (
        "Dialogs",
        &[
            ("Tab / Shift+Tab", "Move between buttons"),
            ("Enter / Space", "Activate focused button"),
            ("Mouse click", "Click a button / outside to cancel"),
            ("Esc", "Cancel / close"),
        ],
    ),
];

fn render_help_overlay(frame: &mut Frame, app: &App, screen: HelpScreen, size: Rect) {
    use ratatui::style::{Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};

    let base = Style::default()
        .fg(app.theme.menubar_fg)
        .bg(app.theme.menubar_bg);

    // About stays simple prose; Help is the grouped Key | Action table.
    let (title, lines): (&str, Vec<Line>) = match screen {
        HelpScreen::About => (
            "About",
            [
                format!("edit {}", env!("CARGO_PKG_VERSION")),
                env!("CARGO_PKG_DESCRIPTION").to_string(),
                String::new(),
                "A UTF-8 native, DOS-faithful EDIT.COM for the modern terminal.".into(),
                "Runs on Linux, FreeBSD, macOS, and MyOS.".into(),
                String::new(),
                format!("Author: {}", env!("CARGO_PKG_AUTHORS")),
                format!("© 2026 {} — MPL-2.0.", env!("CARGO_PKG_AUTHORS")),
                String::new(),
                "Press Esc to close.".into(),
            ]
            .into_iter()
            .map(Line::from)
            .collect(),
        ),
        HelpScreen::Help => {
            // Key column width = widest key, clamped.
            let key_w = HELP_SECTIONS
                .iter()
                .flat_map(|(_, rows)| rows.iter())
                .map(|(k, _)| k.len())
                .max()
                .unwrap_or(8)
                .min(22);
            let mut out: Vec<Line> = Vec::new();
            for (i, (section, rows)) in HELP_SECTIONS.iter().enumerate() {
                if i > 0 {
                    out.push(Line::from(""));
                }
                out.push(Line::from(Span::styled(
                    section.to_string(),
                    base.add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )));
                for (k, a) in rows.iter() {
                    out.push(Line::from(format!("  {:<kw$}  {}", k, a, kw = key_w)));
                }
            }
            ("Help", out)
        }
    };

    // Box geometry: fit the terminal, leave room for a border + a footer hint.
    let dw = 64u16.min(size.width.max(1));
    let dh = 20u16.min(size.height.max(1));
    let dx = size.x + size.width.saturating_sub(dw) / 2;
    let dy = size.y + size.height.saturating_sub(dh) / 2;
    let dialog_area = Rect::new(dx, dy, dw, dh);

    let inner_h = dh.saturating_sub(2) as usize; // borders
    let body_rows = inner_h.saturating_sub(1); // reserve 1 row for the footer hint
    let total = lines.len();
    let max_scroll = total.saturating_sub(body_rows);
    let scroll = app.help_scroll.min(max_scroll);

    let mut shown: Vec<Line> = lines
        .into_iter()
        .skip(scroll)
        .take(body_rows)
        .collect::<Vec<_>>();
    let more_below = scroll < max_scroll;
    let footer = if total > body_rows {
        format!(
            "↑↓/PgUp/PgDn scroll{}  ·  Esc close",
            if more_below { "  ▼ more" } else { "" }
        )
    } else {
        "Esc close".to_string()
    };
    shown.push(Line::from(Span::styled(
        footer,
        base.add_modifier(Modifier::DIM),
    )));

    let dialog = Paragraph::new(shown)
        .style(base)
        .block(Block::default().title(title).borders(Borders::ALL));

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(dialog, dialog_area);
}
