# Feature Specification: UX completeness hardening (round 2)

**Feature Branch**: `029-ux-completeness-hardening`

**Created**: 2026-06-20

**Status**: Draft

**Input**: A full multi-lane UX audit surfaced a batch of verified crash, correctness, consistency, and
silent-failure defects. This feature fixes them together. Larger feature-sized parity items (mouse
editing inside dialogs, double/triple-click selection, right-click menu, extra F-keys) are explicitly
out of scope and tracked as GitHub issues + ROADMAP rows.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - No operation can crash the editor (Priority: P1)

Common operations on real-world content — opening a file with a Unicode path, deleting a selection that
spans multibyte characters, opening a very large file, or viewing the crash-recovery prompt — must never
panic.

**Why this priority**: These are hard crashes triggerable by ordinary data (non-ASCII paths/text, a big
file). Crashes are the most severe defect class and erode trust.

**Independent Test**: Delete a selection over multibyte text; show the recovery dialog with a long
Unicode path; open an oversized file — none panic.

**Acceptance Scenarios**:

1. **Given** a selection spanning multibyte characters, **When** the user deletes/cuts it, **Then** the
   text is removed and the undo entry is correct, with no panic.
2. **Given** a crash-recovery file whose path is a long Unicode string, **When** the recovery prompt is
   shown, **Then** the path is truncated safely and displayed, with no panic.
3. **Given** a file larger than a sane maximum, **When** the user opens it, **Then** the editor shows a
   clear "file too large" message instead of attempting to load it (no OOM, no panic).
4. **Given** any byte offset, **When** it is converted to a character index internally, **Then** the
   conversion never panics on a non-character-boundary offset.

---

### User Story 2 - Saving never silently loses data (Priority: P1)

When the user saves, they get clear confirmation on success and a clear error on failure — a failed save
must never look like a successful one.

**Why this priority**: A silently failed save is data loss (Constitution VII). Today plain save (Ctrl+S)
shows nothing on success and only logs on failure.

**Independent Test**: Save a writable file → see a "Saved" confirmation. Save a file that can't be
written → see an error message; the buffer stays marked modified.

**Acceptance Scenarios**:

1. **Given** a named, writable buffer, **When** the user saves, **Then** a "Saved" confirmation appears.
2. **Given** a save that fails (e.g. permission denied), **When** the user saves, **Then** an error
   message is shown and the buffer remains marked modified.
3. **Given** an autosave/recovery write that fails, **When** it happens, **Then** the user is notified
   non-intrusively rather than it failing silently.

---

### User Story 3 - Dialogs behave consistently (Priority: P1)

Every confirmation dialog that advertises a key honors it; the save-before-quit prompt cancels on Esc
like its label promises and like every other confirm dialog.

**Why this priority**: A button labeled `Cancel (Esc)` that ignores Esc is a correctness bug and a trust
problem; consistency across dialogs is a core UX expectation.

**Independent Test**: Open the save-before-quit prompt and press Esc → it cancels (nothing is saved or
discarded, the editor returns to normal).

**Acceptance Scenarios**:

1. **Given** the save-before-quit prompt, **When** the user presses Esc, **Then** it cancels (consistent
   with its `Cancel (Esc)` label and the other confirm dialogs).
2. **Given** a menu is open, **When** a Go-to-Line request arrives, **Then** it does not open over the
   menu (modal precedence is respected).

---

### User Story 4 - Saving with a chosen encoding via the browser keeps that encoding (Priority: P1)

When the user picks a save encoding and then chooses a destination through the file browser, the file is
written in the chosen encoding.

**Why this priority**: Silently dropping the chosen encoding writes the file wrong — a data-correctness
defect.

**Independent Test**: Choose a non-UTF-8 encoding, save through the browser to a path, and confirm the
file is written in the chosen encoding and the buffer reflects it.

**Acceptance Scenarios**:

1. **Given** a pending encoding selection, **When** the user completes Save-As through the file browser,
   **Then** the file is written in that encoding (the selection is not dropped).

---

### User Story 5 - The cursor lands where you click, and wide characters align (Priority: P2)

Clicking in the text places the cursor under the pointer even when the line-number gutter is shown and
the view is horizontally scrolled; combining marks, wide (CJK), and emoji characters are measured at
their true display width everywhere (cursor, scrolling, truncation, tabs, fields).

**Why this priority**: Misplaced clicks and misaligned wide characters are visible, constant friction and
violate the project's display-width mandate; but the editor is still usable, so P2.

**Independent Test**: Enable line numbers, click in the text → the cursor lands under the pointer.
Render lines with combining/CJK/emoji → columns align.

**Acceptance Scenarios**:

1. **Given** line numbers are enabled, **When** the user clicks in the text, **Then** the cursor lands at
   the clicked column (the gutter offset is accounted for).
2. **Given** the view is horizontally scrolled, **When** the user clicks, **Then** the cursor accounts
   for the scroll offset.
3. **Given** text containing combining marks, wide CJK, or emoji, **When** it is displayed and navigated,
   **Then** display-width is computed consistently (combining = 0, wide = 2) across the editor, file
   browser, tab bar, and dialog fields.

---

### User Story 6 - Actions give feedback; nothing fails in silence (Priority: P2)

Copy/cut/paste, attempts to edit a read-only buffer, and file-open failures all produce a brief, clear
message instead of silently doing nothing.

**Why this priority**: Silent no-ops make the editor feel broken ("I pressed the key and nothing
happened"); consistent feedback is a quality bar, but not a crash, so P2.

**Independent Test**: Copy with a selection → "Copied"; paste with an empty clipboard → a notice; type
in a read-only buffer → "Buffer is read-only"; open a file that fails → an error with the path/reason.

**Acceptance Scenarios**:

1. **Given** a selection, **When** the user copies or cuts, **Then** a brief confirmation appears; on a
   clipboard failure, an error appears.
2. **Given** an empty clipboard, **When** the user pastes, **Then** a notice appears (no silent no-op).
3. **Given** a read-only buffer, **When** the user attempts to edit, **Then** a "read-only" message
   appears.
4. **Given** a file that cannot be opened, **When** the user opens it (at startup or via the open
   dialog), **Then** an error message naming the path and reason is shown instead of a silent blank
   buffer.

---

### User Story 7 - The Close-buffer action is reachable, and the alternate theme is legible (Priority: P3)

Closing the current buffer works from the keyboard (and a menu item), matching the documentation; the
selected menu item is legible in every bundled theme.

**Why this priority**: Discoverability/consistency polish; the editor works without these, so P3.

**Independent Test**: Press the documented close-buffer shortcut → the current buffer closes. Switch to
the alternate theme → the highlighted menu item is readable.

**Acceptance Scenarios**:

1. **Given** the documented close-buffer shortcut, **When** the user presses it, **Then** the current
   buffer closes (and a File menu item does the same).
2. **Given** the alternate (light) theme, **When** a menu item is selected, **Then** its text is legible
   (contrasting foreground/background).

---

### Edge Cases

- Deleting an empty/reversed selection → no-op, no panic.
- Recovery path shorter than the truncation limit → shown in full.
- A file exactly at the size limit boundary → handled deterministically (documented inclusive/exclusive).
- Clicking past the end of a line / on the gutter → cursor clamps to the line; gutter clicks don't place
  the cursor.
- Paste of empty clipboard, copy with no selection → safe notice / no-op.
- Save failure followed by a successful retry → modified flag and messaging update correctly.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Deleting or cutting a selection MUST extract the removed text by character (not byte)
  indices, tolerate an empty/reversed range, and never panic on multibyte content.
- **FR-002**: The crash-recovery prompt MUST truncate a long path without splitting a character (no
  byte-boundary panic) and remain readable.
- **FR-003**: Byte-to-character conversion MUST never panic on a non-character-boundary byte offset.
- **FR-004**: Opening a file larger than a defined maximum MUST be refused with a clear "file too large"
  message rather than loading it (no OOM/panic). The limit MUST be documented.
- **FR-005**: A plain save MUST show a success confirmation; a failed save MUST show an error and leave
  the buffer marked modified (no silent success-looking failure).
- **FR-006**: Autosave/recovery write failures MUST surface a non-intrusive notice (not silent).
- **FR-007**: The save-before-quit prompt MUST cancel on Esc, consistent with its label and the other
  confirm dialogs.
- **FR-008**: Completing Save-As through the file browser MUST apply any pending encoding selection.
- **FR-009**: A click in the editor text MUST map to the correct column, accounting for the line-number
  gutter and the horizontal scroll offset; a click on the gutter MUST NOT place the cursor mid-text.
- **FR-010**: Display width MUST be computed consistently from a single width function (combining
  marks = 0, East-Asian wide = 2, emoji handled) across the editor, file browser, tab bar, and dialog
  input fields — replacing the divergent custom width helpers.
- **FR-011**: Copy and cut MUST give brief feedback; clipboard failures and empty-clipboard paste MUST
  produce a visible notice rather than a silent no-op.
- **FR-012**: Attempting to edit a read-only buffer MUST show a "read-only" message.
- **FR-013**: File-open failures (at startup and via the open dialog) MUST show an error naming the path
  and reason rather than silently producing a blank buffer.
- **FR-014**: Closing the current buffer MUST be reachable from the keyboard (the documented shortcut)
  and from a File menu item.
- **FR-015**: Every bundled theme MUST render the selected menu item with legible contrast.
- **FR-016**: A Go-to-Line request MUST NOT open while a menu is active (modal precedence respected).
- **FR-017**: All existing behavior MUST be preserved — existing keys, mouse interactions, editing
  semantics, file formats, dialogs, and the crash-log file output are unchanged except where a defect is
  being corrected.
- **FR-018**: No new third-party dependencies may be introduced (Constitution IV).

### Key Entities

- **Display-width function**: The single source of truth for how many terminal columns a grapheme
  occupies (combining = 0, wide = 2), used by all rendering and cursor/column math.
- **Status message**: The transient one-line feedback channel used to confirm actions and report errors.
- **Recovery prompt path**: The (possibly long, possibly Unicode) file path shown in the crash-recovery
  dialog.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero panics across the regression suite for multibyte delete/cut, Unicode recovery paths,
  oversized-file open, and arbitrary byte→char conversion.
- **SC-002**: A failed save is distinguishable from a successful one 100% of the time (message + retained
  modified flag).
- **SC-003**: The save-before-quit prompt cancels on Esc (verified by test), matching every other
  confirm dialog.
- **SC-004**: Saving through the browser preserves the chosen encoding in 100% of cases.
- **SC-005**: With line numbers on, a click lands on the intended column (verified across gutter +
  scroll offsets); wide/combining/emoji characters align under one shared width function.
- **SC-006**: Copy/cut/paste, read-only edits, and file-open failures each produce a visible message
  (no silent no-op) — verified per action.
- **SC-007**: The close-buffer shortcut and menu item both close the current buffer; the alternate theme
  shows a legible selected menu item.
- **SC-008**: No regression in the existing test suite.

## Assumptions

- The maximum openable file size is a fixed, documented constant chosen to prevent OOM on typical
  machines (e.g. a few hundred MB); larger files are refused with a message.
- "Brief feedback" means the existing status-message channel (one line, transient), not a modal.
- The documented close-buffer shortcut is `Ctrl+W` (as `docs/CAPABILITIES.md` already states); this
  feature makes the implementation match the docs.
- Reusing the existing `unicode-width` / `unicode-segmentation` dependencies (already in the tree) is the
  intended way to unify width; no new crates.
- Deferred items (in-dialog mouse text editing & list-item clicks, double/triple-click selection,
  right-click context menu, extra DOS F-keys) are enhancements tracked separately and are NOT part of
  this feature.
