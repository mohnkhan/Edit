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

/// Feature 031: map a horizontal click offset (columns from a field's inner-left
/// edge) to a caret grapheme index in `value`, mirroring the field renderer's
/// visible window: left-aligned when the value fits `field_w`, else the
/// right-anchored tail that fits `field_w`. The 1-column caret glyph the renderer
/// embeds is ignored (a ≤1-col artifact). The result is clamped to
/// `[0, grapheme_count(value)]`.
pub fn field_caret_at(value: &str, field_w: u16, click_offset: u16) -> usize {
    let graphemes: Vec<&str> = value.graphemes(true).collect();
    let total = str_width(value);
    // First visible grapheme index (right-anchor the tail when it overflows).
    let first = if total <= field_w || field_w == 0 {
        0
    } else {
        let mut acc = 0u16;
        let mut count = 0usize;
        for g in graphemes.iter().rev() {
            let w = display_width(g);
            if acc + w > field_w {
                break;
            }
            acc += w;
            count += 1;
        }
        graphemes.len() - count
    };
    // Walk the visible graphemes, accumulating display width, until we pass the
    // click offset — that grapheme is the caret target. Past the end → value len.
    let mut x = 0u16;
    let mut idx = first;
    for g in &graphemes[first..] {
        let w = display_width(g);
        if x + w > click_offset {
            return idx;
        }
        x += w;
        idx += 1;
    }
    idx.min(graphemes.len())
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

    // T003 (Feature 031): field_caret_at maps a click offset to a caret grapheme.
    #[test]
    fn field_caret_fits_left_aligned() {
        // "hello" fits in width 20: offset n → caret n; past end → 5.
        assert_eq!(field_caret_at("hello", 20, 0), 0);
        assert_eq!(field_caret_at("hello", 20, 2), 2);
        assert_eq!(field_caret_at("hello", 20, 4), 4);
        assert_eq!(field_caret_at("hello", 20, 99), 5); // clamp to end
    }

    #[test]
    fn field_caret_empty_value() {
        assert_eq!(field_caret_at("", 10, 0), 0);
        assert_eq!(field_caret_at("", 10, 5), 0);
    }

    #[test]
    fn field_caret_wide_chars() {
        // "aあb": widths 1,2,1. Offset 0→0, 1→1 (start of 'あ'), 2 is mid-'あ'
        // (1+2 > 2 so still index 1), 3→2 ('b'), past→3.
        assert_eq!(field_caret_at("aあb", 20, 0), 0);
        assert_eq!(field_caret_at("aあb", 20, 1), 1);
        assert_eq!(field_caret_at("aあb", 20, 2), 1);
        assert_eq!(field_caret_at("aあb", 20, 3), 2);
        assert_eq!(field_caret_at("aあb", 20, 9), 3);
    }

    #[test]
    fn field_caret_overflow_right_anchored() {
        // "abcdefghij" (10 wide) in a width-4 field shows the tail "ghij"
        // (indices 6..10). Offset 0 → 6, 1 → 7, past → 10.
        assert_eq!(field_caret_at("abcdefghij", 4, 0), 6);
        assert_eq!(field_caret_at("abcdefghij", 4, 1), 7);
        assert_eq!(field_caret_at("abcdefghij", 4, 99), 10);
    }
}
