# Feature Specification: Session Restore

**Feature Branch**: `003-session-restore`

**Created**: 2026-06-18

**Status**: Draft

**Input**: User description: "Feature 003 — Session Restore. On a clean exit (user-initiated quit, not crash), write a session.toml to $XDG_STATE_HOME/edit/ capturing all open buffers (file paths), per-buffer cursor positions (line, column), the current split layout (horizontal/vertical/none, which pane is active), and the active buffer index. On startup without explicit file arguments, detect the session file, prompt the user ("Restore previous session? [Y/n]"), and if confirmed re-open all recorded paths, seek each buffer's cursor to its saved position, and restore the split layout. If a recorded file no longer exists, skip it with a status-bar warning rather than failing the whole restore. If all recorded files have gone missing, fall back to a blank buffer. The session file must be human-readable TOML. A --no-session flag suppresses the prompt entirely. The feature must integrate with the existing XDG directory helpers in src/ and the clean-exit path already used by the crash-recovery subsystem."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Restore Previous Session on Startup (Priority: P1)

A user opens the editor, edits three files across a split-view layout, then quits normally
(File > Quit or Ctrl+Q). The next time they launch the editor without specifying any files,
they are prompted "Restore previous session? [Y/n]". Pressing Enter (or Y) re-opens all
three files with each cursor on the exact line and column it was when the editor was closed,
and the split layout is exactly as it was.

**Why this priority**: This is the core value proposition — users lose no context when
returning to work after closing the editor. It is independently valuable without any of the
lower-priority stories.

**Independent Test**: Launch editor, open two files in split view, position cursors,
quit cleanly, relaunch, confirm restore prompt, verify files open at correct cursor
positions and layout.

**Acceptance Scenarios**:

1. **Given** the editor has previously been quit cleanly with two files open in a vertical
   split, **When** the editor is launched with no file arguments, **Then** the restore prompt
   appears, pressing Y reopens both files in a vertical split with cursors at their saved
   line/column positions.
2. **Given** the editor is launched with no prior session file, **When** it starts,
   **Then** no restore prompt appears and a blank buffer opens normally.
3. **Given** the restore prompt is shown, **When** the user presses N or Escape,
   **Then** the editor opens with a blank buffer and the session file is left unchanged.

---

### User Story 2 - Handle Missing Files Gracefully (Priority: P2)

After saving a session, a user deletes or moves one of the recorded files before relaunching
the editor. When they confirm the restore prompt, the editor reopens the files that still exist,
shows a status-bar warning for each missing file, and does not crash or abort the restore.

**Why this priority**: Without graceful degradation the feature becomes a liability — a
missing file would block access to all other restored files. This story ensures the feature
remains safe under real-world file system changes.

**Independent Test**: Create a session with two files, delete one, relaunch, confirm restore,
verify the surviving file opens correctly and a warning appears for the missing one.

**Acceptance Scenarios**:

1. **Given** the session records two files and one has been deleted, **When** the user
   confirms the restore, **Then** the surviving file opens at its saved cursor position
   and the status bar displays a warning naming the missing file.
2. **Given** the session records files and all have been deleted, **When** the user confirms
   the restore, **Then** a blank buffer opens and the status bar shows a warning that no
   session files could be found.
3. **Given** a recorded file exists but is no longer readable (permission denied), **When**
   the user confirms the restore, **Then** that file is skipped with a warning and other
   files restore normally.

---

### User Story 3 - Suppress Session Prompt via Flag (Priority: P3)

A developer scripting or batch-using the editor passes `--no-session` on the command line.
The restore prompt is skipped entirely and the editor opens with a blank buffer, regardless
of whether a session file exists.

**Why this priority**: Allows headless/non-interactive use without user interaction blocking
startup. It is a quality-of-life feature for power users and scripting, not required for
the core restore flow.

**Independent Test**: Create a session file, launch with `--no-session`, verify no prompt
appears and the editor opens a blank buffer.

**Acceptance Scenarios**:

1. **Given** a valid session file exists, **When** the editor is launched with `--no-session`,
   **Then** no restore prompt is shown and the editor opens a blank buffer.
2. **Given** the editor is launched with both `--no-session` and explicit file arguments,
   **Then** the specified files open normally and no restore prompt appears.

---

### User Story 4 - File Arguments Bypass Session Restore (Priority: P4)

A user launches the editor with one or more explicit file paths on the command line. No
restore prompt appears — the specified files open directly, just as in the original
EDIT.COM behavior.

**Why this priority**: Preserves the expected CLI contract. Users passing files explicitly
should not be interrupted by a session prompt.

**Independent Test**: Create a session file, launch with an explicit file path, verify
no prompt and the specified file opens.

**Acceptance Scenarios**:

1. **Given** a session file exists and the editor is launched with an explicit file path,
   **When** it starts, **Then** the specified file opens with no restore prompt.

---

### Edge Cases

- What happens if the session file is corrupt or has invalid TOML?
  The editor silently ignores the corrupt file, shows a status-bar warning, and opens a
  blank buffer. The corrupt file is overwritten on the next clean exit.
- What happens if the session file records a directory path instead of a file?
  The directory path is skipped with a status-bar warning (same as a missing file).
- What happens if the editor crashes (not a clean exit)?
  No session file is written. The existing recovery mechanism handles crash-exit separately.
  On the next launch a session restore prompt does NOT appear for crashed sessions; only
  the existing crash-recovery prompt is shown.
- What happens if `$XDG_STATE_HOME` is not writable?
  The session save silently fails with a logged warning; the editor does not show an error
  to the user on exit. On the next launch, no restore prompt appears.
- What happens if the session records more than 20 buffers?
  All recorded buffers are restored (no arbitrary cap). This is expected behavior for power
  users with many files open.
- What happens if a restored cursor position (line or column) exceeds the actual length
  of the reopened file? (The file may have been edited externally between sessions.)
  The cursor is clamped to the last line of the file and to the last column of that line.
  No warning is shown; this is transparent to the user.
- What happens if the TUI fails to initialize before the restore prompt can be shown
  (e.g., terminal too small, ncurses init error)?
  The restore prompt is skipped. The editor handles TUI init failure through its existing
  error path (typically exiting with a message to stderr).
- What happens if both a crash-recovery and a session restore are pending on startup?
  The crash-recovery prompt is shown first. The session restore prompt is shown
  immediately after, only if the user dismisses or completes the crash recovery. The two
  flows are independent and sequential.
- What happens if `--no-session` is passed and a crash recovery is also pending?
  `--no-session` suppresses the session restore prompt only. The crash-recovery prompt
  is shown normally, unaffected by `--no-session`.
- What happens if a `.session.toml.tmp` file is found on startup (orphaned from a
  previous crashed write)?
  The orphaned `.tmp` file is silently deleted on startup. No warning is shown to the
  user; the deletion is logged at `debug` level.
- What happens if `$XDG_STATE_HOME` is set to a path that does not exist as a directory?
  The editor attempts `create_dir_all` on the path. If creation succeeds, the session
  file is written normally. If creation fails, the behavior is identical to "not writable":
  the session save silently fails with a `warn`-level log entry and no error is shown.
- What about keyboard accessibility of the restore prompt beyond Y/N/Enter/Escape?
  The restore prompt is a simple Y/N dialog. Tab traversal and screen-reader support are
  explicitly out of scope for v1.x. Y, y, Enter (confirm) and N, n, Escape, Ctrl+C
  (decline) are the only required inputs.
- What log levels are used for session operations?
  Successful session save and load: `info`. Failures (unwritable directory, corrupt file,
  missing path, unknown schema version): `warn`. Path resolution details and tmp-file
  cleanup: `debug` (visible only with `--debug`).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: On a user-initiated clean exit, the editor MUST write a session file to
  `$XDG_STATE_HOME/edit/session.toml` capturing: the ordered list of open buffer file paths
  (in visual tab order, left to right as displayed in the tab bar),
  each buffer's last known cursor position (1-based line and column), the split layout type
  (none, horizontal, vertical), which pane was active, and the index of the active buffer.
- **FR-002**: The session file MUST be written only on clean exits. A clean exit is any
  of: the File > Quit menu action, the Ctrl+Q keybinding, or any quit keybinding
  explicitly registered in the user's keymap. All other terminations are non-clean exits
  and MUST NOT trigger a session write: SIGTERM (system shutdown or `kill` command),
  SIGKILL, SIGSEGV, process panic or abort, and OOM kill are all non-clean exits.
  SIGKILL cannot be intercepted and no session write is possible; this is expected.
  SIGTERM MUST be treated as a non-clean exit (no session write).
- **FR-003**: On startup with no explicit file arguments and no `--no-session` flag, the
  editor MUST check for `$XDG_STATE_HOME/edit/session.toml`. If the file exists and is
  valid, the editor MUST prompt the user: "Restore previous session? [Y/n]". The prompt
  accepts Y, y, or Enter to confirm and N, n, Escape, or Ctrl+C to decline;
  matching is case-insensitive.
- **FR-004**: If the user confirms the restore prompt (Y or Enter), the editor MUST reopen
  all recorded buffer paths in the recorded order, seek each buffer's cursor to its saved
  position, and restore the split layout and active pane.
- **FR-005**: If a recorded buffer path no longer exists or is unreadable at restore time,
  the editor MUST skip that path and display a status-bar warning naming the missing file.
  Other recorded files that do exist MUST still be restored. Before opening any path from
  the session file, the editor MUST validate it using the existing `security::sanitize`
  path helper; paths containing `../` traversal sequences or symlinks resolving outside
  the working tree MUST be treated as missing (skip with status-bar warning). Dangling
  symlinks and symlink loops MUST also be treated as unreadable and skipped with a
  status-bar warning. If only one file of a two-pane split is successfully restored, the
  split layout MUST collapse to a single pane displaying the surviving file.
- **FR-006**: If all recorded buffer paths are missing or unreadable, the editor MUST open
  a blank buffer and display a status-bar warning.
- **FR-007**: If the user declines the restore prompt (N, Escape, or Ctrl+C), the editor
  MUST open a blank buffer and leave the session file unchanged on disk.
- **FR-008**: If the `--no-session` flag is passed, the editor MUST skip the restore
  prompt and open normally (with any explicitly supplied files or a blank buffer). The
  `--no-session` flag suppresses the session restore prompt only; it does not affect the
  existing crash-recovery prompt, which operates independently.
- **FR-009**: The session file format MUST be valid, human-readable TOML. All of the
  following fields are required: `version` (integer schema version), `active_buffer`
  (0-based index into the buffers array), `split_layout` (string: "none", "horizontal",
  or "vertical"), `active_pane` (integer 0 or 1; MUST be 0 when `split_layout` is
  "none"), and a `[[buffers]]` array where each entry has: `path` (string, stored
  as-opened — absolute if the file was opened with an absolute path, relative to the
  working directory otherwise), `cursor_line` (integer, 1-based), and `cursor_col`
  (integer, 1-based). TOML parsers MUST use lenient mode: extra fields not listed above
  are silently ignored and do not trigger the corrupt-file path.
- **FR-010**: If the session file exists but is corrupt or invalid, the editor MUST treat
  it as absent: skip the restore prompt, display a status-bar warning to the user, log a
  `warn`-level message, and open a blank buffer. The corrupt file MUST be overwritten on
  the next clean exit using the same atomic tmp-rename sequence as FR-001. A session file
  is considered corrupt if any of the following conditions are true: (a) the file contains
  malformed TOML syntax that fails to parse; (b) any required field listed in FR-009 is
  absent from the parsed TOML; (c) `version` is present but its value is not a recognized
  schema version (see FR-013); (d) `active_buffer` is ≥ the number of entries in the
  `[[buffers]]` array; (e) any `cursor_line` or `cursor_col` value is less than 1.
  Parsing MUST NOT panic or propagate an unhandled error to the user regardless of file
  content, including adversarially crafted TOML with deeply nested tables or very large
  values.
- **FR-011**: Session save and load MUST use the existing XDG state directory helper
  (`$XDG_STATE_HOME/edit/`), the same directory already used by logs and crash reports.
  When `$XDG_STATE_HOME` is not set, the fallback path MUST be
  `$HOME/.local/state/edit/session.toml`. When the state directory does not exist, the
  editor MUST attempt to create it via `create_dir_all`; if creation fails the session
  save silently fails with a `warn`-level log entry and no error is shown to the user.
- **FR-012**: The `--no-session` flag MUST be documented in `--help` output and in the
  man page.
- **FR-013**: If the session file's `version` field value is not 1 (i.e., a future or
  unknown schema version), the editor MUST treat the file as absent per FR-010: skip the
  restore prompt, log a `warn`-level message identifying the unrecognized version, and
  open a blank buffer. The file MUST be overwritten on the next clean exit.

### Key Entities

- **Session**: A persistent record of the editor's last clean-exit state. Contains a list
  of buffer entries, the active buffer index, and the split layout descriptor.
- **Buffer Entry**: One entry in the session's buffer list. Attributes: file path (absolute
  or as-opened), cursor line (1-based), cursor column (1-based).
- **Split Layout**: Describes the visible pane arrangement: none (single pane), horizontal
  (top/bottom), or vertical (left/right). Includes which pane was active.
- **Status-Bar Warning**: An ephemeral one-line message displayed in the editor's status
  bar. For session restore warnings, the message auto-dismisses after 5 seconds or on the
  next user keystroke. When multiple warnings are queued they are shown sequentially.
  Session warnings have lower display priority than modal error dialogs.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can close and reopen the editor and be back at the exact same file,
  line, and column within 3 seconds of process invocation (wall-clock), without any
  manual navigation.
- **SC-002**: The restore prompt and full session reload (for any number of recorded
  buffers) complete within 2 seconds of the editor reaching the interactive state.
- **SC-003**: When one or more recorded files are missing, 100% of the remaining valid
  files are successfully restored and at least one status-bar warning is shown per missing
  file — the restore never silently drops a restorable file.
- **SC-004**: Launching with `--no-session` shows no restore prompt in 100% of cases,
  regardless of session file state.
- **SC-005**: The session file is absent after a crash exit in 100% of cases — session
  writes occur only on clean exits. "Crash exit" includes SIGSEGV, process panic or
  abort, OOM kill, SIGKILL, SIGTERM, and any involuntary termination not listed in
  FR-002.
- **SC-006**: A corrupt or partial session file never causes an error dialog, crash, or
  blocked startup — the editor always reaches an interactive state.
- **SC-007**: The session write on clean exit completes within 500 ms and does not
  perceptibly delay the editor's visible close. If the atomic write has not completed
  within 500 ms, it is abandoned (the tmp file is cleaned up on next startup per the
  orphaned-tmp edge case) and the editor exits immediately.

## Assumptions

- Scratch buffers (new files not yet saved to disk) are not recorded in the session.
  Only buffers with an on-disk path are serialized. This avoids persisting anonymous
  content that has no stable path to restore.
- The session file is single-instance; the editor does not handle concurrent sessions
  from multiple editor processes sharing the same `$XDG_STATE_HOME`. Last-writer wins.
- The split layout supports at most two panes (as per the existing multi-file US6 baseline).
  Sessions with a single pane or a two-pane split are both supported; three-or-more-pane
  layouts are not in scope (not currently supported by the editor).
- Cursor position is the insertion point at the time of exit, not a visual selection range.
  Restoring a selection range is out of scope for this feature.
- The session TOML schema version starts at 1. Future schema changes will increment this
  version; a reader encountering an unknown version falls back to the "treat as absent"
  behavior (FR-013).
- The restore prompt is shown in the editor's TUI, not as a terminal prompt before the UI
  starts, to remain consistent with the DOS-faithful UI principle.
