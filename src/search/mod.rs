//! Search and replace subsystem.
//!
//! Sub-modules:
//! - [`highlight`] — match-span styling for the renderer.
//!
//! This module provides:
//! - [`SearchDirection`]  — forward or backward traversal.
//! - [`CharRange`]        — a half-open `[start, end)` range of char indices.
//! - [`SearchState`]      — per-session search/replace state.
//! - [`SearchEngine`]     — stateless find-all implementation (plain & regex).

#![allow(dead_code, unused_variables, unused_imports)]

pub mod highlight;

use regex::Regex;

// ---------------------------------------------------------------------------
// SearchDirection
// ---------------------------------------------------------------------------

/// Direction of the next-/prev-match traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

impl Default for SearchDirection {
    fn default() -> Self {
        SearchDirection::Forward
    }
}

// ---------------------------------------------------------------------------
// CharRange
// ---------------------------------------------------------------------------

/// A half-open `[start, end)` range expressed as Unicode char (scalar) indices.
///
/// Both indices refer to positions in the rope's flat char sequence, not to
/// line/column coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharRange {
    /// Inclusive start char index.
    pub start: usize,
    /// Exclusive end char index.
    pub end: usize,
}

// ---------------------------------------------------------------------------
// SearchState
// ---------------------------------------------------------------------------

/// All mutable search-and-replace state for one editor session.
#[derive(Debug, Default)]
pub struct SearchState {
    /// The current search query string.
    pub query: String,
    /// The replacement string, if in replace mode.
    pub replacement: Option<String>,
    /// Whether to interpret `query` as a regular expression.
    pub regex_mode: bool,
    /// Whether the search is case-sensitive.
    pub case_sensitive: bool,
    /// Whether the search should wrap from end-of-document back to start
    /// (or start-of-document back to end for backward searches).
    pub wrap: bool,
    /// Direction of traversal for find-next / find-prev.
    pub direction: SearchDirection,
    /// All matches found by the last call to [`SearchEngine::find_all`].
    pub matches: Vec<CharRange>,
    /// Index into `matches` pointing at the currently highlighted match.
    pub active_match: Option<usize>,
}

// ---------------------------------------------------------------------------
// SearchEngine
// ---------------------------------------------------------------------------

/// Stateless search engine.  All inputs are passed as arguments; the engine
/// carries no state of its own.
pub struct SearchEngine;

impl SearchEngine {
    /// Find all occurrences of `query` in `rope` and return their char-index
    /// ranges.
    ///
    /// # Arguments
    /// - `rope`           — the document to search.
    /// - `query`          — the search term or regex pattern.
    /// - `regex_mode`     — when `true`, `query` is compiled as a [`Regex`].
    /// - `case_sensitive` — when `false`, matching is case-insensitive.
    ///
    /// # Returns
    /// A `Vec<CharRange>` with one entry per match, in document order.
    /// Returns an empty vec if `query` is empty or no matches are found.
    /// If `regex_mode` is `true` and the pattern fails to compile, returns an
    /// empty vec (the caller should surface the compile error separately).
    pub fn find_all(
        rope: &crate::buffer::rope::EditorRope,
        query: &str,
        regex_mode: bool,
        case_sensitive: bool,
    ) -> Vec<CharRange> {
        if query.is_empty() {
            return Vec::new();
        }

        let text = rope.to_string();

        if regex_mode {
            Self::find_all_regex(&text, query, case_sensitive)
        } else {
            Self::find_all_plain(&text, query, case_sensitive)
        }
    }

    // -----------------------------------------------------------------------
    // Plain-text search
    // -----------------------------------------------------------------------

    fn find_all_plain(text: &str, query: &str, case_sensitive: bool) -> Vec<CharRange> {
        let mut results = Vec::new();

        // Build comparison-friendly versions.
        let (haystack, needle): (String, String) = if case_sensitive {
            (text.to_owned(), query.to_owned())
        } else {
            (text.to_lowercase(), query.to_lowercase())
        };

        if needle.is_empty() {
            return results;
        }

        // Walk byte offsets of all occurrences.
        let mut search_from = 0usize;
        while search_from <= haystack.len() {
            match haystack[search_from..].find(needle.as_str()) {
                None => break,
                Some(rel_byte) => {
                    let byte_start = search_from + rel_byte;
                    let byte_end = byte_start + needle.len();

                    // Convert byte offsets to char indices using the *original* text.
                    let char_start = byte_offset_to_char(text, byte_start);
                    let char_end = byte_offset_to_char(text, byte_end);

                    results.push(CharRange {
                        start: char_start,
                        end: char_end,
                    });

                    // Advance past this match (at least one byte to avoid infinite loop).
                    search_from = byte_end.max(byte_start + 1);
                }
            }
        }

        results
    }

    // -----------------------------------------------------------------------
    // Regex search
    // -----------------------------------------------------------------------

    fn find_all_regex(text: &str, pattern: &str, case_sensitive: bool) -> Vec<CharRange> {
        // Build the regex, optionally wrapping with (?i) for case-insensitive.
        let full_pattern = if case_sensitive {
            pattern.to_owned()
        } else {
            format!("(?i){}", pattern)
        };

        let re = match Regex::new(&full_pattern) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        re.find_iter(text)
            .map(|m| {
                let char_start = byte_offset_to_char(text, m.start());
                let char_end = byte_offset_to_char(text, m.end());
                CharRange {
                    start: char_start,
                    end: char_end,
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Byte-to-char helper
// ---------------------------------------------------------------------------

/// Convert a UTF-8 byte offset into a Unicode char (scalar value) index by
/// counting how many chars precede `byte_offset` in `text`.
///
/// This is O(n) but is only called during a search operation, not on every
/// keystroke.
fn byte_offset_to_char(text: &str, byte_offset: usize) -> usize {
    text[..byte_offset.min(text.len())].chars().count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::rope::EditorRope;

    fn rope(s: &str) -> EditorRope {
        EditorRope::from_str(s)
    }

    // -----------------------------------------------------------------------
    // Plain-text search
    // -----------------------------------------------------------------------

    #[test]
    fn find_all_plain_single_match() {
        let r = rope("foo bar foo");
        let matches = SearchEngine::find_all(&r, "bar", false, true);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].start, 4);
        assert_eq!(matches[0].end, 7);
    }

    #[test]
    fn find_all_plain_multiple_matches() {
        let r = rope("foo bar foo");
        let matches = SearchEngine::find_all(&r, "foo", false, true);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].start, 0);
        assert_eq!(matches[0].end, 3);
        assert_eq!(matches[1].start, 8);
        assert_eq!(matches[1].end, 11);
    }

    #[test]
    fn find_all_plain_no_match() {
        let r = rope("foo bar foo");
        let matches = SearchEngine::find_all(&r, "baz", false, true);
        assert!(matches.is_empty());
    }

    #[test]
    fn find_all_plain_case_insensitive() {
        let r = rope("Hello HELLO hello");
        let matches = SearchEngine::find_all(&r, "hello", false, false);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn find_all_plain_case_sensitive_subset() {
        let r = rope("Hello HELLO hello");
        let matches = SearchEngine::find_all(&r, "hello", false, true);
        // Only the lowercase "hello"
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn find_all_empty_query_returns_empty() {
        let r = rope("foo bar");
        let matches = SearchEngine::find_all(&r, "", false, true);
        assert!(matches.is_empty());
    }

    #[test]
    fn find_all_multibyte_chars() {
        // "café" — char indices differ from byte indices for é.
        let r = rope("café café");
        let matches = SearchEngine::find_all(&r, "café", false, true);
        assert_eq!(matches.len(), 2);
        // First match: chars 0..4 (c=0, a=1, f=2, é=3)
        assert_eq!(matches[0].start, 0);
        assert_eq!(matches[0].end, 4);
        // Second match: chars 5..9 (space=4)
        assert_eq!(matches[1].start, 5);
        assert_eq!(matches[1].end, 9);
    }

    // -----------------------------------------------------------------------
    // Regex search
    // -----------------------------------------------------------------------

    #[test]
    fn find_all_regex_simple_pattern() {
        let r = rope("abc 123 def 456");
        let matches = SearchEngine::find_all(&r, r"\d+", true, true);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn find_all_regex_case_insensitive() {
        let r = rope("Foo FOO foo");
        let matches = SearchEngine::find_all(&r, "foo", true, false);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn find_all_regex_invalid_pattern_returns_empty() {
        let r = rope("foo bar");
        // Unclosed group is an invalid regex.
        let matches = SearchEngine::find_all(&r, "(unclosed", true, true);
        assert!(matches.is_empty());
    }

    // -----------------------------------------------------------------------
    // SearchState default
    // -----------------------------------------------------------------------

    #[test]
    fn search_state_default() {
        let s = SearchState::default();
        assert!(s.query.is_empty());
        assert!(s.replacement.is_none());
        assert!(!s.regex_mode);
        assert!(!s.case_sensitive);
        assert!(!s.wrap);
        assert_eq!(s.direction, SearchDirection::Forward);
        assert!(s.matches.is_empty());
        assert!(s.active_match.is_none());
    }
}
