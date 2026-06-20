# Data Model: Word-wise navigation, selection, and deletion

Editor operations only; no persisted data.

## grapheme_class (existing, `App::grapheme_class`)

- 0 = word (Unicode alphanumeric or `_`), 1 = whitespace, 2 = other. Shared with double-click (030).

## next_word_pos (new, pure)

- **Input**: direction + current cursor `(line, gcol)` + buffer content.
- **Output**: target `(line, gcol)` on a grapheme boundary (possibly an adjacent line); equals the cursor
  at a buffer end (no-op signal). Right = start of next token (skip current run + following whitespace);
  Left = start of preceding token (skip preceding whitespace + token run); crosses line boundaries.

## Operations (new wrappers)

- **move_word(dir)**: selection → None; cursor → `next_word_pos(dir)`; scroll into view.
- **move_word_selecting(dir)**: anchor = current selection anchor or cursor; cursor → `next_word_pos(dir)`;
  selection = anchor↔cursor (None if empty).
- **delete_word(dir)**: read-only → message + no-op; else if a selection exists → delete it; else set
  selection = cursor↔`next_word_pos(dir)` and delete it via `delete_selection` (one undo step; cursor at
  range start). No-op when target == cursor.

## Actions & keymap (new)

- `MoveWordLeft`/`MoveWordRight`, `SelectWordLeft`/`SelectWordRight`, `DeleteWordLeft`/`DeleteWordRight`.
- Bindings: `Ctrl+Left`/`Ctrl+Right`, `Ctrl+Shift+Left`/`Ctrl+Shift+Right`, `Ctrl+Backspace`/`Ctrl+Delete`.
- Existing bindings unchanged.
