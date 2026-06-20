# Feature Specification: Interactive (clickable + draggable) scrollbars

**Feature Branch**: `024-draggable-scrollbar`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "Make the feature-021 scrollbars interactive — click the track to page toward
the click, drag the thumb to scroll proportionally. Applies to the editor (vertical + horizontal) and the
list/overlay bars. Editor drag scrolls the viewport only (cursor unchanged), and must not conflict with
text drag-selection. Scope: scrollbar mouse interaction only. Decisions: track-click pages by one
viewport; the editor vertical thumb is draggable."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Click the track to page (Priority: P1)

When a scrollbar is shown, the user clicks the track above or below the thumb and the view pages toward
the click: clicking above the thumb scrolls back by one viewport, below scrolls forward by one viewport.
This applies to the editor's vertical bar (and the horizontal bar's left/right in non-wrap mode) and to
the file browser, Help/About, and encoding/plugin list bars.

**Why this priority**: A clickable track is the most common scrollbar interaction and gives mouse users
fast, coarse navigation; it's the core of making the bars interactive.

**Independent Test**: With content taller than the view, click the track below the thumb → the view
scrolls forward by ~one viewport and the thumb moves down; click above the thumb → it scrolls back; at
the ends, clicking past the limit clamps.

**Acceptance Scenarios**:

1. **Given** a visible vertical scrollbar, **When** the user clicks the track below the thumb, **Then**
   the view scrolls forward by one viewport (bounded) and the thumb moves toward the click.
2. **Given** a visible vertical scrollbar, **When** the user clicks the track above the thumb, **Then**
   the view scrolls back by one viewport (bounded).
3. **Given** the editor in non-wrap mode with a horizontal scrollbar, **When** the user clicks left/right
   of the horizontal thumb, **Then** the view pages left/right by one viewport width (bounded).
4. **Given** any surface with a scrollbar (file browser, Help/About, encoding, plugin), **When** the user
   clicks its track, **Then** that surface pages toward the click.

### User Story 2 - Drag the thumb to scroll (Priority: P1)

The user presses on the scrollbar thumb and drags along the track; the view scrolls proportionally so the
thumb follows the cursor. Releasing ends the drag. In the editor this scrolls the viewport only — the
cursor is not moved — and it never starts a text selection.

**Why this priority**: Dragging the thumb is the precise, expected way to scrub through a large file or
list; together with US1 it makes the bars fully interactive.

**Independent Test**: Press on the thumb and move the cursor toward the bottom of the track → the scroll
offset increases proportionally and the thumb tracks the cursor; release → further mouse moves no longer
scroll; in the editor the text cursor and any selection are unchanged.

**Acceptance Scenarios**:

1. **Given** the pointer is pressed on a scrollbar thumb, **When** the user moves the cursor along the
   track, **Then** the scroll offset updates proportionally to the cursor position (bounded at both ends).
2. **Given** a thumb drag is in progress, **When** the user releases the button, **Then** the drag ends
   and subsequent moves do not scroll.
3. **Given** an editor thumb drag, **When** it scrolls, **Then** the text cursor is not moved and no text
   selection is created or extended.

### User Story 3 - Existing mouse/keyboard behavior is unchanged (Priority: P1)

Adding scrollbar interaction does not change anything else: clicking in the text still places the cursor;
press-drag in the text body still selects (feature 017); the wheel (feature 023), keyboard scrolling,
dialog buttons (020), and dialog list/field clicks all behave exactly as before. Only presses/drags that
start on a scrollbar are newly meaningful.

**Why this priority**: A regression in text selection or click-to-place would be a serious usability
loss; this must be airtight given the press/drag overlap with feature 017.

**Independent Test**: Press-drag inside the text body → selection as before (no scroll). Click a dialog
button → activates as before. Roll the wheel → scrolls as before. Only a press starting on a scrollbar
scrolls instead of selecting.

**Acceptance Scenarios**:

1. **Given** a press-drag that starts inside the text body (not on a scrollbar), **When** performed,
   **Then** it selects text exactly as before (no scrolling).
2. **Given** a press that starts on a scrollbar, **When** it becomes a drag, **Then** it scrolls and does
   NOT start a text selection.
3. **Given** any existing click target (text, dialog button, list entry, menu), **When** clicked, **Then**
   it behaves exactly as before.

### Edge Cases

- **No scrollbar shown** (content fits): clicks in that region fall through to existing behavior (e.g.
  editor click places the cursor); there is no bar to interact with.
- **Click exactly on the thumb (not the track)**: begins a drag (no immediate page jump).
- **Drag beyond the track ends**: the offset clamps at first/last; no over-scroll, no panic.
- **Release outside the track / outside the window**: the drag ends cleanly.
- **Resize during a drag**: the bar geometry recomputes; the drag continues against the new geometry or
  ends safely — no panic.
- **Tiny scrollbar (thumb fills the track)**: clicking/dragging is a no-op (nothing to scroll).
- **Editor split view**: the bar belongs to the pane it's drawn in; interacting scrolls that pane.
- **Modal open**: only the modal's scrollbar is interactive (the editor beneath is not).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Clicking a scrollbar **track** MUST page the view toward the click by one viewport (up/back
  if above/left of the thumb, down/forward if below/right), bounded at the ends.
- **FR-002**: Pressing on a scrollbar **thumb** and dragging MUST scroll the view proportionally to the
  cursor position along the track, with the thumb following the cursor, bounded at both ends.
- **FR-003**: Releasing the button MUST end the drag; subsequent cursor moves MUST NOT scroll.
- **FR-004**: Interaction MUST apply to every surface that draws a feature-021 scrollbar: the editor's
  vertical bar, the editor's horizontal bar (non-wrap), and the file browser, Help/About, and
  encoding/plugin list bars.
- **FR-005**: In the editor, scrollbar click/drag MUST adjust the viewport offset only — the text cursor
  MUST NOT move and no text selection MUST be created or extended.
- **FR-006**: A press/drag that starts on a scrollbar MUST NOT trigger the existing text drag-selection
  (feature 017) or click-to-place-cursor; conversely a press/drag that starts off any scrollbar MUST
  behave exactly as before.
- **FR-007**: All other mouse/keyboard behavior — wheel (023), keyboard scrolling, dialog buttons (020),
  dialog list/field clicks, menu interaction — MUST be unchanged.
- **FR-008**: When a modal/overlay is open, only that surface's scrollbar MUST be interactive.
- **FR-009**: Scrollbar interaction MUST be bounded (no over-scroll/underflow) and MUST NOT panic at any
  terminal size, at the ends, on resize, on release outside the window, or when no bar is shown.
- **FR-010**: After any scrollbar interaction, the drawn thumb position MUST match the resulting scroll
  offset (the bar reflects the view).

### Key Entities

- **Scrollbar region**: the drawn track + thumb of a surface's bar, with its rect and the surface's
  (content length, viewport length, current offset) — the same geometry the renderer uses.
- **Scrollbar drag**: an in-progress thumb drag binding the active bar/surface + axis until the button is
  released; while active, mouse moves map to scroll offsets instead of text selection.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every surface that shows a scrollbar can be scrolled by clicking its track (pages toward the
  click) and by dragging its thumb (proportional), bounded.
- **SC-002**: In the editor, scrollbar interaction never moves the text cursor and never starts/extends a
  selection.
- **SC-003**: 100% of existing mouse gestures (text click, text press-drag selection, dialog/button
  clicks, wheel) and keyboard scrolling behave identically to before (zero regression), verified by tests.
- **SC-004**: No scrollbar interaction panics or over-scrolls across terminal sizes, at the ends, during
  resize, or on release outside the track.

## Assumptions

- **Track click pages by one viewport** (per decision): up/left for above/left of the thumb, down/right
  for below/right.
- **Editor thumb is draggable** (per decision): drag scrolls the viewport only; a press whose start cell
  is on a scrollbar enters "scrollbar drag" mode and suppresses text drag-selection for that gesture.
- **Proportional drag**: the cursor's fractional position along the track maps to the same fraction of the
  scrollable range (clamped).
- **Reuse feature-021 geometry**: the same per-surface bar rect + (content, viewport, offset) used to
  render the bar is used to hit-test and map clicks/drags, so the interactive region equals the drawn one.
- **Scope**: scrollbar mouse interaction only — no new keyboard actions, no config, no change to wheel,
  selection, buttons, or list/field clicks.
- Builds on feature 021 (scrollbars), 023 (wheel scroll model + per-surface scroll offsets), and 017
  (mouse drag-selection, whose anchor logic must be guarded).
