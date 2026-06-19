//! Soft-wrap computation cache (Feature 005).
//!
//! [`WrapCache`] maps each logical line to the byte offsets of its
//! visual sub-lines for a given viewport width.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::buffer::rope::EditorRope;

// Word-boundary set (per FR-003): space, tab, comma, period, semicolon, colon, hyphen, slash.
const WORD_BREAK_CHARS: &[char] = &[' ', '\t', ',', '.', ';', ':', '-', '/'];

/// Computed wrap-point cache for a single viewport width.
///
/// `visual_starts[L]` is a sorted list of byte offsets within logical line L
/// where visual sub-lines start.  The first entry is always 0.
pub struct WrapCache {
    /// Viewport width (display columns) used when this cache was computed.
    pub viewport_width: u16,
    /// Buffer version counter at time of computation.
    pub text_version: u64,
    /// For each logical line index: sorted Vec of byte offsets where visual
    /// sub-lines begin (always starts with 0).
    pub visual_starts: Vec<Vec<u32>>,
    /// Flat map: visual_row -> (logical_line as u32, start_byte as u32).
    pub visual_line_map: Vec<(u32, u32)>,
}

impl WrapCache {
    /// Compute or recompute the wrap cache for the given rope.
    pub fn compute(rope: &EditorRope, viewport_width: u16, text_version: u64) -> Self {
        let line_count = rope.line_count();
        let mut visual_starts: Vec<Vec<u32>> = Vec::with_capacity(line_count);

        for l in 0..line_count {
            let line = rope.line_slice(l);
            let starts = compute_line_wrap_starts(&line, viewport_width);
            visual_starts.push(starts);
        }

        // Flatten into visual_line_map
        let total: usize = visual_starts.iter().map(|v| v.len()).sum();
        let mut visual_line_map: Vec<(u32, u32)> = Vec::with_capacity(total);
        for (l, starts) in visual_starts.iter().enumerate() {
            for &byte_off in starts {
                visual_line_map.push((l as u32, byte_off));
            }
        }

        WrapCache {
            viewport_width,
            text_version,
            visual_starts,
            visual_line_map,
        }
    }

    /// Returns true if the cache is stale and must be recomputed.
    pub fn is_stale(&self, viewport_width: u16, text_version: u64) -> bool {
        self.viewport_width != viewport_width || self.text_version != text_version
    }

    /// Map a visual row index to (logical_line, start_byte_offset).
    /// Returns None if visual_row is out of range.
    pub fn visual_to_logical(&self, visual_row: usize) -> Option<(usize, u32)> {
        self.visual_line_map
            .get(visual_row)
            .map(|&(l, b)| (l as usize, b))
    }

    /// Total number of visual rows across all logical lines.
    pub fn total_visual_rows(&self) -> usize {
        self.visual_line_map.len()
    }

    /// Number of visual rows for a single logical line.
    pub fn visual_row_count(&self, logical_line: usize) -> usize {
        self.visual_starts
            .get(logical_line)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

/// Compute the sorted list of visual-sub-line start byte offsets for a single
/// logical line string with the given viewport width.
///
/// Always returns at least `vec![0]`.
fn compute_line_wrap_starts(line: &str, viewport_width: u16) -> Vec<u32> {
    let mut starts: Vec<u32> = vec![0];

    // Empty line → just the origin.
    if line.is_empty() {
        return starts;
    }

    // Guard: extremely narrow viewport (< 2 cols) — let App handle the guard.
    if viewport_width < 2 {
        return starts;
    }

    let width = viewport_width as usize;
    let mut col: usize = 0;
    let mut byte_off: usize = 0;
    // byte offset of the current visual sub-line's start
    let mut segment_start: usize = 0;
    // byte offset immediately after the last word-boundary grapheme seen
    let mut last_break: usize = 0;
    // column count at which the current segment started (used to recalc after break)
    let mut last_break_col: usize = 0;

    for g in UnicodeSegmentation::graphemes(line, true) {
        let gbytes = g.len();
        let gw_raw = UnicodeWidthStr::width(g);
        // Treat zero-width graphemes as 0 for column counting (combining chars etc.)
        let gw = gw_raw;

        if col + gw > width {
            // Need to wrap. Decide where.
            let break_at: usize;
            let col_after: usize;

            if last_break > segment_start {
                // Soft break at last word boundary
                break_at = last_break;
                // Recompute col from break point to current byte_off
                let fragment = &line[break_at..byte_off];
                col_after = fragment
                    .graphemes(true)
                    .map(UnicodeWidthStr::width)
                    .sum::<usize>();
            } else {
                // Hard break at the current grapheme boundary (before current grapheme)
                break_at = byte_off;
                col_after = 0;
            }

            starts.push(break_at as u32);
            segment_start = break_at;
            last_break = break_at;
            col = col_after;
            // Don't advance byte_off or col by the current grapheme yet —
            // we'll do that below by letting the loop continue naturally.
            // But we need to add the current grapheme's width now.
            col += gw;
            byte_off += gbytes;

            // Update last_break if this grapheme itself is a word-break char
            if g.chars().all(|c| WORD_BREAK_CHARS.contains(&c)) {
                last_break = byte_off;
                last_break_col = col;
            }
            continue;
        }

        // No wrap needed this grapheme.
        col += gw;
        byte_off += gbytes;

        // Track word-break position after this grapheme
        if g.chars().all(|c| WORD_BREAK_CHARS.contains(&c)) {
            last_break = byte_off;
            last_break_col = col;
        }
    }

    // Suppress unused warning
    let _ = last_break_col;

    starts
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a WrapCache from a multi-line string.
    fn cache_from_str(s: &str, width: u16) -> WrapCache {
        let rope = EditorRope::from_str(s);
        WrapCache::compute(&rope, width, 1)
    }

    #[test]
    fn test_ascii_word_wrap() {
        // "hello world foo" width=8 → "hello " is 6 cols, fits; then "world" needs 5 cols
        // but col(6)+"w"(1) = 7 ≤ 8, ok; col 7+"o"=8 ≤ 8 ok; col 8+"r"=9 > 8 → wrap
        // Actually let's re-examine: "hello " = 6 cols, then 'w' at col 7 ≤ 8 ok,
        // 'o' at col 8 ≤ 8 ok, 'r' col 9 > 8 → last_break is after 'hello ' (byte 6)
        // So break at byte 6: first visual row = "hello ", second = "world foo"
        let rope = EditorRope::from_str("hello world foo");
        let cache = WrapCache::compute(&rope, 8, 1);
        let starts = &cache.visual_starts[0];
        assert!(starts.contains(&0), "starts must begin with 0");
        // The soft break should happen at byte 6 (after "hello ")
        assert!(
            starts.contains(&6),
            "expected break at byte 6 for 'hello ', got: {:?}",
            starts
        );
    }

    #[test]
    fn test_no_whitespace_hard_break() {
        // "AAAAAAAAAA" (10 chars), width=5 → hard break at byte 5
        let rope = EditorRope::from_str("AAAAAAAAAA");
        let cache = WrapCache::compute(&rope, 5, 1);
        let starts = &cache.visual_starts[0];
        assert!(starts.contains(&0));
        assert!(
            starts.contains(&5),
            "expected hard break at byte 5, got: {:?}",
            starts
        );
        assert_eq!(cache.visual_row_count(0), 2);
    }

    #[test]
    fn test_cjk_double_width_no_split() {
        // 5 × '字' (each 3 bytes, 2 visual cols), width=6
        // col 0+2=2, col 2+2=4, col 4+2=6 → fits (6==6 means "col + gw > width" is 6+2=8>6)
        // So after 3 chars (col=6) the 4th '字' would push col to 8 > 6 → break at byte 9
        // Result: [0, 9] → 2 visual rows: "字字字" + "字字"
        let rope = EditorRope::from_str("字字字字字");
        let cache = WrapCache::compute(&rope, 6, 1);
        let starts = &cache.visual_starts[0];
        assert_eq!(starts[0], 0);
        // '字' is 3 UTF-8 bytes; after 3 chars = 9 bytes
        assert!(
            starts.contains(&9),
            "expected break at byte 9 (before 4th '字'), got: {:?}",
            starts
        );
        assert_eq!(cache.visual_row_count(0), 2);
    }

    #[test]
    fn test_empty_line() {
        let rope = EditorRope::from_str("");
        let cache = WrapCache::compute(&rope, 80, 1);
        assert_eq!(cache.visual_starts[0], vec![0]);
        assert_eq!(cache.visual_row_count(0), 1);
    }

    #[test]
    fn test_line_exactly_at_width() {
        // "hello" = 5 ASCII chars, width=5 → no wrap (5 == 5 is not > 5)
        let rope = EditorRope::from_str("hello");
        let cache = WrapCache::compute(&rope, 5, 1);
        assert_eq!(cache.visual_starts[0], vec![0]);
        assert_eq!(cache.visual_row_count(0), 1);
    }

    #[test]
    fn test_line_one_over_width() {
        // "hello!" with width=5 → col after "hello" = 5, then '!' pushes 5+1=6>5 → wrap
        // No word-break char in "hello", so last_break == segment_start == 0
        // → hard break at byte_off=5
        let rope = EditorRope::from_str("hello!");
        let cache = WrapCache::compute(&rope, 5, 1);
        let starts = &cache.visual_starts[0];
        assert_eq!(starts[0], 0);
        assert!(
            starts.contains(&5),
            "expected break at byte 5, got: {:?}",
            starts
        );
        assert_eq!(cache.visual_row_count(0), 2);
    }

    #[test]
    fn test_visual_to_logical_roundtrip() {
        // Three-line rope, each line long enough to wrap
        let rope = EditorRope::from_str("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC");
        let cache = WrapCache::compute(&rope, 5, 42);
        // Each 10-char line at width=5 should produce exactly 2 visual rows
        for vr in 0..cache.total_visual_rows() {
            let (logical, _start_byte) = cache.visual_to_logical(vr).expect("in-range");
            // logical line index must be valid
            assert!(logical < 3, "logical={logical} out of range for vr={vr}");
        }
    }

    #[test]
    fn test_is_stale() {
        let rope = EditorRope::from_str("hello");
        let cache = WrapCache::compute(&rope, 80, 1);
        assert!(!cache.is_stale(80, 1), "same params must not be stale");
        assert!(cache.is_stale(79, 1), "changed width must be stale");
        assert!(cache.is_stale(80, 2), "changed version must be stale");
        assert!(cache.is_stale(79, 2), "both changed must be stale");
    }
}
