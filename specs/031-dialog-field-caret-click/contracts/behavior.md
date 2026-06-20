# Contract: Caret-on-click in dialog text fields

## field_caret_at(value, field_w, click_offset) -> grapheme index

- Value fits (`str_width(value) <= field_w`): result = the grapheme whose display-width span contains
  `click_offset`, counting from 0; `click_offset` past the text → `grapheme_count(value)` (end).
- Value overflows: the visible window is the right-anchored tail fitting `field_w`; `click_offset` maps
  within that tail to the absolute grapheme index; clamped to `[0, grapheme_count(value)]`.
- Multibyte/wide/combining: widths via the shared `ui::width::display_width` (combining=0, wide=2).
- Never panics; empty value → 0.

## US1 — Find/Replace

| Action | Effect |
|---|---|
| Click on a visible char in the Find/Replace field | `caret` moves to that grapheme; the field is focused |
| Click past the text | `caret` clamps to the value end |
| Click on the label/border | caret unchanged |
| Click on a button | button activates (unchanged) |

## US2 — file-browser Name field

| Action | Effect |
|---|---|
| Left / Right | caret −1 / +1 (clamped) |
| Home / End | caret to 0 / value length |
| Type a char | inserted at the caret; caret advances (append still works at end) |
| Backspace | removes the grapheme before the caret |
| Click in the field box | caret moves to the clicked grapheme |
| Existing flows (filter/append, Enter activate, ← parent, Esc) | unchanged |

## US3 — Go-to-Line input

Same caret behavior as US2, digits-only; Enter jumps (clamped to range) and Esc cancels — unchanged.

## No-regression

- Existing keys, mouse (buttons, list rows, outside-click), editing elsewhere, file formats, and
  dialog confirm/cancel flows unchanged. No new dependencies. Geometry drawn == clickable per field.
