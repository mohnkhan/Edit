# Phase 0 Research: Interactive scrollbars

## Existing machinery (from code survey)

- `src/ui/scrollbar.rs` (feature 021): `render_vertical/render_horizontal` (draw via ratatui `Scrollbar`,
  only on overflow), `is_needed`. ratatui's `Scrollbar` does **not** expose thumb position or hit-testing.
- `src/ui/mod.rs`: `editor_panes(area, soft_wrap) -> (text, vbar, hbar?)` (private) gives the editor bar
  rects; `render_editor_scrollbars` feeds content/viewport/pos (lines or `total_visual_rows`;
  visible-line max width for the h-bar).
- `src/ui/file_browser.rs`: list bar over `Rect::new(inner_left, list_top, iw, list_rows)` when entries
  overflow; metrics `entries.len()` / `list_rows` / `scroll`.
- `render_help_overlay`: bar over `Rect::new(dx+1, dy+1, dw-2, body_rows)`; metrics total lines /
  body_rows / `help_scroll`. Encoding & plugin dialogs: similar bar over their interior, metrics from
  entry count / body rows / cursor.
- `src/app.rs::handle_mouse_event`: normalize → left-Drag (feature-017 selection via `drag_anchor`) →
  wheel (023) → Press/Left guard → modal button blocks → file-browser entry hit-test → editor click.
- Editor viewport scroll already exists: `wheel_scroll_editor(buf_idx, down, step)` (023) +
  `buffers[].scroll_offset`.

## Decision 1 — Own thumb/offset math in `scrollbar.rs`

**Decision**: Add pure helpers:
- `thumb_span(track_len, content, viewport, pos) -> (start, len)` — thumb length `≈ max(1, track*viewport/content)`, start `≈ (track-len)*pos/(content-viewport)`, clamped.
- `pos_to_offset(track_len, content, viewport, click) -> offset` — inverse: map a 0-based click position
  along the track to a scroll offset, clamped to `[0, content-viewport]`.
- A classification `hit_zone(track_len, content, viewport, pos, click) -> Above | Thumb | Below`.

**Rationale**: ratatui exposes no hit-testing, so we compute our own; keeping it pure makes it unit-test
the math directly and lets render + hit-test agree (both derive the thumb from the same formula). Match
ratatui's "fills the track when content fits / min length 1" behavior so the drawn and computed thumbs
align closely (exactness isn't required — classification + proportional map is what matters).

## Decision 2 — `scrollbar_regions()` for the active surface

**Decision**: One App method returns the interactive bars for the currently-active surface (modal wins,
else the editor pane under the cursor): each region = `{ rect, axis, content, viewport, offset, kind }`
where `kind` identifies how to apply a new offset (editor buf_idx / file browser / help / encoding /
plugin). The press-check and drag-apply both use it.

**Rationale**: Centralizes the geometry (mirrors feature-020's button approach) so drawn == interactive
and there's one place to add/adjust a surface. Avoids scattering rect math across the mouse handler.

## Decision 3 — Press-check ordered before selection/entry/click

**Decision**: On a left **press**, before the feature-017 drag-anchor/editor-click and before the modal
entry/button handlers, check whether the press is inside an active scrollbar region. If on the **track**
(above/below thumb) → page by one viewport (apply offset, return). If on the **thumb** → start a
`scrollbar_drag` (record kind + axis + track geometry) and return. Bars live in reserved columns/rows
that don't overlap buttons/text/entries, so checking first is safe and prevents a bar press from
selecting an entry or placing the cursor.

**Rationale**: The file-browser `hit_test` classifies by row only, so a bar-column press would otherwise
select an entry; the editor press would set a drag anchor. Checking the bar first resolves both (FR-006).

## Decision 4 — `scrollbar_drag` state guards feature-017

**Decision**: Add `scrollbar_drag: Option<ScrollbarDrag>` to App. While `Some`, the left-**Drag** branch
maps the cursor position along the stored track to an offset via `pos_to_offset` and applies it (editor =
viewport-only, cursor untouched), instead of extending a text selection. **Release** clears it. The
feature-017 selection drag only runs when `scrollbar_drag` is `None` and a `drag_anchor` exists.

**Rationale**: Cleanly separates a scrollbar drag from a text-selection drag using the gesture's starting
region; release-outside just clears state (no panic).

## Decision 5 — Track-click pages; editor viewport-only

**Decision**: Track click pages by one viewport (per user decision), reusing the viewport size already
known per surface (editor `viewport_height`/content width; list `list_rows`/body rows). Editor
click/drag both adjust `scroll_offset` only (reuse the feature-023 clamp), never the cursor.

## Testing strategy (Constitution V — TDD)

- **Unit** (`scrollbar.rs`): `thumb_span` (fills track when content≤viewport; min len 1; start within
  bounds; monotonic in pos), `pos_to_offset` (click at top→0, bottom→max, clamped, monotonic), `hit_zone`.
- **Integration** (`scrollbar_interaction.rs`): synth a press on the editor v-bar track below the thumb →
  `scroll_offset.0` increases ~one viewport, cursor unchanged; press on thumb + drag down → proportional
  increase; press-drag in the text body still selects; press on the bar does not select; file-browser /
  Help bar clicks scroll those; modal open → editor not scrolled.

## No open clarifications

Both decisions (track-click pages; editor thumb draggable, viewport-only) are fixed. No NEEDS
CLARIFICATION remains.
