# Feature Specification: Mouse-wheel scrolling (app-wide)

**Feature Branch**: `023-mouse-wheel-scroll`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "Mouse-wheel scrolling, app-wide (feature 023). Wheel (ScrollUp/ScrollDown)
events are normalized but dropped everywhere because the mouse handler only acts on left-button press;
the editor, file browser, Help/About, and encoding/plugin dialogs all ignore the wheel. Route wheel
events to the surface under the cursor (or the open modal/overlay). Decisions: editor wheel scrolls the
viewport only (cursor may go off-screen); 3 lines per notch. Wheel handling only — no change to existing
keyboard/click/drag behavior."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scroll the editor with the wheel (Priority: P1)

While editing a file taller than the window, the user rolls the mouse wheel and the editor view scrolls
up or down. The wheel scrolls the viewport (the text shown) without moving the cursor; the feature-021
vertical scrollbar tracks the new position. Scrolling stops cleanly at the top and bottom of the file.

**Why this priority**: The editor is the primary surface and the most common place to want wheel
scrolling on a large file; it's the headline of the report.

**Independent Test**: Open a file with more lines than the window; roll the wheel down → the first
visible line advances by the scroll step; roll up → it retreats; at the very top, rolling up does
nothing (no underflow); the cursor position is unchanged by scrolling.

**Acceptance Scenarios**:

1. **Given** a file taller than the viewport, **When** the user rolls the wheel down over the editor,
   **Then** the view scrolls down by the configured step and the vertical scrollbar thumb moves down.
2. **Given** the view is scrolled, **When** the user rolls the wheel up, **Then** the view scrolls up by
   the step.
3. **Given** the view is at the top (or bottom), **When** the user rolls the wheel up (or down) further,
   **Then** the view stays at the top (or bottom) — no over-scroll, no panic.
4. **Given** the user scrolls with the wheel, **When** the view moves, **Then** the cursor position is
   not changed by the scroll itself.

### User Story 2 - Scroll lists and overlays with the wheel (Priority: P1)

The wheel also scrolls the other scrollable surfaces: the file browser listing, the Help/About screens,
and the encoding/plugin list dialogs. Rolling the wheel moves through their content the same direction as
the wheel, bounded by their start/end.

**Why this priority**: The user reported the bug in Help; the file browser and dialogs share the same
gap. Consistent wheel behavior across every scrollable surface is the expected outcome.

**Independent Test**: Open the file browser on a long directory and roll the wheel → the listing scrolls
(the selection/visible window advances); open Help with overflowing content and roll the wheel → the
cheat sheet scrolls and its scrollbar tracks; both stop at their ends.

**Acceptance Scenarios**:

1. **Given** a file browser with more entries than fit, **When** the user rolls the wheel, **Then** the
   listing scrolls in the wheel's direction, bounded at the first/last entry.
2. **Given** the Help or About overlay with overflowing content, **When** the user rolls the wheel,
   **Then** the content scrolls and its scrollbar reflects the new position.
3. **Given** any of these surfaces at its scroll limit, **When** the user keeps rolling, **Then** it
   stays at the limit (no over-scroll, no panic).

### User Story 3 - Existing mouse/keyboard behavior is unchanged (Priority: P1)

Adding wheel handling does not change any existing interaction: left-click placement and dialog/button
clicks, click-drag text selection (feature 017), keyboard navigation and scrolling, and all dialog
actions behave exactly as before. Only previously-ignored wheel events now do something.

**Why this priority**: A regression in click/drag/keyboard would make the feature a net negative;
preserving current behavior is as important as adding the wheel.

**Independent Test**: Exercise a left-click in the editor, a button click in a dialog, a press-drag
selection, and keyboard PgUp/PgDn — all behave identically to before; only the wheel is newly active.

**Acceptance Scenarios**:

1. **Given** any existing mouse gesture (click, click-on-button, press-drag-release), **When** performed,
   **Then** it behaves exactly as before this feature.
2. **Given** the wheel is rolled while no scrollable surface is relevant (e.g. over the menu bar/status
   bar), **When** the event arrives, **Then** nothing happens (the event is ignored, no panic).

### Edge Cases

- **At the top/bottom (or first/last entry)**: further wheel input in that direction is a no-op (clamp).
- **Content fits entirely** (no overflow): the wheel is a no-op; no scrollbar appears.
- **Wheel over a non-scrollable region** (menu bar, status bar, dialog chrome): ignored, no panic.
- **Modal open**: when a dialog/overlay is open, the wheel scrolls that surface (the modal), not the
  editor underneath.
- **Soft-wrap editor**: wheel scrolls by visual rows consistently with keyboard scrolling.
- **Split view**: the wheel scrolls the pane under the cursor.
- **Cursor off-screen after scrolling**: allowed (viewport-only scroll); the next cursor move/keypress
  brings it back into view as it does today.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Mouse-wheel up/down events MUST scroll the relevant scrollable surface; they MUST no longer
  be silently discarded.
- **FR-002**: Over the main editor, the wheel MUST scroll the viewport vertically by the configured step
  without moving the cursor.
- **FR-003**: When a modal/overlay is open (file browser, Help/About, encoding select, plugin manager),
  the wheel MUST scroll that surface rather than the editor beneath it.
- **FR-004**: The file browser MUST scroll its listing on wheel input; Help/About MUST scroll their
  content; the encoding/plugin lists MUST scroll their content.
- **FR-005**: Wheel scrolling MUST be bounded — at the top/bottom (or first/last item) further wheel
  input in that direction is a no-op (no over-scroll, no underflow/overflow, no panic).
- **FR-006**: Each wheel notch MUST scroll by a small, consistent step (default 3 lines/rows).
- **FR-007**: Where a scrollbar exists (feature 021), it MUST reflect the position after wheel scrolling.
- **FR-008**: Wheel handling MUST NOT change any existing mouse behavior (left-click placement, dialog/
  button clicks, press-drag text selection) or any keyboard navigation/scrolling or dialog action.
- **FR-009**: A wheel event over a non-scrollable region MUST be ignored without side effects.
- **FR-010**: Wheel handling MUST NOT panic on any terminal size, at scroll limits, with empty content,
  or in split view / soft-wrap.

### Key Entities

- **Wheel event**: a normalized scroll-up/scroll-down input carrying the cursor position, routed to the
  surface under the cursor (or the open modal).
- **Scrollable surface**: editor viewport, file-browser listing, Help/About content, encoding/plugin
  list — each with an existing scroll offset that the wheel adjusts within bounds.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Rolling the wheel scrolls the editor, the file browser, Help/About, and the encoding/plugin
  dialogs — every scrollable surface responds.
- **SC-002**: Wheel scrolling is bounded: it never scrolls past the first/last line or entry and never
  panics, across terminal sizes, empty content, split view, and soft-wrap.
- **SC-003**: 100% of existing mouse gestures (click, button click, press-drag selection) and keyboard
  scrolling behave identically to before (zero regression), verified by tests.
- **SC-004**: Where a scrollbar is shown, its thumb position matches the post-wheel scroll offset.

## Assumptions

- **Editor wheel = viewport-only** (per the stated default): the wheel changes the scroll offset, not the
  cursor; the cursor may scroll off-screen until the next cursor-moving key/click (standard editor
  behavior).
- **Scroll step = 3 lines/rows per notch** (per the stated default), applied per wheel event.
- **Routing**: the wheel targets the open modal/overlay if one is open; otherwise the editor pane under
  the cursor. Wheel over the menu/status bar or dialog chrome is ignored.
- **List dialogs**: for the file browser, wheel advances the listing window (consistent with its existing
  up/down scrolling); for Help/encoding/plugin, the wheel adjusts their existing scroll/cursor offset.
- **Scope**: wheel handling only — no new actions, no change to keyboard/click/drag, no new config key
  (the step is a fixed constant).
- Builds on the normalized wheel events already produced by the input layer and the per-surface scroll
  state from features 012/018/021.
