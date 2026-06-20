# Contract: Interactive scrollbars

Behavioral contract the tests assert against.

## Mapping helpers (`src/ui/scrollbar.rs`, pure)

- `thumb_span(track_len, content, viewport, pos) -> (start, len)`:
  - `content <= viewport` ⇒ `(0, track_len)` (full track; nothing to scroll).
  - else `len = max(1, track_len*viewport/content)`, `start ∈ [0, track_len-len]`, monotonic in `pos`.
- `pos_to_offset(track_len, content, viewport, click) -> offset`: clamped to `[0, content-viewport]`;
  `click` at track start → 0, at track end → max; monotonic.
- `hit_zone(...) -> Above | Thumb | Below` relative to the computed thumb span.

## Track click (page)

| Where | Effect |
|---|---|
| Above the thumb (vertical) / left (horizontal) | `offset = (offset - viewport).clamp(0, max)` |
| Below the thumb (vertical) / right (horizontal) | `offset = (offset + viewport).clamp(0, max)` |

Applied to the surface whose bar was clicked; bounded; thumb moves toward the click.

## Thumb drag

- Press on the thumb starts a drag (no immediate jump).
- Subsequent moves: `offset = pos_to_offset(cursor_along_track - track_start, …)` — proportional, bounded.
- Release ends the drag; later moves don't scroll.
- Editor drag: adjusts `scroll_offset` only — text cursor unchanged, no selection created/extended.

## Surfaces

Editor vertical bar, editor horizontal bar (non-wrap only), file browser, Help/About, encoding, plugin —
each interactive only when its bar is drawn (content overflows). Modal open ⇒ only the modal's bar is
interactive; the editor beneath is not.

## Ordering & no-regression

- A left **press** is tested against the active surface's scrollbar regions **before** the feature-017
  drag-anchor / editor click and before the modal entry/button handlers.
- A press **on a scrollbar** does not place the cursor, select an entry, or start a text selection.
- A press **off all scrollbars** behaves exactly as before (text click places cursor; text press-drag
  selects; dialog buttons/entries click; menu works).
- The wheel (023), keyboard scrolling, and dialog actions are unchanged.

## Bounds / resilience

- No over-scroll/underflow; clamp at both ends.
- No panic at any terminal size, when no bar is shown, when the thumb fills the track (no-op), on resize
  mid-drag, or on release outside the track/window.

## Coupling with feature 021

- After interaction the drawn thumb reflects the new offset (renderer reads the same offset the
  interaction wrote).
