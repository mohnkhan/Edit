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
            let base = ratatui::style::Style::default()
                .fg(app.theme.menubar_fg)
                .bg(app.theme.menubar_bg);
            let is_replace = d.mode == DialogMode::Replace;

            // Caret marker inside the focused field's text.
            let with_caret = |text: &str, focused: bool| -> String {
                if !focused {
                    return text.to_string();
                }
                let mut out = String::new();
                for (i, g) in
                    unicode_segmentation::UnicodeSegmentation::graphemes(text, true).enumerate()
                {
                    if i == d.caret {
                        out.push('│');
                    }
                    out.push_str(g);
                }
                let len = unicode_segmentation::UnicodeSegmentation::graphemes(text, true).count();
                if d.caret >= len {
                    out.push('│');
                }
                out
            };

            let count = match (d.query.is_empty(), app.search_state.active_match) {
                (true, _) => String::new(),
                (false, Some(i)) => format!("  {}/{}", i + 1, app.search_state.matches.len()),
                (false, None) => {
                    if app.search_state.matches.is_empty() {
                        "  not found".to_string()
                    } else {
                        format!("  {} matches", app.search_state.matches.len())
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

            let mut lines: Vec<ratatui::text::Line> = Vec::new();
            lines.push(ratatui::text::Line::from(format!(
                "Find:    {}{}",
                with_caret(&d.query, d.focus == DialogField::Query),
                count
            )));
            if is_replace {
                lines.push(ratatui::text::Line::from(format!(
                    "Replace: {}",
                    with_caret(&d.replacement, d.focus == DialogField::Replacement)
                )));
            }
            lines.push(ratatui::text::Line::from(opts));
            let hint = if is_replace {
                "Enter replace · Ctrl+A all · Tab field · F3/F2 next/prev · Esc close"
            } else {
                "Enter find · F3/F2 next/prev · Esc close"
            };
            lines.push(ratatui::text::Line::from(hint));

            let title = if is_replace { " Replace " } else { " Find " };
            let dialog = ratatui::widgets::Paragraph::new(lines).style(base).block(
                ratatui::widgets::Block::default()
                    .title(title)
                    .borders(ratatui::widgets::Borders::ALL),
            );
            let dw = 70u16.min(size.width);
            let dh = if is_replace { 6u16 } else { 5u16 }.min(size.height);
            let dx = size.x + size.width.saturating_sub(dw) / 2;
            // Place near the top so it doesn't hide the current match.
            let dy = size.y + 1;
            let dialog_area = ratatui::layout::Rect::new(dx, dy, dw, dh);
            frame.render_widget(ratatui::widgets::Clear, dialog_area);
            frame.render_widget(dialog, dialog_area);
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

/// Render the Help / About overlay (Feature 011), centred over the editor.
fn render_help_overlay(frame: &mut Frame, app: &App, screen: HelpScreen, size: Rect) {
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};

    let (title, lines): (&str, Vec<String>) = match screen {
        HelpScreen::Help => (
            "Help",
            vec![
                "Menus:  F10 / Alt+<letter> open • arrows move • Enter select • Esc close".into(),
                "        Mouse: click a menu title or a dropdown item".into(),
                "".into(),
                "File:   Ctrl+N new • Ctrl+O open • Ctrl+S save • F12 save-as-encoding".into(),
                "Edit:   Ctrl+Z undo • Ctrl+Y redo • Ctrl+X/C/V cut/copy/paste • Ctrl+A all".into(),
                "Search: Ctrl+F find • F3 next • F2 prev • Ctrl+H replace".into(),
                "View:   Alt+Z soft-wrap".into(),
                "Quit:   Ctrl+Q".into(),
                "".into(),
                "Press Esc to close.".into(),
            ],
        ),
        HelpScreen::About => (
            "About",
            vec![
                format!("edit {}", env!("CARGO_PKG_VERSION")),
                env!("CARGO_PKG_DESCRIPTION").to_string(),
                "".into(),
                "A UTF-8 native, DOS-faithful EDIT.COM for the modern terminal.".into(),
                "Runs on Linux, FreeBSD, macOS, and MyOS.".into(),
                "".into(),
                format!("Author: {}", env!("CARGO_PKG_AUTHORS")),
                format!(
                    "© 2026 {} — Licensed under MPL-2.0.",
                    env!("CARGO_PKG_AUTHORS")
                ),
                "Created as the built-in editor for the MyOS project.".into(),
                "".into(),
                "Press Esc to close.".into(),
            ],
        ),
    };

    let text: Vec<ratatui::text::Line> = lines.into_iter().map(ratatui::text::Line::from).collect();
    let dh = (text.len() as u16 + 2).min(size.height);
    let dw = 72u16.min(size.width);
    let dx = size.x + size.width.saturating_sub(dw) / 2;
    let dy = size.y + size.height.saturating_sub(dh) / 2;
    let dialog_area = Rect::new(dx, dy, dw, dh);

    let dialog = Paragraph::new(text)
        .style(
            ratatui::style::Style::default()
                .fg(app.theme.menubar_fg)
                .bg(app.theme.menubar_bg),
        )
        .block(Block::default().title(title).borders(Borders::ALL));

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(dialog, dialog_area);
}
