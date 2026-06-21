//! Task T032: User interface subsystem root.
//!
//! Exports all UI sub-modules and provides the top-level [`Ui::render`]
//! function that composes the full terminal frame from application state.

#![allow(dead_code, unused_variables, unused_imports)]

pub mod buttons;
pub mod contextmenu;
pub mod dialog;
pub mod editor;
pub mod file_browser;
pub mod menubar;
pub mod plugin_manager;
pub mod scrollbar;
pub mod statusbar;
pub mod tabbar;
pub mod theme;
pub mod width;
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
        // Feature 027: with 2+ buffers a one-row tab bar sits between the menu
        // bar and the editor, shrinking the editor by exactly one row. The row
        // count here must agree with `App::editor_top()` (the geometry source).
        let tab_bar_visible = app.tab_bar_visible();
        let constraints: &[Constraint] = if tab_bar_visible {
            &[
                Constraint::Length(1), // menu bar
                Constraint::Length(1), // tab bar
                Constraint::Min(1),    // editor
                Constraint::Length(1), // status bar
            ]
        } else {
            &[
                Constraint::Length(1), // menu bar
                Constraint::Min(1),    // editor
                Constraint::Length(1), // status bar
            ]
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(size);

        let menubar_area = chunks[0];
        let tab_bar_area = if tab_bar_visible {
            Some(chunks[1])
        } else {
            None
        };
        let editor_area = if tab_bar_visible {
            chunks[2]
        } else {
            chunks[1]
        };
        let statusbar_area = if tab_bar_visible {
            chunks[3]
        } else {
            chunks[2]
        };

        // ── Editor area ───────────────────────────────────────────────────────
        let buf = app.active_buffer();
        let show_line_numbers = app.config.line_numbers;

        match app.split_mode {
            SplitMode::Single => {
                // Feature 021: reserve the right column (vertical bar) and bottom
                // row (horizontal bar, non-wrap) and draw scrollbars there.
                let (text, vbar, hbar) = editor_panes(editor_area, app.soft_wrap);
                let wrap_starts = app.wrap_cache.as_ref().map(|c| c.visual_starts.as_slice());
                let editor_widget = EditorWidget::new(
                    buf,
                    app.theme,
                    show_line_numbers,
                    app.soft_wrap,
                    wrap_starts,
                )
                .with_matches(&app.search_state.matches, app.search_state.active_match);
                frame.render_widget(editor_widget, text);
                render_editor_scrollbars(frame, app, buf, text, vbar, hbar);
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
                let (l_text, l_vbar, l_hbar) = editor_panes(left_area, app.soft_wrap);
                frame.render_widget(
                    EditorWidget::new(
                        &app.buffers[0],
                        app.theme,
                        show_line_numbers,
                        app.soft_wrap,
                        app.wrap_cache.as_ref().map(|c| c.visual_starts.as_slice()),
                    ),
                    l_text,
                );
                render_editor_scrollbars(frame, app, &app.buffers[0], l_text, l_vbar, l_hbar);
                let right_buf_idx = if app.buffers.len() > 1 {
                    app.active_idx.max(1)
                } else {
                    0
                };
                let (r_text, r_vbar, r_hbar) = editor_panes(right_area, app.soft_wrap);
                frame.render_widget(
                    EditorWidget::new(
                        &app.buffers[right_buf_idx],
                        app.theme,
                        show_line_numbers,
                        app.soft_wrap,
                        app.wrap_cache.as_ref().map(|c| c.visual_starts.as_slice()),
                    ),
                    r_text,
                );
                render_editor_scrollbars(
                    frame,
                    app,
                    &app.buffers[right_buf_idx],
                    r_text,
                    r_vbar,
                    r_hbar,
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

        // ── Tab bar (Feature 027) ─────────────────────────────────────────────
        // Shown only with 2+ buffers; uses the same geometry as the mouse
        // hit-testing in `App::handle_mouse_event`. Rendered BEFORE the menu bar so
        // an open menu's dropdown (which drops from row 0 into the tab-bar row and
        // below) overlays the tab bar instead of being painted over by it
        // (Feature 033: z-order fix — the first dropdown item was hidden).
        if let Some(area) = tab_bar_area {
            tabbar::render_tab_bar(
                frame.buffer_mut(),
                area,
                &app.buffers,
                app.active_idx,
                app.theme,
            );
        }

        // ── Menu bar ──────────────────────────────────────────────────────────
        // Rendered AFTER the editor/status bar (and the tab bar) so an open
        // dropdown overlays the content below it, but BEFORE the modal dialog
        // overlays so dialogs stay on top.
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
            // Feature 020: the outer rect grows by a button row (computed in the
            // shared `find_replace_rect`); content stops above the button area.
            let dialog_area = find_replace_rect(d, size);
            let dx = dialog_area.x;
            let dy = dialog_area.y;
            let dw = dialog_area.width;
            let dh = dialog_area.height;

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
            // Reserve the bottom 4 interior rows for the boxed button row.
            let bottom = (dy + dh).saturating_sub(1).saturating_sub(4); // first reserved row
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

            // Feature 020: boxed buttons (mode-dependent) in the bottom rows.
            let labels = app.interactive_button_labels();
            let rects = crate::ui::buttons::button_rects(dialog_area, &labels);
            crate::ui::buttons::render_buttons(
                frame.buffer_mut(),
                &rects,
                &labels,
                app.interactive_focus_is_button().unwrap_or(usize::MAX),
                app.theme,
            );
        }

        // T015 — Encoding select dialog overlay.
        if let Some(cursor_idx) = app.pending_encoding_select {
            use crate::ui::dialog::EncodingSelectDialog;
            let dialog = EncodingSelectDialog {
                cursor_idx,
                theme: app.theme,
                button_focus: app.interactive_focus_is_button(),
            };
            frame.render_widget(dialog, size);
        }

        // Feature 025 — Go-to-Line prompt overlay.
        if let Some(ref entry) = app.pending_goto_line {
            let base = ratatui::style::Style::default()
                .fg(app.theme.menubar_fg)
                .bg(app.theme.menubar_bg);
            // Feature 031: embed the caret glyph at the caret position (mid-string).
            let caret = app.pending_goto_line_caret.min(entry.len());
            let body = format!("Go to line: {}▏{}", &entry[..caret], &entry[caret..]);
            // A compact centered box; width fits the prompt + padding, clamped.
            let dw = (body.len() as u16 + 4).clamp(20, size.width.max(1));
            let dh = 3u16.min(size.height.max(1));
            let dx = size.x + size.width.saturating_sub(dw) / 2;
            let dy = size.y + size.height.saturating_sub(dh) / 2;
            let area = ratatui::layout::Rect::new(dx, dy, dw, dh);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(
                ratatui::widgets::Paragraph::new(body).style(base).block(
                    ratatui::widgets::Block::default()
                        .title("Go to Line")
                        .borders(ratatui::widgets::Borders::ALL)
                        .style(base),
                ),
                area,
            );
        }

        // Feature 012 — File browser overlay (Open / Save As).
        if let Some(ref browser) = app.file_browser {
            use crate::ui::file_browser::FileBrowserWidget;
            let widget = FileBrowserWidget {
                browser,
                theme: app.theme,
                button_focus: app.interactive_focus_is_button(),
            };
            frame.render_widget(widget, size);
        }

        // Feature 011 — Help / About overlay.
        if let Some(screen) = app.pending_help {
            render_help_overlay(frame, app, screen, size);
        }

        // Feature 008 — Plugin manager dialog (Feature 020: + boxed Close button).
        if app.pending_plugin_manager {
            let body = crate::ui::plugin_manager::manager_body(
                &app.plugin_host,
                app.plugin_manager_cursor,
            );
            let dialog_area = crate::ui::plugin_manager::manager_rect(
                &app.plugin_host,
                app.plugin_manager_cursor,
                size,
            );
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
            frame.render_widget(ratatui::widgets::Clear, dialog_area);
            frame.render_widget(dialog, dialog_area);
            // Feature 021: vertical scrollbar when the plugin list overflows the
            // body rows (body = interior minus the 4-row button area); position
            // tracks the cursor so the user sees where the highlight sits.
            let body_rows = (dialog_area.height as usize).saturating_sub(2 + 4);
            crate::ui::scrollbar::render_vertical(
                frame.buffer_mut(),
                Rect::new(
                    dialog_area.x + 1,
                    dialog_area.y + 1,
                    dialog_area.width.saturating_sub(2),
                    body_rows as u16,
                ),
                app.plugin_host.registry.instances.len(),
                body_rows,
                app.plugin_manager_cursor,
                app.theme,
            );
            // Boxed Close button in the bottom interior rows.
            let labels = app.interactive_button_labels();
            let rects = crate::ui::buttons::button_rects(dialog_area, &labels);
            crate::ui::buttons::render_buttons(
                frame.buffer_mut(),
                &rects,
                &labels,
                app.interactive_focus_is_button().unwrap_or(usize::MAX),
                app.theme,
            );
        }

        // Feature 030 (US3) — editor right-click context menu, drawn last so it
        // overlays everything (it only opens when no other modal is active).
        if let Some(menu) = app.context_menu() {
            crate::ui::contextmenu::render(frame.buffer_mut(), size, menu, app.theme);
        }
    }
}

/// Feature 021: split an editor pane area into `(text, vertical_bar, horizontal_bar?)`.
/// The vertical scrollbar always takes the rightmost column; the horizontal
/// scrollbar takes the bottom row in non-wrap mode only. The bars are inset so
/// they never overlap at the bottom-right corner.
pub(crate) fn editor_panes(area: Rect, soft_wrap: bool) -> (Rect, Rect, Option<Rect>) {
    let vbar_w: u16 = if area.width >= 2 { 1 } else { 0 };
    let hbar_h: u16 = if !soft_wrap && area.height >= 2 { 1 } else { 0 };
    let text_w = area.width - vbar_w;
    let text_h = area.height - hbar_h;
    let text = Rect::new(area.x, area.y, text_w, text_h);
    let vbar = Rect::new(area.x + text_w, area.y, vbar_w, text_h);
    let hbar = if hbar_h > 0 {
        Some(Rect::new(area.x, area.y + area.height - 1, text_w, 1))
    } else {
        None
    };
    (text, vbar, hbar)
}

/// Feature 021: maximum display width among the editor's currently visible
/// logical lines — a cheap, viewport-bounded measure for the horizontal
/// scrollbar's content length (avoids scanning the whole file each frame).
pub(crate) fn max_visible_line_width(buf: &crate::buffer::Buffer, text: Rect) -> usize {
    use unicode_width::UnicodeWidthStr;
    let start = buf.scroll_offset.0;
    let total = buf.rope.line_count();
    let end = (start + text.height as usize).min(total);
    let mut max = 0usize;
    for i in start..end {
        let line = buf.rope.line_slice(i);
        let w = UnicodeWidthStr::width(line.trim_end_matches('\n'));
        if w > max {
            max = w;
        }
    }
    max
}

/// Feature 021: draw the editor's vertical (+ horizontal, non-wrap) scrollbars in
/// the reserved strips. Inputs mirror the geometry the editor widget was given
/// and the scroll math in `App` (see `viewport_height` / `content_width`).
fn render_editor_scrollbars(
    frame: &mut Frame,
    app: &App,
    buf: &crate::buffer::Buffer,
    text: Rect,
    vbar: Rect,
    hbar: Option<Rect>,
) {
    // Vertical: lines (or total visual rows in soft-wrap) vs visible rows.
    let viewport_v = text.height as usize;
    let content_v = if app.soft_wrap {
        app.wrap_cache
            .as_ref()
            .map(|c| c.total_visual_rows())
            .unwrap_or_else(|| buf.rope.line_count())
    } else {
        buf.rope.line_count()
    };
    crate::ui::scrollbar::render_vertical(
        frame.buffer_mut(),
        vbar,
        content_v,
        viewport_v,
        buf.scroll_offset.0,
        app.theme,
    );
    // Horizontal (non-wrap only): max visible-line width vs content width.
    if let Some(hbar) = hbar {
        let gutter: u16 = if app.config.line_numbers { 4 } else { 0 };
        let viewport_h = text.width.saturating_sub(gutter) as usize;
        let content_h = max_visible_line_width(buf, text);
        crate::ui::scrollbar::render_horizontal(
            frame.buffer_mut(),
            hbar,
            content_h,
            viewport_h,
            buf.scroll_offset.1,
            app.theme,
        );
    }
}

/// Feature 020: outer rect of the Find/Replace dialog, grown to fit a boxed
/// button row (Find / [Replace / Replace All] / Close). Shared by the renderer
/// and the app's mouse hit-testing so clicks land on the drawn buttons.
pub fn find_replace_rect(
    d: &crate::ui::dialog::FindReplaceDialog,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    use crate::ui::dialog::DialogMode;
    let n_fields: u16 = if d.mode == DialogMode::Replace { 2 } else { 1 };
    let content_h = n_fields * 4 + 2; // fields + options + hint
    let dw = 70u16.min(area.width.max(1));
    // +2 outer borders, +4 for the button row (1-row gap + 3-row box).
    let dh = (content_h + 2 + 4).min(area.height.max(1));
    let dx = area.x + area.width.saturating_sub(dw) / 2;
    let dy = area.y + 1; // near the top so it doesn't hide the current match
    ratatui::layout::Rect::new(dx, dy, dw, dh)
}

/// Feature 031: the inner text rect of each Find/Replace input field, matching
/// `render_find_field`'s geometry (label row + a 3-row bordered box; the text sits
/// at the box's middle row). Shared with the click hit-test so drawn == clickable.
/// Query text is at `dy+3`, replacement (replace mode) at `dy+7`; x = `dx+2`,
/// width = `dw-4`.
pub fn find_replace_field_rects(
    d: &crate::ui::dialog::FindReplaceDialog,
    area: ratatui::layout::Rect,
) -> Vec<(crate::ui::dialog::DialogField, ratatui::layout::Rect)> {
    use crate::ui::dialog::{DialogField, DialogMode};
    let da = find_replace_rect(d, area);
    let text_x = da.x + 2;
    let text_w = da.width.saturating_sub(4);
    let mut out = vec![(
        DialogField::Query,
        ratatui::layout::Rect::new(text_x, da.y + 3, text_w, 1),
    )];
    if d.mode == DialogMode::Replace {
        out.push((
            DialogField::Replacement,
            ratatui::layout::Rect::new(text_x, da.y + 7, text_w, 1),
        ));
    }
    out
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

/// Feature 024: number of content lines the Help/About overlay renders (for
/// scrollbar geometry). Mirrors the line-building in [`render_help_overlay`].
pub(crate) fn help_total_lines(screen: HelpScreen) -> usize {
    match screen {
        HelpScreen::About => 10,
        HelpScreen::Help => {
            let rows: usize = HELP_SECTIONS.iter().map(|(_, r)| 1 + r.len()).sum();
            rows + HELP_SECTIONS.len().saturating_sub(1) // blank separators
        }
    }
}

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

    // Feature 021: reserve a footer hint row + a 3-row boxed Close button (with a
    // 1-row gap) at the bottom interior, plus a right-edge vertical scrollbar.
    let inner_h = dh.saturating_sub(2) as usize; // borders
    let button_reserved = 4usize; // 1-row gap + 3-row boxed button
    let body_rows = inner_h.saturating_sub(1 + button_reserved); // 1 row = footer hint
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
            "↑↓/PgUp/PgDn scroll{}",
            if more_below { "  ▼ more" } else { "" }
        )
    } else {
        String::new()
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

    // Vertical scrollbar over the body's right interior column (only on overflow).
    crate::ui::scrollbar::render_vertical(
        frame.buffer_mut(),
        Rect::new(dx + 1, dy + 1, dw.saturating_sub(2), body_rows as u16),
        total,
        body_rows,
        scroll,
        app.theme,
    );

    // Boxed Close button in the bottom interior rows (key hint on the label).
    let labels = [crate::ui::buttons::HELP_CLOSE_LABEL];
    let rects = crate::ui::buttons::button_rects(dialog_area, &labels);
    crate::ui::buttons::render_buttons(frame.buffer_mut(), &rects, &labels, 0, app.theme);
}

/// Feature 021: the Help/About overlay's outer rect + Close button rects, shared
/// by the renderer and the mouse hit-test so a click lands on the drawn button.
pub fn help_close_button_rects(size: Rect) -> Vec<Rect> {
    let dw = 64u16.min(size.width.max(1));
    let dh = 20u16.min(size.height.max(1));
    let dx = size.x + size.width.saturating_sub(dw) / 2;
    let dy = size.y + size.height.saturating_sub(dh) / 2;
    let dialog_area = Rect::new(dx, dy, dw, dh);
    crate::ui::buttons::button_rects(dialog_area, &[crate::ui::buttons::HELP_CLOSE_LABEL])
}
