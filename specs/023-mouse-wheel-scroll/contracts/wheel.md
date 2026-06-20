# Contract: Mouse-wheel scrolling

Behavioral contract the tests assert against. `step = 3`.

## Routing (one target per event; modal wins)

| Precondition | ScrollDown | ScrollUp |
|---|---|---|
| Help/About open | `help_scroll += step` (clamped to content by render) | `help_scroll = help_scroll.saturating_sub(step)` |
| Encoding select open | cursor `= (cursor + step).min(n-1)` | cursor `= cursor.saturating_sub(step)` |
| File browser open | `move_down(vis)` ×step | `move_up(vis)` ×step |
| Plugin manager open | cursor `= (cursor + step).min(n-1)` | cursor `= cursor.saturating_sub(step)` |
| Find/Replace open | no-op | no-op |
| none (editor) | `scroll_offset.0 = (off+step).min(content-1)` | `scroll_offset.0 = off.saturating_sub(step)` |

- Editor scroll is **viewport-only**: the cursor (`buffers[i].cursor`) is unchanged by a wheel event.
- Split view: the editor target buffer is chosen by the cursor column (left/right half).
- `content` = `total_visual_rows()` in soft-wrap, else `line_count()`.

## Bounds

- At the top, ScrollUp is a no-op (offset stays 0); at the bottom, ScrollDown clamps (offset stays
  `content-1`). Same for list cursors at `0` / `n-1`. No panic, no over-scroll.
- Content that fits the viewport: editor offset stays 0 (clamp); no scrollbar appears (feature 021).

## No-regression

- A wheel event returns before the Press/Left click path — so click placement, dialog/button clicks, and
  press-drag selection (feature 017) are never triggered by the wheel.
- Conversely, left-press / drag / keyboard scrolling behave exactly as before (the wheel block only
  matches `ScrollUp`/`ScrollDown`).
- Wheel over a non-scrollable area with no modal and an editor that fits: no state change.

## Scrollbar coupling (feature 021)

- After a wheel scroll, any visible scrollbar reflects the new offset (it reads the same `scroll_offset` /
  `help_scroll` / `FileBrowser.scroll`). No separate update needed.
