//! Split from app.rs (Feature 041): mouse.

use super::*;

impl App {
    // ── Feature 011 — Mouse menu interaction ──────────────────────────────────

    /// Handle a raw mouse event: open top-level menus, activate dropdown items,
    /// close menus on an outside click, or reposition the editor cursor.
    ///
    /// Uses [`hit_test_menu`] with the same geometry the menu bar renders with,
    /// so clicks land on exactly what is drawn — for both built-in and plugin
    /// menus, and for dropdown items (which previously could not be clicked).
    pub fn handle_mouse_event(&mut self, me: crossterm::event::MouseEvent) -> io::Result<()> {
        let Some(ev) = normalize_mouse(me) else {
            return Ok(());
        };

        // Feature 030 (US3): while the context menu is open it is modal — a press
        // on an item activates it, a press elsewhere dismisses. (Wheel/drag ignored.)
        if let Modal::ContextMenu(menu) = self.modal {
            if ev.kind == NormalizedMouseKind::Press {
                let (w, h) = self.terminal_size;
                let rect = crate::ui::contextmenu::menu_rect(
                    &menu,
                    ratatui::layout::Rect::new(0, 0, w, h),
                );
                if ev.button == MouseButton::Left {
                    if let Some(idx) = crate::ui::contextmenu::hit_test(rect, ev.col, ev.row) {
                        let act = crate::ui::contextmenu::ITEMS[idx].1.clone();
                        self.close_modal();
                        return self.handle_action(act);
                    }
                }
                // Press outside the menu (or non-left) dismisses it.
                self.close_modal();
            }
            return Ok(());
        }

        // Feature 030 (US3): a right-click in the editor opens the context menu —
        // but only when no other modal/menu is active (modal precedence, FR-010).
        if ev.kind == NormalizedMouseKind::Press && ev.button == MouseButton::Right {
            let (_, term_rows) = self.terminal_size;
            let in_editor = ev.row >= self.editor_top() && ev.row + 1 < term_rows;
            let any_modal = self.open_button_dialog().is_some()
                || self.interactive_dialog().is_some()
                || self.help_screen().is_some()
                || self.goto_line_digits().is_some()
                || self.menu_bar.is_active();
            if in_editor && !any_modal {
                self.modal =
                    Modal::ContextMenu(crate::ui::contextmenu::ContextMenu::new(ev.col, ev.row));
            }
            return Ok(());
        }

        // Feature 024: while a scrollbar thumb drag is active, mouse drags scroll
        // (proportional) instead of selecting text. Released below.
        if ev.kind == NormalizedMouseKind::Drag && self.scrollbar_drag.is_some() {
            let Some(d) = self.scrollbar_drag else {
                return Ok(());
            };
            let click = match d.axis {
                ScrollAxis::Vertical => ev.row.saturating_sub(d.track_start),
                ScrollAxis::Horizontal => ev.col.saturating_sub(d.track_start),
            } as usize;
            let off = crate::ui::scrollbar::pos_to_offset(
                d.track_len as usize,
                d.content,
                d.viewport,
                click,
            );
            self.apply_scroll_target(d.target, off, d.viewport);
            return Ok(());
        }

        // Feature 024: a button release ends any scrollbar drag.
        if ev.kind == NormalizedMouseKind::Release {
            self.scrollbar_drag = None;
            return Ok(());
        }

        // Feature 017: a left-drag in the editor extends the selection from the
        // anchor set on the preceding press (only when no modal/menu is active).
        if ev.kind == NormalizedMouseKind::Drag && ev.button == MouseButton::Left {
            if let Some(anchor) = self.drag_anchor {
                if !self.menu_bar.is_active() {
                    self.handle_mouse_click(ev.col, ev.row);
                    self.update_selection_to_cursor(anchor);
                }
            }
            return Ok(());
        }

        // Feature 023: mouse-wheel scrolling. Routed to the open modal/overlay
        // (modal wins), else the editor pane under the cursor. Placed BEFORE the
        // Press/Left guard so wheel events are not dropped, and returns so the
        // click/cursor paths below are never entered for a wheel event.
        if matches!(
            ev.kind,
            NormalizedMouseKind::ScrollUp | NormalizedMouseKind::ScrollDown
        ) {
            let down = ev.kind == NormalizedMouseKind::ScrollDown;
            let step = WHEEL_STEP;
            if let Some(s) = self.help_scroll_mut() {
                *s = if down {
                    s.saturating_add(step)
                } else {
                    s.saturating_sub(step)
                };
            } else if let Modal::EncodingSelect { row: idx } = self.modal {
                let n = crate::ui::dialog::ENCODING_OPTIONS.len();
                self.set_encoding_select(if down {
                    (idx + step).min(n - 1)
                } else {
                    idx.saturating_sub(step)
                });
            } else if self.file_browser().is_some() {
                let (w, h) = self.terminal_size;
                let vis = ratatui::layout::Rect::new(0, 0, w, h);
                if let Some(fb) = self.file_browser_mut() {
                    let rows = fb.visible_rows(vis);
                    for _ in 0..step {
                        if down {
                            fb.move_down(rows);
                        } else {
                            fb.move_up(rows);
                        }
                    }
                }
            } else if self.is_plugin_manager_open() {
                let n = self.plugin_host.registry.instances.len();
                if n > 0 {
                    if let Some(c) = self.plugin_manager_cursor_mut() {
                        *c = if down {
                            (*c + step).min(n - 1)
                        } else {
                            c.saturating_sub(step)
                        };
                    }
                }
            } else if self.find_replace().is_some() || self.goto_line_digits().is_some() {
                // Find/Replace and Go-to-Line have no scrollable content — ignore.
            } else {
                // Editor: ignore the menu/tab rows (above editor_top) and the
                // status-bar row (last).
                let (w, term_rows) = self.terminal_size;
                if ev.row >= self.editor_top() && ev.row + 1 < term_rows {
                    let buf_idx = if matches!(self.split_mode, crate::ui::SplitMode::Vertical) {
                        if ev.col >= w / 2 && self.buffers.len() > 1 {
                            self.active_idx.max(1)
                        } else {
                            0
                        }
                    } else {
                        self.active_idx
                    };
                    self.wheel_scroll_editor(buf_idx, down, step);
                }
            }
            return Ok(());
        }

        // Only left-button presses drive the menu / cursor for now.
        if ev.kind != NormalizedMouseKind::Press || ev.button != MouseButton::Left {
            return Ok(());
        }

        // Feature 024: a press on a scrollbar — page on the track, drag on the
        // thumb. Checked BEFORE the editor click / feature-017 drag-anchor and the
        // modal entry/button handlers, so a bar press never selects or places the
        // cursor. Bars occupy reserved cells that don't overlap those targets.
        for r in self.scrollbar_regions(ev.col, ev.row) {
            let inside = ev.col >= r.rect.x
                && ev.col < r.rect.x + r.rect.width
                && ev.row >= r.rect.y
                && ev.row < r.rect.y + r.rect.height;
            if !inside {
                continue;
            }
            let (track_start, click, track_len) = match r.axis {
                ScrollAxis::Vertical => (r.rect.y, ev.row.saturating_sub(r.rect.y), r.rect.height),
                ScrollAxis::Horizontal => (r.rect.x, ev.col.saturating_sub(r.rect.x), r.rect.width),
            };
            let zone = crate::ui::scrollbar::hit_zone(
                track_len as usize,
                r.content,
                r.viewport,
                r.offset,
                click as usize,
            );
            let max_off = r.content.saturating_sub(r.viewport);
            match zone {
                crate::ui::scrollbar::HitZone::Above => {
                    let off = r.offset.saturating_sub(r.viewport);
                    self.apply_scroll_target(r.target, off, r.viewport);
                }
                crate::ui::scrollbar::HitZone::Below => {
                    let off = (r.offset + r.viewport).min(max_off);
                    self.apply_scroll_target(r.target, off, r.viewport);
                }
                crate::ui::scrollbar::HitZone::Thumb => {
                    self.scrollbar_drag = Some(ScrollbarDrag {
                        target: r.target,
                        axis: r.axis,
                        track_start,
                        track_len,
                        content: r.content,
                        viewport: r.viewport,
                    });
                }
            }
            return Ok(());
        }

        // Feature 021 — Help/About overlay: a click on the boxed Close button
        // dismisses it (same effect as Esc). Other clicks are inert (modal).
        if self.help_screen().is_some() {
            let (w, h) = self.terminal_size;
            let rects = crate::ui::help_close_button_rects(ratatui::layout::Rect::new(0, 0, w, h));
            if crate::ui::buttons::hit_test_buttons(&rects, ev.col, ev.row).is_some() {
                self.close_modal();
            }
            return Ok(());
        }

        // Feature 031 (#58) — Go-to-Line: a click in the digit field positions the
        // caret. Geometry mirrors the render: a centered box of width
        // `(19 + digits.len()).clamp(20, w)`; the digits start after the border +
        // the "Go to line: " (12-col) prefix.
        if let Some(entry) = self.goto_line_digits().map(|s| s.to_owned()) {
            // Feature 039 (FR-006): hit-test the digit field via the shared rect.
            if let Some(fr) = self.goto_line_field_rect() {
                if ev.row == fr.y && ev.col >= fr.x && ev.col < fr.x + fr.width {
                    let new_caret =
                        crate::ui::width::field_caret_at(&entry, fr.width, ev.col - fr.x);
                    if let Modal::GotoLine { caret, .. } = &mut self.modal {
                        *caret = new_caret;
                    }
                }
            }
            return Ok(());
        }

        // Feature 020 — interactive/list dialog buttons: a click on a boxed
        // button activates it directly (buttons win over the list/entry hit-test
        // that follows). Uses the same geometry the renderer drew with.
        self.ensure_dialog_focus();
        if self.interactive_dialog().is_some() {
            if let Some(rect) = self.interactive_dialog_rect() {
                let labels = self.interactive_button_labels();
                let rects = crate::ui::buttons::button_rects(rect, &labels);
                if let Some(i) = crate::ui::buttons::hit_test_buttons(&rects, ev.col, ev.row) {
                    self.activate_interactive_button(i);
                    return Ok(());
                }
                // Feature 030 (#53): a click on a list row selects that row and
                // focuses the list (primary control). Buttons were checked first.
                match self.interactive_dialog() {
                    Some(InteractiveDialog::EncodingSelect) => {
                        if let Some(idx) = crate::ui::dialog::encoding_row_hit(rect, ev.col, ev.row)
                        {
                            self.set_encoding_select(idx);
                            self.dialog_focus = 0;
                            return Ok(());
                        }
                    }
                    Some(InteractiveDialog::PluginManager) => {
                        if let Some(idx) = crate::ui::plugin_manager::manager_row_hit(
                            &self.plugin_host,
                            rect,
                            ev.col,
                            ev.row,
                        ) {
                            if let Some(c) = self.plugin_manager_cursor_mut() {
                                *c = idx;
                            }
                            self.dialog_focus = 0;
                            return Ok(());
                        }
                    }
                    Some(InteractiveDialog::FindReplace) => {
                        // Feature 031 (#58): a click in a field's text box moves the
                        // caret to the clicked grapheme and focuses that field.
                        let (w, h) = self.terminal_size;
                        let full = ratatui::layout::Rect::new(0, 0, w, h);
                        if let Some(d) = self.find_replace() {
                            let fields = crate::ui::find_replace_field_rects(d, full);
                            for (field, fr) in fields {
                                if ev.row == fr.y && ev.col >= fr.x && ev.col < fr.x + fr.width {
                                    let value = match field {
                                        crate::ui::dialog::DialogField::Query => &d.query,
                                        crate::ui::dialog::DialogField::Replacement => {
                                            &d.replacement
                                        }
                                    };
                                    let caret = crate::ui::width::field_caret_at(
                                        value,
                                        fr.width,
                                        ev.col - fr.x,
                                    );
                                    if let Some(d) = self.find_replace_mut() {
                                        d.set_focus(field);
                                        d.caret = caret;
                                    }
                                    self.dialog_focus = match field {
                                        crate::ui::dialog::DialogField::Query => 0,
                                        crate::ui::dialog::DialogField::Replacement => 1,
                                    };
                                    return Ok(());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            // Not on a button or row: fall through to dialog-specific handling (file
            // browser entry clicks below; the other dialogs ignore the click).
        }

        // Feature 012 — file browser: a single click selects the row; a second
        // click on the same row (within DOUBLE_CLICK_MS) activates it (enter
        // folder / open file). This matches file-dialog convention and avoids a
        // double-click on a folder navigating in and then immediately opening
        // whatever file lands under the cursor in the new listing. A click
        // outside the box cancels.
        if self.file_browser().is_some() {
            let (w, h) = self.terminal_size;
            let area = ratatui::layout::Rect::new(0, 0, w, h);
            // Feature 031 (#58): a click inside the Name/path field box positions
            // the caret there (checked before the list/outside hit-test).
            if let Some(fr) = self.file_browser().map(|fb| fb.field_text_rect(area)) {
                if ev.row == fr.y && ev.col >= fr.x && ev.col < fr.x + fr.width {
                    if let Some(fb) = self.file_browser_mut() {
                        fb.caret_click(fr, ev.col);
                    }
                    return Ok(());
                }
            }
            let Some(hit) = self
                .file_browser()
                .map(|fb| fb.hit_test(area, ev.col, ev.row))
            else {
                return Ok(());
            };
            match hit {
                BrowserHit::Entry(idx) => {
                    let now = Instant::now();
                    let double = self.last_browser_click.is_some_and(|(prev, t)| {
                        prev == idx
                            && now.duration_since(t) <= Duration::from_millis(DOUBLE_CLICK_MS)
                    });
                    if double {
                        self.last_browser_click = None;
                        let outcome = self.file_browser_mut().map(|fb| fb.activate_index(idx));
                        if let Some(outcome) = outcome {
                            self.apply_browse_outcome(outcome);
                        }
                    } else {
                        // First click: just move the highlight to the row.
                        self.last_browser_click = Some((idx, now));
                        if let Some(fb) = self.file_browser_mut() {
                            fb.selected = idx;
                        }
                    }
                }
                BrowserHit::Outside => {
                    self.last_browser_click = None;
                    self.close_modal();
                }
                BrowserHit::Inside => self.last_browser_click = None,
            }
            return Ok(());
        }

        // Feature 016 — confirm/dismiss dialogs: a click on a boxed button
        // activates it; a click outside the dialog cancels (where safe). Uses the
        // same geometry the renderer drew with.
        self.ensure_dialog_focus();
        if self.open_button_dialog().is_some() {
            if let Some(rect) = self.button_dialog_rect() {
                let labels = self.dialog_button_labels();
                let rects = crate::ui::buttons::button_rects(rect, &labels);
                if let Some(i) = crate::ui::buttons::hit_test_buttons(&rects, ev.col, ev.row) {
                    self.activate_dialog_button(i);
                } else {
                    let inside = ev.col >= rect.x
                        && ev.col < rect.x + rect.width
                        && ev.row >= rect.y
                        && ev.row < rect.y + rect.height;
                    if !inside {
                        if let Some(c) = self.dialog_cancel_index() {
                            self.activate_dialog_button(c);
                        }
                    }
                }
            }
            return Ok(());
        }

        // Modal dialogs win: ignore menu/editor mouse while one is open.
        // (ContextMenu / FileBrowser are handled earlier and have already returned,
        // so they cannot be the active modal here.)
        if matches!(
            self.modal,
            Modal::SavePrompt
                | Modal::SessionRestore(_)
                | Modal::RevertConfirm(_)
                | Modal::CloseConfirm(_)
        ) || self.encoding_select_row().is_some()
            || self.help_screen().is_some()
            || self.pending_external_change.is_some()
            || !self.pending_plugin_consent.is_empty()
            || self.is_plugin_manager_open()
            || self.find_replace().is_some()
            || self.goto_line_digits().is_some()
        {
            return Ok(());
        }

        // Feature 027 — tab bar: a click on the tab row switches buffers (label)
        // or closes one (`[x]`), and never reaches the editor (FR-008). Uses the
        // same geometry as the renderer. A click on the row outside any tab is a
        // no-op. Reached only when no modal is open (guarded above).
        //
        // Feature 039: the tab bar handles clicks on its row only when it is the
        // topmost layer there per `LAYER_PRECEDENCE`. When a menu dropdown is open
        // it sits above the tab bar (feature 033 paints it on top), so it owns the
        // tab row — including the first dropdown item — and `hit_test_menu` below
        // routes the click (otherwise that item is unreachable with 2+ buffers).
        if self.top_row_owner() == Layer::TabBar && ev.row + 1 == self.editor_top() {
            let area = ratatui::layout::Rect::new(0, ev.row, self.terminal_size.0, 1);
            for r in crate::ui::tabbar::tab_hit_regions(area, &self.buffers, self.active_idx) {
                if ev.col == r.close_rect.x {
                    self.tab_close_clicked(r.idx);
                    return Ok(());
                }
                if ev.col >= r.label_rect.x && ev.col < r.label_rect.x + r.label_rect.width {
                    // Feature 043: route through activate_buffer so the wrap cache is
                    // invalidated — otherwise the clicked tab renders with the previous
                    // tab's wrap (ghost wrap + misaligned line numbers).
                    self.activate_buffer(r.idx);
                    return Ok(());
                }
            }
            return Ok(());
        }

        let menus = self.resolved_menus();
        let toggle_states = [(Action::ToggleSoftWrap, self.active_buffer().soft_wrap)];
        let term_width = self.terminal_size.0;

        match hit_test_menu(
            &menus,
            &self.menu_bar.state,
            &toggle_states,
            term_width,
            ev.col,
            ev.row,
        ) {
            MenuHit::TopLevel(idx) => {
                // Clicking the title of the already-open menu closes it (toggle).
                if let MenuState::DropDown { top_idx, .. } = self.menu_bar.state {
                    if top_idx == idx {
                        self.menu_bar.close_menu();
                        return Ok(());
                    }
                }
                self.menu_bar.open_menu(idx, &menus);
            }
            MenuHit::Item { top_idx, item_idx } => {
                self.menu_bar.state = MenuState::DropDown { top_idx, item_idx };
                if let Some(action) = self.menu_bar.select_item(&menus) {
                    return self.handle_action(action);
                }
            }
            MenuHit::Outside => {
                if self.menu_bar.is_active() {
                    self.menu_bar.close_menu();
                } else {
                    // Editor click: position the cursor first (maps the click to a
                    // line/grapheme col).
                    self.handle_mouse_click(ev.col, ev.row);
                    // Feature 030: classify single/double/triple click.
                    match self.next_editor_click_count(ev.col, ev.row) {
                        2 => self.select_word_at_cursor(),
                        3 => self.select_line_at_cursor(),
                        _ => {
                            // Single click: clear selection, set the drag anchor so
                            // a following drag selects (Feature 017).
                            self.active_buffer_mut().selection = None;
                            self.drag_anchor = Some(self.active_buffer().cursor);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // ── T111 — Mouse click cursor repositioning ───────────────────────────────

    /// Reposition the cursor when the user clicks inside the editor area.
    ///
    /// `col` and `row` are 0-based terminal coordinates.  Row 0 is the menu bar
    /// and the last row is the status bar, so editor rows are `1..terminal_rows-1`.
    pub fn handle_mouse_click(&mut self, col: u16, row: u16) {
        let (term_cols, term_rows) = self.terminal_size;

        // Feature 027: the editor starts at `editor_top()` (below the menu bar and,
        // when shown, the tab bar). Rows above it and the status row are not editor.
        let top = self.editor_top();
        if row < top || row >= term_rows.saturating_sub(1) {
            return;
        }
        // Feature 021: the editor reserves its rightmost column for the vertical
        // scrollbar and (non-wrap) its bottom row for the horizontal scrollbar;
        // clicks on those reserved cells must not move the cursor.
        if col >= term_cols.saturating_sub(1) {
            return;
        }
        if !self.active_buffer().soft_wrap && row == term_rows.saturating_sub(2) {
            return;
        }

        let clicked_row = (row - top) as usize; // 0-based editor row

        // Feature 029: the line-number gutter occupies `gutter` columns on the
        // left; the text area starts after it. Map the raw terminal column into the
        // text area (a click on the gutter clamps to column 0 via saturating_sub).
        let gutter: u16 = if self.config.line_numbers { 4 } else { 0 };
        let col = col.saturating_sub(gutter);

        // Soft-wrap mode: map (visual_row, visual_col) → (logical_line, grapheme_col).
        if self.active_buffer().soft_wrap {
            if let Some(ref cache) = self.wrap_cache {
                let scroll_vr = self.active_buffer().scroll_offset.0;
                let visual_row = scroll_vr + clicked_row;
                if let Some((logical_line, start_byte_u32)) = cache.visual_to_logical(visual_row) {
                    let start_byte = start_byte_u32 as usize;
                    let line_str = self.active_buffer().rope.line_slice(logical_line);
                    // Compute which segment end byte is.
                    let seg_end = {
                        let starts = &cache.visual_starts[logical_line];
                        let seg_idx_opt = starts.iter().position(|&b| b as usize == start_byte);
                        seg_idx_opt
                            .and_then(|si| starts.get(si + 1))
                            .map(|&b| b as usize)
                            .unwrap_or(line_str.len())
                    };
                    // Effective text column accounting for '»' marker on continuation rows.
                    let is_continuation = start_byte > 0;
                    let text_col_start: usize = if is_continuation { 1 } else { 0 };
                    let target_vcol = col as usize;

                    let mut vis_col = text_col_start;
                    let mut found_gcol: usize = {
                        // Default: walk logical line to find byte=start_byte → grapheme col.
                        line_str
                            .graphemes(true)
                            .scan(0usize, |b, g| {
                                let c = *b;
                                *b += g.len();
                                Some((c, g))
                            })
                            .take_while(|(b, _)| *b < start_byte)
                            .count()
                    };
                    let mut cur_byte = start_byte;

                    for grapheme in line_str[start_byte..seg_end].graphemes(true) {
                        let gw = unicode_segmentation_width(grapheme) as usize;
                        if vis_col + gw > target_vcol {
                            break;
                        }
                        vis_col += gw;
                        cur_byte += grapheme.len();
                        found_gcol += 1;
                    }

                    let _ = cur_byte; // used for iteration side effects

                    let new_vcol = CursorPos::visual_col_from_grapheme_col(
                        &self.active_buffer().rope,
                        logical_line,
                        found_gcol,
                    );
                    let buf = self.active_buffer_mut();
                    buf.cursor = CursorPos {
                        line: logical_line,
                        grapheme_col: found_gcol,
                        visual_col: new_vcol,
                    };
                    self.clamp_scroll();
                    return;
                }
            }
        }

        // Normal mode (non-wrap): existing logic.
        let buf = self.active_buffer();
        let scroll_line = buf.scroll_offset.0;
        let target_line = scroll_line + clicked_row;
        let line_count = buf.rope.line_count();
        if target_line >= line_count {
            return;
        }

        let line_str = buf.rope.line_slice(target_line);
        // Feature 029: account for the horizontal scroll offset — the first visible
        // text column is `scroll_offset.1`, so the absolute clicked column is
        // `scroll_offset.1 + col` (col already has the gutter removed).
        let target_x: u16 = (buf.scroll_offset.1 as u16).saturating_add(col);
        let mut visual_x: u16 = 0;
        let mut found_gcol: usize = 0;

        for (gcol, grapheme) in line_str.graphemes(true).enumerate() {
            let w = unicode_segmentation_width(grapheme);
            if visual_x + w > target_x {
                found_gcol = gcol;
                break;
            }
            visual_x += w;
            found_gcol = gcol + 1;
        }

        let new_vcol = CursorPos::visual_col_from_grapheme_col(
            &self.active_buffer().rope,
            target_line,
            found_gcol,
        );

        let buf = self.active_buffer_mut();
        buf.cursor = CursorPos {
            line: target_line,
            grapheme_col: found_gcol,
            visual_col: new_vcol,
        };
        self.clamp_scroll();
    }
}
