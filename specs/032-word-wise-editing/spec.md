# Feature Specification: Word-wise navigation, selection, and deletion

**Feature Branch**: `032-word-wise-editing`

**Created**: 2026-06-21

**Status**: Draft

**Input**: The editor only moves and deletes one character at a time; add the standard word-wise editing
keys (Ctrl+Left/Right, Ctrl+Shift+Left/Right, Ctrl+Backspace, Ctrl+Delete), reusing the word-boundary
classification from feature 030's double-click selection. No new dependencies.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Move the cursor by word (Priority: P1)

A user holds Ctrl and presses Left/Right to jump the cursor a whole word at a time instead of one
character, so they can navigate text quickly.

**Why this priority**: Word-wise movement is a near-universal editing expectation and the foundation the
selection and deletion stories build on.

**Independent Test**: Place the cursor mid-line and press Ctrl+Right / Ctrl+Left → the cursor jumps to
the next / previous word boundary, crossing line ends sensibly.

**Acceptance Scenarios**:

1. **Given** the cursor inside or before a word, **When** the user presses Ctrl+Right, **Then** the
   cursor moves to the start of the next word (skipping any intervening spaces/symbols).
2. **Given** the cursor inside or after a word, **When** the user presses Ctrl+Left, **Then** the cursor
   moves to the start of the current/previous word.
3. **Given** the cursor at end of line, **When** the user presses Ctrl+Right, **Then** it moves to the
   first word of the next line; **Given** the cursor at column 0, **When** Ctrl+Left, **Then** it moves
   to the end of the previous line.
4. **Given** an active selection, **When** the user presses Ctrl+Left/Right (without Shift), **Then** the
   selection is cleared and the cursor moves; the cursor is scrolled into view.
5. **Given** the cursor at the very start/end of the buffer, **When** Ctrl+Left/Right, **Then** it stays
   put (no movement, no panic).

---

### User Story 2 - Select by word (Priority: P2)

A user holds Ctrl+Shift and presses Left/Right to extend the selection a word at a time, then copies or
cuts it.

**Why this priority**: A natural companion to US1 and to the existing Shift+Arrow selection; valuable but
the editor is usable without it.

**Independent Test**: From a cursor position, press Ctrl+Shift+Right twice → the selection spans the next
two words; Copy yields exactly that text.

**Acceptance Scenarios**:

1. **Given** a cursor position, **When** the user presses Ctrl+Shift+Right, **Then** the selection extends
   to the next word boundary (same boundary rule as US1).
2. **Given** a cursor position, **When** the user presses Ctrl+Shift+Left, **Then** the selection extends
   to the previous word boundary.
3. **Given** a word-wise selection, **When** the user copies or cuts, **Then** exactly the selected text
   is used.
4. **Given** the cursor at a buffer boundary, **When** Ctrl+Shift in that direction, **Then** the
   selection is unchanged (no panic).

---

### User Story 3 - Delete by word (Priority: P1)

A user presses Ctrl+Backspace to delete the word before the cursor, or Ctrl+Delete to delete the word
after it, so they can erase a token in one keystroke.

**Why this priority**: Word deletion is one of the most-used editing shortcuts; high everyday value.

**Independent Test**: With the cursor after a word, press Ctrl+Backspace → the word (and adjoining
whitespace per the boundary rule) is removed in one undoable step.

**Acceptance Scenarios**:

1. **Given** the cursor after a word, **When** the user presses Ctrl+Backspace, **Then** the text from the
   previous word boundary to the cursor is deleted as one undo step, and the cursor lands at the deletion
   point.
2. **Given** the cursor before a word, **When** the user presses Ctrl+Delete, **Then** the text from the
   cursor to the next word boundary is deleted as one undo step.
3. **Given** an active selection, **When** the user presses Ctrl+Backspace/Delete, **Then** the selection
   is deleted (consistent with Backspace/Delete on a selection).
4. **Given** a read-only buffer, **When** the user presses Ctrl+Backspace/Delete, **Then** nothing is
   deleted and the read-only message is shown.
5. **Given** the cursor at the start/end of the buffer, **When** the respective key is pressed, **Then**
   it is a safe no-op.

---

### Edge Cases

- Word-wise move/delete across an empty line → moves to / deletes through the line boundary correctly.
- Multibyte / wide (CJK) / combining text → boundaries fall on grapheme boundaries; no panic.
- Long runs of spaces or symbols → treated as their own word class (a run of like characters).
- Undo after a word delete restores exactly the removed text and the prior cursor position.
- Terminals that do not distinguish Ctrl+Arrow → the accelerator is simply absent; per-character keys
  still work (graceful degradation).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor MUST move the cursor to the previous / next word boundary on Ctrl+Left /
  Ctrl+Right, using the same word classification as feature 030's double-click (a maximal run of word
  characters = Unicode alphanumeric plus `_`; whitespace and other-symbol runs are separate classes),
  operating on grapheme boundaries.
- **FR-002**: Word-wise movement MUST cross line boundaries: Ctrl+Right at end of line moves to the next
  line's first word start; Ctrl+Left at column 0 moves to the end of the previous line; at the buffer
  start/end it is a no-op.
- **FR-003**: Plain word-wise movement MUST clear any active selection and scroll the cursor into view,
  consistent with existing plain cursor movement.
- **FR-004**: Ctrl+Shift+Left / Ctrl+Shift+Right MUST extend the selection by a word in each direction
  (same boundary rule), consistent with the existing Shift+Arrow selection, and the result MUST be usable
  by Copy/Cut.
- **FR-005**: Ctrl+Backspace MUST delete from the cursor back to the previous word boundary, and
  Ctrl+Delete MUST delete from the cursor forward to the next word boundary, each as a single undo step;
  with an active selection, both MUST delete the selection instead.
- **FR-006**: Word-wise deletion MUST respect the read-only guard (no change; the read-only message is
  shown) and MUST be a safe no-op at the buffer start/end.
- **FR-007**: New keybindings — Ctrl+Left→move-word-left, Ctrl+Right→move-word-right,
  Ctrl+Shift+Left→select-word-left, Ctrl+Shift+Right→select-word-right, Ctrl+Backspace→delete-word-left,
  Ctrl+Delete→delete-word-right — MUST be added without changing any existing binding.
- **FR-008**: All existing behavior MUST be preserved — per-character movement/selection/deletion, the
  existing Shift+Arrow selection, Copy/Cut/Paste, undo/redo, and every other key are unchanged.
- **FR-009**: No new third-party dependencies may be introduced (Constitution IV).

### Key Entities

- **Word boundary**: a position between two grapheme runs of different classes (word / whitespace /
  other), shared with feature 030's double-click selection so behavior is consistent.
- **Word target**: the cursor position reached by one word-wise step in a direction, computed from the
  buffer content and the current cursor (may be on an adjacent line).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Ctrl+Left/Right move the cursor by whole words, including across line boundaries, with no
  panic at any position — verified by test.
- **SC-002**: Ctrl+Shift+Left/Right extend the selection by words and Copy returns exactly that text —
  verified by test.
- **SC-003**: Ctrl+Backspace/Delete remove exactly the word range in one undo step, no-op safely at
  buffer ends, and are blocked (with a message) in a read-only buffer — verified by test.
- **SC-004**: The new bindings are present and no existing binding changes — verified by test.
- **SC-005**: No regression in the existing test suite; no new dependencies.

## Assumptions

- "Word" matches feature 030's double-click classification (Unicode alphanumeric plus `_` is a word
  character; whitespace and other symbols form their own runs); a word-wise step skips a leading run of
  non-word characters then stops at the next class change, matching common editor behavior.
- Word-wise movement reuses the existing cursor-move + scroll-into-view + selection-clearing paths; word
  deletion reuses the existing single-undo-step deletion path.
- Home/End remain line-local (feature 028); Ctrl+Home/Ctrl+End (document start/end) are out of scope for
  this feature.
- Each story is independently shippable; implementing any subset still leaves a working editor.
