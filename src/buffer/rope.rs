//! Task T014: EditorRope — a thin, ergonomic wrapper around [`ropey::Rope`].
//!
//! All public methods operate on Unicode scalar values (char indices) or
//! grapheme clusters, never raw bytes, so callers never need to think about
//! UTF-8 internals.

#![allow(dead_code)]

use unicode_segmentation::UnicodeSegmentation;

// ---------------------------------------------------------------------------
// EditorRope
// ---------------------------------------------------------------------------

/// A gap-buffer / rope-based text container suitable for large files.
///
/// All positional arguments (`char_idx`, `line_idx`, …) use the same
/// coordinate space as `ropey`: 0-based char (Unicode scalar) indices.
pub struct EditorRope(ropey::Rope);

impl EditorRope {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Create an empty rope.
    pub fn new() -> Self {
        EditorRope(ropey::Rope::new())
    }

    /// Create a rope pre-populated with `s`.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        EditorRope(ropey::Rope::from_str(s))
    }

    // -----------------------------------------------------------------------
    // Mutation
    // -----------------------------------------------------------------------

    /// Insert `s` at Unicode char index `char_idx`.
    pub fn insert_str(&mut self, char_idx: usize, s: &str) {
        self.0.insert(char_idx, s);
    }

    /// Delete a char-index range (exclusive end) from the rope.
    pub fn delete_range(&mut self, range: std::ops::Range<usize>) {
        self.0.remove(range);
    }

    // -----------------------------------------------------------------------
    // Line queries
    // -----------------------------------------------------------------------

    /// Total number of lines (at least 1 even for an empty rope).
    pub fn line_count(&self) -> usize {
        // ropey counts the trailing newline as a line terminator, not as a
        // line of its own, so `len_lines()` already behaves the way we want.
        self.0.len_lines()
    }

    /// Return line `line_idx` as a `String`, with any trailing `\n` / `\r\n`
    /// stripped.
    pub fn line_slice(&self, line_idx: usize) -> String {
        let line = self.0.line(line_idx);
        let s: String = line.chars().collect();
        // Strip trailing newline characters.
        s.trim_end_matches(['\n', '\r']).to_owned()
    }

    // -----------------------------------------------------------------------
    // Grapheme queries
    // -----------------------------------------------------------------------

    /// All grapheme clusters on `line_idx`, each as an owned `String`.
    /// The trailing newline (if any) is excluded.
    pub fn graphemes_on_line(&self, line_idx: usize) -> Vec<String> {
        let line = self.line_slice(line_idx);
        line.graphemes(true).map(|g| g.to_owned()).collect()
    }

    /// Number of grapheme clusters on `line_idx` (excluding trailing newline).
    pub fn grapheme_count_on_line(&self, line_idx: usize) -> usize {
        let line = self.line_slice(line_idx);
        line.graphemes(true).count()
    }

    // -----------------------------------------------------------------------
    // Global char / byte conversions
    // -----------------------------------------------------------------------

    /// Total number of Unicode scalar values (chars) in the rope.
    pub fn char_count(&self) -> usize {
        self.0.len_chars()
    }

    /// Char (Unicode scalar) index of the first character of line `line_idx`.
    /// Clamped to the document length for out-of-range lines (Feature 015).
    pub fn line_to_char(&self, line_idx: usize) -> usize {
        if line_idx >= self.0.len_lines() {
            return self.0.len_chars();
        }
        self.0.line_to_char(line_idx)
    }

    /// Convert a char (Unicode scalar) index to a UTF-8 byte index.
    ///
    /// This is implemented by materialising the rope as a `String` and then
    /// walking the char indices, because `ropey` 0.6 does not expose a public
    /// `char_to_byte` method.  For hot paths, prefer caching the `String` form.
    pub fn char_to_byte(&self, char_idx: usize) -> usize {
        let s = self.0.to_string();
        s.char_indices()
            .nth(char_idx)
            .map(|(byte_pos, _)| byte_pos)
            .unwrap_or(s.len())
    }

    /// Convert a UTF-8 byte index to a char (Unicode scalar) index.
    ///
    /// Walks the materialised string to count how many chars precede `byte_idx`.
    pub fn byte_to_char(&self, byte_idx: usize) -> usize {
        let s = self.0.to_string();
        s[..byte_idx].chars().count()
    }

    // -----------------------------------------------------------------------
    // Serialisation
    // -----------------------------------------------------------------------

    /// Collect the entire rope contents into a `String`.
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// Default
// ---------------------------------------------------------------------------

impl Default for EditorRope {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rope() -> EditorRope {
        EditorRope::from_str("Hello\nWorld\nFoo")
    }

    // --- Construction -------------------------------------------------------

    #[test]
    fn new_is_empty() {
        let r = EditorRope::new();
        assert_eq!(r.char_count(), 0);
    }

    #[test]
    fn from_str_char_count() {
        let r = EditorRope::from_str("abc");
        assert_eq!(r.char_count(), 3);
    }

    // --- Mutation -----------------------------------------------------------

    #[test]
    fn insert_str_at_start() {
        let mut r = EditorRope::from_str("world");
        r.insert_str(0, "hello ");
        assert_eq!(r.to_string(), "hello world");
    }

    #[test]
    fn insert_str_at_end() {
        let mut r = EditorRope::from_str("hello");
        r.insert_str(5, " world");
        assert_eq!(r.to_string(), "hello world");
    }

    #[test]
    fn delete_range_removes_chars() {
        let mut r = EditorRope::from_str("hello world");
        r.delete_range(5..11);
        assert_eq!(r.to_string(), "hello");
    }

    // --- Line queries -------------------------------------------------------

    #[test]
    fn line_count_multiline() {
        let r = make_rope();
        // "Hello\n", "World\n", "Foo" → 3 lines
        assert_eq!(r.line_count(), 3);
    }

    #[test]
    fn line_slice_strips_newline() {
        let r = make_rope();
        assert_eq!(r.line_slice(0), "Hello");
        assert_eq!(r.line_slice(1), "World");
        assert_eq!(r.line_slice(2), "Foo");
    }

    #[test]
    fn line_slice_crlf_stripped() {
        let r = EditorRope::from_str("Line1\r\nLine2");
        assert_eq!(r.line_slice(0), "Line1");
        assert_eq!(r.line_slice(1), "Line2");
    }

    // --- Grapheme queries ---------------------------------------------------

    #[test]
    fn graphemes_on_ascii_line() {
        let r = EditorRope::from_str("abc\nxyz");
        let g = r.graphemes_on_line(0);
        assert_eq!(g, vec!["a", "b", "c"]);
    }

    #[test]
    fn grapheme_count_ascii() {
        let r = EditorRope::from_str("hello\n");
        assert_eq!(r.grapheme_count_on_line(0), 5);
    }

    #[test]
    fn graphemes_multibyte() {
        // "café" = ['c','a','f','é'] — é is a two-byte UTF-8 sequence.
        let r = EditorRope::from_str("café");
        let g = r.graphemes_on_line(0);
        assert_eq!(g, vec!["c", "a", "f", "\u{00E9}"]);
        assert_eq!(r.grapheme_count_on_line(0), 4);
    }

    // --- Char / byte conversion ---------------------------------------------

    #[test]
    fn char_to_byte_ascii() {
        let r = EditorRope::from_str("hello");
        // In pure ASCII char and byte indices coincide.
        assert_eq!(r.char_to_byte(3), 3);
    }

    #[test]
    fn byte_to_char_ascii() {
        let r = EditorRope::from_str("hello");
        assert_eq!(r.byte_to_char(3), 3);
    }

    #[test]
    fn char_to_byte_multibyte() {
        // 'é' occupies 2 bytes; the char at index 1 starts at byte 1,
        // but the char after it ('!') starts at byte 3.
        let r = EditorRope::from_str("é!");
        assert_eq!(r.char_to_byte(0), 0); // 'é' starts at byte 0
        assert_eq!(r.char_to_byte(1), 2); // '!' starts at byte 2
    }

    // --- to_string ----------------------------------------------------------

    #[test]
    fn to_string_roundtrip() {
        let src = "Hello\nWorld\n";
        let r = EditorRope::from_str(src);
        assert_eq!(r.to_string(), src);
    }
}
