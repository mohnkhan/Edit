//! Task T032: User interface subsystem root.
//!
//! Exports all UI sub-modules and provides the top-level [`Ui::render`]
//! function that composes the full terminal frame from application state.

#![allow(dead_code, unused_variables, unused_imports)]

pub mod dialog;
pub mod editor;
pub mod menubar;
pub mod statusbar;
pub mod theme;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::App;
use crate::ui::{
    editor::EditorWidget,
    menubar::MenuBarWidget,
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
                Constraint::Length(1),  // menu bar
                Constraint::Min(1),     // editor
                Constraint::Length(1),  // status bar
            ])
            .split(size);

        let menubar_area = chunks[0];
        let editor_area = chunks[1];
        let statusbar_area = chunks[2];

        // ── Menu bar ─────────────────────────────────────────────────────────
        let menubar = MenuBarWidget::new(app.theme, &app.menu_bar);
        frame.render_widget(menubar, menubar_area);

        // ── Editor area ───────────────────────────────────────────────────────
        let buf = app.active_buffer();
        let show_line_numbers = app.config.line_numbers;

        match app.split_mode {
            SplitMode::Single => {
                let editor_widget = EditorWidget::new(buf, app.theme, show_line_numbers);
                frame.render_widget(editor_widget, editor_area);
            }
            SplitMode::Vertical => {
                let half_width = editor_area.width / 2;
                let left_area = Rect::new(editor_area.x, editor_area.y, half_width, editor_area.height);
                let right_area = Rect::new(
                    editor_area.x + half_width,
                    editor_area.y,
                    editor_area.width - half_width,
                    editor_area.height,
                );
                frame.render_widget(
                    EditorWidget::new(&app.buffers[0], app.theme, show_line_numbers),
                    left_area,
                );
                let right_buf_idx = if app.buffers.len() > 1 {
                    app.active_idx.max(1)
                } else {
                    0
                };
                frame.render_widget(
                    EditorWidget::new(&app.buffers[right_buf_idx], app.theme, show_line_numbers),
                    right_area,
                );
            }
        }

        // ── Status bar ────────────────────────────────────────────────────────
        let status_bar = StatusBar::new(
            buf,
            app.theme,
            app.active_idx,
            app.buffers.len(),
        );
        frame.render_widget(status_bar, statusbar_area);

        // ── Dialogs (overlaid) ────────────────────────────────────────────────
        // When app.pending_save_prompt is set, render the save-prompt dialog.
        if app.pending_save_prompt {
            use crate::ui::dialog::SavePromptDialog;
            let filename = buf
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "[No Name]".to_string());
            // We need a 'static str for SavePromptDialog but we have a String.
            // Render a plain paragraph instead to avoid lifetime issues until
            // the dialog API is refined in T041.
            let dialog = ratatui::widgets::Paragraph::new(format!(
                "Save changes to {}?  [S]ave / [D]iscard / [C]ancel",
                filename
            ))
            .style(
                ratatui::style::Style::default()
                    .fg(app.theme.menubar_fg)
                    .bg(app.theme.menubar_bg),
            )
            .block(
                ratatui::widgets::Block::default()
                    .title("Unsaved Changes")
                    .borders(ratatui::widgets::Borders::ALL),
            );

            // Centered overlay rect (fixed 60×5)
            let dw = 60u16.min(size.width);
            let dh = 5u16.min(size.height);
            let dx = size.x + size.width.saturating_sub(dw) / 2;
            let dy = size.y + size.height.saturating_sub(dh) / 2;
            let dialog_area = ratatui::layout::Rect::new(dx, dy, dw, dh);

            frame.render_widget(ratatui::widgets::Clear, dialog_area);
            frame.render_widget(dialog, dialog_area);
        }
    }
}
