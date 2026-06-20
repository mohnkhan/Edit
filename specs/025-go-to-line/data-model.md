# Phase 1 Data Model: Go to Line

UI-state only — no persistence/config.

## New state (in `src/app.rs`)

| Field | Type | Role |
|---|---|---|
| `pending_goto_line` | `Option<String>` | `Some(digits)` while the prompt is open; `None` otherwise |

## New action (in `src/input/keymap.rs`)

| Action | Binding | Menu |
|---|---|---|
| `Action::GoToLine` | `Ctrl+G` | Search ▸ "Go to Line" (mnemonic `g`) |

## Behavior mapping

```
open:    Action::GoToLine, no other modal open → pending_goto_line = Some(String::new())
type:    InsertChar(d) where d.is_ascii_digit() → push d
edit:    Backspace → pop last char
cancel:  MenuClose (Esc) → pending_goto_line = None  (cursor unchanged)
confirm: InsertNewline →
           let s = pending_goto_line.take();
           if let Ok(n) = s.parse::<usize>() {           // empty / non-numeric → no move
               let count = rope.line_count();
               let line1 = n.clamp(1, count);            // 0/below → 1; > count → count
               set_cursor_lc(line1 - 1, 0);              // column 1 + scroll into view
           }
other:   consumed (buffer not modified)
```

Oversized input that overflows `usize` is treated as the last line (parse error → clamp/last, or saturate
before clamp). Empty buffer ⇒ `line_count() == 1` ⇒ always line 1.

## Modal integration

`pending_goto_line.is_some()` is added to the existing modal guards so the editor ignores
clicks/wheel/scrollbar gestures while the prompt is open, and only one modal is shown at a time.

## Invariants

- The prompt never modifies buffer text.
- The resulting cursor line is always in `[0, line_count-1]`; the target line is visible after the jump.
- No panic on empty buffer, oversized input, or tiny terminal.
