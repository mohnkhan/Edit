//! Split from app.rs (Feature 041): dialogs.

use super::*;

impl App {
    // ── Feature 016 — dialog buttons (confirm/dismiss dialogs) ────────────────

    /// Initialize the focused dialog control once per dialog opening; reset when no
    /// dialog is open. Called from render and before key/mouse handling so focus is
    /// correct even without a prior frame.
    ///
    /// - Confirm/dismiss dialogs (Feature 016): focus the safe default button.
    /// - Interactive dialogs (Feature 020): focus the **primary control** (stop 0 —
    ///   the field/list), NOT a button. Feature 028 fix: without this, `dialog_focus`
    ///   carried over from a previous dialog could land on a button when an
    ///   interactive dialog opened, so typed characters were swallowed and the caret
    ///   hidden (the Save-As "can't type / can't see what I type" bug).
    pub(super) fn ensure_dialog_focus(&mut self) {
        if self.open_button_dialog().is_some() {
            if !self.dialog_focus_init {
                self.dialog_focus = self.dialog_default_focus();
                self.dialog_focus_init = true;
            }
        } else if self.interactive_dialog().is_some() {
            if !self.dialog_focus_init {
                self.dialog_focus = 0; // primary control (field/list)
                self.dialog_focus_init = true;
            }
        } else {
            self.dialog_focus_init = false;
        }
    }

    /// The currently-open confirm/dismiss dialog that has a button bar, if any.
    /// Order matches modal precedence.
    pub(super) fn open_button_dialog(&self) -> Option<ButtonDialog> {
        // Precedence preserved from the original flag chain (Feature 039): the
        // synchronously-opened button dialogs live in `self.modal`; the two async
        // ones (`pending_external_change`, `pending_plugin_consent`) keep their
        // original slots between `SavePrompt` and `RevertConfirm` / at the tail.
        match &self.modal {
            Modal::SessionRestore(_) => Some(ButtonDialog::SessionRestore),
            Modal::SavePrompt => Some(ButtonDialog::SavePrompt),
            _ if self.pending_external_change.is_some() => Some(ButtonDialog::ExternalChange),
            Modal::RevertConfirm(_) => Some(ButtonDialog::RevertConfirm),
            Modal::CloseConfirm(_) => Some(ButtonDialog::CloseConfirm),
            _ if !self.pending_plugin_consent.is_empty() => Some(ButtonDialog::PluginConsent),
            _ => None,
        }
    }

    /// Ordered button labels for the open confirm/dismiss dialog (tab order).
    pub fn dialog_button_labels(&self) -> Vec<&'static str> {
        // Feature 021: each label carries its activating key. Dispatch
        // (`activate_dialog_button`) keys on the button index, never this text.
        match self.open_button_dialog() {
            Some(ButtonDialog::SessionRestore) => vec!["Restore (Enter)", "Decline (Esc)"],
            Some(ButtonDialog::SavePrompt) => vec!["Save (S)", "Discard (D)", "Cancel (Esc)"],
            Some(ButtonDialog::ExternalChange) => vec!["Reload (Enter)", "Keep (Esc)"],
            Some(ButtonDialog::RevertConfirm) => vec!["Revert (Enter)", "Cancel (Esc)"],
            Some(ButtonDialog::PluginConsent) => vec!["Allow (Enter)", "Deny (Esc)"],
            Some(ButtonDialog::CloseConfirm) => vec!["Save (S)", "Discard (D)", "Cancel (Esc)"],
            None => vec![],
        }
    }

    /// Safe default-focused button index for the open dialog (R6).
    pub fn dialog_default_focus(&self) -> usize {
        match self.open_button_dialog() {
            Some(ButtonDialog::SavePrompt) => 2,     // Cancel
            Some(ButtonDialog::ExternalChange) => 1, // Keep
            Some(ButtonDialog::RevertConfirm) => 1,  // Cancel
            Some(ButtonDialog::PluginConsent) => 1,  // Deny
            Some(ButtonDialog::CloseConfirm) => 2,   // Cancel
            _ => 0,
        }
    }

    /// Whether clicking outside the dialog box cancels it (all current ones have a
    /// safe cancel).
    pub fn dialog_supports_outside_cancel(&self) -> bool {
        self.open_button_dialog().is_some()
    }

    /// Button index treated as "cancel/no/keep" for an outside click.
    pub(super) fn dialog_cancel_index(&self) -> Option<usize> {
        match self.open_button_dialog()? {
            ButtonDialog::SessionRestore => Some(1), // Decline
            ButtonDialog::SavePrompt => Some(2),     // Cancel
            ButtonDialog::ExternalChange => Some(1), // Keep
            ButtonDialog::RevertConfirm => Some(1),  // Cancel
            ButtonDialog::PluginConsent => Some(1),  // Deny
            ButtonDialog::CloseConfirm => Some(2),   // Cancel
        }
    }

    /// Run the choice for button `idx` of the open confirm/dismiss dialog, reusing
    /// the existing handlers so a button == the corresponding key shortcut.
    pub fn activate_dialog_button(&mut self, idx: usize) {
        match self.open_button_dialog() {
            Some(ButtonDialog::SessionRestore) => {
                if idx == 0 {
                    self.do_restore_session();
                }
                self.close_modal();
            }
            Some(ButtonDialog::SavePrompt) => match idx {
                0 => self.prompt_save_and_quit(),
                1 => self.prompt_discard_and_quit(),
                _ => self.prompt_cancel_quit(),
            },
            Some(ButtonDialog::ExternalChange) => {
                if let Some(ec) = self.pending_external_change.take() {
                    if idx == 0 {
                        self.reload_from_disk(ec.buf_idx);
                    } else if let Some(b) = self.buffers.get_mut(ec.buf_idx) {
                        b.modified = true;
                    }
                }
            }
            Some(ButtonDialog::RevertConfirm) => {
                let b = match self.modal {
                    Modal::RevertConfirm(b) => Some(b),
                    _ => None,
                };
                self.close_modal();
                if idx == 0 {
                    if let Some(b) = b {
                        self.reload_from_disk(b);
                    }
                }
            }
            Some(ButtonDialog::PluginConsent) => self.consent_decide(idx == 0),
            Some(ButtonDialog::CloseConfirm) => {
                // Feature 027: operate on the stored (clicked) index, not the
                // active buffer (M1). Save → save then close; Discard → close;
                // Cancel → dismiss, nothing closes. A failed save keeps the
                // dialog open so no changes are silently lost (Principle VII).
                let taken = match self.modal {
                    Modal::CloseConfirm(bidx) => Some(bidx),
                    _ => None,
                };
                self.close_modal();
                if let Some(bidx) = taken {
                    match idx {
                        0 => match self.buffers.get(bidx).map(|b| b.save()) {
                            Some(Ok(())) => {
                                if let Some(path) = self.buffers[bidx].path.clone() {
                                    self.self_write_times.insert(path, Instant::now());
                                }
                                self.close_buffer_at(bidx);
                            }
                            Some(Err(e)) => {
                                log::error!("Save failed on tab close: {}", e);
                                self.modal = Modal::CloseConfirm(bidx); // keep open
                            }
                            None => {}
                        },
                        1 => self.close_buffer_at(bidx),
                        _ => {} // Cancel — already cleared above
                    }
                }
            }
            None => {}
        }
    }

    /// Title + body lines for the open confirm dialog (Feature 016). Centralized
    /// here so the overlay render and mouse hit-test share identical geometry.
    pub(super) fn dialog_view_text(&self) -> Option<(&'static str, Vec<String>)> {
        let kind = self.open_button_dialog()?;
        let active_name = || {
            self.buffers
                .get(self.active_idx)
                .and_then(|b| b.path.as_ref())
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "[No Name]".to_string())
        };
        let v = match kind {
            ButtonDialog::SessionRestore => (
                "Restore Session",
                vec!["Restore previous session?".to_string()],
            ),
            ButtonDialog::SavePrompt => (
                "Unsaved Changes",
                vec![format!("Save changes to {}?", active_name())],
            ),
            ButtonDialog::ExternalChange => {
                let mut lines = vec!["File changed on disk.".to_string()];
                if let Some(ec) = &self.pending_external_change {
                    if self.buffers.get(ec.buf_idx).map(|b| b.modified) == Some(true) {
                        lines.push("WARNING: unsaved changes will be lost.".to_string());
                    }
                }
                ("External Change", lines)
            }
            ButtonDialog::RevertConfirm => (
                "Revert",
                vec![
                    format!("Revert {} to last saved version?", active_name()),
                    "Unsaved changes will be lost.".to_string(),
                ],
            ),
            ButtonDialog::PluginConsent => {
                let name = self
                    .pending_plugin_consent
                    .first()
                    .map(|m| m.id.clone())
                    .unwrap_or_default();
                (
                    "Plugin Consent",
                    vec![format!("Allow plugin '{}' to run?", name)],
                )
            }
            ButtonDialog::CloseConfirm => {
                // Name the buffer being closed (the stored index, not active).
                let name = match self.modal {
                    Modal::CloseConfirm(i) => Some(i),
                    _ => None,
                }
                .and_then(|i| self.buffers.get(i))
                .and_then(|b| b.path.as_ref())
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "[No Name]".to_string());
                (
                    "Unsaved Changes",
                    vec![format!("Save changes to {} before closing?", name)],
                )
            }
        };
        Some(v)
    }

    /// Overlay rect for the open confirm dialog, sized to fit its body + button
    /// row. Shared by the renderer and mouse hit-testing so clicks land on the
    /// drawn buttons (Feature 016).
    pub fn button_dialog_rect(&self) -> Option<ratatui::layout::Rect> {
        let (_title, body) = self.dialog_view_text()?;
        let labels = self.dialog_button_labels();
        let (tw, th) = self.terminal_size;
        let body_w = body
            .iter()
            .map(|l| unicode_width::UnicodeWidthStr::width(l.as_str()) as u16)
            .max()
            .unwrap_or(0);
        // Each button: width(label)+4, plus 1-col gaps.
        let buttons_w: u16 = labels
            .iter()
            .map(|l| unicode_width::UnicodeWidthStr::width(*l) as u16 + 4)
            .sum::<u16>()
            + labels.len().saturating_sub(1) as u16;
        let inner = body_w.max(buttons_w);
        let dw = (inner + 4).clamp(24, tw.max(24)).min(tw.max(1));
        // borders(2) + body lines + gap(1) + button row(3)
        let dh = (body.len() as u16 + 6).min(th.max(1));
        let dx = tw.saturating_sub(dw) / 2;
        let dy = th.saturating_sub(dh) / 2;
        Some(ratatui::layout::Rect::new(dx, dy, dw, dh))
    }

    /// Everything the renderer needs to draw the open confirm dialog:
    /// `(rect, title, body lines, button labels, focused index)`. `None` when no
    /// button-dialog is open (Feature 016).
    #[allow(clippy::type_complexity)]
    pub fn button_dialog_render(
        &self,
    ) -> Option<(
        ratatui::layout::Rect,
        &'static str,
        Vec<String>,
        Vec<&'static str>,
        usize,
    )> {
        let rect = self.button_dialog_rect()?;
        let (title, body) = self.dialog_view_text()?;
        let labels = self.dialog_button_labels();
        let focus = self.dialog_focus.min(labels.len().saturating_sub(1));
        Some((rect, title, body, labels, focus))
    }

    // ── Feature 020 — interactive/list dialog focus ring ──────────────────────
    //
    // The four interactive dialogs (encoding select, plugin manager, Find/Replace,
    // file browser) reuse `dialog_focus` as a ring index: stop 0 (and stop 1 for
    // Find/Replace in replace mode) is the primary control; later stops are boxed
    // buttons. `Tab`/`Shift+Tab` move the index; a button is activated by
    // Enter/Space or a click. While the primary control is focused, the dialog's
    // existing keys behave exactly as before.

    /// The currently-open interactive/list dialog, if any. These are mutually
    /// exclusive in practice; the order is a defensive precedence.
    pub(super) fn interactive_dialog(&self) -> Option<InteractiveDialog> {
        if self.find_replace().is_some() {
            Some(InteractiveDialog::FindReplace)
        } else if self.encoding_select_row().is_some() {
            Some(InteractiveDialog::EncodingSelect)
        } else if self.file_browser().is_some() {
            Some(InteractiveDialog::FileBrowser)
        } else if self.is_plugin_manager_open() {
            Some(InteractiveDialog::PluginManager)
        } else {
            None
        }
    }

    /// Number of primary-control focus stops that precede the buttons in the ring
    /// (1 for the list/browser dialogs; 1 in Find mode and 2 in Replace mode).
    pub(super) fn interactive_field_stops(&self) -> usize {
        match self.interactive_dialog() {
            Some(InteractiveDialog::FindReplace) => match self.find_replace().map(|d| d.mode) {
                Some(DialogMode::Replace) => 2,
                _ => 1,
            },
            Some(_) => 1,
            None => 0,
        }
    }

    /// Ordered boxed-button labels for the open interactive dialog (tab order
    /// after the primary control). Mode-aware for Find/Replace and the file
    /// browser.
    pub fn interactive_button_labels(&self) -> Vec<&'static str> {
        // Feature 021: labels carry their activating key; dispatch
        // (`activate_interactive_button`) keys on index + mode, not this text.
        match self.interactive_dialog() {
            Some(InteractiveDialog::EncodingSelect) => vec!["OK (Enter)", "Cancel (Esc)"],
            Some(InteractiveDialog::PluginManager) => vec!["Close (Esc)"],
            Some(InteractiveDialog::FileBrowser) => {
                let save = matches!(
                    self.file_browser().map(|b| b.mode),
                    Some(crate::ui::file_browser::BrowseMode::Save)
                );
                if save {
                    vec!["Save (Enter)", "Cancel (Esc)"]
                } else {
                    vec!["Open (Enter)", "Cancel (Esc)"]
                }
            }
            Some(InteractiveDialog::FindReplace) => {
                let replace = matches!(
                    self.find_replace().map(|d| d.mode),
                    Some(DialogMode::Replace)
                );
                if replace {
                    vec![
                        "Find (Enter)",
                        "Replace",
                        "Replace All (Ctrl+A)",
                        "Close (Esc)",
                    ]
                } else {
                    vec!["Find (Enter)", "Close (Esc)"]
                }
            }
            None => vec![],
        }
    }

    /// Total focus stops in the ring (primary-control stops + buttons).
    pub(super) fn interactive_ring_len(&self) -> usize {
        self.interactive_field_stops() + self.interactive_button_labels().len()
    }

    /// `Some(button_index)` when `dialog_focus` is on a button rather than the
    /// primary control; `None` when the primary control is focused (or no
    /// interactive dialog is open).
    pub fn interactive_focus_is_button(&self) -> Option<usize> {
        self.interactive_dialog()?;
        let fs = self.interactive_field_stops();
        if self.dialog_focus >= fs {
            Some(self.dialog_focus - fs)
        } else {
            None
        }
    }

    /// Keep `FindReplaceDialog.focus` in sync with the ring's field stops so the
    /// edited/rendered field matches the focused stop (stop 0 → Query, stop 1 →
    /// Replacement). No-op when a button stop is focused.
    pub(super) fn sync_find_replace_focus(&mut self) {
        let f = self.dialog_focus;
        if let Some(d) = self.find_replace_mut() {
            let field = match f {
                0 => DialogField::Query,
                1 if d.mode == DialogMode::Replace => DialogField::Replacement,
                _ => return,
            };
            d.set_focus(field);
        }
    }

    /// Outer overlay `Rect` for the open interactive dialog — the single geometry
    /// source shared by the renderer and mouse hit-testing so a click always
    /// lands on the button that was drawn.
    pub fn interactive_dialog_rect(&self) -> Option<ratatui::layout::Rect> {
        let (tw, th) = self.terminal_size;
        let area = ratatui::layout::Rect::new(0, 0, tw, th);
        match self.interactive_dialog()? {
            InteractiveDialog::EncodingSelect => {
                Some(crate::ui::dialog::encoding_dialog_rect(area))
            }
            InteractiveDialog::PluginManager => Some(crate::ui::plugin_manager::manager_rect(
                &self.plugin_host,
                self.plugin_manager_cursor(),
                area,
            )),
            InteractiveDialog::FileBrowser => self.file_browser().map(|fb| fb.box_rect(area)),
            InteractiveDialog::FindReplace => self
                .find_replace()
                .map(|d| crate::ui::find_replace_rect(d, area)),
        }
    }

    /// Run the action bound to button `idx` of the open interactive dialog. Each
    /// maps onto an action the dialog already performs (no new actions).
    pub fn activate_interactive_button(&mut self, idx: usize) {
        match self.interactive_dialog() {
            Some(InteractiveDialog::EncodingSelect) => {
                if idx == 0 {
                    // Feature 046: checked list access — a stale selection row can't
                    // index past ENCODING_OPTIONS (no-op instead of panic).
                    if let Modal::EncodingSelect { row: sel } = self.modal {
                        if let Some(enc) = crate::ui::dialog::ENCODING_OPTIONS.get(sel).map(|o| o.0)
                        {
                            self.close_modal();
                            self.do_save_as_encoding(enc);
                        }
                    }
                } else {
                    self.close_modal();
                }
            }
            Some(InteractiveDialog::PluginManager) => {
                // Sole button is Close.
                self.close_modal();
            }
            Some(InteractiveDialog::FileBrowser) => {
                if idx == 0 {
                    if let Some(outcome) = self.file_browser_mut().map(|fb| fb.activate()) {
                        self.apply_browse_outcome(outcome);
                    }
                } else {
                    self.close_modal();
                }
            }
            Some(InteractiveDialog::FindReplace) => {
                // Dispatch on (mode, index), not label text (labels carry key hints).
                // Find mode ring buttons: [Find, Close]; Replace mode:
                // [Find, Replace, Replace All, Close].
                let replace = matches!(
                    self.find_replace().map(|d| d.mode),
                    Some(DialogMode::Replace)
                );
                if replace {
                    match idx {
                        0 => self.run_find_from_dialog(),
                        1 => self.replace_current_from_dialog(),
                        2 => self.replace_all_from_dialog(),
                        _ => self.close_find_replace(),
                    }
                } else {
                    match idx {
                        0 => self.run_find_from_dialog(),
                        _ => self.close_find_replace(),
                    }
                }
            }
            None => {}
        }
    }

    /// File ▸ Revert (Feature 014): reload the active buffer from its last saved
    /// version on disk, discarding in-editor changes. No-op with a notice when the
    /// buffer was never saved; asks for confirmation when there are unsaved changes.
    pub fn handle_revert(&mut self) {
        let idx = self.active_idx;
        if self.buffers[idx].path.is_none() {
            self.status_message = Some("Nothing to revert (never saved)".to_string());
            return;
        }
        if self.buffers[idx].modified {
            // Confirm before discarding unsaved changes.
            self.modal = Modal::RevertConfirm(idx);
        } else {
            // Clean buffer — reload directly (harmless re-read).
            self.reload_from_disk(idx);
        }
    }

    /// Write the active buffer to `path` (File ▸ Save As).
    pub fn do_save_as(&mut self, path: PathBuf) {
        // Feature 029: honor an encoding chosen before the destination was picked
        // (the Save-As-Encoding → file-browser flow). Previously this path ignored
        // `pending_save_as_encoding`, silently writing the file in the old encoding.
        if let Some(enc) = self.pending_save_as_encoding.take() {
            self.active_buffer_mut().encoding = enc;
        }
        match self.active_buffer_mut().save_as(path.clone()) {
            Ok(()) => {
                self.active_buffer_mut().modified = false;
                self.active_buffer_mut().undo_stack.mark_saved(); // Feature 014
                                                                  // Feature 007: suppress the watcher event from our own write.
                self.self_write_times.insert(path.clone(), Instant::now());
                // Feature 049: a Save-As destination is a recently-used file.
                self.record_recent(&path.to_string_lossy());
                self.status_message = Some(format!("Saved as {}", path.display()));
            }
            Err(e) => {
                log::error!("save_as failed for {:?}: {}", path, e);
                self.status_message = Some(format!("Save As failed: {e}"));
            }
        }
    }

    /// Select the entire buffer (Edit ▸ Select All / Ctrl+A).
    pub fn select_all(&mut self) {
        let buf = self.active_buffer_mut();
        let line_count = buf.rope.line_count();
        if line_count == 0 {
            return;
        }
        let last_line = line_count - 1;
        let last_gcol = buf.rope.grapheme_count_on_line(last_line);
        let anchor = CursorPos {
            line: 0,
            grapheme_col: 0,
            visual_col: 0,
        };
        let active_vcol = CursorPos::visual_col_from_grapheme_col(&buf.rope, last_line, last_gcol);
        let active = CursorPos {
            line: last_line,
            grapheme_col: last_gcol,
            visual_col: active_vcol,
        };
        buf.selection = Some(Selection { anchor, active });
        buf.cursor = active;
    }

    /// Undo the most recent edit (Edit ▸ Undo / Ctrl+Z).
    pub fn handle_undo(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        let op = {
            let buf = self.active_buffer_mut();
            buf.undo_stack.undo(&mut buf.rope)
        };
        match op {
            Some(op) => {
                self.apply_history_cursor(undo_target_idx(&op));
                self.status_message = Some("Undo".to_string());
            }
            None => self.status_message = Some("Nothing to undo".to_string()),
        }
    }

    /// Redo the most recently undone edit (Edit ▸ Redo / Ctrl+Y).
    pub fn handle_redo(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        let op = {
            let buf = self.active_buffer_mut();
            buf.undo_stack.redo(&mut buf.rope)
        };
        match op {
            Some(op) => {
                self.apply_history_cursor(redo_target_idx(&op));
                self.status_message = Some("Redo".to_string());
            }
            None => self.status_message = Some("Nothing to redo".to_string()),
        }
    }

    /// Shared post-undo/redo bookkeeping: mark dirty, drop selection, invalidate
    /// the wrap cache, and move the cursor to `char_idx` (clamped to the buffer).
    pub(super) fn apply_history_cursor(&mut self, char_idx: usize) {
        let (line, gcol) = self.line_col_for_char_idx(char_idx);
        let buf = self.active_buffer_mut();
        // Feature 014: undo/redo may return the content to the saved baseline —
        // derive Modified from the undo history instead of forcing it true.
        buf.refresh_modified();
        buf.selection = None;
        let vcol = CursorPos::visual_col_from_grapheme_col(&buf.rope, line, gcol);
        buf.cursor = CursorPos {
            line,
            grapheme_col: gcol,
            visual_col: vcol,
        };
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);
        self.clamp_scroll();
    }

    /// Convert a rope char index into a `(line, grapheme_col)` position — the
    /// inverse of [`Self::char_idx_for`]. Clamps past-end indices to the end.
    pub(super) fn line_col_for_char_idx(&self, char_idx: usize) -> (usize, usize) {
        let buf = self.active_buffer();
        let line_count = buf.rope.line_count();
        let mut remaining = char_idx;
        for line in 0..line_count {
            let line_str = buf.rope.line_slice(line);
            let line_chars = line_str.chars().count();
            if remaining <= line_chars {
                let mut acc = 0usize;
                let mut gcol = 0usize;
                for g in line_str.graphemes(true) {
                    if acc >= remaining {
                        break;
                    }
                    acc += g.chars().count();
                    gcol += 1;
                }
                return (line, gcol);
            }
            remaining -= line_chars + 1; // +1 for the line's trailing newline
        }
        let last = line_count.saturating_sub(1);
        (last, buf.rope.grapheme_count_on_line(last))
    }
}
