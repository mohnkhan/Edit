# Phase 0 Research: Go to Line

## Existing machinery (from code survey)

- `App::set_cursor_lc(line, gcol)` (`src/app.rs:1894`) sets the cursor and calls `clamp_scroll`, which
  scrolls the cursor's line into the viewport (both wrap and non-wrap). So a jump = `set_cursor_lc(line-1,
  0)` — cursor at line start + scrolled into view, for free.
- `rope.line_count()` gives the clamp bound.
- The encoding dialog is the template for a simple modal text entry: `pending_encoding_select:
  Option<usize>` opened by an action, handled by a dedicated `handle_action` intercept that consumes all
  keys and returns. Go to Line mirrors this with `Option<String>`.
- `src/input/keymap.rs`: `Action` enum + `default_map` (`map.insert("Ctrl+G", …)`) + `action_from_string`
  + menu mapping. `Ctrl+G` is currently unbound (only Ctrl+F=Find etc.).
- `src/ui/menubar.rs::SEARCH_MENU`: array of `MenuItem { label, action, mnemonic }`.
- `handle_mouse_event` ignores the editor when a modal is open (a list of `pending_*` checks); the wheel
  (023) and scrollbar (024) likewise route to the modal or skip the editor.

## Decision 1 — Reuse `set_cursor_lc` for the jump

**Decision**: On confirm, `set_cursor_lc(target_line0, 0)` where `target_line0 = clamp(n,1,line_count)-1`.

**Rationale**: It already moves the cursor to column 1 and scrolls the line into view via `clamp_scroll`
(satisfies FR-003/SC-003) with no new scroll code.

## Decision 2 — Prompt state as `Option<String>` (encoding-dialog pattern)

**Decision**: `pending_goto_line: Option<String>` holds the in-progress digits. Opened by
`Action::GoToLine` when no other modal is open; a dedicated `handle_action` intercept handles
`InsertChar(d)` (push if `d.is_ascii_digit()`), `Backspace` (pop), `InsertNewline` (confirm), `MenuClose`
(cancel), and consumes everything else.

**Rationale**: Matches the established modal-input pattern (encoding select), so routing and "buffer not
edited while open" come naturally; minimal new surface.

## Decision 3 — Clamp + invalid handling

**Decision**: On Enter, parse the field as a number; empty or non-parsing → close with no movement;
otherwise clamp to `[1, line_count]` and jump. The field already rejects non-digits on input, so parsing
only fails on empty.

**Rationale**: FR-004/FR-006/SC-002 — predictable, no surprise jumps; oversized input clamps to the last
line without overflow (parse to `usize`, saturating; if it overflows `usize`, treat as last line).

## Decision 4 — Treat as a modal in mouse/wheel/scrollbar guards

**Decision**: Add `pending_goto_line.is_some()` to the modal conditions so the editor doesn't receive
clicks/wheel/scrollbar gestures while the prompt is open, and so only one modal shows at a time.

**Rationale**: Consistency with features 020/021/023/024 (FR-007); avoids input leaking to the editor.

## Decision 5 — Shortcut + menu placement

**Decision**: `Ctrl+G` (unbound today) + a `SEARCH_MENU` item "Go to Line" (mnemonic `g`), beside
Find/Replace.

**Rationale**: DOS EDIT.COM placed Go-to-Line under Search; `Ctrl+G` is the common shortcut.

## Testing strategy (Constitution V — TDD)

- **Unit**: clamp helper (`n` clamped to `[1, line_count]`, including `0`→1 and `>count`→count and
  overflow→count); digit-only field edit.
- **Integration**: `Ctrl+G` opens the prompt; typing digits + Enter moves the cursor to that line's start
  and scrolls it into view (`scroll_offset` adjusts); over-range clamps to last; `0`/below clamps to
  first; Esc leaves the cursor unchanged; empty/non-numeric Enter doesn't move; while open, an editor
  key (e.g. a letter) does not modify the buffer.

## No open clarifications

Defaults fixed (1-based; column 1; Ctrl+G; Search menu; invalid = no-op). No NEEDS CLARIFICATION remains.
