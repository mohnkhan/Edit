# Data Model: Caret-on-click in dialog text fields

UI input-state changes only; no persisted data.

## field_caret_at (pure helper, `src/ui/width.rs`)

- **Signature (intent)**: `field_caret_at(value: &str, field_w: u16, click_offset: u16) -> usize`.
- **Contract**: returns a grapheme index into `value`. Visible window = left-aligned when
  `str_width(value) <= field_w`, else the right-anchored tail fitting `field_w`. Walks visible graphemes
  by `display_width` until exceeding `click_offset`. Clamped to `[0, grapheme_count(value)]`. Ignores the
  1-column caret glyph artifact.

## Find/Replace field (`FindReplaceDialog`, existing)

- `caret: usize` (grapheme index) — already present, with `move_left/right`, `insert_char`, `backspace`.
- **New**: a click sets `focus` to the clicked field and `caret = field_caret_at(value, rect.width,
  click_col - rect.x)`.

## File-browser Name field (`FileBrowser`)

- **New**: `caret: usize` (grapheme index into `filename`, default = end).
- **Rules**: `push_char` inserts at `caret` then `caret += 1`; `backspace` removes the grapheme before
  `caret` then `caret -= 1`; `move_left/right` ±1 clamped; `move_home/end` → 0 / len; a click →
  `caret = field_caret_at(...)`. Clearing the field resets `caret = 0`.

## Go-to-Line input (`App`)

- **New**: `pending_goto_line_caret: usize` (grapheme index into the digit string; reset on open).
- **Rules**: digit insert at caret (digits-only preserved); Backspace before caret; Left/Right/Home/End
  move (clamped); click → `field_caret_at(...)`.

## Field text rects (renderer-shared)

- Find/Replace: `find_replace_field_rects(d, area)` → text rects for query (+ replacement in replace
  mode), matching `render_find_field`.
- File browser: field text rect from `compute_layout` (`field_box` interior).
- Go-to-Line: value origin `dx + 1 + "Go to line: ".len()`, row `dy + 1`, width `dw - 2 - 12`.
