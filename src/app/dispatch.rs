//! Split from app.rs (Feature 041): dispatch.

use super::*;

impl App {
    // ── Action dispatch ──────────────────────────────────────────────────────

    /// Build the resolved composite menu list (built-in + active plugin menus).
    /// Recomputed on demand so mid-session plugin enable/disable is reflected.
    pub(super) fn resolved_menus(&self) -> Vec<ResolvedMenu> {
        resolve_menus(
            &self.plugin_host.registry.menu_items(),
            &self.recent_files.paths,
        )
    }

    /// Open the dropdown for top-level menu `idx`, clamped against the resolved
    /// menu count.
    pub(super) fn open_menu_idx(&mut self, idx: usize) {
        let menus = self.resolved_menus();
        self.menu_bar.open_menu(idx, &menus);
    }

    pub fn handle_action(&mut self, action: Action) -> io::Result<()> {
        self.ensure_dialog_focus();

        // Feature 030 (US3): the editor context menu is modal while open — Up/Down
        // move focus, Enter/Space activate the focused item (routing to the real
        // action), Esc dismisses; all other keys are consumed.
        if let Modal::ContextMenu(mut menu) = self.modal {
            match &action {
                Action::MoveDown => {
                    menu.focus_next();
                    self.modal = Modal::ContextMenu(menu);
                }
                Action::MoveUp => {
                    menu.focus_prev();
                    self.modal = Modal::ContextMenu(menu);
                }
                Action::InsertNewline | Action::InsertChar(' ') => {
                    // Feature 046: checked access — a stale focus can't index past ITEMS.
                    if let Some(item) = crate::ui::contextmenu::ITEMS.get(menu.focus) {
                        let act = item.1.clone();
                        self.close_modal();
                        return self.handle_action(act);
                    }
                }
                Action::MenuClose | Action::Quit => {
                    self.close_modal();
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 016: button focus + activation for confirm/dismiss dialogs.
        // Tab/Shift+Tab move focus; Enter/Space activate the focused button. All
        // other keys (letter shortcuts, Esc) fall through to the per-dialog guards
        // below, which still handle them.
        {
            let n = self.dialog_button_labels().len();
            if n > 0 {
                match &action {
                    // Feature 028: arrow keys move between buttons too (the button
                    // row is horizontal, so Right/Down = next, Left/Up = prev),
                    // consistent with Tab/Shift+Tab.
                    Action::FocusNextField | Action::MoveRight | Action::MoveDown => {
                        self.dialog_focus = crate::ui::buttons::next(self.dialog_focus, n);
                        return Ok(());
                    }
                    Action::FocusPrevField | Action::MoveLeft | Action::MoveUp => {
                        self.dialog_focus = crate::ui::buttons::prev(self.dialog_focus, n);
                        return Ok(());
                    }
                    Action::InsertNewline | Action::InsertChar(' ') => {
                        let idx = self.dialog_focus.min(n - 1);
                        self.activate_dialog_button(idx);
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        // Feature 020: focus-ring movement for the interactive/list dialogs.
        // Tab/Shift+Tab cycle the whole ring (primary control + buttons). All
        // other keys fall through to the per-dialog guards below, which consult
        // `dialog_focus` to decide between primary-control behavior and button
        // activation.
        {
            let ring = self.interactive_ring_len();
            if ring > 1 {
                // Feature 028: when a BUTTON is focused, arrow keys also cycle the
                // ring (consistent with Tab). When the primary control is focused,
                // arrows fall through to the list/field below (unchanged behavior).
                let on_button = self.interactive_focus_is_button().is_some();
                match &action {
                    Action::FocusNextField => {
                        self.dialog_focus = crate::ui::buttons::next(self.dialog_focus, ring);
                        self.sync_find_replace_focus();
                        return Ok(());
                    }
                    Action::FocusPrevField => {
                        self.dialog_focus = crate::ui::buttons::prev(self.dialog_focus, ring);
                        self.sync_find_replace_focus();
                        return Ok(());
                    }
                    Action::MoveRight | Action::MoveDown if on_button => {
                        self.dialog_focus = crate::ui::buttons::next(self.dialog_focus, ring);
                        self.sync_find_replace_focus();
                        return Ok(());
                    }
                    Action::MoveLeft | Action::MoveUp if on_button => {
                        self.dialog_focus = crate::ui::buttons::prev(self.dialog_focus, ring);
                        self.sync_find_replace_focus();
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        // When the session restore dialog is active, only Y/y/Enter (confirm)
        // and N/n/Escape/Quit (decline) are forwarded; everything else is
        // dropped silently so the dialog stays visible.
        if matches!(self.modal, Modal::SessionRestore(_)) {
            match &action {
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'Y') => {
                    self.do_restore_session();
                    self.close_modal();
                }
                Action::InsertNewline => {
                    self.do_restore_session();
                    self.close_modal();
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'N') => {
                    self.close_modal();
                }
                Action::Quit | Action::MenuClose => {
                    self.close_modal();
                }
                _ => {}
            }
            return Ok(());
        }

        // When the save-before-quit prompt is active, only S / D / C are valid.
        // All other actions are silently dropped so the prompt stays visible.
        if matches!(self.modal, Modal::SavePrompt) {
            match &action {
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'S') => {
                    self.prompt_save_and_quit();
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'D') => {
                    self.prompt_discard_and_quit();
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'C') => {
                    self.prompt_cancel_quit();
                }
                // Feature 029: Esc cancels, matching the "Cancel (Esc)" label and
                // every other confirm dialog (was previously ignored).
                Action::MenuClose | Action::Quit => {
                    self.prompt_cancel_quit();
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 007 — External-change dialog intercept: only Y/Enter (reload)
        // and N/Esc (keep) are forwarded while the dialog is active.
        if self.pending_external_change.is_some() {
            match &action {
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'Y') => {
                    if let Some(ec) = self.pending_external_change.take() {
                        self.reload_from_disk(ec.buf_idx);
                    }
                }
                Action::InsertNewline => {
                    if let Some(ec) = self.pending_external_change.take() {
                        self.reload_from_disk(ec.buf_idx);
                    }
                }
                Action::ReloadFile => {
                    if let Some(ec) = self.pending_external_change.take() {
                        self.reload_from_disk(ec.buf_idx);
                    }
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'N') => {
                    if let Some(ec) = self.pending_external_change.take() {
                        if let Some(b) = self.buffers.get_mut(ec.buf_idx) {
                            b.modified = true;
                        }
                    }
                }
                Action::MenuClose | Action::DismissExternalChange => {
                    if let Some(ec) = self.pending_external_change.take() {
                        if let Some(b) = self.buffers.get_mut(ec.buf_idx) {
                            b.modified = true;
                        }
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 014 — Revert confirmation intercept: Y/Enter discards changes
        // and reloads from disk; N/Esc cancels (buffer untouched).
        if let Modal::RevertConfirm(buf_idx) = self.modal {
            match &action {
                Action::InsertNewline => {
                    self.close_modal();
                    self.reload_from_disk(buf_idx);
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'Y') => {
                    self.close_modal();
                    self.reload_from_disk(buf_idx);
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'N') => {
                    self.close_modal();
                }
                Action::MenuClose | Action::Quit => {
                    self.close_modal();
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 027 — tab `[x]` close confirmation intercept: only S (save +
        // close), D (discard + close), C/Esc (cancel) are valid; all other input
        // is dropped so the modal stays visible. Mirrors the save-before-quit
        // prompt so the `(S)`/`(D)` label hints are accurate.
        if matches!(self.modal, Modal::CloseConfirm(_)) {
            match &action {
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'S') => {
                    self.activate_dialog_button(0);
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'D') => {
                    self.activate_dialog_button(1);
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'C') => {
                    self.activate_dialog_button(2);
                }
                Action::MenuClose | Action::Quit => {
                    self.activate_dialog_button(2);
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 015 — Find/Replace dialog intercept. While open, keystrokes
        // edit the dialog fields and drive the search; the buffer is only touched
        // by an explicit Replace/Replace-All. All input is consumed.
        if self.find_replace().is_some() {
            let is_replace = matches!(
                self.find_replace().map(|d| d.mode),
                Some(DialogMode::Replace)
            );
            // Dialog-global keys (work regardless of which stop is focused):
            // close, option toggles, and match navigation. Feature 020 keeps
            // these unchanged from feature 015.
            match &action {
                Action::MenuClose | Action::Quit => {
                    self.close_find_replace();
                    return Ok(());
                }
                Action::ToggleSearchCase => {
                    if let Some(d) = self.find_replace_mut() {
                        d.case_sensitive = !d.case_sensitive;
                    }
                    self.run_find_from_dialog();
                    return Ok(());
                }
                Action::ToggleSearchWrap => {
                    if let Some(d) = self.find_replace_mut() {
                        d.wrap ^= true;
                    }
                    return Ok(());
                }
                Action::ToggleSearchRegex => {
                    if let Some(d) = self.find_replace_mut() {
                        d.regex = !d.regex;
                    }
                    self.run_find_from_dialog();
                    return Ok(());
                }
                Action::ToggleSearchWholeWord => {
                    if let Some(d) = self.find_replace_mut() {
                        d.whole_word = !d.whole_word;
                    }
                    self.run_find_from_dialog();
                    return Ok(());
                }
                Action::FindNext => {
                    self.find_next();
                    return Ok(());
                }
                Action::FindPrev => {
                    self.find_prev();
                    return Ok(());
                }
                // Ctrl+A → Replace All (only while the Replace dialog is open).
                Action::SelectAll if is_replace => {
                    self.replace_all_from_dialog();
                    return Ok(());
                }
                _ => {}
            }
            // Button focused: Enter/Space activate it; text-editing keys are
            // ignored (they belong to the fields, which are not focused).
            if let Some(btn) = self.interactive_focus_is_button() {
                if matches!(&action, Action::InsertNewline | Action::InsertChar(' ')) {
                    self.activate_interactive_button(btn);
                }
                return Ok(());
            }
            // A field stop is focused: edit the field / run the per-mode action.
            match &action {
                Action::InsertChar(c) => {
                    if let Some(d) = self.find_replace_mut() {
                        d.insert_char(*c);
                    }
                }
                Action::Backspace => {
                    if let Some(d) = self.find_replace_mut() {
                        d.backspace();
                    }
                }
                Action::MoveLeft => {
                    if let Some(d) = self.find_replace_mut() {
                        d.move_left();
                    }
                }
                Action::MoveRight => {
                    if let Some(d) = self.find_replace_mut() {
                        d.move_right();
                    }
                }
                Action::InsertNewline => {
                    if is_replace {
                        self.replace_current_from_dialog();
                    } else {
                        self.run_find_from_dialog();
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 025 — Go-to-Line prompt intercept: digits edit the number,
        // Enter jumps (clamped) to the line start, Esc cancels; everything else is
        // consumed so the buffer is never modified while the prompt is open.
        if matches!(self.modal, Modal::GotoLine { .. }) {
            // Feature 031: caret-aware digit editing. The value is ASCII digits, so
            // the caret index equals a byte offset.
            match &action {
                Action::InsertChar(c) if c.is_ascii_digit() => {
                    if let Modal::GotoLine { digits, caret } = &mut self.modal {
                        let c0 = (*caret).min(digits.len());
                        digits.insert(c0, *c);
                        *caret = c0 + 1;
                    }
                }
                Action::Backspace => {
                    if let Modal::GotoLine { digits, caret } = &mut self.modal {
                        let c0 = (*caret).min(digits.len());
                        if c0 > 0 {
                            digits.remove(c0 - 1);
                            *caret = c0 - 1;
                        }
                    }
                }
                Action::MoveLeft => {
                    if let Modal::GotoLine { caret, .. } = &mut self.modal {
                        *caret = caret.saturating_sub(1);
                    }
                }
                Action::MoveRight => {
                    if let Modal::GotoLine { digits, caret } = &mut self.modal {
                        *caret = (*caret + 1).min(digits.len());
                    }
                }
                Action::MoveLineStart => {
                    if let Modal::GotoLine { caret, .. } = &mut self.modal {
                        *caret = 0;
                    }
                }
                Action::MoveLineEnd => {
                    if let Modal::GotoLine { digits, caret } = &mut self.modal {
                        *caret = digits.len();
                    }
                }
                Action::InsertNewline => {
                    // Take the digits out and close the prompt (mem::take → None).
                    let entry = match std::mem::take(&mut self.modal) {
                        Modal::GotoLine { digits, .. } => digits,
                        other => {
                            self.modal = other;
                            String::new()
                        }
                    };
                    if let Ok(n) = entry.parse::<usize>() {
                        let count = self.active_buffer().rope.line_count();
                        let line1 = n.clamp(1, count.max(1));
                        self.set_cursor_lc(line1 - 1, 0);
                    }
                    // Empty / non-numeric → closed with no movement.
                }
                Action::MenuClose | Action::Quit => {
                    self.close_modal();
                }
                _ => {}
            }
            return Ok(());
        }

        // T012 — Encoding-dialog intercept: when the dialog is open, only
        // Up/Down (navigate), Enter (confirm), and Esc/MenuClose (cancel) are
        // processed; all other actions are silently consumed.
        if let Modal::EncodingSelect { row: idx } = self.modal {
            let n = crate::ui::dialog::ENCODING_OPTIONS.len();
            // Esc always cancels, from any focus stop.
            if matches!(&action, Action::MenuClose) {
                self.close_modal();
                return Ok(());
            }
            // Button focused (OK/Cancel): Enter/Space activate; arrows no-op.
            if let Some(btn) = self.interactive_focus_is_button() {
                if matches!(&action, Action::InsertNewline | Action::InsertChar(' ')) {
                    self.activate_interactive_button(btn);
                }
                return Ok(());
            }
            // List focused: existing navigation/confirm behavior (feature 004).
            // Feature 028: PageUp/PageDown jump by a page, clamped (no wrap).
            match &action {
                Action::MoveUp => {
                    self.set_encoding_select((idx + n - 1) % n);
                }
                Action::MoveDown => {
                    self.set_encoding_select((idx + 1) % n);
                }
                Action::MovePageDown => {
                    self.set_encoding_select((idx + DIALOG_LIST_PAGE).min(n - 1));
                }
                Action::MovePageUp => {
                    self.set_encoding_select(idx.saturating_sub(DIALOG_LIST_PAGE));
                }
                Action::InsertNewline => {
                    // Feature 046: checked list access (stale row → no-op, not panic).
                    if let Some(enc) = crate::ui::dialog::ENCODING_OPTIONS.get(idx).map(|o| o.0) {
                        self.close_modal();
                        self.do_save_as_encoding(enc);
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 012 — File browser intercept (Open/Save). Arrow keys move,
        // Enter/Right activate, Left/Backspace go to parent, printable chars edit
        // the filename/path field, Esc cancels. All other actions are consumed so
        // the browser stays modal over the buffer.
        if self.file_browser().is_some() {
            // Esc always cancels, from any focus stop.
            if matches!(&action, Action::MenuClose) {
                self.close_modal();
                return Ok(());
            }
            // Button focused (Open|Save / Cancel): Enter/Space activate; other
            // navigation/edit keys no-op so they don't mutate the listing.
            if let Some(btn) = self.interactive_focus_is_button() {
                if matches!(&action, Action::InsertNewline | Action::InsertChar(' ')) {
                    self.activate_interactive_button(btn);
                }
                return Ok(());
            }
            // Browser focused: existing navigation/edit behavior (feature 012).
            let vis = {
                let (w, h) = self.terminal_size;
                self.file_browser()
                    .map(|b| b.visible_rows(ratatui::layout::Rect::new(0, 0, w, h)))
                    .unwrap_or(0)
            };
            let mut outcome: Option<BrowseOutcome> = None;
            if let Some(fb) = self.file_browser_mut() {
                // Feature 031: while a filename is being typed, Left/Right/Home/End
                // edit the field caret; with an empty field they keep the list
                // navigation semantics (← parent, → activate).
                let editing = !fb.filename.is_empty();
                match &action {
                    Action::MoveUp => fb.move_up(vis),
                    Action::MoveDown => fb.move_down(vis),
                    Action::MovePageUp => fb.page_up(vis),
                    Action::MovePageDown => fb.page_down(vis),
                    Action::MoveLeft if editing => fb.caret_left(),
                    Action::MoveRight if editing => fb.caret_right(),
                    Action::MoveLineStart if editing => fb.caret_home(),
                    Action::MoveLineEnd if editing => fb.caret_end(),
                    Action::MoveLeft => fb.enter_parent(),
                    Action::MoveRight | Action::InsertNewline => outcome = Some(fb.activate()),
                    Action::Backspace => fb.backspace(),
                    Action::InsertChar(c) => fb.push_char(*c),
                    _ => {}
                }
            }
            if let Some(outcome) = outcome {
                self.apply_browse_outcome(outcome);
            }
            return Ok(());
        }

        // Feature 011/018 — Help / About overlay: arrows/PageUp-Down scroll the
        // cheat sheet; Esc/Enter/Quit close; other input is consumed (modal).
        // Feature 028: Home/End jump to top/bottom and every scroll is clamped to
        // the content so keyboard scrolling stays in range.
        if let Some(screen) = self.help_screen() {
            let (max_scroll, page) = self.help_view_metrics(screen);
            match &action {
                Action::MoveDown => {
                    if let Some(s) = self.help_scroll_mut() {
                        *s = (*s + 1).min(max_scroll);
                    }
                }
                Action::MoveUp => {
                    if let Some(s) = self.help_scroll_mut() {
                        *s = s.saturating_sub(1);
                    }
                }
                Action::MovePageDown => {
                    if let Some(s) = self.help_scroll_mut() {
                        *s = (*s + page).min(max_scroll);
                    }
                }
                Action::MovePageUp => {
                    if let Some(s) = self.help_scroll_mut() {
                        *s = s.saturating_sub(page);
                    }
                }
                Action::MoveLineStart => {
                    if let Some(s) = self.help_scroll_mut() {
                        *s = 0;
                    }
                }
                Action::MoveLineEnd => {
                    if let Some(s) = self.help_scroll_mut() {
                        *s = max_scroll;
                    }
                }
                Action::MenuClose | Action::InsertNewline | Action::Quit => {
                    self.close_modal();
                }
                // A printable key also dismisses (legacy behavior), except none
                // that we use for scrolling above.
                Action::InsertChar(_) => self.close_modal(),
                _ => {}
            }
            return Ok(());
        }

        // Feature 008 — Plugin consent dialog intercept: Enter/Y = allow, Esc/N = deny.
        if !self.pending_plugin_consent.is_empty() {
            match &action {
                Action::InsertNewline => self.consent_decide(true),
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'Y') => {
                    self.consent_decide(true)
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'N') => {
                    self.consent_decide(false)
                }
                Action::MenuClose | Action::Quit => self.consent_decide(false),
                _ => {}
            }
            return Ok(());
        }

        // Feature 008 — Plugin manager dialog intercept: Up/Down navigate,
        // Space/Enter toggle enabled, Esc closes.
        if self.is_plugin_manager_open() {
            let n = self.plugin_host.registry.instances.len();
            // Esc/Quit always close, from any focus stop.
            if matches!(&action, Action::MenuClose | Action::Quit) {
                self.close_modal();
                return Ok(());
            }
            // Button focused (Close): Enter/Space activate; arrows no-op.
            if let Some(btn) = self.interactive_focus_is_button() {
                if matches!(&action, Action::InsertNewline | Action::InsertChar(' ')) {
                    self.activate_interactive_button(btn);
                }
                return Ok(());
            }
            // List focused: existing navigation/toggle behavior (feature 008).
            // Feature 028: PageUp/PageDown jump by a page, clamped (no wrap).
            match &action {
                Action::MoveUp if n > 0 => {
                    if let Some(c) = self.plugin_manager_cursor_mut() {
                        *c = (*c + n - 1) % n;
                    }
                }
                Action::MoveDown if n > 0 => {
                    if let Some(c) = self.plugin_manager_cursor_mut() {
                        *c = (*c + 1) % n;
                    }
                }
                Action::MovePageDown if n > 0 => {
                    if let Some(c) = self.plugin_manager_cursor_mut() {
                        *c = (*c + DIALOG_LIST_PAGE).min(n - 1);
                    }
                }
                Action::MovePageUp if n > 0 => {
                    if let Some(c) = self.plugin_manager_cursor_mut() {
                        *c = c.saturating_sub(DIALOG_LIST_PAGE);
                    }
                }
                Action::InsertChar(' ') | Action::InsertNewline => {
                    self.plugin_manager_toggle_current();
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 009 — Menu-bar navigation intercept. Placed AFTER all modal
        // dialog guards (modals win — FR-012) and BEFORE the normal action match.
        // Routes navigation/selection keys to the menu state machine over the
        // resolved (built-in + plugin) menu list, and consumes all other actions
        // so navigation never mutates the buffer (FR-006).
        if self.menu_bar.is_active() {
            // Ctrl+Q must still quit while a menu is open: close the menu and
            // fall through to the normal quit handling in the main match below.
            if matches!(action, Action::Quit) {
                self.menu_bar.close_menu();
            } else {
                let menus = self.resolved_menus();
                match action {
                    Action::MoveUp => self.menu_bar.navigate_up(&menus),
                    Action::MoveDown => self.menu_bar.navigate_down(&menus),
                    Action::MoveLeft => self.menu_bar.navigate_left(&menus),
                    Action::MoveRight => self.menu_bar.navigate_right(&menus),
                    Action::MenuClose => self.menu_bar.close_menu(),
                    Action::InsertNewline => {
                        if let Some(selected) = self.menu_bar.select_item(&menus) {
                            return self.handle_action(selected);
                        }
                    }
                    // Switching/opening menus while the bar is active.
                    Action::MenuFile => self.menu_bar.open_menu(0, &menus),
                    Action::MenuEdit => self.menu_bar.open_menu(1, &menus),
                    Action::MenuSearch => self.menu_bar.open_menu(2, &menus),
                    Action::MenuView => self.menu_bar.open_menu(3, &menus),
                    Action::MenuOptions => self.menu_bar.open_menu(4, &menus),
                    Action::MenuHelp => self.menu_bar.open_menu(5, &menus),
                    Action::MenuOpen(idx) => self.menu_bar.open_menu(idx, &menus),
                    Action::Menu => self.menu_bar.activate_bar(),
                    // Feature 013: mnemonic accelerator typed while the bar is active.
                    // In a dropdown, a matching letter activates the item (like Enter)
                    // and a non-match is an inert no-op (does NOT jump to another menu).
                    // At the top level, a matching letter opens that menu. Letters never
                    // edit the buffer while the bar is active (FR-004/FR-007).
                    Action::InsertChar(c) => match self.menu_bar.state {
                        MenuState::DropDown { .. } => {
                            if let Some(selected) = self.menu_bar.select_item_by_mnemonic(&menus, c)
                            {
                                return self.handle_action(selected);
                            }
                        }
                        MenuState::TopActive(_) => {
                            self.menu_bar.open_menu_by_mnemonic(&menus, c);
                        }
                        MenuState::Inactive => {}
                    },
                    // Everything else is consumed (no buffer mutation) while open.
                    _ => {}
                }
                return Ok(());
            }
        }

        match action {
            Action::Quit => self.handle_quit(),
            Action::Resize(w, h) => self.handle_resize(w, h),
            Action::Tick => self.handle_tick(),

            // Cursor movement — T025
            Action::MoveUp => self.move_cursor(Direction::Up),
            Action::MoveDown => self.move_cursor(Direction::Down),
            Action::MoveLeft => self.move_cursor(Direction::Left),
            Action::MoveRight => self.move_cursor(Direction::Right),
            Action::MoveLineStart => self.move_line_start(),
            Action::MoveLineEnd => self.move_line_end(),
            Action::MovePageUp => self.move_page_up(),
            Action::MovePageDown => self.move_page_down(),
            Action::MoveDocStart => self.move_doc_start(),
            Action::MoveDocEnd => self.move_doc_end(),
            // Feature 032: word-wise movement.
            Action::MoveWordLeft => self.move_word(Direction::Left),
            Action::MoveWordRight => self.move_word(Direction::Right),

            // Feature 017: Shift+navigation — extend the selection while moving.
            Action::SelectLeft => self.move_cursor_selecting(Direction::Left),
            Action::SelectRight => self.move_cursor_selecting(Direction::Right),
            Action::SelectUp => self.move_cursor_selecting(Direction::Up),
            Action::SelectDown => self.move_cursor_selecting(Direction::Down),
            Action::SelectLineStart => self.select_line_start(),
            Action::SelectLineEnd => self.select_line_end(),
            // Feature 032: word-wise selection.
            Action::SelectWordLeft => self.move_word_selecting(Direction::Left),
            Action::SelectWordRight => self.move_word_selecting(Direction::Right),

            // Text insertion — T026
            Action::InsertChar(c) => self.insert_char(c),
            Action::InsertNewline => self.insert_newline(),

            // Deletion — T027
            Action::Backspace => self.delete_backward(),
            Action::Delete => self.delete_forward(),
            // Feature 032: word-wise deletion.
            Action::DeleteWordLeft => self.delete_word(Direction::Left),
            Action::DeleteWordRight => self.delete_word(Direction::Right),

            // File browser (Feature 012). Open/Save As show the navigable browser.
            Action::Open => {
                self.modal = Modal::FileBrowser(FileBrowser::open(
                    self.browser_start_dir(),
                    BrowseMode::Open,
                ));
            }
            Action::SaveAs => {
                self.modal = Modal::FileBrowser(FileBrowser::open(
                    self.browser_start_dir(),
                    BrowseMode::Save,
                ));
            }

            // Feature 049: open a recent file by its index in the recent list.
            Action::OpenRecent(idx) => self.open_recent(idx),

            // File operations (Feature 011).
            Action::New => self.new_buffer(),
            Action::Close => self.close_active_buffer(),
            Action::Revert => self.handle_revert(),

            // Edit operations (Feature 011) — previously unhandled, so both the
            // menu items and the Ctrl+Z/Y/X/C/V/A shortcuts were dead.
            Action::Undo => self.handle_undo(),
            Action::Redo => self.handle_redo(),
            Action::Cut => self.cut_selection(),
            Action::Copy => self.copy_selection(),
            Action::Paste => self.paste_clipboard(),
            Action::SelectAll => self.select_all(),

            // View toggle (Feature 011).
            Action::ToggleLineNumbers => {
                self.config.line_numbers = !self.config.line_numbers;
            }

            // Help menu (Feature 011).
            Action::Help => {
                self.modal = Modal::Help {
                    screen: HelpScreen::Help,
                    scroll: 0,
                };
            }
            Action::About => {
                self.modal = Modal::Help {
                    screen: HelpScreen::About,
                    scroll: 0,
                };
            }

            // Save prompt responses (T033)
            Action::Save => self.handle_save_action(),

            // T013 — Save As Encoding dialog trigger
            Action::SaveAsEncoding => {
                if !self.buffers.is_empty() {
                    let idx = Self::encoding_to_idx(self.active_buffer().encoding);
                    self.set_encoding_select(idx);
                }
            }

            // Search and replace (Feature 015 — interactive dialogs)
            Action::Find => self.open_find_dialog(),
            Action::FindNext => self.find_next(),
            Action::FindPrev => self.find_prev(),
            Action::FindReplace => self.open_replace_dialog(),
            // Feature 025: open the Go-to-Line prompt (only when no other modal is
            // already open; the intercept above handles it once open).
            Action::GoToLine => {
                if self.open_button_dialog().is_none()
                    && self.interactive_dialog().is_none()
                    && self.help_screen().is_none()
                    && self.goto_line_digits().is_none()
                    && !self.menu_bar.is_active()
                // Feature 029: don't open over a menu
                {
                    // Feature 031: caret starts at 0.
                    self.modal = Modal::GotoLine {
                        digits: String::new(),
                        caret: 0,
                    };
                }
            }
            // Search-option toggles and Tab focus are only meaningful inside an
            // open Find/Replace dialog (handled by the intercept above); inert here.
            Action::ToggleSearchCase
            | Action::ToggleSearchWrap
            | Action::ToggleSearchRegex
            | Action::ToggleSearchWholeWord
            | Action::FocusNextField
            | Action::FocusPrevField => {}

            // Menu navigation (T048 / Feature 009). Alt+<letter> opens a
            // dropdown directly; F10 (`Menu`) enters the top-level highlight
            // (no dropdown) — the DOS-faithful entry path (FR-015).
            Action::MenuFile => self.open_menu_idx(0),
            Action::MenuEdit => self.open_menu_idx(1),
            Action::MenuSearch => self.open_menu_idx(2),
            Action::MenuView => self.open_menu_idx(3),
            Action::MenuOptions => self.open_menu_idx(4),
            Action::MenuHelp => self.open_menu_idx(5),
            Action::MenuClose => self.menu_bar.close_menu(),
            Action::Menu => self.menu_bar.activate_bar(),
            Action::MenuOpen(idx) => self.open_menu_idx(idx),

            // Multi-buffer navigation (T066)
            Action::NextBuffer => self.next_buffer(),
            Action::PrevBuffer => self.prev_buffer(),

            // Split-view toggle (T067)
            Action::SplitView => {
                self.split_mode = match self.split_mode {
                    crate::ui::SplitMode::Single => crate::ui::SplitMode::Vertical,
                    crate::ui::SplitMode::Vertical => crate::ui::SplitMode::Single,
                };
            }

            // Syntax-highlight toggle (T077)
            Action::ToggleHighlight => self.toggle_highlight(),

            // Theme cycle (T081)
            Action::ToggleTheme => {
                let new_name = match self.theme.name {
                    "classic" => "high-contrast",
                    "high-contrast" => "plain",
                    _ => "classic",
                };
                self.set_theme(new_name);
            }

            // Soft-wrap toggle (Feature 005)
            Action::ToggleSoftWrap => self.handle_toggle_soft_wrap()?,

            // Feature 008 — Plugin API
            Action::OpenPluginManager => {
                self.modal = Modal::PluginManager { cursor: 0 };
            }
            Action::PluginMenuActivated(plugin_id, item_id) => {
                let content = self.active_buffer().rope.to_string();
                if let Some(msg) = self
                    .plugin_host
                    .dispatch_menu_action(&plugin_id, &item_id, &content)
                {
                    self.status_message = Some(msg);
                }
            }

            _ => {
                log::debug!("Unhandled action: {:?}", action);
            }
        }
        Ok(())
    }
}
