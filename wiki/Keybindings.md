# Keybindings

The complete default keyboard reference for `edit`. Keybindings can be customized in
[`config.toml`](Configuration.md) and extended by [plugins](Plugin-Development.md) — note that the
safety-critical **Save** and **Quit** actions cannot be overridden by a plugin.

## File operations

| Key | Action |
|---|---|
| `Ctrl+S` | Save current file |
| `Ctrl+Shift+S` | Save As |
| `F12` | Save As Encoding dialog (select output encoding) |
| `Ctrl+O` | Open file dialog |
| `Ctrl+N` | New file |
| `Ctrl+Q` | Quit (prompts if unsaved changes) |

## Editing

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

## Navigation

| Key | Action |
|---|---|
| Arrow keys | Move cursor |
| `Ctrl+Left` / `Ctrl+Right` | Word left / word right |
| `Home` | Beginning of line |
| `End` | End of line |
| `Ctrl+Home` | Beginning of file |
| `Ctrl+End` | End of file |
| `PgUp` / `PgDn` | Page up / page down |

## Selection

| Key | Action |
|---|---|
| `Shift+Arrow` | Extend selection |
| `Ctrl+A` | Select all |

## Search and replace

| Key | Action |
|---|---|
| `Ctrl+F` | Open Find dialog |
| `Ctrl+H` | Open Find and Replace dialog |
| `F3` | Find next |
| `Shift+F3` | Find previous |

## Multi-file / buffers

| Key | Action |
|---|---|
| `Ctrl+Tab` | Next buffer |
| `Ctrl+Shift+Tab` | Previous buffer |
| `Ctrl+W` | Close current buffer |
| `F6` | Switch to next split pane |

## View

| Key | Action |
|---|---|
| `Alt+Z` | Toggle soft-wrap mode (non-DOS extension) |

## Menu activation

The pull-down menus are fully operable from the keyboard (Feature 009).

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

Plugin-contributed top-level menus appear in the menu bar **between Options and Help** and are
navigable and activatable with the same keys.

## Plugin manager dialog (Options › Plugins)

| Key | Action |
|---|---|
| `Up` / `Down` | Navigate the plugin list |
| `Space` | Toggle the selected plugin on/off |
| `Esc` | Close the dialog |

## Consent dialog (first run of a new plugin)

| Key | Action |
|---|---|
| `Enter` | Allow the plugin |
| `Esc` | Deny the plugin |
