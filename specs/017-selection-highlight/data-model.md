# Phase 1 Data Model: Visible text selection

## Existing types (unchanged shape)

- `Selection { anchor: CursorPos, active: CursorPos }` (`src/buffer/mod.rs`). Selected range = ordered
  span between `anchor` and `active`. Empty when `anchor == active` → treat as no selection.
- `Buffer.selection: Option<Selection>`.

## Derived: ordered range + per-cell test

- `selection_ordered(sel) -> ((line,gcol) start, (line,gcol) end)` with `start <= end` (handles reverse).
- A rendered cell at `(line, gcol)` is highlighted iff `start <= (line,gcol) < end`. Newline at a line's
  end is considered part of the selection for full middle lines (visual fill to line end is optional).

## `Action` (new) — `src/input/keymap.rs`

`SelectLeft`, `SelectRight`, `SelectUp`, `SelectDown`, `SelectLineStart`, `SelectLineEnd`, bound to
`Shift+Left/Right/Up/Down/Home/End`. (key_to_string already emits `Shift+` for these non-char keys.)

## `App` behavior — `src/app.rs`

- `move_cursor_selecting(dir)`: anchor at cursor if no selection; move cursor (reuse `move_cursor`
  internals without clearing); set `selection = Some({anchor, active: cursor})`; `None` if empty.
- `select_line_start()` / `select_line_end()`: same, to line bounds.
- `move_cursor(dir)` (plain): clears `selection` (then moves) — non-shift movement deselects.
- Mouse: Press in editor → set cursor + `anchor`, `selection=None`; Drag in editor → move cursor +
  `selection=Some({anchor,active})`.
- Edits (insert/newline/backspace/delete/paste): replace the selection if present (existing
  `delete_selection`) then apply; ensure `selection=None` after.

## Render — `src/ui/editor.rs`

`EditorWidget` reads `buffer.selection`; in both render paths, when drawing a cell whose `(line, gcol)`
is in the ordered selection range, add `Modifier::REVERSED` (applied after syntax/match styles, before
the cursor cell which keeps its own style).

## Invariants

- Highlighted cells == the ordered selection range, direction-independent (FR-008).
- Only visible cells styled; grapheme/scroll/soft-wrap correct (FR-007).
- Selection distinct from search-match highlight (FR-001); cursor cell still distinct.
- Non-shift move / edit / single click clears selection (FR-004/FR-006).
