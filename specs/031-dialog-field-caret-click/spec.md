# Feature Specification: Caret-on-click in dialog text fields

**Feature Branch**: `031-dialog-field-caret-click`

**Created**: 2026-06-20

**Status**: Draft

**Input**: Close #58 (the deferred remainder of #53): let users click inside a dialog text field to
position the caret at the clicked character. Only the Find/Replace fields have a caret model today; the
Go-to-Line input and the file-browser Name field are append-only, so they need a caret/editing model
before click-to-position works. No new dependencies.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Click to position the caret in Find/Replace (Priority: P1) — closes part of #58

A user editing the Find or Replace text clicks partway into the field; the caret moves to the clicked
character so they can edit there, instead of only being able to type at the end.

**Why this priority**: Find/Replace already supports mid-string editing (Left/Right + insert), so
click-to-position is the natural completion and the highest-value of the three fields.

**Independent Test**: Type a query, click on an interior character → the caret lands there; typing
inserts at that point.

**Acceptance Scenarios**:

1. **Given** the Find query (or Replace replacement) field with text, **When** the user clicks on a
   visible character, **Then** the caret moves to that character and the field is focused.
2. **Given** the user clicks past the end of the text, **Then** the caret clamps to the end.
3. **Given** the text is longer than the field is wide (right-anchored), **When** the user clicks a
   visible character, **Then** the caret maps to the correct character within the visible window.
4. **Given** a click on the field's label/border (not the text interior), **Then** the caret is
   unchanged (no spurious move); a click on a button still activates it.

---

### User Story 2 - Caret editing + click in the file-browser Name field (Priority: P2) — closes part of #58

The file-browser filename field becomes a proper single-line input: the user can move the caret with
Left/Right/Home/End, insert and delete mid-string, and click to position the caret — not just append.

**Why this priority**: Filenames can be long and need correction mid-string; today the field is
append-only. Independent of US1/US3.

**Independent Test**: Type a name, press Left twice, type a character → it inserts mid-string; click
earlier in the field → the caret moves there.

**Acceptance Scenarios**:

1. **Given** the Name field with text, **When** the user presses Left/Right/Home/End, **Then** the caret
   moves accordingly (clamped to the value bounds).
2. **Given** the caret mid-string, **When** the user types or presses Backspace, **Then** the character
   is inserted/removed at the caret (not only at the end).
3. **Given** the user clicks inside the field box, **Then** the caret moves to the clicked character
   (clamped to the value end).
4. **Given** existing flows (typing to filter/append at end, Enter/activation, folder navigation,
   Esc/cancel), **Then** they behave as before.

---

### User Story 3 - Caret editing + click in the Go-to-Line input (Priority: P2) — closes part of #58

The Go-to-Line prompt becomes a proper input: caret movement (Left/Right/Home/End), insert/delete
mid-string, and click-to-position — while still accepting only digits.

**Why this priority**: Consistency and correction for the line-number entry; small and independent.

**Independent Test**: Type digits, move the caret left, insert a digit mid-string, click to reposition.

**Acceptance Scenarios**:

1. **Given** the Go-to-Line input with digits, **When** the user moves the caret and types a digit,
   **Then** it inserts at the caret; non-digits are still rejected.
2. **Given** the user clicks inside the input, **Then** the caret moves to the clicked character.
3. **Given** existing flows (Enter jumps and clamps to range, Esc cancels), **Then** they are unchanged.

---

### Edge Cases

- Click before the first / after the last character → caret clamps to 0 / value length.
- Click on an empty field → caret at 0, no panic.
- Multibyte / wide (CJK) / combining text → the click maps to a grapheme boundary using display width.
- A value wider than the field (right-anchored visible window) → the click maps within the visible tail.
- Backspace/Left at the start, Right/End at the end → no underflow/overflow, no panic.
- The 1-column caret glyph in the rendered field is a display artifact; the click maps to the value, so
  being off by that single column near the caret is acceptable and must not misplace by more.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A shared helper MUST map a horizontal click offset (columns from a field's inner-left
  edge) to a grapheme index in the field value, using the same visible-window logic the renderer uses
  (left-aligned when the value fits the field width; right-anchored tail when it overflows) and the
  shared display-width function; the result MUST be clamped to `[0, value_grapheme_len]`.
- **FR-002**: Clicking inside the Find query or Replace replacement field MUST move that field's caret to
  the mapped grapheme and focus the field; clicks on the label/border MUST NOT move the caret and button
  clicks MUST still activate.
- **FR-003**: The file-browser Name field MUST gain a caret: Left/Right/Home/End move it (clamped),
  typing inserts at the caret, Backspace deletes before the caret, and a click in the field box positions
  the caret — all preserving existing filtering/append-at-end/activation/navigation behavior.
- **FR-004**: The Go-to-Line input MUST gain a caret with the same movement/insert/delete/click behavior,
  while still accepting digits only and preserving Enter-jump/clamp and Esc-cancel.
- **FR-005**: Each field's rendered caret MUST appear at the caret position (mid-string when applicable),
  matching where editing and clicks act.
- **FR-006**: Click-to-caret geometry MUST match the renderer (drawn == clickable) for each field,
  derived from the same field rect the renderer draws into.
- **FR-007**: All existing behavior MUST be preserved — existing keys, mouse (buttons, list rows,
  outside-click), editing semantics elsewhere, file formats, and dialog confirm/cancel flows are
  unchanged except for the additions above.
- **FR-008**: No new third-party dependencies may be introduced (Constitution IV).

### Key Entities

- **Field caret**: a grapheme index into a single-line field value marking the insert/edit point; moved
  by arrows/Home/End and by clicks; clamped to `[0, len]`.
- **field_caret_at helper**: pure function mapping `(value, field_width, click_offset)` to a caret
  grapheme index via the renderer's visible-window logic and display width.
- **Field text rect**: the per-dialog inner text area (origin + width) shared by the renderer and the
  click hit-test.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In the Find and Replace fields, clicking a visible character positions the caret there
  (and at the end when clicking past the text), including when the value is right-anchored — verified by
  test.
- **SC-002**: The file-browser Name field and the Go-to-Line input support Left/Right/Home/End, mid-string
  insert/delete, and click-to-position — verified by test, with prior behavior intact.
- **SC-003**: Click-to-caret maps correctly over ASCII and multibyte/wide text with no panic at any
  boundary.
- **SC-004**: No regression in the existing test suite; no new dependencies. Closing all three stories
  closes #58.

## Assumptions

- "Visible window" matches the existing field rendering: the value is shown left-aligned when it fits the
  field width and right-anchored (tail visible) when it overflows; the helper reverses that, ignoring the
  1-column caret glyph artifact.
- Home/End move the caret to value start/end within the field (they are field-local while a field is
  focused, not editor cursor movement).
- The Go-to-Line input remains digits-only; the caret/editing additions do not change which characters
  are accepted.
- Each story is independently shippable; implementing any subset still leaves working dialogs.
