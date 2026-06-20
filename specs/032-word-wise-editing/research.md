# Research: Word-wise navigation, selection, and deletion

No NEEDS CLARIFICATION; decisions grounded in the existing code.

## D1 — Word boundary definition (reuse `grapheme_class`)

**Finding**: Feature 030 added `App::grapheme_class(g) -> u8` (0 word = alphanumeric/`_`, 1 whitespace,
2 other) for double-click selection.

**Decision**: Reuse it for word-wise movement/deletion so all "word" semantics in the editor are
identical. No second definition.

**Alternatives**: `unicode-segmentation`'s `unicode_word_indices` — rejected: would diverge from
double-click and split punctuation differently; the class-run rule is consistent and already tested.

## D2 — `next_word_pos(dir)` (the one new computation)

**Decision**: Compute the target `(line, gcol)` over the active buffer's graphemes:
- **Right**: let `len` = grapheme count of the cursor line. If `gcol >= len`: target `(line+1, 0)` if a
  next line exists, else stay. Else: `i = gcol`; consume the run of `class(graphemes[i])` (`i` advances
  while same class), then consume a following whitespace run; target `(line, i)`.
- **Left**: if `gcol == 0`: target `(prev_line, prev_len)` if a previous line exists, else stay. Else:
  `i = gcol`; step `i -= 1`; consume a preceding whitespace run (`while i>0 && class(i-1)==ws`), then
  consume the preceding token run (`while i>0 && class(i-1)==class(i-1 initial token)`); target
  `(line, i)`.

This yields "start of next/previous token," matching the spec and common editors; line-crossing handled
explicitly; buffer ends are no-ops (target == cursor).

**Rationale**: Pure, grapheme-based, multibyte-safe; small and unit-testable in isolation.

## D3 — Movement & selection wrappers

**Decision**: `move_word(dir)` = clear selection + `set_cursor_lc(next_word_pos)` (mirrors `move_cursor`).
`move_word_selecting(dir)` = anchor = `selection_anchor_or_cursor()`, `set_cursor_lc(next_word_pos)`,
`update_selection_to_cursor(anchor)` (mirrors `move_cursor_selecting`). Both reuse the existing
scroll-into-view in `set_cursor_lc`/`clamp_scroll`.

## D4 — Deletion as one undo step

**Decision**: `delete_word(dir)`: `deny_if_readonly()` guard first (read-only message, no-op). If there is
an active selection, delete it (existing behavior). Otherwise compute `target = next_word_pos(dir)`; if
`target == cursor`, no-op; else set `selection` to span cursor↔target and call the existing
`delete_selection()` (which removes the range char-safely, pushes a single `EditOp::Delete`, and places
the cursor at the range start). This guarantees one undo step and reuses the proven deletion path.

**Alternatives**: A bespoke range delete — rejected: `delete_selection` already handles undo, cursor, and
multibyte correctly.

## D5 — Keybindings

**Finding**: `key_to_string` already encodes Ctrl/Shift + Left/Right/Backspace/Delete (e.g.
`"Ctrl+Left"`, `"Ctrl+Shift+Right"`, `"Ctrl+Backspace"`, `"Ctrl+Delete"`); none are bound.

**Decision**: Add to `default_map`: `Ctrl+Left→MoveWordLeft`, `Ctrl+Right→MoveWordRight`,
`Ctrl+Shift+Left→SelectWordLeft`, `Ctrl+Shift+Right→SelectWordRight`, `Ctrl+Backspace→DeleteWordLeft`,
`Ctrl+Delete→DeleteWordRight`. Add the six `Action` variants + `action_from_str` arms. Terminals that
don't report Ctrl+Arrow distinctly just don't match (graceful degradation); per-character keys unaffected.

## Testing approach

TDD. Unit (`app.rs`): `next_word_pos` over `"foo  bar_baz, café"` and multibyte, mid-word/in-space, at
line ends (crossing), at buffer ends (no-op); `move_word` clears selection; `move_word_selecting` builds
the right selection; `delete_word` removes the expected range in one undo step, deletes an active
selection, no-ops at ends, and is blocked read-only. Unit (`keymap.rs`): the six bindings map and
existing bindings are unchanged. Integration (`tests/integration/word_editing.rs`): drive the actions and
assert cursor/selection/buffer via `selection_text()` and rope content.
