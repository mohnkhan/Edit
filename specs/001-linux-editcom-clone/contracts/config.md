# Contract: Configuration File Schema

**Feature**: Linux EDIT.COM Clone
**Format**: TOML
**Location**: `$XDG_CONFIG_HOME/edit/config.toml` (fallback: `~/.config/edit/config.toml`)
**Version**: 1.0.0

## Full Schema with Defaults

```toml
# edit configuration file
# All fields are optional — omit to use the default.

# Default encoding for new files and files without a detectable encoding.
# Values: "utf-8" | "cp437" | "cp850" | "iso-8859-1" | "windows-1252"
default_encoding = "utf-8"

# Color theme.
# Values: "classic" (DOS blue) | "high-contrast" | "plain" (terminal defaults)
theme = "classic"

# Seconds between auto-save recovery file writes.
# Range: 10–300. Values outside this range are clamped with a warning.
autosave_interval = 30

# Show line numbers in the left gutter.
line_numbers = false

# Enable syntax highlighting.
highlight = true

# Enable mouse event handling.
mouse = true

# Log verbosity level.
# Values: "error" | "warn" | "info" | "debug"
log_level = "warn"

# Keybinding overrides.
# Each key maps a key sequence string to an action name.
# Unknown action names are rejected with an error logged at startup.
[keybindings]
# "Ctrl+S" = "save"       # (already the default — shown for illustration)
# "Ctrl+Q" = "quit"
# "F5"     = "save"

# Theme color overrides (only used when theme = "custom").
# Colors: terminal color names ("black", "red", "green", "yellow", "blue",
#         "magenta", "cyan", "white", "bright_*") or hex RGB ("#RRGGBB").
# [colors]
# background       = "blue"
# foreground       = "white"
# menubar_bg       = "cyan"
# menubar_fg       = "black"
# highlight_keyword = "yellow"
# highlight_string  = "green"
# highlight_comment = "bright_black"
```

## Valid Action Names (for `[keybindings]`)

| Action | Description |
|--------|-------------|
| `save` | Save active buffer |
| `save_as` | Save active buffer to a new path |
| `open` | Open file dialog |
| `close` | Close active buffer |
| `quit` | Quit editor |
| `cut` | Cut selection to clipboard |
| `copy` | Copy selection to clipboard |
| `paste` | Paste from clipboard |
| `undo` | Undo last edit |
| `redo` | Redo last undone edit |
| `select_all` | Select entire buffer |
| `find` | Open find dialog |
| `find_next` | Go to next search match |
| `find_prev` | Go to previous search match |
| `find_replace` | Open find & replace dialog |
| `menu` | Activate menu bar |
| `menu_file` | Open File menu |
| `menu_edit` | Open Edit menu |
| `menu_search` | Open Search menu |
| `menu_view` | Open View menu |
| `menu_options` | Open Options menu |
| `menu_help` | Open Help menu |
| `help` | Show built-in help screen |
| `toggle_line_numbers` | Toggle line number gutter |
| `toggle_highlight` | Toggle syntax highlighting |
| `split_view` | Toggle split view |
| `next_buffer` | Switch to next open buffer |
| `prev_buffer` | Switch to previous open buffer |

## Error Handling

- **Unknown key**: logged as `WARN`; key is ignored.
- **Unknown action name**: logged as `ERROR`; binding is ignored (default kept).
- **Conflicting binding**: logged as `WARN`; user override wins.
- **Type mismatch**: logged as `ERROR`; field reverts to default.
- **Missing file**: silently use all defaults; no error.
- **Unparseable TOML**: logged as `ERROR` with line number; all defaults used.
