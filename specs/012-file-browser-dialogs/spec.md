# Feature Specification: File Browser Dialogs

**Feature Branch**: `012-file-browser-dialogs`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "Replace the plain path-text Open and Save As dialogs (and any other file-system dialog) with a navigable file browser that displays the current directory's folders and files, lets the user move through the directory tree, and select a file to open or a name/location to save — operable by both keyboard (arrows/Enter/Esc) and mouse. Reuse the existing modal/overlay and mouse hit-test patterns; keep UTF-8 correctness; validate paths via src/security/sanitize.rs."

## Clarifications

### Session 2026-06-20

- Q: Mouse activation model in the file browser? → A: Single-click enters folders and picks files
  (one click; Enter/arrows also work).
- Q: Besides browsing, allow typing a path/name directly? → A: Yes — an editable field accompanies
  the browser (filename in Save; full path that jumps the browser in Open).
- Q: Hidden/dot-files visibility? → A: Show by default; no toggle this iteration.
- Q: Should `Ctrl+S` on an unnamed (new) buffer open the Save browser? → A: Yes.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open a file by browsing (Priority: P1)

A user chooses **File ▸ Open** (or `Ctrl+O`) and is shown the contents of a starting directory:
its sub-folders and files, listed and visually distinguished. They move the highlight with the
arrow keys (or click with the mouse), step into a folder to see its contents, step back up to the
parent, and finally choose a file — which opens it into a new buffer. They never have to know or
type a full path.

**Why this priority**: Opening files is the single most common file-system interaction and is the
primary complaint — the current dialog forces the user to type a full path blind. This story is the
minimum that makes the editor usable like a normal editor.

**Independent Test**: Launch the editor, open the browser, navigate into a known sub-directory,
select a file, and confirm its contents load into the active view — entirely without typing a path.

**Acceptance Scenarios**:

1. **Given** the Open browser is showing a directory, **When** the user highlights a sub-folder and
   activates it, **Then** the listing updates to show that folder's contents and the displayed
   current-directory path updates accordingly.
2. **Given** the Open browser is showing a directory, **When** the user activates the parent entry
   (`..`), **Then** the listing moves up one level (no-op at the filesystem root).
3. **Given** the Open browser is showing a directory, **When** the user activates a regular file,
   **Then** the dialog closes and that file is opened into a buffer.
4. **Given** the Open browser is open, **When** the user presses Escape (or clicks outside it),
   **Then** the dialog closes and no file is opened.

---

### User Story 2 - Save to a chosen folder and name by browsing (Priority: P1)

A user chooses **File ▸ Save As** and is shown the same browsable view. They navigate to the folder
where the file should live, type (or confirm) a filename, and confirm — the active buffer is written
to that folder under that name. Saving an as-yet-unnamed buffer (e.g. via `Ctrl+S`) also brings up
this browser so the user can pick a destination.

**Why this priority**: Saving to a new location is the second core file-system task; without a
browsable destination picker the Save experience has the same blind-path problem as Open.

**Independent Test**: Open the Save browser, navigate to a writable directory, enter a filename,
confirm, and verify a file with that name and the buffer's contents exists in that directory.

**Acceptance Scenarios**:

1. **Given** the Save browser is open on a directory, **When** the user enters a filename and
   confirms, **Then** the buffer is written to `<current directory>/<filename>` and the dialog closes.
2. **Given** the Save browser is open, **When** the user navigates into a different folder before
   confirming, **Then** the file is written into that newly-selected folder.
3. **Given** the user highlights an existing file in the Save browser, **When** they select it as the
   target, **Then** its name populates the filename field (so it can be overwritten deliberately).
4. **Given** an unnamed buffer, **When** the user triggers a save, **Then** the Save browser appears
   so a destination can be chosen.

---

### User Story 3 - Consistent navigation by keyboard and mouse (Priority: P2)

The browser behaves the same whether driven by keyboard or mouse: arrow keys / clicks move the
highlight, Enter / click activates, Escape / outside-click cancels, and a long listing scrolls to
keep the highlighted entry visible.

**Why this priority**: The user explicitly asked for both input methods; consistent behaviour is
what makes the dialog feel like "any other sane editor". It builds on P1 rather than gating it.

**Independent Test**: Perform the same open-a-nested-file task twice — once using only the keyboard,
once using only the mouse — and confirm identical results.

**Acceptance Scenarios**:

1. **Given** a directory with more entries than fit on screen, **When** the user moves the highlight
   past the visible window, **Then** the list scrolls so the highlighted entry stays visible.
2. **Given** the browser is open, **When** the user uses the mouse to enter folders and pick a file,
   **Then** the outcome matches the equivalent keyboard sequence.

---

### Edge Cases

- **Unreadable directory** (permission denied): the browser shows a clear, non-fatal message and
  stays on the previous readable directory rather than crashing or showing a blank list.
- **Filesystem root**: the parent (`..`) entry is a no-op at `/`.
- **Empty directory**: shows just the `..` entry (where applicable) and no files.
- **Very long names / non-ASCII (UTF-8) names**: displayed correctly and truncated to fit without
  breaking layout or splitting multi-byte characters.
- **Save with an empty filename**: confirming does nothing (stays open) — no zero-named file.
- **Save into a non-writable directory**: a clear error is shown; the dialog stays open.
- **Symlinks / `..` path components**: resolved/validated through the existing path sanitizer.
- **Cancelling** at any point leaves the editor state unchanged.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Open and Save file dialogs MUST present the current directory's entries — its
  sub-folders and files — as a visible, navigable list, instead of a blank path field.
- **FR-002**: Folders MUST be visually distinguishable from files, and a parent-directory entry
  (`..`) MUST be available except at the filesystem root.
- **FR-003**: Users MUST be able to move the selection highlight through the listing with the
  keyboard (up/down) and with the mouse.
- **FR-003a**: A single mouse click on an entry MUST act on it directly — entering a highlighted
  folder or picking a file — matching what the keyboard's Enter does (no separate confirm click).
- **FR-006a**: An editable text field MUST accompany the browser: in Save mode it holds the filename;
  in Open mode the user MAY type a full path to jump the browser to that location.
- **FR-002a**: Hidden/dot-files MUST be shown in the listing by default.
- **FR-004**: Users MUST be able to descend into a highlighted folder and ascend to the parent
  folder, with the displayed current-directory path always reflecting the location being viewed.
- **FR-005**: In Open mode, activating a regular file MUST open it into a buffer and close the dialog.
- **FR-006**: In Save mode, the user MUST be able to specify a filename and confirm to write the
  active buffer into the currently-viewed directory under that name.
- **FR-007**: Triggering a save on an unnamed buffer MUST present the Save browser to choose a
  destination.
- **FR-008**: Escape MUST cancel any file dialog with no change to editor state; the dialog MUST also
  be dismissable by the mouse.
- **FR-009**: A listing longer than the visible area MUST scroll to keep the highlighted entry in view.
- **FR-010**: The dialog MUST behave identically whether operated by keyboard or mouse for the same
  logical actions (move, activate, cancel).
- **FR-011**: All displayed names MUST render as valid UTF-8 and MUST be truncated, never corrupted,
  when too long to fit.
- **FR-012**: Every chosen path (open target, save destination) MUST be validated through the
  project's path sanitizer before any file is read or written.
- **FR-013**: Filesystem errors (unreadable directory, unwritable destination) MUST be surfaced to
  the user without crashing and without losing the current dialog.
- **FR-014**: The file dialogs MUST obey the existing modal precedence — they sit above the editor
  but below higher-priority modals — and only one file dialog is open at a time.

### Key Entities *(include if feature involves data)*

- **File Browser session**: the transient state of an open dialog — its mode (Open or Save), the
  directory currently being viewed, the ordered list of entries, the highlighted entry, the scroll
  position, the in-progress filename (Save mode), and any current error notice.
- **Directory entry**: one item in the listing — a display name and whether it is a folder, a regular
  file, or the parent (`..`) shortcut.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can open a file located two directory levels away from the start directory
  without typing any part of a path.
- **SC-002**: A user can save the current buffer to a chosen existing directory under a new name
  without typing any directory path (only the filename).
- **SC-003**: 100% of file-open and file-save flows that previously required typing a full path can
  be completed by browsing instead.
- **SC-004**: The same open/save task can be completed using only the keyboard and, separately, using
  only the mouse, with identical results.
- **SC-005**: Attempting to enter an unreadable directory or save to an unwritable one never crashes
  the editor and always leaves a usable dialog with a visible explanation.
- **SC-006**: Directory and file names containing multi-byte UTF-8 characters display correctly and
  remain selectable.

## Assumptions

- **Starting directory**: the browser opens at the directory of the active buffer's file when it has
  one, otherwise the process's current working directory.
- **Mouse activation model** (clarified): a single click acts directly — clicking a folder enters
  it, clicking a file picks it — mirroring the keyboard's Enter. Arrow keys + Enter remain available.
- **Text entry** (clarified): an editable field accompanies the browser; in Save mode it is the
  filename, and in Open mode the user may type a full path to jump the browser there.
- **Hidden/dot-files** (clarified): shown by default; no visibility toggle this iteration.
- **Save on unnamed buffer** (clarified): `Ctrl+S` on a buffer with no path opens the Save browser.
- **Sort order**: parent (`..`) first, then folders, then files, each alphabetically
  (case-insensitive).
- **Scope**: this replaces the existing Open and Save As path-text dialogs; the Save-As-Encoding
  dialog remains an encoding picker (it may chain into the Save browser when a filename is needed but
  the encoding selection UI itself is unchanged).
- **Platform/runtime**: reuses the existing modal-overlay, mouse hit-testing, theming, and path
  sanitizer already present in the editor; no new external dependencies.
