# Phase 0 Research: Mouse-wheel scrolling (app-wide)

## Existing machinery (from code survey)

- `src/input/mouse.rs` already maps crossterm `MouseEventKind::ScrollUp/ScrollDown` to
  `NormalizedMouseKind::ScrollUp/ScrollDown` with `col`/`row` (unit-tested).
- `src/app.rs::handle_mouse_event` (≈3282): normalizes, handles a left **drag** (feature-017 selection),
  then `if ev.kind != Press || button != Left { return Ok(()) }` — which **drops every wheel event**.
  After that come the modal/button click branches and the editor click.
- Per-surface scroll state already exists: editor `buffers[i].scroll_offset.0`; `viewport_height()`;
  `wrap_cache.total_visual_rows()`; `FileBrowser::move_up/move_down(visible_rows)` (scroll via
  `ensure_visible`) + `visible_rows(area)`; `help_scroll` (clamped by `render_help_overlay`);
  `pending_encoding_select` (cursor index); `plugin_manager_cursor`.

## Decision 1 — Wheel block placed before the Press/Left guard

**Decision**: Insert a `match ev.kind { ScrollUp | ScrollDown => … }` block immediately after the drag
block and **before** the `Press/Left` early-return, returning `Ok(())` after handling.

**Rationale**: That guard is exactly why the wheel is ignored; handling above it is the minimal, correct
fix and keeps all existing click/drag/press logic untouched below.

## Decision 2 — Modal-wins routing

**Decision**: If an overlay/modal is open, the wheel scrolls it (precedence: Help/About → encoding →
file browser → plugin manager; Find/Replace ignored). Otherwise the wheel scrolls the editor pane under
the cursor.

**Rationale**: Matches the existing modal precedence in `handle_action`/`handle_mouse_event`; prevents
the editor scrolling under an open dialog (FR-003).

## Decision 3 — Editor viewport-only scroll, clamped

**Decision**: `scroll_offset.0 = (scroll_offset.0 ± step).clamp(0, max)` where
`max = content_rows.saturating_sub(1)`, `content_rows = total_visual_rows()` (soft-wrap) or
`rope.line_count()` (non-wrap). Cursor is not touched. In split view, choose the buffer by the cursor
column (left half = `buffers[0]`, right half = the right pane's buffer).

**Rationale**: The stated default (viewport-only, 3 lines); clamping to `content_rows-1` keeps at least
one line visible and prevents over-scroll. The feature-021 scrollbar reads `scroll_offset`, so it tracks
for free. `max` via `content_rows-1` is a safe bound that never panics even if `viewport_height` is large.

**Alternatives**: move the cursor by N (rejected — user default is viewport-only); clamp by
`content-viewport` (slightly tighter, but `content-1` is simpler and equally panic-free).

## Decision 4 — File browser: move selection by step

**Decision**: For the file browser, call `move_up`/`move_down(visible_rows)` `step` times.

**Rationale**: Reuses the tested navigation that scrolls via `ensure_visible` and keeps the highlight
valid; consistent with arrow-key behavior and avoids a detached scroll offset that a later keypress would
snap back. Bounds are already handled by `move_up/move_down` (no wrap).

## Decision 5 — Encoding / plugin lists: clamp the cursor by step

**Decision**: Move `pending_encoding_select` / `plugin_manager_cursor` by ±step, **clamped** to
`[0, n-1]` (not wrapped). Help/About: `help_scroll ± step` (render already clamps the upper bound).

**Rationale**: These lists render all entries from the cursor/scroll; nudging the cursor by the step is
the natural wheel mapping; clamping (vs the keyboard's wrap) matches wheel expectations.

## Decision 6 — Fixed step constant (no config)

**Decision**: `const WHEEL_STEP: usize = 3;` in `src/app.rs`. No config key.

**Rationale**: YAGNI; 3 is the conventional notch step. A config key can be a later follow-up if asked.

## Testing strategy (Constitution V — TDD)

- **Unit** (`src/app.rs`): editor wheel scroll increments/decrements `scroll_offset.0` by the step,
  clamps at 0 and at the bottom, leaves the cursor unchanged; soft-wrap uses the visual-row bound.
- **Integration** (`tests/integration/mouse_wheel.rs`): synthesize `MouseEventKind::ScrollUp/ScrollDown`
  and assert: editor scrolls (cursor unchanged, bounded); file browser listing scrolls; Help scrolls
  (`help_scroll` changes, bounded at 0); a wheel with a modal open scrolls the modal, not the editor;
  existing click + press-drag selection still behave (no regression).

## No open clarifications

Defaults are fixed (viewport-only; step 3). No `NEEDS CLARIFICATION` remains.
