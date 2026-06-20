# Feature Specification: Editable-field affordance + Help redesign

**Feature Branch**: `018-field-affordance-help`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "where we allow a user to type — for example in the file dialog box where a
user can type path or filename — there is no way for the user to know that he can type anything there.
The Help is completely messed up; the user cannot understand what's being said there."

## Clarifications

### Session 2026-06-20

- Q: How should editable text fields look? → A: **Bordered input box** — each editable field is drawn in
  its own box with a label and an always-visible caret, so it clearly reads as a place to type.
- Q: How should the Help screen be redesigned? → A: **Two-column Key | Action table**, grouped by
  category, wrapped/fit to the terminal and **scrollable** when it doesn't fit.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Know you can type in the file dialog (Priority: P1)

When the Open/Save file dialog is showing, the place to type a filename (Save) or a path to jump to
(Open) is drawn as a clearly-bordered input box with a label and a visible caret, so the user
immediately understands they can type there. In Open mode this box is now visible (previously the
typeable path field was not drawn at all).

**Why this priority**: The literal complaint — users cannot tell a field is typeable; the Open path
field was invisible. This is the core fix.

**Independent Test**: Open the file dialog; a bordered, labeled input box with a caret is visible in both
Open and Save modes; typing updates the box text.

**Acceptance Scenarios**:

1. **Given** the Save file dialog, **When** it is shown, **Then** a bordered input box labeled for the
   filename is visible with a caret, and typing updates it.
2. **Given** the Open file dialog, **When** it is shown, **Then** a bordered input box for the path is
   visible with a caret (the typeable jump-path field is no longer invisible).
3. **Given** the input box, **When** the user types and backspaces, **Then** the box text and caret
   update; pressing the confirm key uses the typed value as before.
4. **Given** a long typed value, **When** it exceeds the box width, **Then** it is shown without
   corrupting the layout (scrolled/truncated within the box).

### User Story 2 - Understand the Help screen (Priority: P1)

The Help screen presents the keyboard shortcuts as a clear, grouped two-column **Key | Action** table
that is readable, fits the terminal, wraps/scrolls instead of truncating, and is organized by category
(File, Edit, Search, View, Selection, Menus, Dialogs). The user can read every entry.

**Why this priority**: The Help is reported as unreadable; a cheat sheet that truncates or is dense is
worse than none. Equal priority to US1 as the second half of the request.

**Independent Test**: Open Help; entries are laid out as aligned Key | Action rows grouped by section;
nothing is cut off (scroll if the list is taller than the screen).

**Acceptance Scenarios**:

1. **Given** Help is open, **When** it renders, **Then** shortcuts appear as aligned Key | Action rows
   under category headings.
2. **Given** a terminal too short to show all rows, **When** Help is open, **Then** the user can scroll
   to read the remaining rows (no silent truncation).
3. **Given** Help is open, **When** the user presses Esc (or the dismiss key), **Then** it closes.
4. **Given** any supported terminal width ≥ the minimum, **When** Help renders, **Then** rows are not
   cut off mid-content (content fits or wraps within the box).

### Edge Cases

- **Empty field**: the input box still renders (empty) with a caret so it's obviously typeable.
- **UTF-8 / wide input**: box text and caret are grapheme-correct; the caret never splits a character.
- **Very small terminal**: the file dialog and Help degrade without panicking; Help scrolls; the input
  box clamps.
- **Open-mode field vs list**: showing the path box must not break list navigation or selection.
- **Help longer than the screen**: scroll indicators / wrap so the bottom rows are reachable.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Editable fields in the file dialog (Save filename; Open jump-path) MUST be rendered as a
  bordered input box with a label and a visible caret, clearly indicating the field is typeable.
- **FR-002**: The Open-mode path field MUST be visible (it was previously not rendered), so users can see
  they may type a path to jump to.
- **FR-003**: Typing, backspacing, and confirming in the field MUST behave as before; only the
  presentation changes (plus the now-visible Open field).
- **FR-004**: Field text and caret MUST be UTF-8/grapheme-correct and MUST not corrupt the layout when
  the text is longer than the box (scroll/truncate within the box).
- **FR-005**: The Help screen MUST present shortcuts as a grouped, aligned two-column **Key | Action**
  table organized by category.
- **FR-006**: The Help screen MUST NOT silently truncate content — it MUST fit/wrap within its box and be
  **scrollable** when there are more rows than fit, with a cue that more content exists.
- **FR-007**: Help MUST remain dismissable (Esc / existing dismiss keys) and MUST stay modal.
- **FR-008**: These changes MUST NOT regress file-dialog navigation/selection, the search/selection
  highlighting, or other dialogs; the field box and Help table degrade gracefully on small terminals
  without panicking.

### Key Entities *(include if feature involves data)*

- **Input field box**: a labeled, bordered single-line editor showing the field text and a caret; backs
  the file-dialog filename/path entry.
- **Help entry / section**: a `(key, action)` row grouped under a category heading; the Help screen is a
  scrollable list of these.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In both Open and Save file-dialog modes, a bordered, labeled input box with a caret is
  visible (Open mode no longer hides the path field) — verifiable in the rendered output.
- **SC-002**: Typing N characters into the field shows them in the box with the caret after them; the
  confirm action uses exactly the typed value.
- **SC-003**: The Help screen shows every shortcut as an aligned Key | Action row grouped by section,
  with zero rows cut off (scroll reaches the last row) at the minimum terminal size.
- **SC-004**: No regression: file-dialog browse/select, search/selection highlighting, and other dialogs
  behave as before; no panic at small sizes.

## Assumptions

- The bordered input box reuses the editor's existing box-drawing/theme conventions (consistent with the
  feature-016 boxed buttons); the caret is a terminal attribute/marker that degrades gracefully.
- Find/Replace fields already show a label and a caret (feature 015); converting them to the same
  bordered-box style is a consistency follow-up, tracked separately, not required for this feature.
- The Help content is the existing set of shortcuts, reorganized into the table; About is unchanged
  except for any shared layout/scroll improvements.
- Scrolling Help uses simple line scrolling (e.g. arrow/PageUp-Down) within the modal.
