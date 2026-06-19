# Feature Specification: External File Modification Detection

**Feature Branch**: `007-external-file-watch`

**Created**: 2026-06-19

**Status**: Draft

**Input**: User description: "Feature 007: External File Modification Detection. When a file currently open in the editor is modified by an external process (e.g. saved by another editor, overwritten by a build tool), detect the change and prompt the user to reload or keep their in-editor version. Use inotify (Linux) via the notify crate; poll as fallback. Deferred from v0.1.0 (Issue #3) because inotify integration adds complexity and Linux-specific code paths requiring careful design to avoid races with the auto-save subsystem."

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Detect External Change and Prompt to Reload (Priority: P1)

A developer has `main.rs` open in the editor and runs `cargo fmt` in a separate terminal. `cargo fmt` overwrites `main.rs`. The editor detects the on-disk change within a few seconds and displays a dialog: **"File changed on disk. Reload? [Y/n]"**. The file indicator in the status bar changes to show the file has been modified externally.

**Why this priority**: This is the primary motivating behavior — without the detection and prompt, users silently lose external changes or unknowingly clobber them on next save. It is independently testable and delivers standalone value.

**Independent Test**: Open any file in the editor; overwrite it from the shell (`echo "new" > file`); observe the reload dialog appears within 5 seconds.

**Acceptance Scenarios**:

1. **Given** a file is open in the editor and has no unsaved changes, **When** an external process overwrites it, **Then** the editor displays a reload prompt within 5 seconds on inotify-capable systems (within 10 seconds on polling fallback).
2. **Given** the reload prompt is shown, **When** the user presses `Y` or `Enter`, **Then** the buffer is replaced with the current on-disk content and the status bar shows no "modified" indicator.
3. **Given** the reload prompt is shown, **When** the user presses `N` or `Esc`, **Then** the buffer retains the previous in-editor content and the status bar shows a "modified" indicator (treated as unsaved changes).

---

### User Story 2 — Reload with Unsaved Changes Warning (Priority: P2)

A developer is editing `config.toml` with several unsaved changes when a build script overwrites the file. The editor detects the change and shows an enhanced prompt: **"File changed on disk. You have unsaved changes. Reload and discard edits? [Y/n]"**. The user can choose to reload (losing unsaved work) or keep their in-editor version.

**Why this priority**: Without this warning, a user who chooses reload would silently discard their own work. The unsaved-changes variant is critical for data safety.

**Independent Test**: Make an edit in any file (don't save), then overwrite it from the shell; observe the enhanced prompt includes the unsaved-changes warning.

**Acceptance Scenarios**:

1. **Given** a file is open with unsaved changes, **When** an external process modifies the file, **Then** the reload prompt explicitly mentions that unsaved changes will be lost.
2. **Given** the unsaved-changes variant of the prompt, **When** the user confirms reload, **Then** the buffer is replaced with disk content and the undo history is cleared.
3. **Given** the unsaved-changes variant of the prompt, **When** the user declines, **Then** the buffer retains the unsaved edits and the file is considered locally modified (no data loss).

---

### User Story 3 — File Deleted While Open (Priority: P3)

A developer has a file open in the editor. A `git checkout` or `make clean` operation deletes the file from disk. The editor detects the deletion and shows a notification: **"File deleted on disk. Buffer kept in memory."** The user can continue editing in memory and save to recreate the file.

**Why this priority**: File deletion is a separate event class from modification. Silently ignoring a deletion (or worse, crashing) would be surprising. This delivers completeness for the detection feature, but the core value (modification detection) is already in US1/US2.

**Independent Test**: Open a file, then delete it with `rm` from the shell; observe the notification appears and the buffer remains editable.

**Acceptance Scenarios**:

1. **Given** a file is open, **When** an external process deletes the backing file, **Then** the editor displays a non-blocking notification (status bar or toast) indicating the file was deleted.
2. **Given** the deletion notification is shown, **When** the user saves (`Ctrl+S`), **Then** the file is recreated on disk with the current buffer content.
3. **Given** a deleted file whose buffer remains open, **When** the user closes the buffer without saving, **Then** the editor behaves identically to closing any other unsaved buffer (prompts if modified).

---

### User Story 4 — Disable Watching via CLI Flag (Priority: P3)

A system administrator deploys the editor in an environment where filesystem events are unreliable (NFS mounts, FUSE filesystems). They launch the editor with `--no-watch` to suppress all file-change detection. No reload prompts appear regardless of external activity.

**Why this priority**: The `--no-watch` escape hatch prevents the watcher from being disruptive in unusual environments. Required for correctness on unreliable filesystems.

**Independent Test**: Launch `edit --no-watch file.txt`; overwrite the file from the shell; verify no dialog appears.

**Acceptance Scenarios**:

1. **Given** the editor is launched with `--no-watch`, **When** any watched file is modified externally, **Then** no reload prompt or notification is shown.
2. **Given** `--no-watch` is active, **When** the user saves (`Ctrl+S`), **Then** the file is saved normally (the flag only suppresses watching, not saving).

---

### Edge Cases

- What happens when the same file is open in two split panes? → A single watch is registered per file path; both panes are reloaded together.
- What happens when multiple rapid external writes occur (e.g., 10 writes in 1 second from a build process)? → Events are coalesced: only one reload prompt appears per "change burst" (debounced over ~1 second).
- What happens when the editor's own auto-save writes to disk? → The write MUST NOT trigger a reload prompt. The detection mechanism MUST distinguish editor-initiated writes from external writes.
- What happens when the file is replaced atomically (e.g., `mv tmpfile original`)? → The rename/create event is treated as a modification event; the reload prompt appears.
- What happens when a binary file overwrites a text file? → The reload path goes through the existing binary-file rejection check; if the new content is not valid UTF-8 or a known encoding, the editor presents an encoding error rather than silently corrupting the buffer.
- What happens on an NFS mount where events are unreliable or delayed? → The polling fallback (configurable interval, default 5 seconds) provides eventual detection; `--no-watch` can suppress watching entirely.
- What happens if the file path is on a filesystem that does not support inotify (e.g., /proc, /sys)? → The watcher silently skips those paths; no prompt, no error.
- What happens when a new unnamed buffer (no backing file) is open? → No watch is registered; unnamed buffers are unaffected.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When any buffer's backing file is modified by an external process, the editor MUST detect the change and display a reload prompt to the user.
- **FR-002**: The reload prompt MUST state the filename and offer two explicit choices: reload from disk, or keep the current in-editor content.
- **FR-003**: When the buffer has unsaved changes at the time an external modification is detected, the reload prompt MUST include an explicit warning that reloading will discard those changes.
- **FR-004**: If the user chooses to reload, the buffer content MUST be replaced with the current on-disk file content; the reload MUST go through the existing encoding detection and validation pipeline (no raw-byte bypass); the undo history MUST be cleared (the replacement is external, not an editor edit).
- **FR-005**: If the user declines to reload, the buffer MUST be marked as having unsaved changes (editor-side modified state), even if no in-editor edits were made, so the user is prompted before closing.
- **FR-006**: When a backing file is deleted by an external process, the editor MUST display a non-blocking notification (distinct from the reload prompt) and retain the buffer content in memory.
- **FR-007**: The detection mechanism MUST NOT produce a reload prompt in response to writes performed by the editor itself (auto-save, `Ctrl+S`, Save As).
- **FR-008**: Multiple rapid external writes to the same file within a 1-second window MUST be coalesced into a single reload prompt (debouncing).
- **FR-009**: The `--no-watch` CLI flag MUST disable all file-watching for the session; no reload prompts or deletion notifications MUST appear when this flag is set.
- **FR-010**: File watching MUST be implemented using the OS-native event mechanism on supported platforms (inotify on Linux, kqueue on BSD/macOS); a polling fallback MUST be available for filesystems where native events are unavailable.
- **FR-011**: Each unique file path is watched at most once, regardless of how many buffers or split panes reference it.
- **FR-012**: File watching MUST have no perceptible impact on editor startup time or keystroke latency (well within the performance baselines in the constitution).

### Key Entities

- **FileWatcher**: The background component that monitors registered file paths for changes. Receives OS-native events or polls on a configurable interval. Sends change notifications to the editor event loop.
- **WatchEvent**: A notification that a monitored file was modified, deleted, renamed, or recreated. Carries the file path and event type.
- **ReloadDialog**: The modal prompt shown to the user when an external modification is detected. Offers "Reload" and "Keep" choices; includes unsaved-changes warning when applicable.
- **DeletionNotice**: A non-blocking status-bar notification shown when a monitored file is deleted.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: When a file open in the editor is overwritten externally, the user sees a reload prompt within **5 seconds** on inotify-capable Linux systems, and within **10 seconds** on polling-fallback systems.
- **SC-002**: Choosing "Reload" produces a buffer that is byte-for-byte identical to the on-disk file content (modulo encoding transcoding), with no silent data loss.
- **SC-003**: The editor's own auto-save write MUST NOT trigger a reload prompt in **100%** of test runs (zero false positives from self-writes).
- **SC-004**: With `--no-watch`, overwriting a file from the shell produces **zero** reload prompts or notifications in the editor session.
- **SC-005**: Editor startup time and keystroke latency remain within the constitution's performance baselines (≤2 s startup, ≤50 ms keystroke) when file watching is active.
- **SC-006**: 10 rapid writes to the same file within 1 second produce exactly **1** reload prompt (debounce verified by automated test).

---

## Assumptions

- The `notify` crate (v6.x, the Rust cross-platform filesystem notification library) will be added to `Cargo.toml` as the cross-platform watcher backend. It wraps inotify on Linux, kqueue on BSD/macOS, and FSEvents on macOS, with an automatic polling fallback.
- The reload prompt is modal (blocks other input until dismissed), consistent with existing modal dialogs (`SavePromptDialog`, session-restore dialog). Non-modal toasts are used only for deletion notices.
- Reloading a buffer clears the undo history, since the new content is a completely external replacement. This trade-off is acceptable and documented; no merge/diff is attempted.
- The auto-save write-suppression mechanism tracks the file path and a 2-second grace window after every editor-initiated write; any watcher event during this window for the same path is suppressed. This is an implementation detail; the requirement is that self-writes produce no prompt (FR-007).
- Polling fallback interval defaults to 5 seconds. The polling interval is not exposed in the UI for feature 007 (deferred to a config-file option in a later feature).
- File watching is **enabled by default**. Users who need to disable it pass `--no-watch` or can add `no_watch = true` to `config.toml`.
- The scope for this feature is modification and deletion detection only. File creation (new file appears in the same directory), rename-to (another file is renamed to match the open path), and directory watching are out of scope for feature 007.
- On Linux kernel < 2.6.36 (very old; not in the supported matrix), inotify is still available but some edge cases may differ. The supported kernel baseline is Linux ≥ 4.4 per the constitution.
- The feature does not add a "merge" or "diff" mode; choosing "Keep" or "Reload" is binary.
