//! Feature 005 — soft-wrap helpers (split from app.rs, Feature 041).

use super::*;

impl App {
    // ── Feature 005 — Soft-wrap helpers ──────────────────────────────────────

    /// Viewport content width: terminal columns minus the gutter (if line numbers
    /// on) and minus the editor's rightmost vertical-scrollbar column (Feature 021).
    pub(super) fn content_width(&self) -> u16 {
        let gutter: u16 = if self.config.line_numbers { 4 } else { 0 };
        self.active_pane_width()
            .saturating_sub(gutter)
            .saturating_sub(1)
    }

    /// Feature 048: terminal-column width of the ACTIVE editor pane — full width in
    /// single view, the active half in a vertical split (left = `w/2`, right = the
    /// remainder). Wrap geometry must use the pane width, not the full terminal.
    fn active_pane_width(&self) -> u16 {
        let full = self.terminal_size.0;
        match self.split_mode {
            crate::ui::SplitMode::Single => full,
            crate::ui::SplitMode::Vertical => {
                let half = full / 2;
                if self.active_idx == 0 {
                    half
                } else {
                    full - half
                }
            }
        }
    }

    /// Feature 048: content width of the NON-active visible pane in a vertical split
    /// (the sibling half minus gutter + scrollbar). Used to build `wrap_cache_alt`.
    pub(super) fn alt_pane_content_width(&self) -> u16 {
        let gutter: u16 = if self.config.line_numbers { 4 } else { 0 };
        let full = self.terminal_size.0;
        let half = full / 2;
        let alt_w = if self.active_idx == 0 {
            full - half
        } else {
            half
        };
        alt_w.saturating_sub(gutter).saturating_sub(1)
    }

    /// Feature 048: the buffer index shown in the NON-active visible pane of a
    /// vertical split, if any (left pane = `buffers[0]`, right pane =
    /// `buffers[active_idx.max(1)]`). `None` in single view or with one buffer.
    pub(super) fn alt_visible_buffer(&self) -> Option<usize> {
        if !matches!(self.split_mode, crate::ui::SplitMode::Vertical) || self.buffers.len() < 2 {
            return None;
        }
        if self.active_idx == 0 {
            Some(self.active_idx.max(1)) // right pane
        } else {
            Some(0) // left pane
        }
    }

    /// Feature 048: wrap visual-starts for a *visible* pane's buffer — the active
    /// buffer uses the primary cache, the non-active visible pane uses the alt
    /// cache, anything else has none. Lets the renderer wrap each pane correctly.
    pub(crate) fn pane_wrap_starts(&self, buf_idx: usize) -> Option<&[Vec<u32>]> {
        if buf_idx == self.active_idx {
            self.wrap_cache.as_ref().map(|c| c.visual_starts.as_slice())
        } else if self.wrap_alt_for == Some(buf_idx) {
            self.wrap_cache_alt
                .as_ref()
                .map(|c| c.visual_starts.as_slice())
        } else {
            None
        }
    }

    /// Feature 048: (re)compute the wrap caches for the visible pane(s) — the active
    /// buffer's `wrap_cache` at the active pane width, and (in a vertical split) the
    /// non-active visible pane's `wrap_cache_alt` at its own width. Called each frame
    /// before render (and by tests). Each is rebuilt only when stale.
    pub(super) fn refresh_wrap_caches(&mut self) {
        if self.active_buffer().soft_wrap {
            let w = self.content_width();
            if self
                .wrap_cache
                .as_ref()
                .map(|c| c.is_stale(w, self.wrap_text_gen))
                .unwrap_or(true)
            {
                self.wrap_cache = Some(crate::ui::wrap::WrapCache::compute(
                    &self.active_buffer().rope,
                    w,
                    self.wrap_text_gen,
                ));
            }
        }
        match self.alt_visible_buffer() {
            Some(i) if self.buffers[i].soft_wrap => {
                let w = self.alt_pane_content_width();
                let stale = self.wrap_alt_for != Some(i)
                    || self
                        .wrap_cache_alt
                        .as_ref()
                        .map(|c| c.is_stale(w, self.wrap_text_gen))
                        .unwrap_or(true);
                if stale {
                    self.wrap_cache_alt = Some(crate::ui::wrap::WrapCache::compute(
                        &self.buffers[i].rope,
                        w,
                        self.wrap_text_gen,
                    ));
                    self.wrap_alt_for = Some(i);
                }
            }
            _ => {
                self.wrap_cache_alt = None;
                self.wrap_alt_for = None;
            }
        }
    }

    /// Compute the global visual row index for the cursor position (using wrap cache).
    pub(super) fn cursor_visual_row(&self) -> usize {
        let cache = match self.wrap_cache.as_ref() {
            Some(c) => c,
            None => return self.active_buffer().cursor.line,
        };
        let cursor = self.active_buffer().cursor;
        let line_str = self.active_buffer().rope.line_slice(cursor.line);
        let cursor_byte: usize = line_str
            .graphemes(true)
            .take(cursor.grapheme_col)
            .map(|g| g.len())
            .sum();
        let starts = match cache.visual_starts.get(cursor.line) {
            Some(s) => s,
            None => return 0,
        };
        let seg_idx = starts
            .partition_point(|&s| (s as usize) <= cursor_byte)
            .saturating_sub(1);
        let rows_before: usize = (0..cursor.line)
            .map(|l| cache.visual_starts.get(l).map(|v| v.len()).unwrap_or(1))
            .sum();
        rows_before + seg_idx
    }

    /// Persist `self.config` to `$XDG_CONFIG_HOME/edit/config.toml` using
    /// an atomic tmp-rename. On failure: log warn, set status message, do not revert.
    pub(super) fn save_config_to_disk(&mut self) {
        let config_path = crate::config::config_path();
        let tmp_path = config_path.with_extension("toml.tmp");

        let toml_str = match toml::to_string(&self.config) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Config serialize failed: {}", e);
                self.status_message = Some(format!("Config save failed: {}", e));
                return;
            }
        };

        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Err(e) = std::fs::write(&tmp_path, &toml_str) {
            log::warn!("Config write failed: {}", e);
            self.status_message = Some(format!("Config save failed: {}", e));
            return;
        }

        if let Err(e) = std::fs::rename(&tmp_path, &config_path) {
            log::warn!("Config rename failed: {}", e);
            self.status_message = Some(format!("Config save failed: {}", e));
        }
    }

    /// Toggle soft-wrap for the **active tab** (Feature 044: wrap is per-buffer).
    /// Handles the width guard, the active-buffer cache rebuild/drop, and the
    /// active buffer's horizontal-scroll reset. Does not touch other tabs and no
    /// longer rewrites `config.soft_wrap` (config is only the default seed).
    pub(super) fn handle_toggle_soft_wrap(&mut self) -> io::Result<()> {
        // Width guard (only applied when turning ON).
        let content_w = self.content_width();
        if !self.active_buffer().soft_wrap && content_w < 10 {
            self.status_message =
                Some("Terminal too narrow for soft wrap (min 10 columns)".to_string());
            return Ok(());
        }

        let now_on = !self.active_buffer().soft_wrap;
        self.active_buffer_mut().soft_wrap = now_on;
        // Reset the active tab's horizontal scroll either way (wrap on: no h-scroll;
        // wrap off: start from column 0). Other tabs are untouched.
        self.active_buffer_mut().scroll_offset.1 = 0;

        if now_on {
            let rope = &self.active_buffer().rope;
            self.wrap_cache = Some(crate::ui::wrap::WrapCache::compute(
                rope,
                content_w,
                self.wrap_text_gen,
            ));
        } else {
            self.wrap_cache = None;
        }

        Ok(())
    }
}
