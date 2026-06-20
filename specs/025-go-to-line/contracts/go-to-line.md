# Contract: Go to Line

Behavioral contract the tests assert against.

## Open / input / confirm / cancel

| Input | Precondition | Effect |
|---|---|---|
| `Action::GoToLine` (Ctrl+G / Search ▸ Go to Line) | no other modal open | open the prompt (`pending_goto_line = Some("")`) |
| `InsertChar(d)` | prompt open, `d` is `0`–`9` | append `d` to the entry |
| `InsertChar(c)` | prompt open, `c` not a digit | ignored (field rejects non-digits) |
| `Backspace` | prompt open, entry non-empty | remove the last digit |
| `InsertNewline` (Enter) | prompt open, entry parses to `n` | jump to line `clamp(n, 1, line_count)` (cursor at column 1, scrolled into view); close |
| `InsertNewline` (Enter) | prompt open, entry empty/non-numeric | close; cursor unchanged |
| `MenuClose` (Esc) | prompt open | close; cursor unchanged |
| any other action | prompt open | consumed; buffer not modified |

## Clamp

- `n > line_count` → line `line_count` (last). `n < 1` (e.g. `0`) → line `1` (first). Oversized/overflowing
  input → last line (no panic).
- Empty buffer (`line_count == 1`) → always line 1.

## Cursor / view

- The cursor lands at column 1 (line start) of the target line.
- The target line is scrolled into the viewport (via the existing `clamp_scroll`).

## Modal / no-regression

- While the prompt is open it captures input — digits/Backspace/Enter/Esc only; the buffer is never
  edited, and editor shortcuts do not act on the buffer.
- Only one modal is open at a time; the editor ignores clicks/wheel/scrollbar gestures while the prompt
  is open.
- Editing, find/replace, and all other dialogs are unchanged; Go to Line only moves the cursor/viewport.

## Resilience

- No panic on any terminal size, empty buffer, or oversized input; the overlay renders width-correctly.
