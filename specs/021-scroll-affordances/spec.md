# Feature Specification: Scroll affordances + dialog button polish

**Feature Branch**: `021-scroll-affordances`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "Scroll affordances + dialog button polish (feature 021). The app's views
scroll but never draw a scrollbar, and the Help/About screens have no Close button. Add scrollbars to
the editor (vertical + horizontal), the file browser, Help/About, and the encoding/plugin dialogs when
content overflows; a Close button on Help/About; and the activating key on every dialog button label.
Decisions: use ratatui's Scrollbar widget; key hints on all dialog buttons; editor gets both bars.
Scope is affordance/visibility only."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See where you are when scrolling content (Priority: P1)

When the content of a scrollable view is larger than the space available, the user sees a scrollbar that
shows how much content there is and where the current view sits within it. This applies to the main
editor view (a large file scrolled vertically, and long lines scrolled horizontally), the file browser
listing (a directory with more entries than fit), the Help/About screens, and the encoding-select and
plugin-manager lists.

**Why this priority**: Today every one of these views scrolls silently — a long directory or large file
looks truncated with no cue that more exists or where you are. This is the core of the report and the
biggest usability win.

**Independent Test**: Open a directory with more entries than visible rows → a vertical scrollbar appears
with a thumb whose position tracks the selection; scroll a large file → the editor's vertical scrollbar
thumb moves; move the cursor past the right edge of a long line → a horizontal scrollbar appears and its
thumb tracks the column. Verifiable by inspecting the rendered cells at the view's edges.

**Acceptance Scenarios**:

1. **Given** a view whose content exceeds the visible area, **When** it is shown, **Then** a scrollbar is
   drawn along the relevant edge with a thumb sized and positioned to reflect the visible fraction and
   the current scroll offset.
2. **Given** a view whose content fits entirely, **When** it is shown, **Then** no scrollbar is drawn (or
   it shows a full-length thumb) and no content is hidden behind it.
3. **Given** a scrollbar is shown, **When** the user scrolls (arrows, PgUp/PgDn, cursor movement), **Then**
   the thumb moves to match the new position.
4. **Given** the main editor in normal (non-wrap) mode with a line longer than the view, **When** the user
   moves the cursor along it, **Then** a horizontal scrollbar reflects the horizontal scroll position.
5. **Given** the main editor in soft-wrap mode, **When** it is shown, **Then** only the vertical scrollbar
   is present (there is no horizontal scrolling to indicate).

### User Story 2 - Close the Help and About screens with a button (Priority: P1)

The Help and the About screens show a clearly bordered **Close** button that the user can click with the
mouse to dismiss the screen, and the button's label shows its keyboard shortcut so the user knows they
can also press it.

**Why this priority**: Help/About are currently keyboard-only with no on-screen way out; users reported
not knowing how to close them. A visible, clickable Close button with its key shown fixes that directly.

**Independent Test**: Open Help, click the Close button → Help closes. Open About, press the shown key →
About closes. Verifiable by hit-testing a click at the drawn button and by driving the key.

**Acceptance Scenarios**:

1. **Given** the Help screen is open, **When** it is shown, **Then** a bordered Close button is drawn whose
   label includes its keyboard shortcut.
2. **Given** the Help or About screen is open, **When** the user clicks the Close button, **Then** the
   screen closes (identical to the existing dismiss key).
3. **Given** the Help or About screen is open, **When** the user presses the existing dismiss key (`Esc`),
   **Then** the screen still closes as before.

### User Story 3 - Every dialog button advertises its shortcut (Priority: P2)

Every on-screen dialog button shows the key that activates it as part of its label (for example "Cancel
(Esc)", "OK (Enter)", "Save (Enter)", "Close (Esc)"), so keyboard users can act without guessing and the
buttons are self-documenting. This applies across the confirm/dismiss dialogs, the interactive/list
dialogs, and the new Help/About Close button.

**Why this priority**: It makes the existing buttons discoverable and consistent, but the dialogs are
already usable without it, so it ranks below the missing scrollbars and the missing Close button.

**Independent Test**: Open any dialog with buttons → each button's drawn label contains its activating
key; pressing that key still performs the same action. Verifiable by inspecting rendered labels and by
driving the keys.

**Acceptance Scenarios**:

1. **Given** any dialog with boxed buttons, **When** it is shown, **Then** each button's label includes the
   key that activates it.
2. **Given** a button labeled with a key, **When** the user presses that key, **Then** the button's action
   runs exactly as before this feature (the label is informational; behavior is unchanged).
3. **Given** the key-hint labels are added, **When** a button is clicked or focused-and-activated, **Then**
   it maps to the same action as before (the displayed text change does not break click/focus mapping).

### Edge Cases

- **Content exactly fits**: no scrollbar (or a full-length, non-draggable thumb); no row/column of content
  is hidden behind a bar.
- **Very small terminal**: scrollbars and buttons degrade without corrupting the layout or panicking; if a
  bar cannot fit, content rendering still succeeds.
- **Reserved space**: drawing a scrollbar must not overlap content — the view reserves the edge it occupies
  so text/entries are not hidden under the bar, and mouse mapping accounts for the reserved cells.
- **Resize while open**: scrollbars and buttons re-flow with the view and remain consistent with the drawn
  content (clicks still land where drawn).
- **Empty content** (empty file, empty directory): no spurious scrollbar; views remain usable.
- **Wide/UTF-8 button labels**: adding the key hint keeps button width and click hit-testing correct.
- **Editor with line numbers / split view**: the editor scrollbars appear correctly with the gutter and in
  each split pane without overlapping the divider or the gutter.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The main editor view MUST display a vertical scrollbar indicating the current line position
  within the file, and (in non-wrap mode) a horizontal scrollbar indicating the current column position
  within the longest visible content; in soft-wrap mode only the vertical scrollbar is shown.
- **FR-002**: The file browser listing MUST display a vertical scrollbar when the number of entries
  exceeds the visible rows, with the thumb reflecting the scroll position.
- **FR-003**: The Help and About overlays MUST display a vertical scrollbar when their content exceeds the
  visible area.
- **FR-004**: The encoding-select and plugin-manager dialogs MUST display a vertical scrollbar when their
  list content exceeds the visible area.
- **FR-005**: Each scrollbar's thumb size and position MUST reflect the visible fraction and the current
  scroll offset of its view, and MUST update as the view scrolls.
- **FR-006**: A scrollbar MUST NOT hide any content — the view MUST reserve the edge the scrollbar occupies
  so text/entries are not drawn under it.
- **FR-007**: When a view's content fits entirely within the visible area, the view MUST NOT obscure
  content with a scrollbar (no scrollbar, or a full-length thumb that hides nothing).
- **FR-008**: The Help and About overlays MUST each render a bordered Close button that dismisses the
  overlay when clicked, identical in effect to the existing dismiss key.
- **FR-009**: Every dialog button label MUST include the key that activates it (e.g. "Close (Esc)",
  "Cancel (Esc)", "OK (Enter)", "Save (Enter)"), across the confirm/dismiss dialogs, the interactive/list
  dialogs, and the Help/About Close button.
- **FR-010**: Adding key-hint text to a button label MUST NOT change the button's action, its
  click/focus mapping, or its layout correctness (width and hit-testing remain accurate).
- **FR-011**: All existing scrolling behavior, navigation keys, dialog actions, and dismissal keys MUST be
  unchanged — this feature only adds visual affordances.
- **FR-012**: Scrollbars and buttons MUST render correctly and without panic across terminal sizes
  (smallest supported up to full screen), under resize, with line numbers, and in split-view editing.

### Key Entities

- **Scrollbar indicator**: a per-view visual element bound to (content length, viewport length, current
  offset); it occupies one reserved edge (right for vertical, bottom for horizontal) of its view.
- **Dialog button label**: the displayed text of a button, now composed of its action name plus its
  activating key, while the underlying action mapping stays keyed on a stable identity.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In every scrollable view (editor, file browser, Help/About, encoding/plugin lists) whose
  content overflows, a scrollbar is visible and its thumb position corresponds to the current scroll
  offset.
- **SC-002**: A user can determine, at a glance and without scrolling, whether there is more content above
  or below (and, in the editor, left or right) the current view.
- **SC-003**: The Help and About screens can each be dismissed using only the mouse (via the Close button).
- **SC-004**: 100% of dialog buttons display their activating key, and 100% of previously working keys and
  actions still behave identically (zero behavioral regression), verified by tests.
- **SC-005**: No scrollbar hides content and no view panics or corrupts its layout across terminal sizes
  from the smallest supported to full screen, including split-view and line-number modes.

## Assumptions

- **Scrollbar widget**: the standard scrollbar component of the project's TUI toolkit is used for a
  consistent look across all views (per the user's decision).
- **Editor bars**: the editor shows both a vertical and a horizontal scrollbar in non-wrap mode, and only
  the vertical bar in soft-wrap mode (there is no horizontal scrolling in soft-wrap).
- **Horizontal extent**: the editor's horizontal scrollbar reflects the width of the currently visible
  lines (a cheap, local measure) rather than scanning the whole file; this is an intentional simplification
  that keeps large-file rendering fast while still indicating horizontal position.
- **Key hints on all buttons**: applied app-wide (confirm dialogs, interactive dialogs, Help/About),
  per the user's decision; the displayed key is the dialog's existing primary shortcut for that button.
- **Scrollbar visibility threshold**: a scrollbar is shown only when content exceeds the viewport; views
  that always fit (e.g. a short fixed list) may omit it.
- **No new actions or navigation**: scrolling, list navigation, and dialog actions are unchanged; only the
  visual indicators and button labels change.
- This builds on the dialog-button machinery from features 016 and 020 and the Help redesign from
  feature 018.
