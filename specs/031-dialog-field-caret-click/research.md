# Research: Caret-on-click in dialog text fields

Closing #58; no NEEDS CLARIFICATION. Decisions per the audit of the current field code.

## D1 — The shared `field_caret_at` helper

**Finding**: `field_box_text` (mod.rs) and the file-browser field render show the value left-aligned when
it fits the inner width, else a right-anchored tail; both embed a 1-col caret glyph `▏`.

**Decision**: Add `ui::width::field_caret_at(value: &str, field_w: u16, click_offset: u16) -> usize`:
compute the first visible grapheme (0 when `str_width(value) <= field_w`, else the start of the tail that
fits `field_w`), then walk visible graphemes summing `display_width` until exceeding `click_offset`,
returning that grapheme index; clamp to `[0, grapheme_len]`. It reverses the renderer's window logic and
ignores the caret glyph (a ≤1-col artifact), per the spec contract.

**Rationale**: One pure, unit-testable function shared by all three fields; matches what the user sees.

**Alternatives**: Reverse the exact `field_box_text` including the caret glyph — rejected: the glyph moves
with the caret (a moving target), adds complexity for sub-column gain.

## D2 (US1) — Find/Replace click

**Finding**: `FindReplaceDialog` has `caret` (grapheme index) with `move_left/right`, `insert_char`,
`backspace` already operating at the caret; render is `render_find_field` (text at `dx+2`, query row
`dy+3`, replacement row `dy+7`, width `dw-4`).

**Decision**: Add `find_replace_field_rects(d, area) -> Vec<(DialogField, Rect)>` (the text rects, same
math as `render_find_field`). In `handle_mouse_event`, after the interactive button hit-test, if a click
lands in a field's text rect: set `d.focus` to that field and `d.caret = field_caret_at(value, rect.width,
click_col - rect.x)`. Unit-test the rects against the render rows.

## D3 (US2) — file-browser Name caret model

**Finding**: `FileBrowser.filename: String`, `push_char` appends, `backspace` pops; render uses
`format!("{}▏", filename)` (caret always at end). No caret index, no Left/Right.

**Decision**: Add `caret: usize` (grapheme index, default = end). `push_char` inserts at the caret and
advances it; `backspace` removes the grapheme before the caret; add `move_left/move_right/move_home/
move_end` (clamped). Render: build the shown string by inserting `▏` at the caret grapheme (reuse the
existing right-anchor tail logic). Expose the field text rect from `compute_layout`
(`field_box.x+1, field_box.y+1, field_box.width-2`). Click → `caret = field_caret_at(...)`. Keep
`filename` semantics (filtering, activation, navigation) unchanged; resetting/clearing the field also
resets the caret.

**Rationale**: Minimal first-class single-line input; preserves all existing flows (append still works
when the caret is at end).

## D4 (US3) — Go-to-Line caret model

**Finding**: `pending_goto_line: Option<String>`; render `format!("Go to line: {entry}▏")`; the handler
accepts digits + Backspace + Enter + Esc (append-only).

**Decision**: Add `pending_goto_line_caret: usize` to `App` (reset to the value length whenever the
prompt opens / value changes externally). Handler: digit insert at caret (still digits-only),
Backspace-before-caret, `MoveLeft/MoveRight/MoveLineStart/MoveLineEnd` move the caret (clamped). Render:
embed `▏` at the caret within the digits (value starts at `dx + 1 + "Go to line: ".len()`). Click maps via
`field_caret_at` over the digit value. Enter-jump/clamp and Esc-cancel unchanged.

**Rationale**: Consistent caret behavior; digits-only restriction preserved.

**Alternatives**: Replace `pending_goto_line: Option<String>` with a struct — rejected: a parallel caret
field is less churn across existing call sites/tests.

## D5 — Click routing & precedence

**Decision**: Field-click handling goes in `handle_mouse_event` immediately after the interactive-dialog
button hit-test and the list-row hit-test (feature 030), before the fall-through. Buttons and list rows
keep priority; a click on neither a button, row, nor field interior is the existing no-op/outside
behavior.

## Testing approach

TDD. Unit: `field_caret_at` (fits, overflow/right-anchored, multibyte/wide, clamp, empty); Find/Replace
field rects match render rows; file-browser caret insert/delete/move + field rect; Go-to-Line caret keys.
Integration (`tests/integration/field_caret.rs`): click into a Find field → `d.caret` set; click into the
Name field → caret set; click into Go-to-Line → caret set; arrow/insert mid-string per field.
