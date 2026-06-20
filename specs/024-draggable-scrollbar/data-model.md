# Phase 1 Data Model: Interactive scrollbars

UI-state only. The "data" is one drag-state field plus a derived per-surface region descriptor.

## New state: `ScrollbarDrag` (in `src/app.rs`)

| Field | Type | Role |
|---|---|---|
| `kind` | enum (EditorV{buf_idx} / EditorH{buf_idx} / FileBrowser / Help / Encoding / Plugin) | which surface/axis the drag controls + how to apply an offset |
| `track_start` | u16 | first track cell along the axis (row for vertical, col for horizontal) |
| `track_len` | u16 | track length in cells |
| `content` | usize | scrollable extent |
| `viewport` | usize | visible extent |

App gains `scrollbar_drag: Option<ScrollbarDrag>` (None except between thumb-press and release).

## Derived: scrollbar region descriptor

`scrollbar_regions()` returns, for the currently-active surface (modal wins, else editor pane under the
cursor), zero or more regions:

| Field | Meaning |
|---|---|
| `rect` | the drawn bar rect (the reserved column/row) |
| `axis` | Vertical / Horizontal |
| `content`, `viewport`, `offset` | scroll metrics (same values the renderer used) |
| `kind` | how to read/write the offset (editor buf_idx / file browser / help / encoding / plugin) |

A region exists only when the bar is actually drawn (content > viewport), so the interactive region
equals the drawn one.

## Mapping math (pure, in `src/ui/scrollbar.rs`)

```
max_off            = content.saturating_sub(viewport)
thumb_len          = max(1, round(track_len * viewport / content))           # ≤ track_len
thumb_start(pos)   = round((track_len - thumb_len) * pos / max_off)          # 0..track_len-thumb_len
hit_zone(click)    = Above if click < thumb_start; Below if click >= thumb_start+thumb_len; else Thumb
pos_to_offset(c)   = round(max_off * (c - thumb_len/2) / (track_len - thumb_len)) clamped [0, max_off]
```

- Track click (Above/Below) → page: `offset = (offset ∓ viewport).clamp(0, max_off)`.
- Thumb drag → `offset = pos_to_offset(cursor_along_track - track_start)`.

## Per-kind apply

| kind | offset source/target |
|---|---|
| EditorV{i} | `buffers[i].scroll_offset.0` (clamp via feature-023 helper); cursor untouched |
| EditorH{i} | `buffers[i].scroll_offset.1` (clamp to max content width) |
| FileBrowser | `file_browser.scroll` (then clamp selection visible, or set selection within window) |
| Help | `help_scroll` (render clamps) |
| Encoding / Plugin | the list cursor index, clamped `[0, n-1]` |

## Invariants

- A scrollbar gesture never moves the editor text cursor and never starts/extends a text selection.
- Offsets stay in `[0, max_off]`; no panic on tiny thumb (thumb == track → no-op), resize, or
  release-outside.
- `scrollbar_drag` is cleared on button release; while it is `Some`, the feature-017 selection drag does
  not run.
- The interactive region equals the drawn bar (both derive from the same geometry + thumb formula).
