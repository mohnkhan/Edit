# Contract: WrapCache

**File**: `src/ui/wrap.rs` (new module)

## Public API

```rust
pub struct WrapCache {
    pub viewport_width: u16,
    pub text_version: u64,
    /// visual_starts[logical_line] = sorted Vec of byte offsets where visual sub-lines begin.
    /// Always non-empty; visual_starts[L][0] == 0.
    pub visual_starts: Vec<Vec<u32>>,
    /// Flat list: visual_row -> (logical_line, start_byte_offset).
    /// Built from visual_starts after each recompute.
    pub visual_line_map: Vec<(u32, u32)>,
}

impl WrapCache {
    /// Compute or recompute the wrap cache for the given buffer rope,
    /// viewport width, and version counter.
    pub fn compute(rope: &EditorRope, viewport_width: u16, text_version: u64) -> Self;

    /// True if the cache is stale and must be recomputed before use.
    pub fn is_stale(&self, viewport_width: u16, text_version: u64) -> bool;

    /// Map a visual row index to (logical_line, grapheme_col_of_first_grapheme_on_that_row).
    pub fn visual_to_logical(&self, visual_row: usize) -> Option<(usize, usize)>;

    /// Total number of visual rows across all logical lines.
    pub fn total_visual_rows(&self) -> usize;

    /// Number of visual rows occupied by a single logical line.
    pub fn visual_row_count(&self, logical_line: usize) -> usize;
}
```

## compute() Algorithm

```
For each logical_line L in rope:
  line_str = rope.line_slice(L)
  starts = [0u32]
  col = 0          // running display column
  last_break = 0   // byte offset of last whitespace break opportunity
  last_break_col = 0
  byte_off = 0

  For each grapheme G in line_str.graphemes(true):
    gw = UnicodeWidthStr::width(G)
    // Never split: if gw==2 and col+gw > viewport_width, break before G
    if col + gw > viewport_width:
      break_at = if last_break > starts.last() { last_break } else { byte_off }
      starts.push(break_at as u32)
      col = byte_off - break_at + gw  // column of G after the break
      // reset break opportunity tracking
      last_break = byte_off
      last_break_col = 0
    // Word-boundary set (per FR-003): U+0020 U+0009 U+002C U+002E U+003B U+003A U+002D U+002F
    // Break placed AFTER the boundary char (boundary char ends the preceding visual line).
    if is_word_boundary(G):  // space, tab, comma, period, semicolon, colon, hyphen, slash
      last_break = byte_off + G.len()
      last_break_col = col + gw
    col += gw
    byte_off += G.len()

  visual_starts[L] = starts
```

## Invalidation Contract

`WrapCache::is_stale(w, v)` returns `true` when:
- `self.viewport_width != w`, OR
- `self.text_version != v`

App MUST call `is_stale()` at the start of every render when `soft_wrap == true` and recompute if stale.

## visual_line_map Layout

After `visual_starts` is populated, flatten into:
```
visual_line_map = []
for (L, starts) in visual_starts.iter().enumerate():
  for start_byte in starts:
    visual_line_map.push((L as u32, *start_byte))
```

`visual_line_map[visual_row]` gives `(logical_line, start_byte)` for O(1) lookup.

## Minimum Width Guard

If `viewport_width < 10`: return an empty/disabled cache and let caller show the "too narrow" warning.
