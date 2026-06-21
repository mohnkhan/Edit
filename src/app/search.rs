//! Split from app.rs (Feature 041): search.

use super::*;

impl App {
    // ── T055 — Find next / find prev ─────────────────────────────────────────

    // ── Feature 015 — interactive Find / Replace dialog ────────────────────────

    /// Open the Find dialog (Ctrl+F / Search ▸ Find), seeded with the last query.
    pub fn open_find_dialog(&mut self) {
        let seed = self.search_state.query.clone();
        self.modal = Modal::FindReplace(FindReplaceDialog::new(DialogMode::Find, seed));
    }

    /// Open the Replace dialog (Ctrl+H / Search ▸ Find Replace).
    pub fn open_replace_dialog(&mut self) {
        let seed = self.search_state.query.clone();
        let mut d = FindReplaceDialog::new(DialogMode::Replace, seed);
        if let Some(r) = &self.search_state.replacement {
            d.replacement = r.clone();
        }
        self.modal = Modal::FindReplace(d);
    }

    /// Close the Find/Replace dialog and clear the active match highlights.
    pub fn close_find_replace(&mut self) {
        self.close_modal();
        self.search_state.matches.clear();
        self.search_state.active_match = None;
    }

    /// Char index of the cursor in the active buffer (for "first match at/after
    /// the cursor").
    pub(super) fn cursor_char_index(&self) -> usize {
        let buf = self.active_buffer();
        let text = buf.rope.to_string();
        let mut char_count = 0usize;
        for (line_idx, line_text) in text.split('\n').enumerate() {
            if line_idx == buf.cursor.line {
                let gcols = unicode_segmentation::UnicodeSegmentation::graphemes(line_text, true)
                    .take(buf.cursor.grapheme_col)
                    .map(|g| g.chars().count())
                    .sum::<usize>();
                return char_count + gcols;
            }
            char_count += line_text.chars().count() + 1; // + newline
        }
        char_count
    }

    /// Copy the dialog's query/options into `search_state`, run the search,
    /// highlight matches, and jump to the first match at/after the cursor.
    pub(super) fn run_find_from_dialog(&mut self) {
        let (query, case, regex, whole, wrap, replacement) = {
            let d = match self.find_replace() {
                Some(d) => d,
                None => return,
            };
            (
                d.query.clone(),
                d.case_sensitive,
                d.regex,
                d.whole_word,
                d.wrap,
                d.replacement.clone(),
            )
        };
        self.search_state.query = query.clone();
        self.search_state.case_sensitive = case;
        self.search_state.regex_mode = regex;
        self.search_state.whole_word = whole;
        self.search_state.wrap = wrap;
        self.search_state.replacement = Some(replacement);

        if query.is_empty() {
            self.search_state.matches.clear();
            self.search_state.active_match = None;
            return;
        }

        let matches = {
            let rope = &self.active_buffer().rope;
            SearchEngine::find_all(rope, &query, regex, case, whole)
        };
        let total = matches.len();
        self.search_state.matches = matches;
        if total == 0 {
            self.search_state.active_match = None;
            self.status_message = Some("Not found".to_string());
            return;
        }
        let cursor_char = self.cursor_char_index();
        let idx = self
            .search_state
            .matches
            .iter()
            .position(|m| m.start >= cursor_char)
            .unwrap_or(0);
        self.search_state.active_match = Some(idx);
        self.status_message = Some(format!("Match {}/{}", idx + 1, total));
        self.scroll_to_match(idx);
    }

    /// Replace the current match with the dialog's replacement, then recompute
    /// matches and advance to the next (Enter in Replace mode).
    pub(super) fn replace_current_from_dialog(&mut self) {
        if self.active_buffer().readonly {
            self.status_message = Some("Buffer is read-only".to_string());
            return;
        }
        // Ensure search state reflects the dialog and we have matches.
        if self.search_state.matches.is_empty() || self.search_state.active_match.is_none() {
            self.run_find_from_dialog();
        }
        let replacement = self
            .find_replace()
            .map(|d| d.replacement.clone())
            .unwrap_or_default();
        let idx = match self.search_state.active_match {
            Some(i) => i,
            None => return,
        };
        let range = match self.search_state.matches.get(idx).cloned() {
            Some(r) => r,
            None => return,
        };

        // Capture the deleted text for undo.
        let deleted: String = {
            let full = self.active_buffer().rope.to_string();
            let bs = full
                .char_indices()
                .nth(range.start)
                .map(|(b, _)| b)
                .unwrap_or(full.len());
            let be = full
                .char_indices()
                .nth(range.end)
                .map(|(b, _)| b)
                .unwrap_or(full.len());
            full[bs..be].to_string()
        };
        {
            let buf = self.active_buffer_mut();
            buf.rope.delete_range(range.start..range.end);
            buf.rope.insert_str(range.start, &replacement);
            buf.undo_stack.push(EditOp::Composite(vec![
                EditOp::Delete {
                    at: range.start,
                    text: deleted,
                },
                EditOp::Insert {
                    at: range.start,
                    text: replacement.clone(),
                },
            ]));
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);

        // Recompute matches against the edited document (no stale offsets).
        let (query, case, regex, whole) = (
            self.search_state.query.clone(),
            self.search_state.case_sensitive,
            self.search_state.regex_mode,
            self.search_state.whole_word,
        );
        let matches = {
            let rope = &self.active_buffer().rope;
            SearchEngine::find_all(rope, &query, regex, case, whole)
        };
        let total = matches.len();
        self.search_state.matches = matches;
        if total == 0 {
            self.search_state.active_match = None;
            self.status_message = Some("Replaced 1 — no more matches".to_string());
            return;
        }
        let next = idx.min(total - 1);
        self.search_state.active_match = Some(next);
        self.status_message = Some(format!("Replaced 1 — Match {}/{}", next + 1, total));
        self.scroll_to_match(next);
    }

    /// Replace all occurrences (Ctrl+A in Replace mode), reporting the count.
    pub(super) fn replace_all_from_dialog(&mut self) {
        // Sync the dialog query/replacement/options into search_state first.
        self.sync_search_state_from_find_replace();
        self.replace_all();
    }

    /// Copy the open Find/Replace dialog's query/options into `search_state`.
    /// (Values are cloned out first so `self.search_state` can be mutated without
    /// holding a borrow of the dialog now nested inside `self.modal` — Feature 039.)
    pub(super) fn sync_search_state_from_find_replace(&mut self) {
        if let Some(d) = self.find_replace() {
            let (query, replacement, case_sensitive, regex, whole_word, wrap) = (
                d.query.clone(),
                d.replacement.clone(),
                d.case_sensitive,
                d.regex,
                d.whole_word,
                d.wrap,
            );
            self.search_state.query = query;
            self.search_state.replacement = Some(replacement);
            self.search_state.case_sensitive = case_sensitive;
            self.search_state.regex_mode = regex;
            self.search_state.whole_word = whole_word;
            self.search_state.wrap = wrap;
        }
    }

    /// Jump to the next search match, wrapping at end-of-document when
    /// `search_state.wrap` is `true`.
    ///
    /// If `search_state.matches` is empty the engine is re-run first.  Sets
    /// `status_message` to reflect the result ("Match N/M", "Not found", or
    /// "Search wrapped").
    pub fn find_next(&mut self) {
        // Re-run the search if there are no cached results.
        if self.search_state.matches.is_empty() {
            let query = self.search_state.query.clone();
            let regex_mode = self.search_state.regex_mode;
            let case_sensitive = self.search_state.case_sensitive;
            let whole_word = self.search_state.whole_word;
            let rope = &self.active_buffer().rope;
            self.search_state.matches =
                SearchEngine::find_all(rope, &query, regex_mode, case_sensitive, whole_word);
        }

        let total = self.search_state.matches.len();
        if total == 0 {
            self.status_message = Some("Not found".to_string());
            self.search_state.active_match = None;
            return;
        }

        let next_idx = match self.search_state.active_match {
            None => 0,
            Some(cur) => {
                if cur + 1 < total {
                    cur + 1
                } else if self.search_state.wrap {
                    // Set a "wrapped" message when going past the last match.
                    self.status_message = Some(format!("Search wrapped — Match 1/{}", total));
                    self.search_state.active_match = Some(0);
                    self.scroll_to_match(0);
                    return;
                } else {
                    self.status_message = Some(format!("Match {}/{} (end)", total, total));
                    return;
                }
            }
        };

        self.search_state.active_match = Some(next_idx);
        self.status_message = Some(format!("Match {}/{}", next_idx + 1, total));
        self.scroll_to_match(next_idx);
    }

    /// Jump to the previous search match, wrapping at start-of-document when
    /// `search_state.wrap` is `true`.
    pub fn find_prev(&mut self) {
        // Re-run the search if there are no cached results.
        if self.search_state.matches.is_empty() {
            let query = self.search_state.query.clone();
            let regex_mode = self.search_state.regex_mode;
            let case_sensitive = self.search_state.case_sensitive;
            let whole_word = self.search_state.whole_word;
            let rope = &self.active_buffer().rope;
            self.search_state.matches =
                SearchEngine::find_all(rope, &query, regex_mode, case_sensitive, whole_word);
        }

        let total = self.search_state.matches.len();
        if total == 0 {
            self.status_message = Some("Not found".to_string());
            self.search_state.active_match = None;
            return;
        }

        let prev_idx = match self.search_state.active_match {
            None => total - 1,
            Some(0) => {
                if self.search_state.wrap {
                    self.status_message =
                        Some(format!("Search wrapped — Match {}/{}", total, total));
                    self.search_state.active_match = Some(total - 1);
                    self.scroll_to_match(total - 1);
                    return;
                } else {
                    self.status_message = Some(format!("Match 1/{} (start)", total));
                    return;
                }
            }
            Some(cur) => cur - 1,
        };

        self.search_state.active_match = Some(prev_idx);
        self.status_message = Some(format!("Match {}/{}", prev_idx + 1, total));
        self.scroll_to_match(prev_idx);
    }

    /// Scroll the viewport and reposition the cursor to the match at `idx`.
    pub(super) fn scroll_to_match(&mut self, idx: usize) {
        let char_start = match self.search_state.matches.get(idx) {
            Some(r) => r.start,
            None => return,
        };

        // Convert char index → (line, grapheme_col) by walking the rope.
        let text = self.active_buffer().rope.to_string();
        let mut char_count = 0usize;
        let mut target_line = 0usize;
        let mut target_gcol = 0usize;

        'outer: for (line_idx, line_text) in text.split('\n').enumerate() {
            let graphemes: Vec<&str> =
                unicode_segmentation::UnicodeSegmentation::graphemes(line_text, true).collect();
            for (gcol, g) in graphemes.iter().enumerate() {
                if char_count == char_start {
                    target_line = line_idx;
                    target_gcol = gcol;
                    break 'outer;
                }
                char_count += g.chars().count();
            }
            // Account for the '\n' character between lines.
            if char_count == char_start {
                target_line = line_idx;
                target_gcol = graphemes.len();
                break 'outer;
            }
            char_count += 1; // for '\n'
        }

        let new_vcol = CursorPos::visual_col_from_grapheme_col(
            &self.active_buffer().rope,
            target_line,
            target_gcol,
        );

        let buf = self.active_buffer_mut();
        buf.cursor = CursorPos {
            line: target_line,
            grapheme_col: target_gcol,
            visual_col: new_vcol,
        };
        self.clamp_scroll();
    }

    // ── T057 — Replace All ────────────────────────────────────────────────────

    /// Replace every occurrence of the current search query with the
    /// replacement string.
    ///
    /// Replacements are applied in **reverse** document order so that earlier
    /// char indices remain valid while later ones are being modified.  All
    /// individual delete+insert pairs are wrapped in a single
    /// `EditOp::Composite` so the entire replace-all is one undo step.
    ///
    /// Sets `status_message` to "Replaced N occurrences" (or "No matches").
    pub fn replace_all(&mut self) {
        if self.active_buffer().readonly {
            self.status_message = Some("Buffer is read-only".to_string());
            return;
        }

        let replacement = match &self.search_state.replacement {
            Some(r) => r.clone(),
            None => {
                self.status_message = Some("No replacement string set".to_string());
                return;
            }
        };

        // Re-run find to get fresh positions.
        {
            let query = self.search_state.query.clone();
            let regex_mode = self.search_state.regex_mode;
            let case_sensitive = self.search_state.case_sensitive;
            let whole_word = self.search_state.whole_word;
            let rope = &self.active_buffer().rope;
            self.search_state.matches =
                SearchEngine::find_all(rope, &query, regex_mode, case_sensitive, whole_word);
        }

        let matches = self.search_state.matches.clone();
        if matches.is_empty() {
            self.status_message = Some("No matches".to_string());
            return;
        }

        let count = matches.len();
        let mut ops: Vec<EditOp> = Vec::with_capacity(count * 2);

        // Apply in reverse order (last match first) to keep earlier indices stable.
        for m in matches.iter().rev() {
            let del_len = m.end - m.start;
            // Capture the text that will be deleted for undo.
            let deleted_text: String = {
                let buf = self.active_buffer();
                let full = buf.rope.to_string();
                // Convert char indices to byte indices for slicing.
                let byte_start = full
                    .char_indices()
                    .nth(m.start)
                    .map(|(b, _)| b)
                    .unwrap_or(full.len());
                let byte_end = full
                    .char_indices()
                    .nth(m.end)
                    .map(|(b, _)| b)
                    .unwrap_or(full.len());
                full[byte_start..byte_end].to_string()
            };

            // Apply the deletion.
            {
                let buf = self.active_buffer_mut();
                buf.rope.delete_range(m.start..m.end);
            }
            ops.push(EditOp::Delete {
                at: m.start,
                text: deleted_text,
            });

            // Apply the insertion.
            {
                let buf = self.active_buffer_mut();
                buf.rope.insert_str(m.start, &replacement);
            }
            ops.push(EditOp::Insert {
                at: m.start,
                text: replacement.clone(),
            });
        }

        // Push all ops as one composite undo entry.
        {
            let buf = self.active_buffer_mut();
            buf.undo_stack.push(EditOp::Composite(ops));
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);

        // Invalidate the cached match positions — they are no longer valid.
        self.search_state.matches.clear();
        self.search_state.active_match = None;

        self.status_message = Some(format!(
            "Replaced {} occurrence{}",
            count,
            if count == 1 { "" } else { "s" }
        ));
    }
}
