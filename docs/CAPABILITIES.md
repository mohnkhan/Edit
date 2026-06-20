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

| Key | Action |
|---|---|
| `Shift+Arrow` | Extend selection |
| `Ctrl+A` | Select all |

### Search and Replace

| Key | Action |
|---|---|
| `Ctrl+F` | Open Find dialog |
| `Ctrl+H` | Open Find and Replace dialog |
| `F3` | Find next |
| `Shift+F3` | Find previous |

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
| `Alt+F` | Open File menu (dropdown) |
| `Alt+E` | Open Edit menu (dropdown) |
| `Alt+S` | Open Search menu (dropdown) |
| `Alt+V` | Open View menu (dropdown) |
| `Alt+O` | Open Options menu (dropdown) |
| `Alt+H` | Open Help menu (dropdown) |
| `←` / `→` | Move between top-level menus (wraps; opens the adjacent dropdown) |
| `↑` / `↓` | Move between items within the open dropdown (wraps) |
| `Enter` | Activate the highlighted menu item |
| `Esc` | Close menu / cancel dialog |
| Mouse (left-click) | Click a top-level menu title to open it; click a dropdown item to activate it; click outside to close (Feature 011) |

Plugin-contributed top-level menus appear in the menu bar **between Options and Help** and are
navigable/activatable with the same keys *and the mouse* (Features 009 / 011).

## Menu Structure

### File
- New (`Ctrl+N`)
- Open… (`Ctrl+O`) — opens the **file browser** (navigate folders, pick a file)
- Save (`Ctrl+S`) — on an unnamed buffer, opens the Save file browser
- Save As… — opens the Save **file browser** (navigate to a folder, type a name)
- Save As Encoding… (`F12`) — choose output encoding for this file
- Close
- ----
- Exit (`Ctrl+Q`)

#### File browser (Open / Save As)

A navigable directory listing replaces the old blind path entry:

| Key / action | Effect |
|---|---|
| `↑` / `↓` / single mouse click | Move the selection (list scrolls to keep it visible) |
| `Enter` / `→` / double mouse click | Enter the highlighted folder, or pick the highlighted file (Open) / confirm (Save) |
| `←` / `Backspace` (empty field) | Go to the parent directory |
| type characters | Edit the filename (Save) or an absolute jump-path (Open) |
| `Esc` / click outside | Cancel |

Folders and files are listed (dot-files shown), sorted parent → folders → files; names render
UTF-8-correct and truncate without corruption. All chosen paths are validated by the path sanitizer.

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
- Help — key-binding cheat sheet overlay (`Esc` closes)
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
