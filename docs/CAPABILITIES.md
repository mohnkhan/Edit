# Capabilities

This document lists all user-visible capabilities of the `edit` command: keybindings, menu items,
file formats, and CLI flags.  Update this file whenever any of these change.

## CLI Interface

```
edit [OPTIONS] [FILE...]
```

### Options

| Flag | Description |
|---|---|
| `FILE...` | One or more files to open (multi-file editing, US6) |
| `--encoding <ENC>` | Override file encoding: `utf-8`, `cp437`, `cp850`, `iso-8859-1`, `windows-1252`, `utf-16-le`, `utf-16-be`, `utf-16` |
| `--theme <NAME>` | Override theme: `classic`, `high-contrast`, `plain` |
| `--line-numbers` | Enable line numbers in the gutter |
| `--no-highlight` | Disable syntax highlighting |
| `--no-autosave` | Disable auto-save and crash recovery |
| `--no-session` | Skip the session restore prompt on startup; open a blank buffer |
| `--no-watch` | Disable external file modification watching for this session |
| `--no-plugins` | Disable all plugin loading for this session (does not change saved consent) |
| `--readonly` | Open all files in read-only mode |
| `--locale <LOC>` | Override locale detection (e.g. `C.UTF-8`) |
| `--legacy-cp437` | Enable CP437→UTF-8 transcoding on file open |
| `--debug` | Enable debug logging |
| `--version` | Print version and exit |
| `--help` | Print help and exit |

## Keybindings (Default)

### File Operations

| Key | Action |
|---|---|
| `Ctrl+N` | New buffer |
| `Ctrl+S` | Save current file |
| `F12` | Save As Encoding dialog (select output encoding) |
| `Ctrl+O` | Open file browser |
| `Ctrl+N` | New file |
| `Ctrl+Q` | Quit (prompts if unsaved changes) |

### Editing

| Key | Action |
|---|---|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+X` | Cut selection |
| `Ctrl+C` | Copy selection |
| `Ctrl+V` | Paste |
| `Delete` | Delete character at cursor |
| `Backspace` | Delete character before cursor |
| `Tab` | Insert tab / indent selection |
| `Shift+Tab` | Dedent selection |

### Navigation

| Key | Action |
|---|---|
| Arrow keys | Move cursor |
| `Ctrl+Left` / `Ctrl+Right` | Word left / word right |
| `Home` | Beginning of line |
| `End` | End of line |
| `Ctrl+Home` | Beginning of file |
| `Ctrl+End` | End of file |
| `PgUp` / `PgDn` | Page up / page down |

### Selection

Selected text is shown highlighted (reverse video). Copy/Cut act on the selection; typing or pasting
replaces it; `Backspace`/`Delete` delete it; moving without Shift (or a single click) clears it.

| Key / action | Action |
|---|---|
| `Shift+Arrow` | Extend selection by a character / line |
| `Shift+Home` / `Shift+End` | Extend selection to line start / end |
| `Ctrl+A` | Select all |
| Mouse press-drag-release | Select the dragged range |
| Single click | Move cursor, clear selection |

### Search and Replace

| Key | Action |
|---|---|
| `Ctrl+F` | Open the interactive Find dialog (type term in a bordered, labeled input box with a caret; `Enter` to search; matches highlighted, view jumps to the current match, "X of Y" shown) |
| `Ctrl+H` | Open the interactive Replace dialog (find + replace-with fields, each a bordered, labeled input box; caret in the focused field) |
| `F3` | Find next (wraps) |
| `F2` | Find previous (wraps) |
| `Tab` | Replace dialog: switch between the find and replace fields |
| `Enter` | Find: search / advance · Replace: replace current match and advance |
| `Ctrl+A` | Replace dialog: Replace All (Select-All everywhere else) |
| `Alt+C` / `Alt+A` / `Alt+R` / `Alt+W` | Toggle case-sensitive / wrap-around / regex / whole-word (while a Find/Replace dialog is open) |
| `Esc` | Close the Find/Replace dialog and clear match highlights |

### Multi-File / Buffers

| Key | Action |
|---|---|
| `Ctrl+Tab` | Next buffer |
| `Ctrl+Shift+Tab` | Previous buffer |
| `Ctrl+W` | Close current buffer |
| `F6` | Switch to next split pane |

### View

| Key | Action |
|---|---|
| `Alt+Z` | Toggle soft-wrap mode (non-DOS extension) |

### Menu

| Key | Action |
|---|---|
| `F10` | Activate menu bar (highlight first menu, no dropdown) |
| `Alt` (tapped alone) | Activate menu bar like `F10` — terminal-permitting (Feature 013) |
| `Alt+F` | Open File menu (dropdown) |
| `Alt+E` | Open Edit menu (dropdown) |
| `Alt+S` | Open Search menu (dropdown) |
| `Alt+V` | Open View menu (dropdown) |
| `Alt+O` | Open Options menu (dropdown) |
| `Alt+H` | Open Help menu (dropdown) |
| accelerator letter | Each menu title and item shows one **underlined** accelerator letter (Feature 013). While the bar is active, the letter opens that top-level menu; while a dropdown is open, the letter activates that item (case-insensitive). A non-matching letter leaves the menu open. |
| `←` / `→` | Move between top-level menus (wraps; opens the adjacent dropdown) |
| `↑` / `↓` | Move between items within the open dropdown (wraps) |
| `Enter` | Activate the highlighted menu item |
| `Esc` | Close menu / cancel dialog |
| Mouse (left-click) | Click a top-level menu title to open it; click a dropdown item to activate it; click outside to close (Feature 011) |

### Dialogs (confirm / dismiss prompts — Feature 016)

| Key / action | Effect |
|---|---|
| `Tab` / `Shift+Tab` | Move focus between the dialog's boxed buttons (wraps) |
| `Enter` / `Space` | Activate the focused button |
| letter shortcuts (e.g. `S`/`D`/`C`, `Y`/`N`) | Still choose directly |
| Mouse left-click | Click a button to activate it; click outside the dialog to cancel (where a safe cancel exists) |
| `Esc` | Cancel / close the dialog |

Each dialog opens focused on its safe default (Cancel/No/Keep for destructive prompts). Applies to the
unsaved-changes, session-restore, external-change, revert, and plugin-consent dialogs.

### Interactive / list dialogs (Feature 020)

The encoding selector, plugin manager, Find/Replace, and file browser also have boxed buttons, reached
by a **combined focus ring**: the list/field group is the first focus stop and each button is a further
stop.

| Key / action | Effect |
|---|---|
| `Tab` / `Shift+Tab` | Cycle the whole ring — list/field then each button (wraps) |
| `Enter` / `Space` | While a button is focused: activate it |
| Mouse left-click | Click a button to activate it (file browser: buttons take precedence over entry clicks) |
| `Esc` | Close / cancel the dialog from any focus stop |

Buttons per dialog: encoding selector **OK / Cancel**; plugin manager **Close**; Find/Replace **Find /
[Replace / Replace All] / Close** (mode-dependent); file browser **Open** (or **Save**) **/ Cancel**.
Each dialog opens focused on its primary control, so existing keyboard flows (arrows, typing, `Space`
toggle, `Alt+C/A/R/W`, `Ctrl+A`, `F3/F2`) are unchanged.

### Scrollbars & button key hints (Feature 021)

- **Scrollbars** appear automatically when content overflows: the editor shows a vertical bar (and, in
  normal/non-wrap mode, a horizontal bar); the file browser, Help/About, and the plugin/encoding lists
  show a vertical bar. Bars indicate position only — scrolling is still driven by the usual keys/cursor.
- **Help and About** show a clickable **Close (Esc)** button (mouse-dismissable; `Esc`/Enter still work).
- **Every dialog button label advertises its key**, e.g. `Cancel (Esc)`, `OK (Enter)`, `Save (S)`,
  `Replace All (Ctrl+A)`, `Close (Esc)`. The key behaves exactly as before; the label is informational.

### Mouse-wheel scrolling (Feature 023)

The mouse wheel scrolls whatever is under the cursor (or the open dialog/overlay): the editor view
(viewport only, ~3 lines per notch, cursor unchanged), the file-browser listing, the Help/About screens,
and the encoding/plugin lists. Scrolling is bounded and the scrollbars track it. The wheel does not
change click/drag or keyboard behavior; a wheel over the menu/status bar is ignored.

The scrollbars are also **clickable and draggable** (Feature 024): click the track above/below the thumb
to page by one viewport, or drag the thumb to scroll proportionally. In the editor this scrolls the
viewport only (the cursor stays put); a press that starts on a scrollbar never selects text.

Built-in accelerators follow DOS/standard convention — File: **N**ew, **O**pen, **S**ave, Save **A**s,
Save As **E**ncoding, e**X**it; Edit: **U**ndo, **R**edo, **C**ut, C**o**py, **P**aste, **S**elect All;
Search: **F**ind, Find **N**ext, Find **P**rev, Find **R**eplace; View: **S**plit View, **N**ext Buffer,
**P**rev Buffer, **T**oggle Line Nos, Soft **W**rap; Options: Toggle **H**ighlight, **P**lugins;
Help: **H**elp, **A**bout.

Plugin-contributed top-level menus appear in the menu bar **between Options and Help** and are
navigable/activatable with the same keys *and the mouse* (Features 009 / 011); their accelerator
letters are assigned automatically (Feature 013).

## Menu Structure

### File
- New (`Ctrl+N`)
- Open… (`Ctrl+O`) — opens the **file browser** (navigate folders, pick a file, or type a path into the
  bordered "Go to path:" field to jump there)
- Save (`Ctrl+S`) — on an unnamed buffer, opens the Save file browser
- Save As… — opens the Save **file browser** (navigate to a folder, type a name in the bordered "Name:"
  field)
- Save As Encoding… (`F12`) — choose output encoding for this file
- Revert — reload the buffer from its last saved version on disk, discarding changes (confirms when
  there are unsaved changes; no-op for a never-saved buffer). Menu-only, no keybinding.
- Close
- ----
- Exit (`Ctrl+Q`)

The `[Modified]` indicator clears when undo returns the buffer to its saved/opened content and
reappears on redo or further edits (Feature 014).

#### File browser (Open / Save As)

A navigable directory listing replaces the old blind path entry:

| Key / action | Effect |
|---|---|
| `↑` / `↓` / single mouse click | Move the selection (list scrolls to keep it visible) |
| `Enter` / `→` / double mouse click | Enter the highlighted folder, or pick the highlighted file (Open) / confirm (Save) |
| `←` / `Backspace` (empty field) | Go to the parent directory |
| type characters | Filter the listing live (Feature 022) — see below; in Save mode the text is also the filename to write; an absolute path (`/…`) is a jump target |
| `Esc` / click outside | Cancel |

Folders and files are listed (dot-files shown), sorted parent → folders → files; names render
UTF-8-correct and truncate without corruption. All chosen paths are validated by the path sanitizer.

**Filtering (Feature 022)**: typing filters the listing as you go — a pattern with wildcards (`*.log`,
`te?t.txt`) glob-matches names; plain text matches by case-insensitive substring. Directories and `..`
always stay visible so you can navigate; clearing the field restores the full listing; an absolute path
keeps its jump-to-path behavior.

**Detail columns (Feature 022)**: each file row shows a human-readable size and a modified date
(`YYYY-MM-DD HH:MM`, UTC); folders show `<DIR>`. Columns are aligned and the name truncates when narrow.

### Edit
- Undo (`Ctrl+Z`)
- Redo (`Ctrl+Y`)
- ----
- Cut (`Ctrl+X`)
- Copy (`Ctrl+C`)
- Paste (`Ctrl+V`)
- ----
- Select All (`Ctrl+A`)

### Search
- Find… (`Ctrl+F`)
- Find Next (`F3`)
- Find Previous (`Shift+F3`)
- Replace… (`Ctrl+H`)

### Options
- Theme
  - Classic (DOS blue)
  - High-Contrast
  - Plain
- Line Numbers (toggle)
- Syntax Highlighting (toggle)
- Auto-save (toggle)
- Plugins… (open the plugin manager dialog)

### Plugins (Feature 008)
- Plugins are installed in `$XDG_CONFIG_HOME/edit/plugins/<id>/` as a `plugin.toml` manifest
  plus an optional `plugin.rhai` script (Rhai language).
- Plugin types: syntax highlighters, custom keybindings, menu items.
- Menu-item plugins contribute top-level menus (rendered between Options and Help) that are
  navigable and activatable by keyboard; activation runs the plugin's sandboxed `menu_action`
  and shows the result in the status bar (Feature 009).
- First run of a newly-installed plugin shows a one-time consent dialog
  (`Enter` allow / `Esc` deny); decisions are saved in `$XDG_CONFIG_HOME/edit/plugins.toml`.
- Plugin manager: **Options > Plugins** lists installed plugins and toggles them on/off
  (`Up`/`Down` navigate, `Space` toggle, `Esc` close).
- Sandbox: plugins have no filesystem/process/network access except a permission-gated
  `read_file`; each call is bounded to 50 ms; misbehaving plugins are disabled for the session.
- `--no-plugins` disables all plugins for a session without changing saved consent.

### View
- Split View
- Next Buffer
- Prev Buffer
- Toggle Line Nos
- Soft Wrap (ext) (`Alt+Z`) — non-DOS extension; wraps long lines at terminal width
  - Shows `✓ Soft Wrap (ext)` when soft-wrap is **ON** (check-state indicator, non-DOS extension)
  - Shows `  Soft Wrap (ext)` (2-space indent) when **OFF**, maintaining label alignment

### Help
- Help — key-binding cheat sheet: a grouped, two-column **Key | Action** table (scroll with
  ↑/↓/PgUp/PgDn; `Esc` closes)
- About — program name, version, and copyright (`Esc` closes)

## File Formats

### Encodings Supported

| Encoding | Read | Write | Notes |
|---|---|---|---|
| UTF-8 | Yes | Yes | Default; BOM stripped on read |
| UTF-16 LE | Yes | Yes | Auto-detected by BOM (`FF FE`); BOM written on encode |
| UTF-16 BE | Yes | Yes | Auto-detected by BOM (`FE FF`); BOM written on encode |
| CP437 | Yes | Yes | DOS code page 437; `--legacy-cp437` flag |
| CP850 | Yes | Yes | DOS code page 850 |
| ISO-8859-1 | Yes | Yes | Latin-1 |
| Windows-1252 | Yes | Yes | Windows Western European |

### Line Endings

| Convention | Read | Write |
|---|---|---|
| LF (`\n`) | Yes | Yes (default on Linux) |
| CRLF (`\r\n`) | Yes (auto-detected) | Yes (preserved from original) |

### Syntax Highlighting

| Language | Detection |
|---|---|
| C / C++ | `.c`, `.h`, `.cpp`, `.hpp` |
| Python | `.py` |
| Shell | `.sh`, `.bash` |
| YAML | `.yml`, `.yaml` |
| Markdown | `.md` |

## Themes

| Name | Description |
|---|---|
| `classic` | DOS-faithful blue background, white text (default) |
| `high-contrast` | Black background, bright text for accessibility |
| `plain` | Terminal default colors; no custom background |

## Configuration File

Location: `$XDG_CONFIG_HOME/edit/config.toml` (default `~/.config/edit/config.toml`)

See `src/config/schema.rs` for the full schema and all defaults.

## Recovery Files

Location: `$XDG_STATE_HOME/edit/recovery/` (default `~/.local/state/edit/recovery/`)

Format: `EDIT-RECOVERY-V1` (TOML envelope wrapping the buffer content and metadata).
On startup, if a recovery file exists for an opened file, the user is prompted to restore or discard.

## Logs and Crash Reports

| File | Path |
|---|---|
| Log | `$XDG_STATE_HOME/edit/logs/edit-<date>.log` |
| Crash report | `$XDG_STATE_HOME/edit/crash-<timestamp>.log` |
