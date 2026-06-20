# Feature Specification: Undo-to-clean state and Revert to saved

**Feature Branch**: `014-undo-clean-revert`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "Returning a buffer to its unmodified state. (1) Undo-to-clean: when the user undoes edits back to the exact point that matches the last saved version (or the originally-opened content for a never-saved-but-opened file), the buffer must no longer be marked Modified … Redoing away from that point marks it Modified again … This must remain correct even after divergent edits. (2) Revert to saved: add a File ▸ Revert menu item (and keybinding) that discards all in-editor changes and reloads the buffer from its last saved version on disk, returning to a clean unmodified state."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Undo back to saved clears the Modified flag (Priority: P1)

A user opens (or saves) a file, makes some edits — the buffer shows `[Modified]` — then presses Undo
repeatedly. When the buffer's content once again equals the last saved version (or, for a file that
was opened but never saved, its originally-loaded content), the `[Modified]` indicator disappears: the
editor recognises there is nothing unsaved. If the user then Redoes, the buffer is `[Modified]` again.

**Why this priority**: This is the core correctness fix and the most visible everyday behavior — every
editor and DOS EDIT itself does this. Today undo always re-marks the buffer Modified, so the indicator
is wrong and users can't tell when they've truly backed out their changes.

**Independent Test**: Open a file, make one edit (Modified appears), Undo once → Modified disappears;
Redo → Modified reappears. Fully testable by toggling edits and observing the flag.

**Acceptance Scenarios**:

1. **Given** a saved/opened buffer with no edits, **When** the user makes one edit, **Then** the buffer
   is shown as Modified.
2. **Given** that one edit, **When** the user Undoes it, **Then** the buffer is shown as unmodified
   (clean).
3. **Given** the buffer was just undone to clean, **When** the user Redoes, **Then** the buffer is
   shown as Modified again.
4. **Given** a buffer saved after several edits, **When** the user makes more edits and then Undoes
   back exactly to the saved point, **Then** the buffer is clean; undoing further (before the save
   point) shows Modified again.

---

### User Story 2 - No false "clean" after divergent edits (Priority: P1)

A user saves, undoes part-way, then types something new (a divergent edit that discards the redo
branch). The buffer must never be shown as clean unless its content truly matches the saved version.
Reaching the same number of undo steps via a different edit path must not be mistaken for the saved
state.

**Why this priority**: A false "clean" is worse than the original bug — the user could quit believing
their work is saved when it is not. Correctness of the indicator is safety-critical for data loss.

**Independent Test**: Save; make edit A then edit B; Undo once (back to A); type edit C (diverging);
the buffer must show Modified even if the number of edits since save coincidentally matches. Confirm
the only "clean" state is genuine content equality with the saved version.

**Acceptance Scenarios**:

1. **Given** a saved buffer, **When** the user makes an edit, undoes it (clean), then makes a different
   edit, **Then** the buffer is Modified.
2. **Given** a saved buffer with edits, **When** the user undoes part-way and then makes a new
   (divergent) edit, **Then** the buffer is Modified and can no longer be returned to clean by
   undo/redo alone (the prior saved point became unreachable).
3. **Given** any sequence of edits/undos/redos, **When** the buffer is shown clean, **Then** its
   content equals the last saved (or originally-opened) version.

---

### User Story 3 - Revert to saved discards all changes (Priority: P2)

A user has made changes they want to abandon. They choose **File ▸ Revert** (or its keybinding). The
buffer is restored to its last saved version on disk and shown as clean (unmodified), with the cursor
placed at a sensible position. If the buffer has unsaved changes, the user is asked to confirm before
the changes are discarded.

**Why this priority**: A one-step "throw away my changes" is a common, expected escape hatch and is the
explicit second half of the request. It's P2 because US1/US2 already let a user back out via undo; this
adds a direct, whole-file path.

**Independent Test**: Open a file, edit it, choose Revert, confirm → buffer matches the on-disk file and
is clean. Testable end-to-end via the menu action.

**Acceptance Scenarios**:

1. **Given** a file-backed buffer with unsaved edits, **When** the user invokes Revert and confirms,
   **Then** the buffer content equals the file on disk and the buffer is clean.
2. **Given** a file-backed buffer with unsaved edits, **When** the user invokes Revert and cancels at
   the confirmation, **Then** nothing changes (edits and Modified state preserved).
3. **Given** a clean file-backed buffer (no edits), **When** the user invokes Revert, **Then** it
   reloads from disk with no harm (and no confirmation needed, since nothing would be lost).
4. **Given** a buffer that has never been saved (no file path), **When** the user invokes Revert,
   **Then** the action reports it cannot revert (no saved version exists) and changes nothing.

---

### Edge Cases

- **Never-saved-but-opened file**: the "clean" baseline is the content loaded when the file was opened
  (the originally-opened content equals the on-disk version at open time). Undo back to that content is
  clean.
- **Brand-new empty buffer (no path, no edits)**: starts clean; after an edit it is Modified; undo back
  to empty is clean again. Revert on it is a no-op/“nothing to revert”.
- **Revert when the file no longer exists / is unreadable on disk**: report an error and leave the
  buffer unchanged (do not blank the buffer).
- **Revert with unsaved changes**: must confirm before discarding (avoid silent data loss).
- **Divergent-edit invalidation**: after a new edit that discards the redo branch, a saved point that
  lived in the discarded branch must be treated as unreachable so the buffer is never falsely clean.
- **Save-As to a new path**: the clean baseline updates to the just-written content (subsequent undo to
  that point is clean).
- **Redo across the saved point**: redoing from clean back to a post-save edit marks Modified again.

## Clarifications

### Session 2026-06-20

- Q: What keyboard shortcut should File ▸ Revert use? → A: Menu-only — no keyboard shortcut. Revert is
  reachable via **File ▸ Revert** (and its `R` menu accelerator from feature 013) only. This avoids any
  keybinding collision and keeps the surface minimal.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When the buffer's content returns (via undo/redo) to exactly the last saved version, the
  editor MUST mark the buffer as unmodified (clean) and hide the `[Modified]` indicator.
- **FR-002**: When the buffer's content moves away from the saved version (via redo or a new edit), the
  editor MUST mark the buffer as modified and show the `[Modified]` indicator.
- **FR-003**: For a buffer that was opened from a file but not yet saved in this session, the "clean"
  baseline MUST be the content loaded at open time; undoing to that content MUST show clean.
- **FR-004**: The editor MUST NOT show a buffer as clean unless its content genuinely equals the saved
  (or originally-opened) baseline — in particular, after a divergent edit that discards the redo branch,
  a now-unreachable saved point MUST NOT produce a false clean state.
- **FR-005**: A successful Save (or Save As) MUST set the current content as the new clean baseline, so
  later undo back to that point shows clean and the indicator is correct immediately after saving.
- **FR-006**: The editor MUST provide a **Revert** action available from **File ▸ Revert** (menu-only,
  no keyboard shortcut — clarified 2026-06-20) that reloads the buffer from its last saved version on
  disk and returns it to a clean state.
- **FR-007**: When Revert is invoked on a buffer with unsaved changes, the editor MUST ask the user to
  confirm before discarding; on cancel, nothing changes.
- **FR-008**: After a confirmed Revert, the buffer content MUST equal the on-disk file, the buffer MUST
  be clean, and the cursor MUST be at a valid position within the reloaded content.
- **FR-009**: Revert on a buffer with no associated file (never saved) MUST be a safe no-op that informs
  the user there is no saved version to revert to, changing nothing.
- **FR-010**: If Revert fails to read the file from disk (missing/unreadable), the editor MUST report
  the error and leave the buffer and its Modified state unchanged.
- **FR-011**: These behaviors MUST NOT regress existing undo/redo, save, autosave, or the modified
  indicator for ordinary editing; ordinary edits still mark the buffer Modified.

### Key Entities *(include if feature involves data)*

- **Clean baseline (saved point)**: the buffer state considered "saved" — the content last written to
  disk, or the content loaded at open. The Modified indicator is derived from whether the current
  content matches this baseline. Becomes unreachable when a divergent edit discards the branch that
  contained it.
- **Modified indicator**: the user-visible `[Modified]` status shown when current content ≠ clean
  baseline.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After making N edits and undoing exactly N times on a saved/opened buffer, the buffer is
  shown clean in 100% of cases.
- **SC-002**: Redoing one step from a clean state shows Modified in 100% of cases.
- **SC-003**: In zero cases is the buffer shown clean while its content differs from the saved/opened
  baseline (no false-clean), including after divergent-edit sequences.
- **SC-004**: A user can discard all changes and return to the on-disk version in a single Revert action
  (one menu selection or one keybinding, plus a confirmation when changes exist).
- **SC-005**: Reverting a file-backed buffer yields content byte-for-byte equal to the on-disk file in
  100% of cases (subject to the editor's normal encoding/line-ending handling on load).
- **SC-006**: No regression: existing undo/redo, save, and editing tests continue to pass, and ordinary
  edits still show Modified.

## Assumptions

- The clean/dirty state is derived from whether the current content equals the saved baseline, tracked
  through the existing undo history; this is more reliable than a flag toggled on every edit.
- Revert reloads through the editor's existing file-open path, so encoding detection/transcoding and
  line-ending handling apply exactly as for a normal open (revert == reload).
- Revert is menu-only (no keybinding), reached via File ▸ Revert / its `R` accelerator (clarified
  2026-06-20).
- Confirmation for a destructive Revert reuses the editor's existing modal confirmation style.
- Multiple open buffers each track their own clean baseline independently.
