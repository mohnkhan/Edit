//! Split from app.rs (Feature 041): actions.

use super::*;

impl App {
    // ── T103 — SaveAs action ─────────────────────────────────────────────────

    /// Save the active buffer to a new path and update buffer.path.
    ///
    /// If `pending_save_as_encoding` is set (meaning the user selected an
    /// encoding via the encoding dialog before typing a filename), that
    /// encoding is applied to the buffer before writing and then cleared.
    pub fn handle_save_as(
        &mut self,
        new_path: std::path::PathBuf,
    ) -> Result<(), crate::buffer::BufferError> {
        if let Some(enc) = self.pending_save_as_encoding.take() {
            self.active_buffer_mut().encoding = enc;
        }
        self.active_buffer_mut().save_as(new_path)
    }

    /// Discard a pending encoding selection (called when the user cancels
    /// the filename-input dialog that follows encoding selection).
    pub fn cancel_pending_save_as_encoding(&mut self) {
        self.pending_save_as_encoding = None;
    }

    // ── T066 — Next / previous buffer ────────────────────────────────────────

    /// Cycle forward to the next open buffer, wrapping around.
    /// Feature 028: invalidate the soft-wrap cache so the next frame rebuilds it
    /// for the now-active buffer. The cache stores per-line visual byte offsets for
    /// ONE buffer's content at a given `wrap_text_gen`; whenever the active buffer's
    /// content identity changes (switch / open / close / session restore) the cache
    /// must be considered stale, or the renderer would slice the new buffer's lines
    /// with the old buffer's offsets (the session-restore crash). Bumping the
    /// generation makes `WrapCache::is_stale` true on the next render-loop check.
    pub(super) fn invalidate_wrap_cache(&mut self) {
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);
    }

    pub fn next_buffer(&mut self) {
        if self.buffers.len() <= 1 {
            return;
        }
        self.active_idx = (self.active_idx + 1) % self.buffers.len();
        self.invalidate_wrap_cache();
        self.clamp_scroll();
    }

    /// Cycle backward to the previous open buffer, wrapping around.
    pub fn prev_buffer(&mut self) {
        if self.buffers.len() <= 1 {
            return;
        }
        self.active_idx = if self.active_idx == 0 {
            self.buffers.len() - 1
        } else {
            self.active_idx - 1
        };
        self.invalidate_wrap_cache();
        self.clamp_scroll();
    }

    // ── T069 — Open file into new buffer ─────────────────────────────────────

    /// Open `path` as a new buffer and make it the active buffer.
    ///
    /// Uses [`crate::security::sanitize::validate_path`] to guard against
    /// path-traversal attacks.  On success the new buffer is appended to
    /// `self.buffers` and `active_idx` is updated to point at it.
    pub fn handle_open_file(&mut self, path: PathBuf) {
        use crate::security::sanitize::validate_path;

        let safe_path = match validate_path(&path) {
            Ok(p) => p,
            Err(e) => {
                // Feature 029: surface rejected paths instead of failing silently.
                log::warn!("handle_open_file: path rejected ({:?}): {}", path, e);
                self.status_message = Some(format!("Open failed: {}", path.display()));
                return;
            }
        };

        let default_encoding = crate::encoding::encoding_from_str(&self.config.default_encoding);

        match Buffer::open(safe_path.clone(), default_encoding) {
            Ok(buf) => {
                // Feature 007: watch the newly-opened file.
                if let Some(ref mut fw) = self.file_watcher {
                    if let Err(e) = fw.watch_path(&safe_path) {
                        log::warn!("FileWatcher: could not watch {:?}: {}", safe_path, e);
                    }
                }
                self.buffers.push(buf);
                self.active_idx = self.buffers.len() - 1;
                self.invalidate_wrap_cache();
                let name = safe_path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| safe_path.display().to_string());
                self.status_message = Some(format!("Opened {name}"));
                log::info!("Opened {:?} as buffer {}", safe_path, self.active_idx);
            }
            Err(e) => {
                // Feature 029: surface the failure (path + reason) instead of a
                // silent no-op that leaves the user wondering why nothing opened.
                log::error!("handle_open_file: failed to open {:?}: {}", safe_path, e);
                self.status_message = Some(format!("Open failed: {} — {}", safe_path.display(), e));
            }
        }
    }

    // ── Feature 012 — File browser ─────────────────────────────────────────────

    /// Directory the file browser should open at: the active buffer's parent
    /// directory if it has a path, else the process current directory.
    pub fn browser_start_dir(&self) -> PathBuf {
        if let Some(parent) = self
            .buffers
            .get(self.active_idx)
            .and_then(|b| b.path.as_ref())
            .and_then(|p| p.parent())
        {
            return parent.to_path_buf();
        }
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    /// Apply the result of a browser activation: open/save the chosen file and
    /// close the browser, or leave it open to keep navigating.
    pub fn apply_browse_outcome(&mut self, outcome: BrowseOutcome) {
        match outcome {
            BrowseOutcome::Navigated | BrowseOutcome::None => {}
            BrowseOutcome::OpenFile(path) => {
                self.close_modal();
                self.handle_open_file(path);
            }
            BrowseOutcome::SaveFile(path) => {
                self.close_modal();
                self.do_save_as(path);
            }
        }
    }

    // ── Feature 011 — File / Edit menu actions ────────────────────────────────

    /// Open a fresh empty buffer and make it active (File ▸ New / Ctrl+N).
    pub fn new_buffer(&mut self) {
        self.buffers.push(Buffer::new_empty());
        self.active_idx = self.buffers.len() - 1;
    }

    /// Close the active buffer (File ▸ Close). The sole buffer is replaced by a
    /// fresh empty one so the editor always has something to edit.
    pub fn close_active_buffer(&mut self) {
        self.close_buffer_at(self.active_idx);
    }

    /// Close the buffer at `idx`, adjusting `active_idx` so the same logical
    /// buffer stays active where possible (Feature 027). Closing the last
    /// remaining buffer replaces it with an empty scratch buffer. No prompt — the
    /// caller is responsible for any unsaved-changes confirmation.
    pub fn close_buffer_at(&mut self, idx: usize) {
        if idx >= self.buffers.len() {
            return;
        }
        if self.buffers.len() <= 1 {
            self.buffers[0] = Buffer::new_empty();
            self.active_idx = 0;
            self.invalidate_wrap_cache();
            self.clamp_scroll();
            return;
        }
        self.buffers.remove(idx);
        if self.active_idx > idx {
            self.active_idx -= 1;
        } else if self.active_idx >= self.buffers.len() {
            self.active_idx = self.buffers.len() - 1;
        }
        self.invalidate_wrap_cache();
        self.clamp_scroll();
    }

    /// Feature 027: a tab's `[x]` was clicked. A clean buffer closes immediately;
    /// a modified one opens the [`ButtonDialog::CloseConfirm`] modal (no silent
    /// data loss).
    pub fn tab_close_clicked(&mut self, idx: usize) {
        if idx >= self.buffers.len() {
            return;
        }
        if self.buffers[idx].modified {
            self.modal = Modal::CloseConfirm(idx);
        } else {
            self.close_buffer_at(idx);
        }
    }
}
