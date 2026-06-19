# Contract: EditorWidget — Soft-Wrap Rendering Path

**File**: `src/ui/editor.rs`

## Modified Struct

```rust
pub struct EditorWidget<'a> {
    pub buffer: &'a Buffer,
    pub theme: &'static Theme,
    pub show_line_numbers: bool,
    // NEW:
    pub soft_wrap: bool,
    /// Wrap break points per logical line; empty slice when soft_wrap == false.
    pub wrap_starts: &'a [Vec<u32>],
}
```

## Rendering Contract (soft_wrap == true)

When `soft_wrap == true`, the widget uses `wrap_starts` to enumerate visual rows:

```
visual_row = 0
for (L, line_starts) in wrap_starts.iter().enumerate():
  if visual_row >= visible_rows + scroll_vrow: break

  for (seg_idx, &start_byte) in line_starts.iter().enumerate():
    if visual_row < scroll_vrow:
      visual_row += 1
      continue  // skip rows above viewport

    screen_y = area.top() + (visual_row - scroll_vrow) as u16
    if screen_y >= area.bottom(): break

    is_continuation = seg_idx > 0
    render_gutter(L, is_continuation, screen_y)
    render_segment(line_str[start_byte..next_start_byte], screen_y, is_continuation)
    visual_row += 1
```

### Continuation Marker

On continuation visual lines (`seg_idx > 0`): place `»` (U+00BB) at the leftmost gutter position (column `area.left()`). If the gutter is enabled (line numbers), the `»` replaces the `|` separator at the rightmost gutter column.

### Scroll Offset in Wrap Mode

- `buffer.scroll_offset.0` = first **visual** row visible (not logical line). The App must update this field in visual-row units when `soft_wrap == true`.
- `buffer.scroll_offset.1` = ignored (horizontal scroll suppressed); always treated as 0.

### Cursor Rendering in Wrap Mode

The cursor `CursorPos.visual_col` is an offset within the **logical** line. To render it:
1. Find which visual segment the cursor falls in by scanning `line_starts` for the largest `start_byte ≤ cursor_byte_offset`.
2. The cursor's screen column = `cursor.visual_col - segment_start_visual_col`.
3. If the cursor column ≥ content_width, it is on the next segment — not visible in the current segment row.

### Horizontal Scroll Suppression

When `soft_wrap == true`, `scroll_vcol` is forced to 0 for rendering purposes regardless of `buffer.scroll_offset.1`. The App clamps `scroll_offset.1 = 0` on toggle-on.

## Rendering Contract (soft_wrap == false)

Unchanged — existing horizontal-scroll path used verbatim.
