//! Feature 005 — soft-wrap helpers (split from app.rs, Feature 041).

use super::*;

impl App {
    // ── Feature 005 — Soft-wrap helpers ──────────────────────────────────────

    /// Viewport content width: terminal columns minus the gutter (if line numbers
    /// on) and minus the editor's rightmost vertical-scrollbar column (Feature 021).
    pub(super) fn content_width(&self) -> u16 {
        let gutter: u16 = if self.config.line_numbers { 4 } else { 0 };
        self.terminal_size
            .0
            .saturating_sub(gutter)
            .saturating_sub(1)
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

    /// Toggle soft-wrap on or off. Handles the width guard, cache rebuild/drop,
    /// horizontal-scroll reset, and config persistence.
    pub(super) fn handle_toggle_soft_wrap(&mut self) -> io::Result<()> {
        // Width guard (only applied when turning ON).
        let content_w = self.content_width();
        if !self.soft_wrap && content_w < 10 {
            self.status_message =
                Some("Terminal too narrow for soft wrap (min 10 columns)".to_string());
            return Ok(());
        }

        self.soft_wrap = !self.soft_wrap;
        self.config.soft_wrap = self.soft_wrap;

        if self.soft_wrap {
            // Build cache; reset horizontal scroll on all buffers.
            let rope = &self.active_buffer().rope;
            self.wrap_cache = Some(crate::ui::wrap::WrapCache::compute(
                rope,
                content_w,
                self.wrap_text_gen,
            ));
            for buf in &mut self.buffers {
                buf.scroll_offset.1 = 0;
            }
        } else {
            // Drop cache; reset horizontal scroll for all buffers.
            self.wrap_cache = None;
            for buf in &mut self.buffers {
                buf.scroll_offset.1 = 0;
            }
        }

        self.save_config_to_disk();
        Ok(())
    }
}
