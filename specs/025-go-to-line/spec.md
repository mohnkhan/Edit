# Feature Specification: Go to Line

**Feature Branch**: `025-go-to-line`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "Go to Line (feature 025). Add a Go-to-Line command (Ctrl+G + a Search-menu
item) that opens a small modal prompt; type a 1-based line number, Enter jumps the cursor to that line's
start (scrolled into view), Esc cancels. Out-of-range clamps to first/last; invalid/empty is a no-op.
Modal consistent with existing dialogs. Navigation only."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Jump to a line by number (Priority: P1)

While editing, the user presses `Ctrl+G` (or picks Search ▸ Go to Line), types a line number into a small
prompt, and presses Enter; the cursor moves to the start of that line and the view scrolls so the line is
visible. This is the core capability.

**Why this priority**: Jumping to a known line (from a compiler error, a review comment, a stack trace) is
a fundamental editor navigation that the editor currently lacks entirely.

**Independent Test**: Open a file with many lines, press `Ctrl+G`, type `42`, press Enter → the cursor is
on line 42 (1-based) at column 1 and line 42 is visible.

**Acceptance Scenarios**:

1. **Given** a file with at least N lines, **When** the user invokes Go to Line, types `N`, and presses
   Enter, **Then** the cursor moves to the start (column 1) of line N and the view scrolls to show it.
2. **Given** the prompt is open, **When** the user types digits, **Then** the typed number is shown in the
   prompt; **When** they press Backspace, **Then** the last digit is removed.
3. **Given** the prompt is open, **When** the user presses Enter on a valid number, **Then** the prompt
   closes and the cursor has moved.

### User Story 2 - Cancel / invalid / out-of-range handling (Priority: P2)

The prompt handles non-happy paths gracefully: Esc closes it without moving; an out-of-range number
clamps to the first or last line; an empty or non-numeric entry does nothing (or shows a brief notice)
and does not move the cursor unexpectedly.

**Why this priority**: Robust, predictable behavior on bad input prevents surprise cursor jumps and makes
the feature safe to use quickly.

**Independent Test**: Open the prompt, press Esc → nothing moves. Open it, type `99999` in a 100-line
file, Enter → cursor goes to the last line. Open it, press Enter with an empty field → no movement.

**Acceptance Scenarios**:

1. **Given** the prompt is open, **When** the user presses Esc, **Then** it closes and the cursor is
   unchanged.
2. **Given** a file with M lines, **When** the user enters a number greater than M, **Then** the cursor
   moves to line M (clamped); **When** they enter `0` or a number below 1, **Then** the cursor moves to
   line 1.
3. **Given** the prompt is open, **When** the user presses Enter with an empty or non-numeric field,
   **Then** the cursor does not move and the prompt closes (or stays with a brief notice) without error.

### User Story 3 - Coexists with existing modals and input (Priority: P2)

The Go-to-Line prompt is a modal overlay that behaves like the other dialogs: while it is open it captures
input (typing edits the number; other editor keys don't leak through), and it does not interfere with the
dialog buttons (020), scrollbars (021/024), or wheel (023). Only one modal is open at a time.

**Why this priority**: Consistency with the existing dialog model avoids input-routing bugs, but the
feature is usable even before this is perfected, so it ranks below the core jump.

**Independent Test**: Open the prompt and type — the buffer is not edited; press a non-digit editor
shortcut — it doesn't act on the buffer; `Esc` closes the prompt and normal editing resumes.

**Acceptance Scenarios**:

1. **Given** the Go-to-Line prompt is open, **When** the user types digits/Backspace/Enter/Esc, **Then**
   only the prompt responds and the buffer is not modified.
2. **Given** another modal is already open, **When** Go to Line would open, **Then** only one modal is
   shown at a time (no overlap/corruption).

### Edge Cases

- **Empty buffer / single line**: Go to Line 1 stays on line 1; any number clamps to line 1.
- **Number larger than the file**: clamps to the last line.
- **Zero / negative / leading zeros**: `0`→line 1; `007`→line 7; non-digits are rejected from the field.
- **Very large input** (more digits than any line count): clamps to the last line without overflow.
- **Terminal too small**: the prompt renders without corruption or panic, or degrades gracefully.
- **Cursor column on the target line**: lands at column 1 (line start), regardless of the previous column.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor MUST provide a Go-to-Line command reachable by a keyboard shortcut (`Ctrl+G`) and
  a Search-menu item.
- **FR-002**: Invoking the command MUST open a modal prompt that accepts a line number typed by the user
  (digits, with Backspace to edit) and shows the current entry.
- **FR-003**: Pressing Enter with a valid number MUST move the cursor to the start (column 1) of that
  1-based line and scroll the view so the line is visible, then close the prompt.
- **FR-004**: A number greater than the line count MUST clamp to the last line; a number below 1 MUST
  clamp to the first line.
- **FR-005**: Esc MUST close the prompt without moving the cursor.
- **FR-006**: An empty or non-numeric entry MUST NOT move the cursor unexpectedly; the field MUST reject
  non-digit characters.
- **FR-007**: While the prompt is open it MUST capture input (typing edits the number; editor keys do not
  modify the buffer), and only one modal MUST be open at a time.
- **FR-008**: The command MUST NOT change editing, search/replace, or any other dialog; it only moves the
  cursor/viewport.
- **FR-009**: The prompt MUST render width-correctly and MUST NOT panic on any terminal size, empty
  buffer, or oversized input.

### Key Entities

- **Go-to-Line prompt**: a transient modal holding the in-progress numeric entry; on confirm it yields a
  target line; on cancel it yields nothing.
- **Target line**: a 1-based line number, clamped to `[1, line_count]`, mapped to a cursor position at the
  start of that line.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can move the cursor to any specific line in a file using only the keyboard
  (`Ctrl+G`, number, Enter) in a few seconds.
- **SC-002**: Out-of-range and invalid input never move the cursor incorrectly and never error: numbers
  beyond the file clamp to the last line; below 1 clamp to the first; empty/non-numeric do nothing.
- **SC-003**: The target line is visible after the jump (scrolled into the viewport) 100% of the time.
- **SC-004**: The prompt never modifies buffer contents and never panics across terminal sizes, empty
  buffers, and oversized input.

## Assumptions

- **1-based line numbers** in the UI (the user types the line number as shown in the line-number gutter).
- **Cursor lands at column 1** of the target line (line start); horizontal position is reset.
- **Shortcut is `Ctrl+G`** and the menu item lives under **Search** (DOS EDIT.COM convention), alongside
  Find/Replace.
- **Invalid/empty on Enter** closes the prompt with no movement (a brief status notice is acceptable but
  not required); non-digit keystrokes are ignored by the field.
- **Reuse** the existing cursor-move + ensure-cursor-visible machinery and the modal-input pattern used by
  the other dialogs; no new scroll model.
- **Scope** is navigation only — no change to editing, find/replace, or other dialogs; no new config key.
- Builds on the existing modal-dialog input routing (features 015/020) and viewport/scroll machinery
  (021/023).
