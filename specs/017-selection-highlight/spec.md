# Feature Specification: Visible text selection (highlight, Shift-select, mouse-drag)

**Feature Branch**: `017-selection-highlight`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "select all and search do not do any highlighting, copy paste highlighting
doesnt show up and is not intuitive." (Selections — Select All and any selected range — are never drawn,
and there is no way to select a sub-range; copy/paste is therefore unintuitive.)

## Clarifications

### Session 2026-06-20

- Q: Scope of the selection feature? → A: **Highlight + Shift-select + mouse-drag** — render the
  selection with a highlight, add keyboard selection (Shift+Arrow/Home/End), and add mouse click-drag to
  select. Copy/Cut operate on the selection.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See the selected text highlighted (Priority: P1)

When the user has a selection (via Select All, Shift-select, or mouse drag), the selected characters are
drawn with a distinct highlight so the user can see exactly what is selected. Select All highlights the
whole buffer.

**Why this priority**: Selections are currently invisible, so the user can't tell what will be
copied/cut. This is the core complaint and prerequisite for trusting copy/paste.

**Independent Test**: Select All; the buffer renders highlighted. Verifiable by inspecting rendered cell
styles.

**Acceptance Scenarios**:

1. **Given** a non-empty buffer, **When** the user does Select All, **Then** all characters render with
   the selection highlight.
2. **Given** a selection from (line a, col x) to (line b, col y), **When** the editor renders, **Then**
   exactly the characters in that range are highlighted (including across multiple lines).
3. **Given** no selection, **When** the editor renders, **Then** no characters are highlighted.
4. **Given** a selection highlight and a search-match highlight, **When** both are present, **Then** they
   remain visually distinguishable.

### User Story 2 - Select with the keyboard (Shift) (Priority: P1)

The user holds Shift and presses the arrow keys (and Home/End) to extend a selection from the cursor.
Moving without Shift, or typing, clears the selection. Copy (Ctrl+C) / Cut (Ctrl+X) act on the
selection; Paste inserts at the cursor (replacing a selection if present).

**Why this priority**: Without a way to select a sub-range, copy/paste is unusable for anything but the
whole file. Shift-select is the standard, expected mechanism.

**Acceptance Scenarios**:

1. **Given** the cursor mid-line, **When** the user presses Shift+Right several times, **Then** a
   selection grows one character at a time and is highlighted.
2. **Given** a selection, **When** the user presses an arrow without Shift, **Then** the selection
   clears and the cursor moves.
3. **Given** a selection, **When** the user presses Shift+Home / Shift+End, **Then** the selection
   extends to the start / end of the line.
4. **Given** a selection, **When** the user Copies then Pastes elsewhere, **Then** the selected text is
   inserted at the new cursor position.
5. **Given** a selection, **When** the user types a character or pastes, **Then** the selection is
   replaced by the typed/pasted text.

### User Story 3 - Select with the mouse (drag) (Priority: P2)

The user presses the left button in the editor, drags, and releases to select the text between the press
and release points; the selection is highlighted as they drag. A single click (no drag) places the
cursor and clears any selection.

**Why this priority**: Mouse selection is the most intuitive way for many users and pairs with the
mouse-navigation work; P2 because keyboard select already makes copy/paste usable.

**Acceptance Scenarios**:

1. **Given** the editor, **When** the user presses and drags the mouse, **Then** the text between the
   anchor and the current drag position is selected and highlighted.
2. **Given** a drag selection, **When** the user releases, **Then** the selection is retained and can be
   copied/cut.
3. **Given** any selection, **When** the user single-clicks (press+release at one spot), **Then** the
   selection clears and the cursor moves to the click.

### Edge Cases

- **Empty selection (anchor == active)**: nothing highlighted; treated as no selection.
- **Reverse selection (active before anchor)**: highlights the same range regardless of direction.
- **Multi-line selection**: highlights the partial first/last lines and the full middle lines, including
  the newline region to the line end where appropriate.
- **Selection + horizontal/vertical scroll**: only the visible portion is highlighted, aligned to the
  drawn glyphs (no off-by-one against scroll).
- **UTF-8 / wide characters**: highlight spans whole grapheme cells; never splits a character.
- **Soft-wrap mode**: the highlight follows the wrapped visual rows of the logical selection.
- **Selection then edit/undo**: editing replaces the selection; undo/redo and Select-All interactions
  don't leave a stale highlight pointing at invalid positions.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor MUST render the active buffer's selection with a highlight distinct from normal
  text (and distinct from search-match highlight), covering exactly the selected character range.
- **FR-002**: Select All MUST produce a visible whole-buffer selection.
- **FR-003**: Shift + arrow keys MUST extend/shrink the selection from the current anchor as the cursor
  moves; Shift+Home/Shift+End MUST extend to line start/end.
- **FR-004**: Moving the cursor without Shift, or typing/pasting, MUST clear the selection (typing/paste
  replaces the selected text).
- **FR-005**: Copy and Cut MUST operate on the current selection; Cut MUST remove the selected text;
  both MUST be undoable as today.
- **FR-006**: A left-button press-drag-release in the editor MUST create a selection between the press
  and release positions, highlighted during the drag; a single click (no drag) MUST clear any selection
  and move the cursor.
- **FR-007**: Selection highlighting MUST be correct under horizontal scroll, soft-wrap, and with
  UTF-8/wide characters — aligned to the drawn glyphs, never splitting a character, only the visible
  portion highlighted.
- **FR-008**: Reverse and multi-line selections MUST highlight the correct range regardless of
  direction.
- **FR-009**: These changes MUST NOT regress existing cursor movement, editing, search-match
  highlighting, or the menu/dialog mouse behavior.

### Key Entities *(include if feature involves data)*

- **Selection**: an `anchor` and `active` cursor position; the selected range is the ordered span
  between them. Drives the highlight and what Copy/Cut act on. Empty when `anchor == active`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After Select All on a non-empty buffer, 100% of the buffer's characters render highlighted.
- **SC-002**: Shift+Right N times selects exactly N characters (highlighted), and Copy yields exactly
  those N characters.
- **SC-003**: A mouse drag from A to B selects exactly the text between A and B (highlighted), in 100%
  of tested cases; a single click clears the selection.
- **SC-004**: Moving without Shift or typing clears the selection in 100% of cases.
- **SC-005**: No regression: cursor movement, editing, search highlighting, and menu/dialog mouse behave
  as before.

## Assumptions

- The selection highlight reuses a terminal attribute (e.g. reverse video) or a theme color distinct
  from the yellow search-match highlight; it degrades gracefully where unavailable.
- Copy/Cut already read `buffer.selection`; this feature makes selections visible and adds ways to
  create them, but does not change clipboard mechanics.
- Mouse drag reuses the existing click→cursor mapping (`handle_mouse_click`) for both the anchor (press)
  and active (drag/release) endpoints, so it is soft-wrap aware.
- Shift+navigation reuses the existing cursor-movement logic, adding selection bookkeeping around it.
