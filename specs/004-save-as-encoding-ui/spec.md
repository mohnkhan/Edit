# Feature Specification: Save-As Encoding Selection UI

**Feature Branch**: `004-save-as-encoding-ui`

**Created**: 2026-06-19

**Status**: Draft

**Input**: User description: "An interactive modal listbox dialog inside the TUI that lets
the user pick the output encoding (UTF-16 LE, UTF-16 BE, UTF-8, CP437/CP850) when saving
a file. The transcoding plumbing from feature 002 already exists. This dialog is triggered
via File > Save As Encoding... menu item and/or an F-key binding (F12 or similar). On
confirmation the file is written in the selected encoding using the existing encode pipeline.
On cancel the file is unchanged. The selected encoding is shown in the status bar after a
successful save."

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Save Active Buffer in a Chosen Encoding (Priority: P1)

A user has a UTF-8 file open and needs to save it in UTF-16 LE for a legacy Windows tool.
They open the encoding dialog from the File menu or via the assigned F-key, select
"UTF-16 LE" from the listbox using arrow keys, press Enter, and the file is immediately
written to disk in the selected encoding. The status bar confirms the new encoding, and
all subsequent saves for that buffer use the newly selected encoding.

**Why this priority**: This is the entire value proposition of the feature. Without it the
feature does not exist.

**Independent Test**: Open any file, trigger the encoding dialog, select UTF-16 LE, confirm
— the saved file on disk starts with the UTF-16 LE BOM (`FF FE`) and contains correctly
encoded content.

**Acceptance Scenarios**:

1. **Given** a buffer containing text is open, **When** the user opens the encoding dialog
   and selects UTF-16 LE and confirms, **Then** the file on disk is rewritten in UTF-16 LE
   with BOM and the status bar shows "Saved as UTF-16 LE".
2. **Given** the encoding dialog is open with UTF-16 LE highlighted, **When** the user
   presses Enter, **Then** the save is executed without further prompts.
3. **Given** a buffer whose file path already exists, **When** the user saves with a new
   encoding, **Then** the original file is atomically replaced (no partial write corruption).

---

### User Story 2 — Cancel Without Saving (Priority: P2)

A user accidentally opens the encoding dialog. They press Escape (or select Cancel) and
the dialog closes with no file I/O performed. The buffer and its existing encoding are
unchanged.

**Why this priority**: Safe cancellation is a required complement to any modal dialog.

**Independent Test**: Open the dialog, press Escape — the file on disk is unchanged
(byte-for-byte identical to before); the editor's active encoding for that buffer is
unchanged.

**Acceptance Scenarios**:

1. **Given** the encoding dialog is open, **When** the user presses Escape, **Then** the
   dialog closes and the buffer file is untouched.
2. **Given** the encoding dialog is open, **When** the user presses Escape, **Then** the
   status bar does not display any encoding-change message.

---

### User Story 3 — Encoding Persists for Subsequent Saves (Priority: P3)

After a user saves a buffer with a new encoding via the dialog, pressing the regular Save
shortcut (Ctrl+S / F5) on the same buffer again saves in the same encoding — no re-selection
required.

**Why this priority**: Without persistence, every subsequent save silently reverts to UTF-8,
corrupting files the user believes are UTF-16.

**Independent Test**: Save via the encoding dialog (UTF-16 LE); then Ctrl+S; inspect the
file on disk — it is still UTF-16 LE.

**Acceptance Scenarios**:

1. **Given** a buffer whose encoding was changed via the dialog to UTF-16 BE, **When**
   the user saves again with Ctrl+S, **Then** the file on disk is UTF-16 BE.
2. **Given** a buffer whose encoding is UTF-16 LE, **When** the user opens the encoding
   dialog again, **Then** the listbox pre-selects UTF-16 LE (current buffer encoding).

---

### User Story 4 — Unsaved New Buffer Triggers Save-As Path (Priority: P4)

A user has a new buffer (never saved to disk) and opens the encoding dialog. Since no
file path exists, the editor first prompts for a file name (the existing Save-As filename
dialog), then writes the file in the selected encoding.

**Why this priority**: The feature must compose cleanly with the existing "new file" save
flow, otherwise unsaved buffers are a silent edge case.

**Independent Test**: New buffer (no path), open encoding dialog, pick UTF-16 BE, enter a
filename — file on disk is UTF-16 BE with correct BOM.

**Acceptance Scenarios**:

1. **Given** a buffer with no file path (new, unsaved), **When** the user opens the
   encoding dialog and confirms an encoding, **Then** the filename prompt appears before
   the file is written.
2. **Given** a buffer with no file path, **When** the user cancels the filename prompt,
   **Then** no file is written and the encoding selection is discarded.

---

### Edge Cases

- What happens when the file is read-only (permissions)?  
  → The encoding dialog opens normally. On confirmation the save attempt fails with a
  status-bar error (`"Save failed: permission denied"`); the buffer's encoding is not
  updated; the original file is unchanged.
- What happens when the disk is full or the write fails mid-transfer?  
  → Atomic write (tmp-rename) prevents partial corruption; a status-bar error is shown
  (`"Save failed: <reason>"`); the original file is preserved.
- What happens when the selected encoding cannot represent the buffer content?  
  → The encode call fails; the save fails with a status-bar error
  (`"Save failed: <encoding error>"`); the buffer's encoding is not updated; the file is
  unchanged.
- What happens when the user selects the same encoding already in use?  
  → The save proceeds normally (the write is performed; the result is an identical file on
  disk); no special message. This is the idempotent case.
- What happens when the active buffer is empty (zero bytes of content)?  
  → The save produces a BOM-only file for UTF-16 LE/BE encodings (BOM bytes only, no
  content), or a zero-byte file for all other encodings. No special case required.
- What happens when terminal dimensions are too small to display the dialog?  
  → The dialog is clamped to available terminal size; text is truncated with a trailing `…`
  when the available width is less than the dialog's target width (40 columns); encoding
  selection remains functional down to 20 columns × 5 rows.
- What happens when the terminal is resized while the dialog is open?  
  → The dialog re-renders at the new terminal dimensions on the next frame; the user's
  current list selection is preserved; no state is lost.
- What happens when the active buffer has unsaved changes (dirty buffer)?  
  → The encoding dialog saves the buffer's current in-memory content (including dirty
  changes), equivalent to a regular save. The dirty-flag is cleared on success.
- What happens when no buffer is active?  
  → The Save As Encoding action is a no-op; the encoding dialog does not open.
- What happens when the encoding dialog is triggered while already open (re-entry)?  
  → The second trigger is ignored; only one encoding dialog instance is shown at a time.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor MUST expose a "Save As Encoding..." action reachable from the
  File pull-down menu (as a new menu item after the existing "Save As..." item).
- **FR-002**: The editor MUST expose the same action via a dedicated keyboard shortcut
  (F12, consistent with the existing F-key binding row).
- **FR-003**: Triggering the action MUST open a modal TUI dialog titled "Save As Encoding"
  rendered as an overlay inside the existing ratatui frame, styled with the application's
  theme colors (blue background, matching the DOS-faithful UI). The dialog MUST include a
  hint line reading `[↑↓] Select  [Enter] Save  [Esc] Cancel`.
- **FR-004**: The dialog MUST display a fixed listbox of all supported output
  encodings: UTF-8, UTF-16 LE, UTF-16 BE, CP437, CP850, ISO-8859-1, Windows-1252.
  (Seven items fit without scrolling; no scroll indicator is required.)
- **FR-005**: The listbox MUST pre-select the encoding currently assigned to the active
  buffer when the dialog opens.
- **FR-006**: The user MUST be able to navigate the listbox with the Up/Down arrow keys
  (keyboard-only; mouse input is out of scope). Navigation MUST wrap around: pressing ↑ on
  the first item selects the last item; pressing ↓ on the last item selects the first item.
  The user confirms a selection with Enter.
- **FR-007**: The user MUST be able to dismiss the dialog without saving by pressing Escape.
  All in-progress dialog state (the current list selection) MUST be discarded on dismissal;
  no pending encoding state remains in memory after the dialog closes.
- **FR-008**: On confirmation, the editor MUST write the active buffer's current in-memory
  content (including any unsaved/dirty changes) to disk using the selected encoding via the
  existing encode pipeline (feature 002 infrastructure). The write MUST use the atomic
  tmp-rename pattern: write to a temporary file first, then rename to the target path.
- **FR-009**: On a successful save, the editor MUST update the active buffer's encoding
  to the selected value so that subsequent saves (Ctrl+S / F5) use the new encoding.
  The buffer's encoding MUST NOT be updated until the write fully succeeds — any I/O or
  encoding failure MUST leave the buffer's encoding at its pre-dialog value.
- **FR-010**: On a successful save, the editor MUST display a confirmation message in the
  status bar using the exact label for the selected encoding:
  UTF-8 → `"Saved as UTF-8"`, UTF-16 LE → `"Saved as UTF-16 LE"`,
  UTF-16 BE → `"Saved as UTF-16 BE"`, CP437 → `"Saved as CP437"`,
  CP850 → `"Saved as CP850"`, ISO-8859-1 → `"Saved as ISO-8859-1"`,
  Windows-1252 → `"Saved as Windows-1252"`.
- **FR-011**: If the active buffer has no file path (new, unsaved buffer), the editor MUST
  invoke the existing filename-prompt flow (the same dialog triggered by File > Save As...)
  before writing the file. The selected encoding is held in memory across this prompt; if
  the user cancels the filename prompt, the encoding selection is also discarded.
- **FR-012**: If the write fails for any reason — permission error, disk full, encoding
  failure (the buffer contains characters that cannot be represented in the selected
  encoding), or any other I/O error — the editor MUST display a status-bar error message
  of the form `"Save failed: <reason>"`. The buffer's encoding MUST be reverted to its
  pre-dialog value before the error message is rendered. The original file MUST remain
  intact (the atomic tmp-rename pattern prevents partial writes).
- **FR-013**: The encoding dialog MUST render entirely within the TUI frame without
  spawning a separate OS dialog, subprocess, or terminal. When the terminal is smaller
  than the dialog's target dimensions (40 columns × 11 rows), the dialog MUST clamp to
  the available terminal size and truncate label text with a trailing `…` where the
  available width is insufficient. The dialog MUST remain functional (keyboard input
  accepted) down to 20 columns × 5 rows; below this floor it need not render.

### Key Entities

- **EncodingSelection**: The user's chosen encoding within the dialog session; discarded on
  cancel, applied on confirm.
- **BufferEncoding**: The encoding assigned to an open buffer; set on file open (auto-detect
  or CLI override) and updated on every encoding-dialog save; governs all subsequent saves.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can change a buffer's encoding to any of the 4 list items nearest
  the pre-selected item and save the file in under 5 keystrokes after the triggering
  action (F12 or menu selection). The trigger itself is not counted; the counted keystrokes
  are: list navigation (0–4 ↑/↓ presses) plus Enter (1 press).
- **SC-002**: 100% of files saved via the encoding dialog produce a byte-for-byte valid
  representation of the selected encoding. Verified by round-trip decode: the written file
  is read back and decoded to UTF-8 using the same encoding, and the result MUST be
  byte-for-byte identical to the original buffer content. No mojibake permitted.
- **SC-003**: Cancelling the dialog leaves the file on disk unchanged (byte-identical to
  the pre-dialog state, verified by SHA-256 checksum comparison of the file before and
  after the dialog is opened and dismissed).
- **SC-004**: A file saved with encoding E via the dialog is saved again in encoding E
  on the next Ctrl+S invocation — no silent reversion to UTF-8.
- **SC-005**: The encoding dialog opens and is ready for input within 16 ms of the
  triggering keystroke on standard desktop hardware (no perceptible rendering delay);
  no intermediate frame is rendered without the dialog visible after the trigger.
- **SC-006**: The feature adds no new Cargo dependencies (all required encoding and TUI
  infrastructure is already present from features 002 and 003).

---

## Assumptions

- All seven `EncodingId` variants currently in the registry (`Utf8`, `Utf16Le`, `Utf16Be`,
  `Cp437`, `Cp850`, `Iso8859_1`, `Windows1252`) are appropriate to expose in the dialog;
  no filtering or per-user permission model is required.
- The existing `encode()` function in `src/encoding/transcode.rs` is the authoritative
  write path and handles all 7 `EncodingId` variants for valid UTF-8 input (established
  by feature 002 integration tests); no new transcoding logic is needed.
- The existing atomic-write pattern (write to `.tmp`, rename) used in the session module
  is the correct write strategy for this feature too.
- The F12 key is unbound in the current keymap (verified in `src/input/keymap.rs`) and is
  available for this feature. F12 is the default binding; it may be rebound by the user
  via the standard keymap configuration system.
- The encoding auto-detected at file open (or set via CLI override) is stored as a
  concrete `EncodingId` value in `buf.encoding`. The dialog's pre-selection reads this
  value directly — there is no distinction between "auto-detected" and "user-set" encodings.
- The File menu is the canonical access point; no secondary access via the Options or
  View menus is required.
- "Save As Encoding..." is distinct from "Save As..." (filename change) — they are
  separate menu items and separate actions; the encoding dialog does not change the
  filename.
- The status bar encoding display (already present from feature 002) will reflect the
  new encoding after a successful save without additional work beyond updating `buf.encoding`.
- Mouse support for the listbox is out of scope for this feature (keyboard-only navigation
  is sufficient for the DOS-faithful UX goal).

## Dependencies

- **Feature 002 (Encoding Pipeline)**: Provides `encode()` in `src/encoding/transcode.rs`,
  `EncodingId` enum in `src/encoding/detect.rs`, and the atomic-write pattern. This
  feature extends but does not modify the feature 002 infrastructure.
- **Feature 003 (Session Restore)**: Established the modal dialog overlay pattern
  (`Option<T>` state fields in `App`, ratatui overlay rendering in `src/ui/mod.rs`). This
  feature reuses that pattern for the encoding dialog state.
- **Downstream**: Future features that read `buf.encoding` to determine the write encoding
  (e.g., auto-save, export) depend on the invariant in FR-009 that `buf.encoding` is
  always authoritative and updated only on a fully successful write.
