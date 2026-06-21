//! Split from app.rs (Feature 041): fileops.

use super::*;

impl App {
    // ── T033 — Quit flow ─────────────────────────────────────────────────────

    pub(super) fn handle_quit(&mut self) {
        if self.active_buffer().modified {
            self.modal = Modal::SavePrompt;
            // The actual dialog rendering is handled by the UI layer; here we
            // just gate the quit so the render loop can show the prompt.
            log::debug!("Buffer modified — showing save prompt before quit");
        } else {
            if let Some(data) = self.build_session_data() {
                if let Err(e) = crate::session::save_session(&data) {
                    log::warn!("session save failed: {}", e);
                }
            }
            self.running = false;
        }
    }

    /// Called when the user chooses [S]ave in the save-before-quit prompt.
    pub fn prompt_save_and_quit(&mut self) {
        match self.active_buffer().save() {
            Ok(()) => {
                // Feature 007: record write time for self-write suppression (FR-007).
                if let Some(path) = self.active_buffer().path.clone() {
                    self.self_write_times.insert(path, Instant::now());
                }
                self.close_modal();
                if let Some(data) = self.build_session_data() {
                    if let Err(e) = crate::session::save_session(&data) {
                        log::warn!("session save failed: {}", e);
                    }
                }
                self.running = false;
            }
            Err(e) => {
                log::error!("Save failed: {}", e);
                // Keep the prompt open so the user can decide what to do.
            }
        }
    }

    /// Called when the user chooses [D]iscard in the save-before-quit prompt.
    pub fn prompt_discard_and_quit(&mut self) {
        self.close_modal();
        if let Some(data) = self.build_session_data() {
            if let Err(e) = crate::session::save_session(&data) {
                log::warn!("session save failed: {}", e);
            }
        }
        self.running = false;
    }

    /// Called when the user chooses [C]ancel in the save-before-quit prompt.
    pub fn prompt_cancel_quit(&mut self) {
        self.close_modal();
    }

    // ── Session save/restore ─────────────────────────────────────────────────

    /// Snapshot the current editor state into a [`SessionData`] for saving.
    ///
    /// Returns `None` when there are no saveable buffers (i.e. every open
    /// buffer is an untitled new-file stub with no path on disk).
    pub fn build_session_data(&self) -> Option<crate::session::SessionData> {
        use crate::session::{BufferEntry, SessionData, SplitLayoutKind};

        let buffers: Vec<BufferEntry> = self
            .buffers
            .iter()
            .filter_map(|buf| {
                let path = buf.path.as_ref()?;
                if !path.exists() {
                    return None;
                }
                Some(BufferEntry {
                    path: path.to_string_lossy().into_owned(),
                    cursor_line: (buf.cursor.line + 1) as u32,
                    cursor_col: (buf.cursor.grapheme_col + 1) as u32,
                    // Feature 045: persist this tab's wrap setting.
                    soft_wrap: buf.soft_wrap,
                    // Feature 047: scroll offset, selection, and encoding.
                    scroll_line: buf.scroll_offset.0 as u32,
                    scroll_col: buf.scroll_offset.1 as u32,
                    selection: buf.selection.map(|s| crate::session::SelectionEntry {
                        anchor_line: (s.anchor.line + 1) as u32,
                        anchor_col: (s.anchor.grapheme_col + 1) as u32,
                        active_line: (s.active.line + 1) as u32,
                        active_col: (s.active.grapheme_col + 1) as u32,
                    }),
                    encoding: crate::encoding::encoding_to_str(buf.encoding).to_string(),
                })
            })
            .collect();

        if buffers.is_empty() {
            return None;
        }

        let split_layout = match self.split_mode {
            crate::ui::SplitMode::Single => SplitLayoutKind::None,
            crate::ui::SplitMode::Vertical => SplitLayoutKind::Vertical,
        };

        // active_pane: 0 for single or left pane, 1 for right pane in a split.
        let active_pane = match self.split_mode {
            crate::ui::SplitMode::Single => 0,
            crate::ui::SplitMode::Vertical => {
                if self.active_idx > 0 {
                    1
                } else {
                    0
                }
            }
        };

        Some(SessionData {
            // Feature 045: schema v2 adds per-tab `soft_wrap` (v1 files still load).
            version: 2,
            active_buffer: self.active_idx,
            split_layout,
            active_pane,
            buffers,
        })
    }

    /// Restore a previously saved session: open each recorded buffer, seek
    /// cursors, and apply the saved split layout.
    ///
    /// Called after the user confirms the session restore dialog. T020–T022
    /// (path validation, missing-file handling) are layered on top here.
    pub fn do_restore_session(&mut self) {
        use crate::security::sanitize::{validate_path, PathError};
        use crate::session::SplitLayoutKind;

        let session = match std::mem::take(&mut self.modal) {
            Modal::SessionRestore(s) => s,
            // Not the open overlay — restore state and bail.
            other => {
                self.modal = other;
                return;
            }
        };

        let mut new_buffers: Vec<Buffer> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        for entry in &session.buffers {
            let raw_path = std::path::Path::new(&entry.path);

            // T020: path traversal check.
            let open_path = match validate_path(raw_path) {
                Ok(canonical) => canonical,
                Err(PathError::Traversal) => {
                    log::warn!("session: path traversal rejected: {:?}", raw_path);
                    warnings.push(format!("session: path rejected: {}", raw_path.display()));
                    continue;
                }
                Err(PathError::Io(_)) => {
                    // Non-existent or unreadable — fall through to Buffer::open
                    // which will produce an appropriate error (T021).
                    raw_path.to_path_buf()
                }
            };

            // T021: attempt to open the buffer. Feature 047: decode in the recorded
            // encoding (empty/absent → the default decode, as before).
            let open_encoding = if entry.encoding.is_empty() {
                self.default_encoding
            } else {
                crate::encoding::encoding_from_str(&entry.encoding)
            };
            match Buffer::open(open_path.clone(), open_encoding) {
                Ok(mut buf) => {
                    // Seek cursor to saved position (convert 1-based → 0-based).
                    let target_line = (entry.cursor_line as usize).saturating_sub(1);
                    let target_gcol = (entry.cursor_col as usize).saturating_sub(1);
                    let line_count = buf.rope.line_count();
                    let clamped_line = target_line.min(line_count.saturating_sub(1));
                    let max_gcol = buf.rope.grapheme_count_on_line(clamped_line);
                    let clamped_gcol = target_gcol.min(max_gcol);
                    let vcol = crate::buffer::CursorPos::visual_col_from_grapheme_col(
                        &buf.rope,
                        clamped_line,
                        clamped_gcol,
                    );
                    buf.cursor = crate::buffer::CursorPos {
                        line: clamped_line,
                        grapheme_col: clamped_gcol,
                        visual_col: vcol,
                    };
                    // Feature 045: restore this tab's saved soft-wrap setting
                    // (v1 sessions have no value → `false` → the configured default).
                    buf.soft_wrap = entry.soft_wrap;
                    // Feature 047: restore scroll offset (clamped to content) and the
                    // active selection (each endpoint clamped; degenerate → none).
                    buf.scroll_offset = (
                        (entry.scroll_line as usize).min(line_count.saturating_sub(1)),
                        entry.scroll_col as usize,
                    );
                    if let Some(sel) = &entry.selection {
                        let clamp = |line1: u32, col1: u32| -> crate::buffer::CursorPos {
                            let l = (line1 as usize)
                                .saturating_sub(1)
                                .min(line_count.saturating_sub(1));
                            let g = (col1 as usize)
                                .saturating_sub(1)
                                .min(buf.rope.grapheme_count_on_line(l));
                            crate::buffer::CursorPos {
                                line: l,
                                grapheme_col: g,
                                visual_col: crate::buffer::CursorPos::visual_col_from_grapheme_col(
                                    &buf.rope, l, g,
                                ),
                            }
                        };
                        let anchor = clamp(sel.anchor_line, sel.anchor_col);
                        let active = clamp(sel.active_line, sel.active_col);
                        // Drop a degenerate (empty) selection.
                        if anchor.line != active.line || anchor.grapheme_col != active.grapheme_col
                        {
                            buf.selection = Some(crate::buffer::Selection { anchor, active });
                        }
                    }
                    // Apply syntax highlighting if configured (plugin highlighter wins).
                    if self.config.highlight {
                        if let Some(ref path) = buf.path.clone() {
                            buf.syntax = self
                                .plugin_host
                                .highlighter_for(path, self.theme)
                                .or_else(|| crate::highlight::detect_highlighter(path));
                        }
                    }
                    new_buffers.push(buf);
                }
                Err(_) => {
                    let display = open_path.display().to_string();
                    log::warn!("session: {} not found or unreadable", display);
                    warnings.push(format!("session: {} not found", display));
                }
            }
        }

        // T022: handle all-failed case.
        if new_buffers.is_empty() {
            // Keep the existing blank buffer; show an error message.
            self.status_message = Some("session: no files could be restored".to_string());
            return;
        }

        // Replace buffers with restored set. Feature 028: the restored content has
        // nothing to do with the old buffer the wrap cache was built for, so it MUST
        // be invalidated here or the soft-wrap renderer slices stale offsets → panic.
        self.buffers = new_buffers;
        self.invalidate_wrap_cache();

        // Feature 007: register watches for newly-restored buffer paths.
        if let Some(ref mut fw) = self.file_watcher {
            for buf in &self.buffers {
                if let Some(ref p) = buf.path {
                    if let Err(e) = fw.watch_path(p) {
                        log::warn!("FileWatcher: could not watch {:?}: {}", p, e);
                    }
                }
            }
        }

        // Restore split layout.
        self.split_mode = match session.split_layout {
            SplitLayoutKind::None => crate::ui::SplitMode::Single,
            SplitLayoutKind::Vertical | SplitLayoutKind::Horizontal => {
                crate::ui::SplitMode::Vertical
            }
        };

        // Clamp active_idx to avoid out-of-bounds (I1) and invalidate the wrap
        // cache for the restored active buffer (Feature 043).
        self.activate_buffer(session.active_buffer);

        // Show first warning in the status bar; log the rest.
        if let Some(first) = warnings.first() {
            self.status_message = Some(first.clone());
        }
        for w in warnings.iter().skip(1) {
            log::warn!("{}", w);
        }
    }

    /// Handle an explicit Save action (Ctrl+S / F5).
    pub fn handle_save_action(&mut self) {
        // Feature 012: an unnamed buffer has no path to save to — open the Save
        // browser so the user can choose a destination.
        if self.active_buffer().path.is_none() {
            self.modal = Modal::FileBrowser(FileBrowser::open(
                self.browser_start_dir(),
                BrowseMode::Save,
            ));
            return;
        }
        match self.active_buffer().save() {
            Ok(()) => {
                self.active_buffer_mut().modified = false;
                // Feature 014: the just-written content is the new clean baseline.
                self.active_buffer_mut().undo_stack.mark_saved();
                // Feature 007: record write time for self-write suppression (FR-007).
                if let Some(path) = self.active_buffer().path.clone() {
                    self.self_write_times.insert(path, Instant::now());
                }
                // Feature 029: confirm the save (was silent; Save-As already does).
                let name = self
                    .active_buffer()
                    .path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "file".to_string());
                self.status_message = Some(format!("Saved {name}"));
                log::info!("Buffer saved");
            }
            Err(e) => {
                // Feature 029: surface the failure (was silent — a failed save
                // looked identical to a successful one). Buffer stays modified.
                log::error!("Save failed: {}", e);
                self.status_message = Some(format!("Save failed: {e}"));
            }
        }
    }

    // ── T014 — do_save_as_encoding ────────────────────────────────────────────

    /// Write the active buffer to disk in `enc`.
    ///
    /// Case A (named buffer): sets `buf.encoding = enc`, calls `buf.save()`;
    /// on success updates the status bar; on failure reverts `buf.encoding`.
    ///
    /// Case B (unnamed buffer): handled in T020.
    pub fn do_save_as_encoding(&mut self, enc: EncodingId) {
        if self.active_buffer().path.is_some() {
            // Case A: named buffer — encode + atomic write.
            let old_enc = self.active_buffer().encoding;
            self.active_buffer_mut().encoding = enc;
            match self.active_buffer().save() {
                Ok(()) => {
                    self.active_buffer_mut().modified = false;
                    self.active_buffer_mut().undo_stack.mark_saved(); // Feature 014
                                                                      // Feature 007: record write time for self-write suppression.
                    if let Some(path) = self.active_buffer().path.clone() {
                        self.self_write_times.insert(path, Instant::now());
                    }
                    let label = Self::label_for_encoding(enc);
                    self.status_message = Some(format!("Saved as {}", label));
                }
                Err(e) => {
                    self.active_buffer_mut().encoding = old_enc;
                    self.status_message = Some(format!("Save failed: {}", e));
                }
            }
        } else {
            // Case B — unnamed buffer: store the chosen encoding so that the
            // next handle_save_as call (once the user provides a filename) will
            // write the file in the selected encoding.
            self.pending_save_as_encoding = Some(enc);
        }
    }

    pub(super) fn handle_resize(&mut self, w: u16, h: u16) {
        self.terminal_size = (w, h);

        // T105 — detect too-small terminal
        self.too_small = w < MIN_WIDTH || h < MIN_HEIGHT;

        // Rebuild wrap cache for new terminal width (Feature 005, T022).
        if self.active_buffer().soft_wrap {
            let content_w = self.content_width();
            let rope = &self.active_buffer().rope;
            self.wrap_cache = Some(crate::ui::wrap::WrapCache::compute(
                rope,
                content_w,
                self.wrap_text_gen,
            ));
            if let Some(ref cache) = self.wrap_cache {
                let total_vr = cache.total_visual_rows();
                let buf = self.active_buffer_mut();
                if buf.scroll_offset.0 >= total_vr {
                    buf.scroll_offset.0 = total_vr.saturating_sub(1);
                }
            }
        }

        // Re-clamp scroll offset so cursor stays visible after resize.
        self.clamp_scroll();
    }

    pub(super) fn handle_tick(&mut self) {
        // US5 — Autosave
        let interval = std::env::var("EDIT_AUTOSAVE_INTERVAL")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(self.config.autosave_interval)
            .clamp(1, 300);

        if !self.config.no_autosave {
            let buf = self.active_buffer();
            if buf.autosave.enabled && buf.modified {
                let elapsed = buf.autosave.last_save_at.elapsed().as_secs() as u32;
                if elapsed >= interval {
                    let ok = crate::buffer::autosave::write_recovery_for_buffer(
                        self.active_buffer_mut(),
                        interval,
                    );
                    // Feature 029: surface a recovery-write failure instead of
                    // losing crash protection silently.
                    if !ok {
                        self.status_message =
                            Some("Autosave failed — crash recovery may be unavailable".to_string());
                    }
                }
            }
        }

        // Feature 007 — drain the file-watcher event queue (non-blocking).
        // Only drain when no dialog is already pending (one prompt at a time).
        if self.pending_external_change.is_none() {
            if let Some(ref mut fw) = self.file_watcher {
                let watched_paths: Vec<PathBuf> =
                    self.buffers.iter().filter_map(|b| b.path.clone()).collect();
                if let Some(event) = fw.try_recv_event(&self.self_write_times, &watched_paths) {
                    match event.kind {
                        crate::watcher::WatchEventKind::Modified => {
                            // Find the buffer index whose path matches.
                            if let Some(buf_idx) = self
                                .buffers
                                .iter()
                                .position(|b| b.path.as_deref() == Some(event.path.as_path()))
                            {
                                self.pending_external_change =
                                    Some(crate::watcher::ExternalChange {
                                        buf_idx,
                                        path: event.path.clone(),
                                        kind: crate::watcher::WatchEventKind::Modified,
                                    });
                            }
                        }
                        crate::watcher::WatchEventKind::Deleted => {
                            let name = event
                                .path
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| event.path.display().to_string());
                            self.watcher_notice = Some(format!(
                                "[{}] File deleted on disk \u{2014} buffer kept in memory",
                                name
                            ));
                        }
                    }
                }
            }
        }
    }

    // ── Feature 007 — Reload from disk ───────────────────────────────────────

    /// Replace buffer at `buf_idx` with the current on-disk content.
    ///
    /// Uses `Buffer::open()` which runs the full encoding detection pipeline
    /// (FR-004 compliance: no raw-byte bypass).  Clears undo history.
    pub fn reload_from_disk(&mut self, buf_idx: usize) {
        let path = match self.buffers.get(buf_idx).and_then(|b| b.path.clone()) {
            Some(p) => p,
            None => return,
        };
        let enc = self.buffers[buf_idx].encoding;
        match Buffer::open(path.clone(), enc) {
            Ok(new_buf) => {
                self.buffers[buf_idx] = new_buf;
                log::info!("Buffer {} reloaded from {:?}", buf_idx, path);
            }
            Err(e) => {
                log::warn!("reload_from_disk failed for {:?}: {}", path, e);
                self.watcher_notice = Some(format!("Reload failed: {}", e));
            }
        }
    }
}
