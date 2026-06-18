# Feature Specification: Linux EDIT.COM Clone

**Feature Branch**: `001-linux-editcom-clone`

**Created**: 2026-06-18

**Status**: Draft

**Input**: User description: "create a Linux EDIT.COM clone spec and take insights from Thoughts.MD"

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Basic File Editing (Priority: P1)

A command-line user opens a text file by passing its path as an argument. They are
immediately presented with the familiar blue-background DOS-style editor interface, with
the filename shown in the title and a status bar displaying row, column, encoding, and
mode. They navigate with arrow keys, Home, End, PageUp, and PageDown; type to insert text;
use Backspace and Delete to remove characters; and save with Ctrl+S or via the File menu.
They quit cleanly with Ctrl+Q, being prompted to save if unsaved changes exist.

**Why this priority**: This is the minimum viable use case. Without reliable open/edit/save,
nothing else matters.

**Independent Test**: Run `edit filename.txt`, make a change, save, quit, verify file on
disk reflects the change.

**Acceptance Scenarios**:

1. **Given** a file exists at the given path, **When** the user runs `edit filename.txt`,
   **Then** the editor opens in full-screen mode, displays the file contents, and shows
   the filename in the title bar.
2. **Given** a path that does not exist, **When** the user runs `edit newfile.txt`,
   **Then** the editor opens with an empty buffer and creates the file on first save.
3. **Given** unsaved changes, **When** the user presses Ctrl+Q, **Then** a dialog asks
   whether to save, discard, or cancel; the file is written only if the user confirms.
4. **Given** a read-only file path, **When** the user opens it with `--readonly`,
   **Then** the editor displays the file and indicates read-only mode; save operations
   are disabled.
5. **Given** a large file (≥ 50 MB), **When** the user opens it, **Then** the editor
   becomes interactive within 3 seconds and remains responsive during scrolling and editing.

---

### User Story 2 — UTF-8 and Unicode Display (Priority: P1)

A user opens a file containing multilingual text — Japanese, Arabic, emoji, and combining
characters — and the editor renders each character correctly, aligns double-width East
Asian characters to two columns, and allows the cursor to navigate character by character
without visual misalignment. The user types Unicode characters directly from their keyboard
and the input appears correctly in the buffer.

**Why this priority**: Unicode correctness is the primary motivation for this project over
existing EDIT.COM clones.

**Independent Test**: Open a file with Japanese, Arabic, and emoji content. Verify visual
alignment, cursor movement, and that the saved file is byte-for-byte identical to the
original.

**Acceptance Scenarios**:

1. **Given** a UTF-8 file with Japanese characters, **When** the user opens it, **Then**
   each kanji occupies exactly two visual columns and the cursor advances two columns per
   character.
2. **Given** a UTF-8 file with combining characters (e.g., é as e + combining acute),
   **When** displayed, **Then** the combined glyph renders as one visual column.
3. **Given** a file with emoji (e.g., 😀), **When** displayed, **Then** the emoji occupies
   two visual columns and the cursor skips over it as a unit.
4. **Given** a terminal configured with a non-UTF-8 locale, **When** the editor starts,
   **Then** a visible warning is shown and the user is advised to set a UTF-8 locale;
   the editor still starts with UTF-8 forced internally.
5. **Given** a legacy CP437-encoded file opened with `--encoding=cp437`, **When** displayed,
   **Then** box-drawing characters and accented letters render as their Unicode equivalents.

---

### User Story 3 — DOS-Style Menu and Keyboard Navigation (Priority: P1)

A user who remembers MS-DOS EDIT.COM opens the editor and immediately recognises the blue
background, the top menu bar (File / Edit / Search / View / Options / Help), and the
bottom status bar. They press Alt+F to open the File menu, navigate with arrow keys,
and invoke commands from the menus. Function keys (F1 Help, F3 Find Next, F5 Save,
F10 Menu) behave exactly as in the original EDIT.COM.

**Why this priority**: Familiarity is the core UX promise of this project. Users who
pick this editor over nano or vi do so precisely because of the EDIT.COM look-and-feel.

**Independent Test**: Launch the editor and navigate every top-level menu item using only
Alt+key and arrow keys; verify all keyboard shortcuts from the EDIT.COM reference mapping.

**Acceptance Scenarios**:

1. **Given** the editor is open, **When** the user presses F10 or Alt+F, **Then** the
   File menu drops down and the user can navigate it with arrow keys.
2. **Given** the File menu is open, **When** the user presses Escape, **Then** the menu
   closes and focus returns to the text area.
3. **Given** a terminal without color support, **When** the editor starts, **Then** it
   falls back to reverse-video for the menu bar and remains fully functional.
4. **Given** a mouse-capable terminal, **When** the user clicks a menu item, **Then**
   the menu opens as if the corresponding Alt key was pressed.
5. **Given** the status bar at the bottom, **When** the cursor moves, **Then** the row
   and column values update in real time.

---

### User Story 4 — Search and Replace (Priority: P1)

A user needs to find all occurrences of a word in a file and optionally replace them. They
open the Search menu or press Ctrl+F, type a search term (plain text or regular expression),
and the editor highlights the first match and scrolls to it. They advance through matches
with F3. They open Find & Replace to substitute terms one at a time or all at once, with
a preview of what will change.

**Why this priority**: Search and replace is a fundamental editing feature required for
any practical use of the editor.

**Independent Test**: Open a file, search for a known term, verify match count and
highlighting, perform replace-all, save, and verify the output file.

**Acceptance Scenarios**:

1. **Given** a search term, **When** the user initiates search, **Then** the first match
   is highlighted and the editor scrolls to its position.
2. **Given** regex mode is enabled, **When** the user enters a valid regex, **Then**
   matches are found according to regex semantics.
3. **Given** an active search, **When** the user presses F3, **Then** the editor advances
   to the next match, wrapping at end of file.
4. **Given** Find & Replace is open, **When** the user confirms Replace All, **Then**
   all occurrences are substituted and a count of replacements is shown.
5. **Given** no matches exist for the search term, **When** the user searches, **Then**
   a "Not found" message appears in the status bar.

---

### User Story 5 — Auto-Save and Crash Recovery (Priority: P1)

A user is editing a long file and their terminal session is unexpectedly terminated — by
a network drop, a crash, or a forced logout. When they restart the editor for the same
file, they are offered the option to recover the last auto-saved version, which contains
all edits from up to 30 seconds before the interruption.

**Why this priority**: Data loss from unexpected termination is a critical reliability gap.
Auto-save prevents it without requiring the user to think about it.

**Independent Test**: Open a file, make edits, kill the editor process without saving,
re-open the file, verify that the recovery offer appears and restores the most recent edits.

**Acceptance Scenarios**:

1. **Given** the editor is running with unsaved changes, **When** 30 seconds elapse,
   **Then** a recovery file is silently written without interrupting the user.
2. **Given** the editor was previously killed without saving, **When** the user opens
   the same file again, **Then** a dialog offers to recover the auto-saved version or
   start fresh.
3. **Given** recovery is accepted, **When** the file loads, **Then** the buffer contains
   the content from the last auto-save point.
4. **Given** recovery is declined, **When** the file loads, **Then** the buffer shows
   the last on-disk version and the recovery file is deleted.
5. **Given** a clean exit (user saved and quit), **When** the file is next opened,
   **Then** no recovery dialog appears.

---

### User Story 6 — Multi-File Editing (Priority: P2)

A user opens multiple files by passing several paths as arguments, or opens additional
files from within the editor via File > Open. They switch between open files using the
View menu or a keyboard shortcut, viewing them in a tabbed interface or a two-panel
split view.

**Why this priority**: Multi-file support significantly improves productivity; however,
single-file editing is already a complete MVP on its own.

**Independent Test**: Open two files, switch between them, edit each independently,
save both, verify both files reflect the correct edits.

**Acceptance Scenarios**:

1. **Given** multiple files passed on the command line, **When** the editor starts,
   **Then** all files are available as separate buffers accessible via the View menu.
2. **Given** two files are open, **When** the user selects split view, **Then** both
   files are visible simultaneously in two panels.
3. **Given** an edited buffer in multi-file mode, **When** the user saves, **Then**
   only the active buffer is written to disk.
4. **Given** two buffers with the same filename but different paths, **When** displayed,
   **Then** the title bar shows enough path context to distinguish them.

---

### User Story 7 — Syntax Highlighting (Priority: P2)

A user opens a source code file (C, Python, Shell script, YAML, or Markdown). The editor
automatically detects the file type from the extension and applies colour-coded syntax
highlighting: keywords in one colour, strings in another, comments in a muted tone.
The user can turn highlighting off from the Options menu or via `--no-highlight`.

**Why this priority**: Highlighting improves readability for developers using the editor
for code; it is optional and must not interfere with editing or encoding correctness.

**Independent Test**: Open a `.py` file, verify keywords and strings are visually
distinguished from plain text; disable highlighting and verify the file appears in plain
colours.

**Acceptance Scenarios**:

1. **Given** a `.c` file, **When** opened, **Then** C keywords, string literals, and
   comments are each rendered in a distinct colour.
2. **Given** highlighting is active on a Python file, **When** the user types a new
   string literal, **Then** it is coloured immediately without noticeable lag.
3. **Given** `--no-highlight` flag or Options > Syntax Off, **When** set, **Then** the
   buffer renders in plain white-on-blue without colour distinctions.
4. **Given** a file with no recognised extension, **When** opened, **Then** highlighting
   is silently disabled and no error is shown.

---

### User Story 8 — Configurable Keybindings and Themes (Priority: P2)

A user who prefers Ctrl+S for save and Ctrl+Q for quit can update a configuration file to
remap any keybinding. A user who prefers a light-on-dark or high-contrast theme can select
one from the Options menu or config file. Changes take effect on the next session start.

**Why this priority**: Customisation increases adoption among users whose muscle memory
differs from the EDIT.COM defaults.

**Independent Test**: Edit the config file to remap a key, restart the editor, verify
the new binding works and the original no longer triggers that action.

**Acceptance Scenarios**:

1. **Given** a keybinding remapped in the config file, **When** the editor starts,
   **Then** the new binding performs the assigned action.
2. **Given** a conflicting binding (two actions assigned to the same key), **When**
   the editor starts, **Then** a warning is logged and the default binding is used.
3. **Given** "high-contrast" theme selected, **When** the editor opens, **Then** the
   colour scheme changes to meet high-contrast accessibility standards.
4. **Given** an invalid config value, **When** the editor starts, **Then** the error
   is logged, the default value is used, and the editor starts normally.

---

### Edge Cases

Each edge case below has a defined resolution (no open questions remain):

- **External modification** (file changes on disk while being edited): Detection of on-disk
  changes during an edit session is **deferred to post-v1.x** (tracked in ROADMAP.md). In
  v1.x, a save overwrites the on-disk version; the recovery/lock mechanism (FR-014) is the
  safety net against concurrent sessions.
- **Disk full / save failure**: Handled per **FR-027** — a Retry/Cancel dialog appears; the
  buffer's unsaved content is preserved (never discarded) so the user can free space and retry.
- **Mixed line endings (CRLF + LF)**: Handled per **FR-007a** below and the `LineEnding` enum
  (data-model.md). The dominant ending detected in the first 512 bytes is preserved on save;
  the rope stores `\n`-only internally.
- **Terminal resized mid-session**: Handled per **FR-026** — the layout reflows; below the
  80×24 minimum a "Terminal too small" notice is shown; no crash or buffer corruption.
- **Auto-save location not writable**: Handled per the recovery contract — the editor falls
  back from `$XDG_RUNTIME_DIR/edit/` to `$TMPDIR/edit-recovery/`; if both fail, auto-save is
  disabled for the session and a one-time warning is logged (editor remains usable).
- **Null bytes / binary content**: Handled per **FR-029** — the file is refused with a
  "Binary file — cannot edit" notice.
- **Very long lines (> terminal width)**: Handled by **horizontal scrolling**, not soft-wrap
  (data-model.md "Long-line handling"), matching MS-DOS EDIT.COM. The cursor stays visible
  via `scroll_offset.1`.

- **FR-007a** (line-ending preservation): The system MUST detect the dominant line ending
  (LF or CRLF) when opening a file and MUST preserve it on save unless the user explicitly
  changes it via Options > Line Endings. Carriage returns are stripped from the internal
  buffer; CRLF is reconstructed on write when the file's ending is CRLF.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST open an existing file when its path is provided as a command-line
  argument and display its contents in the editor buffer.
- **FR-002**: The system MUST create a new empty buffer when a non-existent file path is
  provided, and create the file on first save.
- **FR-003**: The system MUST save the active buffer to disk when the user triggers the
  save command (Ctrl+S or File > Save).
- **FR-004**: The system MUST prompt the user to save, discard, or cancel when quitting
  with unsaved changes.
- **FR-005**: The system MUST render all UTF-8 text correctly, including multibyte
  characters, combining characters, double-width East Asian characters, and emoji. Display
  column widths MUST follow Unicode UAX #11 (East Asian Width): a character classified
  Wide or Fullwidth (e.g. CJK ideographs) MUST occupy exactly 2 terminal columns; combining
  marks (zero-width) MUST occupy 0 columns and compose onto the preceding grapheme; standard
  emoji presentation MUST occupy 2 columns. Cursor navigation MUST advance by whole grapheme
  clusters (per UAX #29), never landing inside a multi-codepoint cluster.
- **FR-006**: The system MUST enforce UTF-8 encoding internally regardless of the system
  locale, and warn the user if the terminal locale is not UTF-8.
- **FR-007**: The system MUST transcode legacy encodings (CP437, CP850, ISO-8859-1,
  Windows-1252) to UTF-8 when the user opens a file with `--encoding=<enc>` or via the
  Options menu. Encoding resolution follows a fixed priority: **(1)** explicit `--encoding`
  flag (when set, it wins even over a conflicting BOM — e.g. `--encoding=cp437` on a
  BOM-bearing file treats the BOM bytes as ordinary CP437 content); **(2)** BOM detection
  (UTF-8/UTF-16); **(3)** heuristic via `chardetng`; **(4)** UTF-8 default when heuristic
  confidence is below 0.6. When a byte sequence is invalid for the resolved encoding, the
  editor MUST NOT crash: it surfaces a decode-error dialog and, if the user chooses "Open
  anyway", substitutes the offending bytes with U+FFFD (replacement character).
- **FR-008**: The system MUST present a DOS-style blue-background interface with a top
  menu bar (File / Edit / Search / View / Options / Help) and a bottom status bar. On
  terminals without color support, ALL chrome elements — menu bar, status bar, dialog boxes,
  and selected-item highlights — MUST degrade to reverse-video (not only the menu bar),
  remaining fully legible and functional.
- **FR-009**: The system MUST support keyboard navigation of all menus via Alt+letter
  and arrow keys, matching the EDIT.COM key mapping.
- **FR-010**: The system MUST support cursor movement via arrow keys, Home, End, PageUp,
  PageDown, and Ctrl+Home/End.
- **FR-011**: The system MUST support cut (Ctrl+X), copy (Ctrl+C), paste (Ctrl+V),
  undo (Ctrl+Z), and redo (Ctrl+Y) operations.
- **FR-012**: The system MUST support incremental search with optional case-sensitivity
  toggle and regex mode, accessible via Ctrl+F or Search menu.
- **FR-013**: The system MUST support Find and Replace with replace-one and replace-all
  operations.
- **FR-014**: The system MUST auto-save a recovery file every 30 seconds while unsaved
  changes exist, and offer recovery on next open if the previous session ended abnormally.
- **FR-015**: The system MUST support opening and switching between multiple files in the
  same session via tabbed or split-panel views.
- **FR-016**: The system MUST provide syntax highlighting for at least five file types:
  C, Python, Shell script, YAML, and Markdown; highlighting MUST be disableable. Highlighting
  is computed per-visible-line (not whole-file) and cached per line, so the SC-005 keystroke
  latency budget (≤ 50 ms) applies unchanged regardless of total file size; on a file ≥ 10 MB
  with highlighting active, per-keystroke highlight recomputation MUST remain within that budget.
- **FR-017**: The system MUST support mouse interaction (click to position cursor, click
  to select menu items) in terminals that report mouse events.
- **FR-018**: The system MUST accept CLI flags: `--encoding`, `--locale`, `--readonly`,
  `--no-autosave`, `--line-numbers`, `--theme`, `--no-highlight`, `--debug`, `--help`,
  `--version`.
- **FR-019**: The system MUST display a built-in help screen via F1 or Help menu,
  and must install a man page accessible via `man edit`.
- **FR-020**: The system MUST read a configuration file at the standard user config path
  and apply settings for default encoding, theme, keybindings, and autosave interval.
- **FR-021**: The system MUST respect file ownership and permissions when saving; it MUST
  NOT escalate privileges. Specifically: the process effective UID and GID MUST be identical
  before and after any `Buffer::save`; the editor MUST NOT invoke `setuid`/`setgid`/`sudo`
  or any privilege-elevation mechanism. On a permission-denied save, the editor surfaces the
  OS error in a dialog (see FR-027) and offers Retry/Cancel only — never an elevation path.
- **FR-022**: The system MUST sanitise all terminal control sequences encountered in file
  content or clipboard data before rendering, to prevent terminal escape injection. At
  minimum, the following byte sequences MUST be neutralised (rendered as visible literal
  text, not interpreted): CSI (`ESC [`), OSC (`ESC ]` … `BEL`/`ST`), DCS (`ESC P` … `ST`),
  APC (`ESC _` … `ST`), PM (`ESC ^` … `ST`), and the lone control characters `0x00`–`0x08`,
  `0x0B`–`0x1F`, `0x7F` (except `\t` `0x09` and `\n` `0x0A`, which are handled normally).
- **FR-023**: The system MUST prevent directory traversal in file-dialog and relative-path
  inputs. The boundary is the editor's **current working directory at launch**: a path that,
  after canonicalisation (resolving `..` and symlinks), escapes above the launch CWD MUST be
  rejected with a clear error UNLESS it was supplied as an explicit absolute path argument on
  the command line (absolute CLI args are trusted; in-editor Open-dialog inputs are not).
- **FR-024**: The system MUST write structured logs to the standard user state directory;
  log verbosity MUST be configurable; `--debug` MUST enable verbose diagnostic output.
- **FR-025**: The system MUST write a crash report to disk when it terminates abnormally.
  The crash report MUST contain only diagnostic metadata — stack/backtrace, resolved locale,
  terminal type, active theme, config path, and the absolute file path of open buffers. It
  MUST NOT contain buffer text content, clipboard data, or search/replace strings.
- **FR-026**: The system MUST require a minimum terminal size of 80 columns × 24 rows. On a
  terminal smaller than the minimum (at launch or after resize), the editor MUST display a
  "Terminal too small (min 80×24)" notice and resume normal rendering once the terminal is
  enlarged, without crashing or corrupting the buffer.
- **FR-027**: The system MUST surface file-write failures (permission denied, disk full /
  ENOSPC, read-only filesystem) in a visible modal dialog offering Retry/Cancel, rather than
  failing silently or discarding the user's unsaved changes.
- **FR-028**: The system MUST support "Save As" to write the active buffer to a new path.
  If the target path already exists, the editor MUST prompt for overwrite confirmation
  (Overwrite / Cancel) before writing. On successful Save As, the buffer's path is updated to
  the new path and the modified flag is cleared.
- **FR-029**: The system MUST refuse to open files containing binary content (detected as a
  null byte `0x00` within the first 512 bytes), showing a "Binary file — cannot edit" notice
  and leaving any currently open buffer unchanged.

### Key Entities

- **Buffer**: An in-memory representation of an open file, tracking content, cursor
  position, undo history, encoding, modification state, and auto-save metadata.
- **Recovery File**: A periodically written snapshot of an unsaved buffer, associated
  with a specific file path, used to restore work after abnormal termination.
- **Configuration**: User-defined settings (keybindings, theme, default encoding,
  autosave interval) persisted in the standard config directory.
- **Keybinding Map**: A mapping from key sequences to editor actions, overridable by the
  user's configuration file; falls back to EDIT.COM defaults.
- **Encoding Profile**: A descriptor for a character encoding (e.g., UTF-8, CP437)
  including the rules for detecting, reading, and writing files in that encoding.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user with no prior knowledge of this editor can open a file, make an
  edit, save, and quit within 60 seconds on their first attempt, guided only by the
  visible menus and F1 help.
- **SC-002**: All multilingual test files (containing Japanese, Arabic, Hebrew, emoji,
  and combining characters) display without visual misalignment in at least three
  distinct terminal emulators.
- **SC-003**: The editor starts and becomes interactive in under 2 seconds on a standard
  modern workstation for files up to 1 MB. **Measured**: wall-clock from process `execve`
  to first completed `terminal.draw()`, via `hyperfine` (median of ≥ 10 runs) in `benches/startup.rs`.
- **SC-004**: A 100 MB UTF-8 file opens and is ready to edit within 3 seconds, with
  no perceptible lag during scrolling or cursor movement. **Measured**: against a fixture of
  predominantly ASCII UTF-8 text (`benches/large_file.rs`); a second informational run uses a
  mixed-Unicode fixture to characterise the wide-character rope-loading cost.
- **SC-005**: Keystroke-to-display latency is under 50 milliseconds for typing and
  cursor movement in normal editing conditions. **Measured**: median elapsed time from
  `crossterm::event::read()` returning a key event to the completion of the resulting
  `terminal.draw()`, captured in `benches/keystroke.rs`.
- **SC-006**: An auto-save recovery round-trip (simulate crash, reopen, recover)
  restores all edits from the last 30 seconds, verified in an automated test
  (`tests/integration/recovery.rs`, T113).
- **SC-007**: A CP437-encoded file round-trips correctly: opened with `--encoding=cp437`,
  all characters display as their Unicode equivalents, and saving back as CP437 produces
  a byte-for-byte identical output file. **Fixture**: `tests/fixtures/cp437_box.bin` —
  the box-drawing frame `C9 CD CD BB / BA 20 20 BA / C8 CD CD BC` (T036); the round-trip
  assertion lives in `tests/integration/encoding_roundtrip.rs` (T109).
- **SC-008**: The editor passes a 72-hour continuous-editing stress test with no crashes,
  and resident memory remaining below 50 MB for a 1 MB file. **Pass criteria**: over the run,
  a loop of insert/delete/search/undo operations executes continuously; peak RSS growth above
  the post-warmup baseline MUST NOT exceed 5 MB (the leak threshold); zero panics. CI runs an
  abbreviated 300-second variant (`EDIT_STRESS_DURATION_SECS=300`); the full 72 h run is manual.
- **SC-009**: The binary is packaged and installable on Ubuntu, Fedora, and Arch Linux
  using their native package managers.
- **SC-010**: All high-priority functional requirements (FR-001 through FR-014) have
  automated test coverage and pass on Linux x86_64, ARM64, macOS, and FreeBSD.

## Assumptions

- Users have a terminal emulator capable of rendering UTF-8; the editor warns but does
  not block on non-UTF-8 terminals.
- The initial release targets Linux x86_64 and ARM64; FreeBSD and macOS support follows
  in the same v1.x release cycle.
- Mouse support is opt-in and gracefully absent on terminals that do not report mouse
  events; all features remain accessible via keyboard alone.
- The configuration file format is YAML or INI; the exact format is decided during
  planning and documented in the developer guide.
- Syntax highlighting covers exactly five languages at launch (C, Python, Shell, YAML,
  Markdown); additional languages are deferred to post-v1.x without a separate spec.
- Plugin/extension API is explicitly out of scope for v1.x and is listed in ROADMAP.md
  as a deferred follow-up item.
- The project has no telemetry or network access by default; any future opt-in telemetry
  requires a governance amendment.
- The "Save As with encoding selection" dialog (FR-007) uses a simple menu prompt; a
  full file-browser dialog is deferred to post-v1.x.
- Line endings: CRLF files are read with CR stripped; saving preserves the dominant
  line ending detected on open (LF or CRLF), unless the user changes it via Options.
- **System clipboard**: cut/copy/paste use the OS clipboard. On a headless or SSH session
  with no display server (no X11/Wayland), clipboard access may fail; in that case the editor
  degrades gracefully — the operation is a no-op and a one-line warning appears in the status
  bar. Clipboard failure MUST NOT panic or abort the editor.
- **WSL clipboard**: under Windows Subsystem for Linux, clipboard interop is best-effort via
  the platform clipboard backend. If no backend is reachable, the same graceful no-op fallback
  applies. A WSL-specific clipboard bridge (e.g. `clip.exe`) is NOT required for v1.x.
- **Minimum terminal size**: the editor assumes at least 80×24 characters (FR-026). Smaller
  terminals show a "Terminal too small" notice until enlarged.
- **Encoding heuristic confidence**: when no `--encoding` flag and no BOM are present, the
  `chardetng` detector is used; if its confidence is below 0.6 the editor falls back to UTF-8.
  This threshold is a tuning constant, documented here so it is not treated as arbitrary.
- **Grapheme segmentation**: cursor/column logic relies on `unicode-segmentation` (UAX #29)
  for grapheme boundaries and `unicode-width` (UAX #11) for column widths, operating over
  `ropey` char indices. These three crates are assumed mutually compatible at the versions
  pinned in `Cargo.toml`; an integration test (T109/T108) guards the assumption.
