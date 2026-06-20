# Render Contract: Find/Replace dialog field boxes

The "interface" this feature exposes is the on-screen rendering of the Find and Replace overlays.
This contract specifies the observable output that tests assert against.

## Find dialog (`Ctrl+F` / Search ▸ Find)

Rendered as a centered/top-anchored bordered modal containing, top to bottom:

1. A label row: `Find what:` (text label above the box).
2. A 3-row bordered input box (`┌─…─┐` / `│ … │` / `└─…─┘`) whose middle row shows the `query` text.
   - When the field is focused (always, in Find mode), a caret glyph `▏` marks the insertion point.
   - When `query` is wider than the inner box width, the text is right-anchored so the caret and the
     trailing characters remain visible.
3. An options row showing the toggles, e.g. `[ ] Case(Alt+C)  [ ] Wrap(Alt+A)  [ ] Regex(Alt+R)  [ ] Word(Alt+W)`
   (an `x` replaces the space when a toggle is on).
4. The match-count indicator near the Find field/label: empty when `query` is empty; `i/N` when there
   is an active match; `N matches` or `not found` otherwise.
5. A hint row: `Enter find · F3/F2 next/prev · Esc close`.

**Title**: ` Find `.

## Replace dialog (`Ctrl+H` / Search ▸ Find Replace)

Same as Find, plus a second labeled bordered input box:

1. `Find what:` label + box (as above).
2. `Replace with:` label + box showing `replacement`.
3. The caret `▏` appears in **only** the focused box (`focus == Query` or `focus == Replacement`);
   the unfocused box shows its text with no caret.
4. Options row (same toggles).
5. Hint row: `Enter replace · Ctrl+A all · Tab field · F3/F2 next/prev · Esc close`.

**Title**: ` Replace `.

## Invariants (assertable)

- **C-1**: Box-drawing border characters (`┌`, `┐`, `└`, `┘`, `─`, `│`) are present around each field.
- **C-2**: The caret glyph `▏` is present in the focused field and absent from the unfocused field.
- **C-3**: The field label text (`Find what:` / `Replace with:`) is present.
- **C-4**: All four option labels (`Case`, `Wrap`, `Regex`, `Word`) are present with `[ ]`/`[x]`.
- **C-5**: The dialog width is clamped to the terminal width and height to the terminal height; on a
  terminal smaller than the dialog's natural size the render produces no panic and does not draw
  outside the frame.
- **C-6**: For text longer than the inner box width, the caret remains visible (right-anchored slice).
- **C-7**: No boxed buttons / focus ring are added (scope guard for issue #38): the hint row and
  field-only focus model are retained.

## Behavioral pass-through (unchanged; covered by existing tests)

- Typing/editing mutate `query` / `replacement` and `caret` (dialog.rs unit tests).
- `Tab` switches `focus` in Replace mode only (dialog.rs unit tests).
- `Alt+C/A/R/W` toggle options; `Enter`/`Ctrl+A`/`F3`/`F2`/`Esc` drive search/replace/navigation/close
  (existing app/search tests).
