# Phase 0 Research: Visible text selection

## R1. Highlight style
**Decision**: render selected cells with `Modifier::REVERSED` (swap fg/bg). Distinct from the yellow
search-match highlight, theme-independent, visible on monochrome terminals, no new theme field.
**Alternatives**: a dedicated theme `selection_bg` — more surface; rejected (REVERSED is enough and
DOS-faithful).

## R2. Selection range → per-line visible spans
**Decision**: normalize the selection to an ordered `(start, end)` of `(line, grapheme_col)` (so reverse
selections work). During the per-line render (which already walks graphemes with a running grapheme
column), a cell is selected iff its `(line, gcol)` falls in `[start, end)`. Multi-line: the first line is
selected from `start.col` to end-of-line, full middle lines, the last line up to `end.col`. Only visible
cells are styled, so horizontal scroll and soft-wrap are handled by the existing loop.
**Rationale**: reuses the feat-015 match-overlay approach but keyed on (line,col) instead of char index;
no separate geometry, so it can't drift from what's drawn.

## R3. Keyboard selecting movement
**Decision**: add `Select{Left,Right,Up,Down,LineStart,LineEnd}` actions bound to `Shift+<key>`. Handler
`move_cursor_selecting(dir)`: if `selection` is `None`, set `anchor = cursor`; move the cursor via the
existing movement; set `selection = Some({anchor, active: cursor})`; if the resulting range is empty,
set `selection = None`. Plain (non-shift) `Move*` and any edit clear the selection (most edit paths
already call `delete_selection`/clear; ensure `move_cursor` clears it).
**Rationale**: reuses existing movement + clamping; minimal new logic.

## R4. Mouse drag selection
**Decision**: on left **Press** in the editor area: `handle_mouse_click` to set the cursor, then set
`anchor = cursor`, `selection = None` (pending). On **Drag**: `handle_mouse_click` to move the cursor,
then `selection = Some({anchor, active: cursor})` (clear if empty). On a press with no drag before
release: the selection stays `None` → single click clears selection + moves cursor. `handle_mouse_event`
currently only acts on `Press`; extend it to also handle `Drag` in the editor region.
**Rationale**: reuses the soft-wrap-aware click→cursor mapping for both endpoints.

## R5. Clearing rules
**Decision**: selection clears on: a non-shift cursor move, a buffer edit (insert/newline/backspace/
delete/paste — paste/typing replaces the selection first), and a single click. Undo/redo already clears
selection. Select-All sets it; Esc does not need to (no selection-specific Esc).
**Rationale**: matches standard editor behavior (FR-004).
