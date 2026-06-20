//! Feature 029: the single source of truth for terminal display width.
//!
//! Previously the codebase had two divergent custom width helpers
//! (`file_browser::grapheme_width`, `app::unicode_segmentation_width`) that only
//! inspected a grapheme's first scalar — so combining marks were counted as width
//! 1 (should be 0) and emoji/ZWJ sequences were mis-measured, causing cursor,
//! scroll, and truncation misalignment. Everything now routes through here, which
//! uses the `unicode-width` tables (combining = 0, East-Asian wide = 2).

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Display columns occupied by a single grapheme cluster.
///
/// Combining marks yield 0, East-Asian wide and most emoji yield 2, everything
/// else 1. Control characters yield 0 (they are never rendered as glyphs; the
/// editor handles tabs separately before calling this).
pub fn display_width(grapheme: &str) -> u16 {
    UnicodeWidthStr::width(grapheme) as u16
}

/// Display columns occupied by a string (sum of its grapheme widths).
pub fn str_width(s: &str) -> u16 {
    s.graphemes(true).map(display_width).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_one() {
        assert_eq!(display_width("a"), 1);
        assert_eq!(display_width(" "), 1);
    }

    #[test]
    fn combining_mark_is_zero() {
        // U+0301 COMBINING ACUTE ACCENT on its own occupies no column.
        assert_eq!(display_width("\u{0301}"), 0);
        // "e" + combining acute renders in one column.
        assert_eq!(str_width("e\u{0301}"), 1);
    }

    #[test]
    fn east_asian_wide_is_two() {
        assert_eq!(display_width("世"), 2);
        assert_eq!(str_width("世界"), 4);
    }

    #[test]
    fn str_width_sums_mixed() {
        // "aあb" = 1 + 2 + 1.
        assert_eq!(str_width("aあb"), 4);
    }
}
