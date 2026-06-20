# Feature Specification: Interaction completeness

**Feature Branch**: `030-interaction-completeness`

**Created**: 2026-06-20

**Status**: Draft

**Input**: Implement the four deferred follow-up issues from the feature-029 UX audit (#53–#56) as four
independent user stories, all completing mouse/keyboard interaction. Reuse existing infrastructure; no
new dependencies.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Mouse interaction inside dialogs (Priority: P1) — closes #53

A mouse user working in a dialog can click directly on its content: clicking a row in a list selects
that entry, and clicking inside a text field puts the caret where they clicked. Today only the dialog's
buttons and the keyboard respond; clicks on the list rows and input fields are ignored.

**Why this priority**: It's the biggest remaining mouse/keyboard parity gap — a mouse user literally
cannot pick an encoding or position the caret in a field by clicking, which feels broken.

**Independent Test**: Open the encoding dialog and click a row → that encoding becomes selected. Open
Find/Replace and click partway into the query field → the caret moves to the clicked character.

**Acceptance Scenarios**:

1. **Given** the encoding-select list (or plugin-manager list) is open, **When** the user clicks a
   visible row, **Then** that row becomes the selected item and the list gains focus.
2. **Given** a dialog text field (Find query/replacement, Go-to-Line input, or file-browser Name/path),
   **When** the user clicks within the field, **Then** the caret moves to the clicked grapheme and that
   field gains focus.
3. **Given** a click lands on a dialog button, **When** it is clicked, **Then** the existing button
   behavior is unchanged (content hit-testing does not interfere with buttons).
4. **Given** a click lands inside the dialog but on neither a row, field, nor button, **Then** nothing
   changes (safe no-op); a click outside the dialog keeps today's behavior.

---

### User Story 2 - Double-click word / triple-click line selection (Priority: P2) — closes #54

In the editor, double-clicking selects the word under the pointer and triple-clicking selects the whole
line, so the user can quickly select text with the mouse and then copy or cut it.

**Why this priority**: A near-universal editor convention; its absence is friction, but single-click +
drag-select already provide a usable path, so P2.

**Independent Test**: Double-click a word → the word is selected (and Copy copies exactly it).
Triple-click → the whole line is selected. A following single click clears the selection.

**Acceptance Scenarios**:

1. **Given** the cursor over a word, **When** the user double-clicks, **Then** the surrounding word is
   selected (word-boundary to word-boundary).
2. **Given** any position on a line, **When** the user triple-clicks, **Then** the whole logical line is
   selected.
3. **Given** an active double/triple-click selection, **When** the user single-clicks elsewhere,
   **Then** the selection clears and the cursor moves there (existing behavior).
4. **Given** a double-click on whitespace or at end of line, **Then** a sensible run is selected (the
   whitespace run / nothing past the end), with no panic.

---

### User Story 3 - Right-click context menu (Priority: P3) — closes #55

Right-clicking in the editor opens a small popup menu with Cut, Copy, Paste, and Select All; the user
can pick an item with the mouse or keyboard.

**Why this priority**: A convenience and discoverability aid; all four actions already have keyboard
shortcuts, so it's the lowest-priority of the four.

**Independent Test**: Right-click in the editor → a menu with Cut/Copy/Paste/Select All appears; click
Copy (or arrow-down + Enter) → the action runs; Esc or an outside click dismisses it.

**Acceptance Scenarios**:

1. **Given** the editor, **When** the user right-clicks, **Then** a context menu (Cut / Copy / Paste /
   Select All) appears near the click.
2. **Given** the context menu is open, **When** the user clicks an item or navigates with Up/Down and
   presses Enter/Space, **Then** the corresponding action runs and the menu closes.
3. **Given** the context menu is open, **When** the user presses Esc or clicks outside it, **Then** it
   dismisses without running anything.
4. **Given** an item that doesn't apply (Cut/Copy with no selection, Paste with empty clipboard),
   **When** chosen, **Then** it is a safe no-op with the existing feedback (no panic).

---

### User Story 4 - DOS-standard F-key bindings (Priority: P2) — closes #56

Common terminal-editor F-keys work as additional accelerators alongside the existing Ctrl bindings,
without changing any existing binding.

**Why this priority**: Cheap, high-familiarity win for keyboard users; small and low-risk.

**Independent Test**: Press F9 → Copy runs; F8 → Cut; F11 → Paste; F6 → next buffer; Shift+F6 →
previous buffer. F1/F2/F3/F5/F10/F12 still do what they did before.

**Acceptance Scenarios**:

1. **Given** the editor, **When** the user presses F6 / Shift+F6, **Then** the next / previous buffer
   becomes active (same as the Ctrl+Tab bindings).
2. **Given** a selection, **When** the user presses F8 / F9, **Then** the selection is cut / copied;
   **When** the user presses F11, **Then** the clipboard is pasted.
3. **Given** the existing F-keys (F1 Help, F2 Find Prev, F3 Find Next, F5 Save, F10 Menu, F12 Save As
   Encoding), **When** pressed, **Then** they behave exactly as before (no shadowing).

---

### Edge Cases

- Clicking a list row beyond the populated entries (empty area below the list) → no selection change.
- Clicking a field when the value is shorter than the click column → caret clamps to end of value.
- Double/triple-click at end of buffer, on an empty line, or on a zero-width/combining cluster → safe
  selection, no panic.
- Right-click while a modal/menu is already open → the modal keeps precedence (no context menu over it).
- Context menu opened near the screen edge → it stays on-screen.
- A new F-key binding must not collide with an existing one (validated by test).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Clicking a visible row in the encoding-select or plugin-manager list MUST select that row
  and focus the list (primary control), using the geometry the renderer draws with.
- **FR-002**: Clicking inside a dialog text field (Find query/replacement, Go-to-Line input,
  file-browser Name/path) MUST move that field's caret to the clicked grapheme (clamped to the value's
  end) and focus that field, mapping columns to graphemes via the shared display-width function.
- **FR-003**: Dialog content hit-testing MUST NOT change existing button-click or outside-click
  behavior, and MUST be a safe no-op when the click is on neither a row, field, nor button.
- **FR-004**: A double-click in the editor MUST select the word under the pointer (a run of
  word-characters, or the adjacent non-word run); a triple-click MUST select the whole logical line.
- **FR-005**: Click-count detection MUST use a bounded time-and-position window; a single click after a
  multi-click selection MUST clear the selection and move the cursor (existing behavior preserved).
- **FR-006**: A double/triple-click selection MUST be the active selection usable by Copy/Cut.
- **FR-007**: A right-click in the editor MUST open a context menu offering Cut, Copy, Paste, and Select
  All, positioned near the click and kept fully on-screen.
- **FR-008**: The context menu MUST be operable by mouse (click an item) and keyboard (Up/Down move,
  Enter/Space activate, Esc dismiss); clicking outside dismisses it; each item routes to the existing
  action and the menu closes after activation.
- **FR-009**: Context-menu items that don't apply MUST be safe no-ops with the existing feedback (no
  panic, no data change).
- **FR-010**: The editor's right-click/context menu MUST respect modal precedence — it MUST NOT open
  while another modal/dialog/menu is active.
- **FR-011**: F6 MUST switch to the next buffer and Shift+F6 to the previous; F8 MUST cut, F9 copy, and
  F11 paste — as additional accelerators that do not replace the existing Ctrl bindings.
- **FR-012**: The new F-key bindings MUST NOT shadow or alter any existing binding (F1/F2/F3/F5/F10/F12
  and all current Ctrl/Alt bindings remain).
- **FR-013**: All existing behavior MUST be preserved — existing keys, existing mouse interactions
  (single-click position, drag-select, wheel, scrollbars, dialog buttons), editing semantics, file
  formats, and dialog flows are unchanged except for the additions above.
- **FR-014**: No new third-party dependencies may be introduced (Constitution IV).

### Key Entities

- **Click-tracker**: Records the last click's time, position, and running count to classify the next
  click as single / double / triple within a bounded window.
- **Context menu**: A transient popup with a fixed item list (Cut/Copy/Paste/Select All), a focused
  index, and an anchor position; rendered as an overlay and hit-tested like the existing menus.
- **Dialog content regions**: The per-dialog clickable areas (list rows; text-field interiors) mapped to
  a selection index or a caret grapheme, shared between renderer and hit-testing.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In the encoding and plugin-manager dialogs, clicking a row selects it; in the Find/Replace,
  Go-to-Line, and file-browser fields, clicking positions the caret — verified per dialog by test.
- **SC-002**: Double-click selects exactly the word and triple-click exactly the line (Copy returns the
  expected text) in 100% of tested cases, including multibyte text, with no panic.
- **SC-003**: The right-click context menu opens, runs each of Cut/Copy/Paste/Select All, and dismisses
  by Esc and by outside-click — verified by test.
- **SC-004**: F6/Shift+F6/F8/F9/F11 perform their actions and every pre-existing F-key is unchanged —
  verified by test.
- **SC-005**: No regression in the existing test suite; no new dependencies.

## Assumptions

- "Word" for double-click means a maximal run of word characters (Unicode alphanumeric plus `_`); a
  double-click on a non-word character selects the adjacent run of like (non-word, non-space) or
  whitespace characters. This mirrors common editors and is panic-free at boundaries.
- The multi-click window reuses the file browser's existing double-click timing convention
  (`DOUBLE_CLICK_MS`) and requires the clicks to be on (approximately) the same cell.
- The context menu reuses the existing menu/button rendering + hit-testing rather than a new widget
  framework; it is a small fixed list, so no scrolling is needed.
- The new F-keys are additive accelerators; the existing Ctrl/Alt bindings remain the primary ones.
- Each user story is independently shippable; implementing any subset still leaves a working editor.
