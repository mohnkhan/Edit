# Feature Specification: UX crash-safety and keyboard navigation hardening

**Feature Branch**: `028-ux-crashfix-keyboard-nav`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: cluster of confirmed crash-safety and keyboard-UX defects reported with screenshots and crash logs (session-restore crash, garbled terminal after a crash, dead/invisible Save-As typing, no arrow-key button movement, no keyboard scrolling in Help, unbound Home/End, no PageUp/PageDown in lists).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Restoring a session never crashes (Priority: P1)

A user launches the editor, is offered "Restore previous session?", and chooses to restore. The editor opens the previously-open files and keeps working — it must never panic, even when soft-wrap is enabled, when some files moved or shrank, or when the previous viewport no longer matches the restored content.

**Why this priority**: This is a hard crash on a common startup path; today it panics and leaves the terminal unusable. It blocks normal use and risks the user's confidence in the tool.

**Independent Test**: Enable soft-wrap, switch/restore to a buffer whose content differs from the previously-rendered one (including an empty line), and render — the app draws the restored content without panicking.

**Acceptance Scenarios**:

1. **Given** soft-wrap is on and a session referencing files with different line lengths, **When** the user restores the session, **Then** the restored files render correctly and the app does not panic.
2. **Given** any active-buffer change while soft-wrap is on (switch to next/previous buffer, open a file, close a buffer), **When** the next frame renders, **Then** the wrap layout matches the now-active buffer's content and no out-of-bounds slice occurs.
3. **Given** a restored buffer whose saved cursor/scroll position is past the new end of file, **When** it renders, **Then** the position is clamped into range with no panic.

---

### User Story 2 - A crash leaves the terminal usable (Priority: P1)

If the editor ever does panic, the user's terminal must be returned to a normal state (cooked mode, primary screen, visible cursor) and the crash message must be readable, so the shell is immediately usable again instead of "hanging" with a garbled display.

**Why this priority**: Even with US1 fixing the known crash, any future panic must fail safe. Today a panic leaves raw mode + the alternate screen active, so the terminal appears frozen and the report is unreadable.

**Independent Test**: Trigger a panic in a build and confirm the terminal is restored (raw mode disabled, alternate screen left, cursor shown) and the crash report is printed legibly to stderr; a crash report file is still written.

**Acceptance Scenarios**:

1. **Given** the editor panics while running, **When** the process exits, **Then** the terminal is in cooked mode on the primary screen with a visible cursor and the panic message is readable.
2. **Given** a panic, **When** the report is produced, **Then** the crash-log file is still written (existing behavior preserved).

---

### User Story 3 - Typing into the Save-As / path field works and is visible (Priority: P1)

When the user opens Save-As (or any interactive dialog with a text field) and types a filename or path, the characters appear in the field with a visible caret, every time, regardless of what dialog was used before.

**Why this priority**: Save-As is a core action; today keystrokes can be silently swallowed because focus lands on a button, making it impossible to name a new file.

**Independent Test**: Open the Save browser fresh (and again after a different dialog was used), type characters, and confirm they accumulate in the filename field and the caret is shown.

**Acceptance Scenarios**:

1. **Given** the Save browser is opened, **When** the user types printable characters, **Then** each character is appended to the filename field and shown with a caret.
2. **Given** a confirm dialog (e.g. a save prompt) was just dismissed, **When** the user next opens an interactive dialog with a field, **Then** focus starts on the field (not a button) so typing works immediately.

---

### User Story 4 - Arrow keys move between dialog buttons (Priority: P2)

In any dialog with more than one button, the user can move the focus between buttons using the arrow keys, not only Tab/Shift+Tab or the mouse.

**Why this priority**: Arrow keys are the intuitive way to move between on-screen buttons; their absence is a discoverability and accessibility gap, but the dialogs are still operable via Tab today.

**Independent Test**: Open a multi-button dialog, press Left/Right (and Up/Down), and confirm the focused button advances/retreats and wraps, matching Tab behavior.

**Acceptance Scenarios**:

1. **Given** a confirm dialog with multiple buttons, **When** the user presses Right (or Down), **Then** focus advances to the next button, wrapping at the end.
2. **Given** the same dialog, **When** the user presses Left (or Up), **Then** focus retreats to the previous button, wrapping at the start.
3. **Given** an interactive dialog whose focus is on a button, **When** the user presses the arrow keys, **Then** focus cycles through the button ring consistently with Tab.

---

### User Story 5 - Help/About scroll and close from the keyboard (Priority: P2)

The user can scroll the Help and About overlays with the keyboard (Up/Down, PageUp/PageDown, Home/End) and dismiss them from the keyboard, without needing a mouse.

**Why this priority**: The cheat-sheet overflows on small terminals; keyboard-only users currently cannot read all of it. Esc already closes, so the surface is not unusable, hence P2.

**Independent Test**: Open Help on a short terminal, press Down/PageDown/End to scroll to the bottom and Up/PageUp/Home to return, and confirm the scroll offset tracks and stays in range; Esc still closes.

**Acceptance Scenarios**:

1. **Given** the Help overlay overflows the viewport, **When** the user presses Down/PageDown/End, **Then** the content scrolls (clamped to the last page) and Up/PageUp/Home scroll back.
2. **Given** the Help/About overlay, **When** the user presses Esc (or the key/keys bound to dismiss), **Then** it closes.

---

### User Story 6 - Home/End and paging work where text and lists live (Priority: P3)

Home and End move the cursor to the start/end of the current line in the editor, and PageUp/PageDown jump by a page in scrollable lists (file browser, encoding select, plugin manager), consistent with the editor.

**Why this priority**: Quality-of-life consistency; the editor and dialogs already support arrow movement, so these are incremental improvements.

**Independent Test**: In the editor press Home/End and confirm the cursor goes to column 0 / line end; in a long file browser listing press PageDown/PageUp and confirm the selection jumps by a page.

**Acceptance Scenarios**:

1. **Given** the cursor mid-line in the editor, **When** the user presses Home then End, **Then** the cursor moves to the first then last column of that line.
2. **Given** a list longer than the visible area, **When** the user presses PageDown/PageUp, **Then** the selection moves by approximately one visible page and stays in range.

---

### Edge Cases

- Restoring a session where every referenced file is missing → keep a blank buffer and show a status message; no crash (existing behavior preserved).
- An empty line (length 0) under a stale wrap layout → renders as a blank line, never slices out of bounds.
- A terminal smaller than the minimum size during/after restore → no panic; the too-small notice path is unaffected.
- Pressing arrow keys on a single-button dialog → focus stays on the only button (no-op, no panic).
- Pressing PageUp/PageDown on an empty or single-item list → no movement, no underflow.
- A copy/cut with a zero-width or reversed selection range → produces empty/clipboard-safe text, no slice panic.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor renderer MUST NOT panic on any combination of buffer content and cached wrap layout; all runtime string slices and indexed accesses into line content MUST be bounds-clamped to the current line length.
- **FR-002**: The soft-wrap cache MUST be invalidated whenever the active buffer's content changes identity or content — specifically on session restore, switching to the next/previous buffer, opening a file into a new buffer, and closing a buffer — so the rendered wrap layout always matches the active buffer.
- **FR-003**: On a panic, the system MUST restore the terminal to a usable state (disable raw mode, leave the alternate screen, disable mouse capture, show the cursor) before writing the human-readable report to the error stream, and MUST still write the crash-log file.
- **FR-004**: Other unchecked runtime operations identified in the audit MUST be hardened against panic/underflow: the copy/cut selection slice MUST tolerate an empty or reversed range, and the file-browser scroll arithmetic MUST not underflow.
- **FR-005**: When an interactive dialog that has a primary text/list control opens, focus MUST start on that primary control (not a button), so typed characters reach the field and the caret is shown.
- **FR-006**: Users MUST be able to move focus between a dialog's buttons using the arrow keys (Left/Right, and Up/Down where the layout is a single row of buttons) in both the confirm/dismiss dialogs and the interactive dialog button rings, with wrap-around consistent with Tab/Shift+Tab.
- **FR-007**: Users MUST be able to scroll the Help and About overlays using Up/Down, PageUp/PageDown, and Home/End, with the scroll offset clamped to the content, and MUST be able to dismiss them from the keyboard.
- **FR-008**: Home and End MUST move the editor cursor to the start and end of the current line, respectively.
- **FR-009**: Scrollable lists (file browser, encoding select, plugin manager) MUST support PageUp/PageDown to move by approximately one visible page, clamped to the list bounds.
- **FR-010**: All existing behavior MUST be preserved: existing dialog keys (Tab/Shift+Tab, Enter/Space, Esc, list navigation, option toggles, match navigation), mouse interactions, editing semantics, file formats, and the crash-log file output are unchanged.
- **FR-011**: No new third-party dependencies may be introduced (Constitution Principle IV).

### Key Entities

- **Wrap cache**: The cached per-line visual-row layout used by soft-wrap rendering; valid only for a specific buffer content generation and width.
- **Dialog focus ring**: The ordered set of focus stops for a dialog (primary control then buttons) that arrow/Tab navigation cycles through.
- **Panic hook**: The process-wide handler that runs on a Rust panic to restore the terminal, print the report, and write the crash log.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Restoring a previous session with soft-wrap enabled completes with zero panics across the regression suite (was: reproducible crash).
- **SC-002**: After any induced panic, the terminal is left in a usable state and the crash message is readable in 100% of cases.
- **SC-003**: Typing a filename in a freshly-opened Save-As dialog shows every typed character on the first attempt, including immediately after another dialog was used.
- **SC-004**: In every multi-button dialog, arrow keys move the focus between buttons (verified by tests for each dialog family).
- **SC-005**: The Help/About overlay can be fully read and dismissed using only the keyboard.
- **SC-006**: No regression in existing dialog, editing, mouse, or crash-log behavior (full existing test suite remains green).

## Assumptions

- The existing crossterm-based terminal teardown sequence used on normal exit is the correct sequence to reuse in the panic hook.
- "Page" for list paging means approximately the number of visible rows, matching the editor's existing PageUp/PageDown semantics.
- For confirm dialogs whose buttons are laid out in a single horizontal row, Left/Right and Up/Down are treated as equivalent previous/next movements.
- Invalidating the wrap cache by forcing a rebuild on the next frame (the existing staleness mechanism) is acceptable and within the editor's render-performance budget.
- Reusing the existing focus-ring, scroll, and cursor-move helpers (no new subsystems) is sufficient; this feature adds no new file formats or persisted data.
