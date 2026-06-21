//! Split from app.rs (Feature 041): editing.

use super::*;

impl App {
    // ── T025 — Cursor movement ────────────────────────────────────────────────

    /// Move the cursor one step in `dir`, clamping to valid positions and
    /// updating `scroll_offset` as necessary.
    /// Compute the cursor position one step in `dir` (no mutation, no selection).
    pub(super) fn next_cursor_pos(&self, dir: Direction) -> (usize, usize) {
        let buf = self.active_buffer();
        let line_count = buf.rope.line_count();
        let cur = buf.cursor;
        match dir {
            Direction::Up => {
                if cur.line == 0 {
                    (0, cur.grapheme_col)
                } else {
                    let target_line = cur.line - 1;
                    let max_gcol = buf.rope.grapheme_count_on_line(target_line);
                    (target_line, cur.grapheme_col.min(max_gcol))
                }
            }
            Direction::Down => {
                if cur.line + 1 >= line_count {
                    (cur.line, cur.grapheme_col)
                } else {
                    let target_line = cur.line + 1;
                    let max_gcol = buf.rope.grapheme_count_on_line(target_line);
                    (target_line, cur.grapheme_col.min(max_gcol))
                }
            }
            Direction::Left => {
                if cur.grapheme_col > 0 {
                    (cur.line, cur.grapheme_col - 1)
                } else if cur.line > 0 {
                    // Wrap to end of previous line
                    let prev = cur.line - 1;
                    let prev_len = buf.rope.grapheme_count_on_line(prev);
                    (prev, prev_len)
                } else {
                    (0, 0)
                }
            }
            Direction::Right => {
                let line_len = buf.rope.grapheme_count_on_line(cur.line);
                if cur.grapheme_col < line_len {
                    (cur.line, cur.grapheme_col + 1)
                } else if cur.line + 1 < line_count {
                    // Wrap to start of next line
                    (cur.line + 1, 0)
                } else {
                    (cur.line, cur.grapheme_col)
                }
            }
        }
    }

    /// Set the cursor to `(line, gcol)` (computing its visual column) and clamp
    /// the viewport. Does not touch the selection.
    pub(super) fn set_cursor_lc(&mut self, line: usize, gcol: usize) {
        let vcol = CursorPos::visual_col_from_grapheme_col(&self.active_buffer().rope, line, gcol);
        self.active_buffer_mut().cursor = CursorPos {
            line,
            grapheme_col: gcol,
            visual_col: vcol,
        };
        self.clamp_scroll();
    }

    pub fn move_cursor(&mut self, dir: Direction) {
        let (new_line, new_gcol) = self.next_cursor_pos(dir);
        // Feature 017: a plain (non-shift) move clears any selection.
        self.active_buffer_mut().selection = None;
        self.set_cursor_lc(new_line, new_gcol);
    }

    /// Feature 017: move the cursor one step in `dir` while extending the
    /// selection from its anchor (Shift+Arrow).
    pub fn move_cursor_selecting(&mut self, dir: Direction) {
        let anchor = self.selection_anchor_or_cursor();
        let (new_line, new_gcol) = self.next_cursor_pos(dir);
        self.set_cursor_lc(new_line, new_gcol);
        self.update_selection_to_cursor(anchor);
    }

    // ── Feature 032 — word-wise movement / selection / deletion ───────────────

    /// Compute the word-target `(line, grapheme_col)` one word step in `dir` from
    /// the cursor, using `grapheme_class` (shared with double-click, feature 030).
    /// Right = start of the next token (consume the current run + following
    /// whitespace); Left = start of the preceding token (consume preceding
    /// whitespace + that token's run). Crosses line boundaries; a buffer end
    /// returns the cursor unchanged (no-op).
    pub(super) fn next_word_pos(&self, dir: Direction) -> (usize, usize) {
        let buf = self.active_buffer();
        let line = buf.cursor.line;
        let gcol = buf.cursor.grapheme_col;
        let graphemes: Vec<String> = buf
            .rope
            .line_slice(line)
            .graphemes(true)
            .map(|s| s.to_string())
            .collect();
        let len = graphemes.len();
        match dir {
            Direction::Right => {
                if gcol >= len {
                    if line + 1 < buf.rope.line_count() {
                        (line + 1, 0)
                    } else {
                        (line, len)
                    }
                } else {
                    let mut i = gcol;
                    let start_class = Self::grapheme_class(&graphemes[i]);
                    while i < len && Self::grapheme_class(&graphemes[i]) == start_class {
                        i += 1;
                    }
                    // Skip a following whitespace run → start of the next token.
                    while i < len && Self::grapheme_class(&graphemes[i]) == 1 {
                        i += 1;
                    }
                    (line, i)
                }
            }
            Direction::Left => {
                if gcol == 0 {
                    if line > 0 {
                        (line - 1, buf.rope.grapheme_count_on_line(line - 1))
                    } else {
                        (line, 0)
                    }
                } else {
                    let mut i = gcol - 1;
                    // Skip a preceding whitespace run.
                    while i > 0 && Self::grapheme_class(&graphemes[i]) == 1 {
                        i -= 1;
                    }
                    // Consume the preceding token run → its start.
                    let tok_class = Self::grapheme_class(&graphemes[i]);
                    while i > 0 && Self::grapheme_class(&graphemes[i - 1]) == tok_class {
                        i -= 1;
                    }
                    (line, i)
                }
            }
            _ => (line, gcol),
        }
    }

    /// Move the cursor one word in `dir` (clears any selection). (US1)
    pub fn move_word(&mut self, dir: Direction) {
        let (l, g) = self.next_word_pos(dir);
        self.active_buffer_mut().selection = None;
        self.set_cursor_lc(l, g);
    }

    /// Extend the selection one word in `dir` (Ctrl+Shift+Arrow). (US2)
    pub fn move_word_selecting(&mut self, dir: Direction) {
        let anchor = self.selection_anchor_or_cursor();
        let (l, g) = self.next_word_pos(dir);
        self.set_cursor_lc(l, g);
        self.update_selection_to_cursor(anchor);
    }

    /// Delete one word in `dir` as a single undo step (Ctrl+Backspace/Delete). With
    /// an active selection, deletes the selection instead. (US3)
    pub fn delete_word(&mut self, dir: Direction) {
        if self.deny_if_readonly() {
            return;
        }
        if self.active_buffer().selection.is_some() {
            self.delete_selection();
            return;
        }
        let cursor = self.active_buffer().cursor;
        let (l, g) = self.next_word_pos(dir);
        if (l, g) == (cursor.line, cursor.grapheme_col) {
            return; // buffer end — nothing to delete
        }
        // Span cursor↔target as a selection, then reuse the char-safe,
        // single-undo-step delete path (which also places the cursor at the start).
        let vcol = CursorPos::visual_col_from_grapheme_col(&self.active_buffer().rope, l, g);
        let target = CursorPos {
            line: l,
            grapheme_col: g,
            visual_col: vcol,
        };
        self.active_buffer_mut().selection = Some(Selection {
            anchor: cursor,
            active: target,
        });
        self.delete_selection();
    }

    /// The current selection's anchor, or the cursor position if there is none.
    pub(super) fn selection_anchor_or_cursor(&self) -> CursorPos {
        let buf = self.active_buffer();
        buf.selection.map(|s| s.anchor).unwrap_or(buf.cursor)
    }

    /// Set `selection` to span `anchor`→cursor, or `None` if empty (Feature 017).
    pub(super) fn update_selection_to_cursor(&mut self, anchor: CursorPos) {
        let buf = self.active_buffer_mut();
        let active = buf.cursor;
        buf.selection = if active.line == anchor.line && active.grapheme_col == anchor.grapheme_col
        {
            None
        } else {
            Some(Selection { anchor, active })
        };
    }

    /// Move the cursor to column 0 of the current line.
    pub fn move_line_start(&mut self) {
        self.active_buffer_mut().selection = None; // Feature 017: plain move clears
        let buf = self.active_buffer_mut();
        buf.cursor.grapheme_col = 0;
        buf.cursor.visual_col = 0;
        self.clamp_scroll();
    }

    /// Move the cursor to the last grapheme of the current line.
    pub fn move_line_end(&mut self) {
        self.active_buffer_mut().selection = None; // Feature 017: plain move clears
        self.cursor_to_line_end();
    }

    /// Place the cursor at the end of its line (no selection change).
    pub(super) fn cursor_to_line_end(&mut self) {
        let line = self.active_buffer().cursor.line;
        let gcol = self.active_buffer().rope.grapheme_count_on_line(line);
        self.set_cursor_lc(line, gcol);
    }

    /// Feature 017: extend the selection to the start of the current line.
    pub fn select_line_start(&mut self) {
        let anchor = self.selection_anchor_or_cursor();
        self.set_cursor_lc(self.active_buffer().cursor.line, 0);
        self.update_selection_to_cursor(anchor);
    }

    /// Feature 017: extend the selection to the end of the current line.
    pub fn select_line_end(&mut self) {
        let anchor = self.selection_anchor_or_cursor();
        self.cursor_to_line_end();
        self.update_selection_to_cursor(anchor);
    }

    /// Move the cursor up by one viewport page.
    pub fn move_page_up(&mut self) {
        let vh = self.viewport_height();
        let buf = self.active_buffer_mut();
        let target_line = buf.cursor.line.saturating_sub(vh);
        let max_gcol = buf.rope.grapheme_count_on_line(target_line);
        let new_gcol = buf.cursor.grapheme_col.min(max_gcol);
        let new_vcol = CursorPos::visual_col_from_grapheme_col(&buf.rope, target_line, new_gcol);
        buf.cursor = CursorPos {
            line: target_line,
            grapheme_col: new_gcol,
            visual_col: new_vcol,
        };
        // Scroll up by the same amount
        buf.scroll_offset.0 = buf.scroll_offset.0.saturating_sub(vh);
        self.clamp_scroll();
    }

    /// Move the cursor down by one viewport page.
    pub fn move_page_down(&mut self) {
        let vh = self.viewport_height();
        let buf = self.active_buffer_mut();
        let line_count = buf.rope.line_count();
        let target_line = (buf.cursor.line + vh).min(line_count.saturating_sub(1));
        let max_gcol = buf.rope.grapheme_count_on_line(target_line);
        let new_gcol = buf.cursor.grapheme_col.min(max_gcol);
        let new_vcol = CursorPos::visual_col_from_grapheme_col(&buf.rope, target_line, new_gcol);
        buf.cursor = CursorPos {
            line: target_line,
            grapheme_col: new_gcol,
            visual_col: new_vcol,
        };
        self.clamp_scroll();
    }

    /// Move cursor to the very first character of the document.
    pub fn move_doc_start(&mut self) {
        let buf = self.active_buffer_mut();
        buf.cursor = CursorPos::default();
        buf.scroll_offset = (0, 0);
    }

    /// Move cursor to the very last line of the document.
    pub fn move_doc_end(&mut self) {
        let buf = self.active_buffer_mut();
        let last_line = buf.rope.line_count().saturating_sub(1);
        let gcol = buf.rope.grapheme_count_on_line(last_line);
        let vcol = CursorPos::visual_col_from_grapheme_col(&buf.rope, last_line, gcol);
        buf.cursor = CursorPos {
            line: last_line,
            grapheme_col: gcol,
            visual_col: vcol,
        };
        self.clamp_scroll();
    }

    // ── Feature 030 — multi-click selection (US2 / #54) ───────────────────────

    /// Classify the current editor left-press as single (1), double (2), or triple
    /// (3) based on the previous press's time and cell. A press within
    /// [`DOUBLE_CLICK_MS`] of the previous one on the same cell increments the
    /// count (wrapping 3 → 1); otherwise it resets to 1.
    pub(super) fn next_editor_click_count(&mut self, col: u16, row: u16) -> u8 {
        let now = Instant::now();
        let count = match self.last_editor_click {
            Some((pc, pr, n, t))
                if pc == col
                    && pr == row
                    && now.duration_since(t) <= Duration::from_millis(DOUBLE_CLICK_MS) =>
            {
                if n >= 3 {
                    1
                } else {
                    n + 1
                }
            }
            _ => 1,
        };
        self.last_editor_click = Some((col, row, count, now));
        count
    }

    /// Classify a grapheme for word-selection: word characters (alphanumeric or
    /// `_`), whitespace, or other (punctuation/symbols). Double-click selects a
    /// maximal run of the same class.
    pub(super) fn grapheme_class(g: &str) -> u8 {
        match g.chars().next() {
            Some(c) if c.is_alphanumeric() || c == '_' => 0, // word
            Some(c) if c.is_whitespace() => 1,               // space
            _ => 2,                                          // other
        }
    }

    /// Select the word (run of same-class graphemes) under the cursor (US2).
    pub(super) fn select_word_at_cursor(&mut self) {
        let buf = self.active_buffer();
        let line = buf.cursor.line;
        let graphemes: Vec<String> = buf
            .rope
            .line_slice(line)
            .graphemes(true)
            .map(|g| g.to_string())
            .collect();
        let len = graphemes.len();
        if len == 0 {
            self.active_buffer_mut().selection = None; // empty line — clear
            return;
        }
        // Clamp the index (a click at end-of-line lands on `len`).
        let idx = buf.cursor.grapheme_col.min(len - 1);
        let class = Self::grapheme_class(&graphemes[idx]);
        let mut start = idx;
        while start > 0 && Self::grapheme_class(&graphemes[start - 1]) == class {
            start -= 1;
        }
        let mut end = idx + 1;
        while end < len && Self::grapheme_class(&graphemes[end]) == class {
            end += 1;
        }
        self.set_selection_on_line(line, start, end);
    }

    /// Select the whole logical line under the cursor (US2).
    pub(super) fn select_line_at_cursor(&mut self) {
        let line = self.active_buffer().cursor.line;
        let len = self.active_buffer().rope.grapheme_count_on_line(line);
        self.set_selection_on_line(line, 0, len);
    }

    /// Set the active selection to `[start, end)` grapheme columns on `line`, with
    /// the cursor at `end`. A degenerate range clears the selection.
    pub(super) fn set_selection_on_line(&mut self, line: usize, start: usize, end: usize) {
        let buf = self.active_buffer_mut();
        if start >= end {
            buf.selection = None;
            return;
        }
        let vcol = |g| CursorPos::visual_col_from_grapheme_col(&buf.rope, line, g);
        let anchor = CursorPos {
            line,
            grapheme_col: start,
            visual_col: vcol(start),
        };
        let active = CursorPos {
            line,
            grapheme_col: end,
            visual_col: vcol(end),
        };
        buf.selection = Some(Selection { anchor, active });
        buf.cursor = active;
    }

    /// Adjust `scroll_offset` so that `cursor` is within the visible viewport.
    /// Feature 034: clamp every buffer's cursor (and selection endpoints) into the
    /// valid range for its current content, so a stale position left by any path
    /// can never cause an out-of-range line access during render. Cheap (a handful
    /// of buffers) and idempotent.
    pub(super) fn clamp_all_cursors(&mut self) {
        for buf in &mut self.buffers {
            let lc = buf.rope.line_count().max(1);
            let clamp = |c: CursorPos, rope: &crate::buffer::rope::EditorRope| -> CursorPos {
                let line = c.line.min(lc - 1);
                let g = c.grapheme_col.min(rope.grapheme_count_on_line(line));
                let visual_col = CursorPos::visual_col_from_grapheme_col(rope, line, g);
                CursorPos {
                    line,
                    grapheme_col: g,
                    visual_col,
                }
            };
            buf.cursor = clamp(buf.cursor, &buf.rope);
            if let Some(sel) = buf.selection {
                buf.selection = Some(crate::buffer::Selection {
                    anchor: clamp(sel.anchor, &buf.rope),
                    active: clamp(sel.active, &buf.rope),
                });
            }
        }
    }

    pub(super) fn clamp_scroll(&mut self) {
        // Clamp to at least 1 row: a tiny/zero terminal frame (possible now that
        // terminal_size follows the real frame, feature 012 follow-up) would make
        // `vh - 1` underflow below. Guarding here keeps editing crash-free on any
        // frame size.
        let vh = self.viewport_height().max(1);

        if self.soft_wrap && self.wrap_cache.is_some() {
            let cursor_vr = self.cursor_visual_row();
            let buf = self.active_buffer_mut();
            if cursor_vr < buf.scroll_offset.0 {
                buf.scroll_offset.0 = cursor_vr;
            } else if cursor_vr >= buf.scroll_offset.0 + vh {
                buf.scroll_offset.0 = cursor_vr.saturating_sub(vh - 1);
            }
        } else {
            let cur_line = self.active_buffer().cursor.line;
            let buf = self.active_buffer_mut();
            if cur_line < buf.scroll_offset.0 {
                buf.scroll_offset.0 = cur_line;
            } else if cur_line >= buf.scroll_offset.0 + vh {
                buf.scroll_offset.0 = cur_line.saturating_sub(vh - 1);
            }
        }
    }

    // ── Char-index helpers ────────────────────────────────────────────────────

    /// Convert the current cursor position to a rope char index.
    pub(super) fn cursor_char_idx(&self) -> usize {
        let buf = self.active_buffer();
        self.char_idx_for(buf.cursor.line, buf.cursor.grapheme_col)
    }

    /// Return the rope char index for a given (line, grapheme_col) position.
    pub(super) fn char_idx_for(&self, line: usize, gcol: usize) -> usize {
        let buf = self.active_buffer();
        // Start of line in chars
        let line_start: usize = (0..line)
            .map(|l| {
                // Each line in the rope includes the trailing \n except possibly the last
                let s = buf.rope.line_slice(l);
                s.chars().count() + 1 // +1 for the \n
            })
            .sum();

        // Walk grapheme clusters to find the char offset within the line
        let line_str = buf.rope.line_slice(line);
        let char_offset: usize = line_str
            .graphemes(true)
            .take(gcol)
            .map(|g| g.chars().count())
            .sum();

        line_start + char_offset
    }

    /// Feature 029: if the active buffer is read-only, set a status message and
    /// return `true` so the caller aborts the edit (previously a silent no-op).
    pub(super) fn deny_if_readonly(&mut self) -> bool {
        if self.active_buffer().readonly {
            self.status_message = Some("Buffer is read-only".to_string());
            true
        } else {
            false
        }
    }

    // ── T026 — Text insertion ─────────────────────────────────────────────────

    /// Insert a single character at the cursor. No-op when buffer is read-only.
    pub fn insert_char(&mut self, c: char) {
        if self.deny_if_readonly() {
            return;
        }
        // Feature 017: typing replaces the current selection.
        if self.active_buffer().selection.is_some() {
            self.delete_selection();
        }

        let char_idx = self.cursor_char_idx();
        let s = c.to_string();

        {
            let buf = self.active_buffer_mut();
            buf.rope.insert_str(char_idx, &s);
            buf.undo_stack.push(EditOp::Insert {
                at: char_idx,
                text: s,
            });
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);

        // Advance cursor right by one grapheme
        self.move_cursor(Direction::Right);
    }

    /// Insert a newline at the cursor, placing the cursor at column 0 of the
    /// new line.
    pub fn insert_newline(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        if self.active_buffer().selection.is_some() {
            self.delete_selection(); // Feature 017: Enter replaces a selection
        }

        let char_idx = self.cursor_char_idx();

        {
            let buf = self.active_buffer_mut();
            buf.rope.insert_str(char_idx, "\n");
            buf.undo_stack.push(EditOp::Insert {
                at: char_idx,
                text: "\n".to_string(),
            });
            buf.modified = true;

            // Move cursor: next line, column 0
            buf.cursor.line += 1;
            buf.cursor.grapheme_col = 0;
            buf.cursor.visual_col = 0;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);

        self.clamp_scroll();
    }

    // ── T027 — Backspace and Delete ───────────────────────────────────────────

    /// Delete the grapheme cluster immediately before the cursor.
    /// No-op at the start of the buffer or when read-only.
    pub fn delete_backward(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        // Feature 017: Backspace with a selection deletes the selection.
        if self.active_buffer().selection.is_some() {
            self.delete_selection();
            return;
        }

        let cur = self.active_buffer().cursor;

        // No-op at absolute beginning of the buffer
        if cur.line == 0 && cur.grapheme_col == 0 {
            return;
        }

        // Find the grapheme to remove (the one immediately before cursor)
        let (del_line, del_gcol) = if cur.grapheme_col > 0 {
            (cur.line, cur.grapheme_col - 1)
        } else {
            // At the start of a line — deleting the newline of the previous line
            let prev_line = cur.line - 1;
            let prev_len = self.active_buffer().rope.grapheme_count_on_line(prev_line);
            (prev_line, prev_len)
        };

        let del_char_idx = self.char_idx_for(del_line, del_gcol);

        // Collect the grapheme text (may be multi-char for combining sequences)
        let deleted_text: String = {
            let buf = self.active_buffer();
            let line_str = buf.rope.line_slice(del_line);
            line_str
                .graphemes(true)
                .nth(del_gcol)
                .unwrap_or("\n") // at line boundary we delete the \n
                .to_string()
        };

        let del_char_len = deleted_text.chars().count();

        {
            let buf = self.active_buffer_mut();
            buf.rope
                .delete_range(del_char_idx..del_char_idx + del_char_len);
            buf.undo_stack.push(EditOp::Delete {
                at: del_char_idx,
                text: deleted_text,
            });
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);

        // Move cursor to the deleted position
        let new_vcol =
            CursorPos::visual_col_from_grapheme_col(&self.active_buffer().rope, del_line, del_gcol);
        let buf = self.active_buffer_mut();
        buf.cursor = CursorPos {
            line: del_line,
            grapheme_col: del_gcol,
            visual_col: new_vcol,
        };

        self.clamp_scroll();
    }

    /// Delete the grapheme cluster at the cursor.
    /// No-op at the end of the buffer or when read-only.
    pub fn delete_forward(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        // Feature 017: Delete with a selection deletes the selection.
        if self.active_buffer().selection.is_some() {
            self.delete_selection();
            return;
        }

        let cur = self.active_buffer().cursor;
        let line_count = self.active_buffer().rope.line_count();
        let gcol_count = self.active_buffer().rope.grapheme_count_on_line(cur.line);

        // Determine whether we're at the last possible position
        let is_last_line = cur.line + 1 >= line_count;
        let is_last_col = cur.grapheme_col >= gcol_count;

        if is_last_line && is_last_col {
            return; // At end of buffer
        }

        let del_char_idx = self.cursor_char_idx();

        // Determine the text being deleted
        let deleted_text: String = if cur.grapheme_col < gcol_count {
            // Delete grapheme at current column
            let buf = self.active_buffer();
            let line_str = buf.rope.line_slice(cur.line);
            line_str
                .graphemes(true)
                .nth(cur.grapheme_col)
                .unwrap_or("\n")
                .to_string()
        } else {
            // At end of line — delete the newline character
            "\n".to_string()
        };

        let del_char_len = deleted_text.chars().count();

        {
            let buf = self.active_buffer_mut();
            buf.rope
                .delete_range(del_char_idx..del_char_idx + del_char_len);
            buf.undo_stack.push(EditOp::Delete {
                at: del_char_idx,
                text: deleted_text,
            });
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);
        // Cursor position stays the same after forward-delete
    }

    // ── T102 — Clipboard cut / copy / paste ──────────────────────────────────

    /// Copy selected text to the system clipboard.
    /// The active buffer's selected text, or `None` when there is no selection.
    ///
    /// Feature 028: `char_idx_for` returns CHAR indices, so this extracts by chars
    /// (byte-slicing a `String` panics on multibyte boundaries) and clamps a
    /// reversed/degenerate range to empty rather than panicking — defense-in-depth
    /// for copy/cut.
    pub fn selection_text(&self) -> Option<String> {
        let buf = self.active_buffer();
        let sel = buf.selection.as_ref()?;
        let (start, end) = sel.ordered_range();
        let s_idx = self.char_idx_for(start.line, start.grapheme_col);
        let e_idx = self.char_idx_for(end.line, end.grapheme_col);
        let full = buf.rope.to_string();
        let total = full.chars().count();
        let lo = s_idx.min(e_idx).min(total);
        let hi = s_idx.max(e_idx).min(total);
        Some(full.chars().skip(lo).take(hi - lo).collect::<String>())
    }

    pub fn copy_selection(&mut self) {
        let text = match self.selection_text() {
            Some(t) => t,
            None => return,
        };
        // Feature 029: give feedback (was silent on both success and failure).
        match arboard::Clipboard::new() {
            Ok(mut cb) => match cb.set_text(text) {
                Ok(()) => self.status_message = Some("Copied".to_string()),
                Err(e) => {
                    log::warn!("Clipboard write failed: {}", e);
                    self.status_message = Some("Clipboard unavailable".to_string());
                }
            },
            Err(e) => {
                log::warn!("Clipboard unavailable: {}", e);
                self.status_message = Some("Clipboard unavailable".to_string());
            }
        }
    }

    /// Cut selected text to the clipboard (copy + delete selection).
    pub fn cut_selection(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        if self.active_buffer().selection.is_none() {
            return;
        }
        self.copy_selection();
        self.delete_selection();
        // Override the "Copied" set by copy_selection (unless the clipboard failed).
        if self.status_message.as_deref() == Some("Copied") {
            self.status_message = Some("Cut".to_string());
        }
    }

    /// Paste text from the system clipboard at the cursor.
    pub fn paste_clipboard(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        // Feature 017: paste replaces the current selection.
        if self.active_buffer().selection.is_some() {
            self.delete_selection();
        }
        let text = match arboard::Clipboard::new() {
            Ok(mut cb) => match cb.get_text() {
                Ok(t) => t,
                Err(e) => {
                    // Feature 029: empty clipboard / read failure is no longer silent.
                    log::warn!("Clipboard read failed: {}", e);
                    self.status_message = Some("Nothing to paste".to_string());
                    return;
                }
            },
            Err(e) => {
                log::warn!("Clipboard unavailable: {}", e);
                self.status_message = Some("Clipboard unavailable".to_string());
                return;
            }
        };
        if text.is_empty() {
            self.status_message = Some("Nothing to paste".to_string());
            return;
        }
        let char_idx = self.cursor_char_idx();
        let char_count = text.chars().count();
        {
            let buf = self.active_buffer_mut();
            buf.rope.insert_str(char_idx, &text);
            buf.undo_stack.push(EditOp::Insert {
                at: char_idx,
                text: text.clone(),
            });
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);
        // Advance cursor by pasted char count
        for _ in 0..char_count {
            self.move_cursor(Direction::Right);
        }
        self.status_message = Some("Pasted".to_string()); // Feature 029: feedback
    }

    /// Delete the current selection from the buffer.
    pub(super) fn delete_selection(&mut self) {
        let sel = match self.active_buffer().selection {
            Some(s) => s,
            None => return,
        };
        let (start, end) = sel.ordered_range();
        let s_idx = self.char_idx_for(start.line, start.grapheme_col);
        let e_idx = self.char_idx_for(end.line, end.grapheme_col);
        if s_idx >= e_idx {
            return;
        }
        // Feature 029: `char_idx_for` returns CHAR indices; extract the deleted
        // text by chars (byte-slicing a String with char indices panics on
        // multibyte content) — same hazard fixed in `copy_selection`/`selection_text`.
        let deleted: String = {
            let buf = self.active_buffer();
            let full = buf.rope.to_string();
            let total = full.chars().count();
            let lo = s_idx.min(e_idx).min(total);
            let hi = s_idx.max(e_idx).min(total);
            full.chars().skip(lo).take(hi - lo).collect()
        };
        {
            let buf = self.active_buffer_mut();
            buf.rope.delete_range(s_idx..e_idx);
            buf.undo_stack.push(EditOp::Delete {
                at: s_idx,
                text: deleted,
            });
            buf.modified = true;
            buf.selection = None;
            buf.cursor = start;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);
    }
}
