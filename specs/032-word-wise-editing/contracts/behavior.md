# Contract: Word-wise navigation, selection, and deletion

Word classes (shared with double-click, feature 030): word = alphanumeric/`_`, whitespace, other.

## US1 — movement

| Key | Effect |
|---|---|
| Ctrl+Right | cursor → start of the next token (skip current run + following whitespace); at EOL → next line col 0; at buffer end → no-op |
| Ctrl+Left | cursor → start of the preceding token (skip preceding whitespace + token run); at col 0 → end of previous line; at buffer start → no-op |

- A plain word move clears any selection and scrolls the cursor into view.

## US2 — selection

| Key | Effect |
|---|---|
| Ctrl+Shift+Right | extend selection to the next word boundary (same rule as Ctrl+Right) |
| Ctrl+Shift+Left | extend selection to the previous word boundary |

- The selection is anchored like Shift+Arrow and is usable by Copy/Cut; collapsing to empty clears it.

## US3 — deletion

| Key | Effect |
|---|---|
| Ctrl+Backspace | delete from the cursor back to the previous word boundary (one undo step) |
| Ctrl+Delete | delete from the cursor forward to the next word boundary (one undo step) |

- With an active selection, both delete the selection instead.
- Read-only buffer → no change, "Buffer is read-only" message.
- Buffer start/end → safe no-op. Undo restores exactly the removed text and the prior cursor.

## No-regression

- Per-character movement/selection/deletion, Shift+Arrow selection, Copy/Cut/Paste, undo/redo, and all
  other keys are unchanged. No new dependencies. Word boundaries match double-click selection.
