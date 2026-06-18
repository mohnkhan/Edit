# Contract: CLI Interface

**Feature**: Linux EDIT.COM Clone
**Version**: 1.0.0

## Synopsis

```
edit [OPTIONS] [FILE...]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `FILE...` | Zero or more file paths to open. Each becomes a separate buffer. If omitted, opens a new empty buffer. |

## Options

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--encoding <ENC>` | `-e` | string | auto-detect | Force file encoding for open and save. Values: `utf-8`, `cp437`, `cp850`, `iso-8859-1`, `windows-1252`. |
| `--locale <LOCALE>` | | string | `$LANG` | Override locale for this session (e.g., `C.UTF-8`). |
| `--readonly` | `-r` | flag | false | Open all files in read-only mode. Save is disabled. |
| `--no-autosave` | | flag | false | Disable auto-save and crash recovery for this session. |
| `--line-numbers` | `-n` | flag | false | Show line numbers in gutter. |
| `--theme <NAME>` | | string | `classic` | Color theme. Values: `classic`, `high-contrast`, `plain`. |
| `--no-highlight` | | flag | false | Disable syntax highlighting. |
| `--debug` | `-d` | flag | false | Enable verbose diagnostic logging; log level overrides config. |
| `--help` | `-h` | flag | — | Print usage summary and exit with code 0. |
| `--version` | `-V` | flag | — | Print `edit <version>` and exit with code 0. |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Clean exit (user saved/discarded and quit). |
| `1` | General error (bad flag, file not found, permission denied). |
| `2` | Startup error (terminal not supported, locale setup failed critically). |
| `130` | Killed by SIGINT (Ctrl+C pressed at OS level, not inside editor). |

## Environment Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `EDIT_LOCALE` | string | unset | Overrides the detected locale (same effect as `--locale`); `--locale` flag wins if both set. |
| `EDIT_AUTOSAVE_INTERVAL` | integer (seconds) | config value (30) | Overrides the autosave/recovery write interval. Primarily for fast integration testing (e.g. `EDIT_AUTOSAVE_INTERVAL=5`). Clamped to 1–300. |
| `EDIT_STRESS_DURATION_SECS` | integer (seconds) | `259200` (72 h) | Used only by the stress test harness (`tests/integration/stress.rs`); CI sets it to `300`. Ignored by the normal binary. |
| `EDIT_DEBUG_RENDER` | `0`/`1` | `0` | When `1`, emits an ncurses/render trace for diagnosing rendering glitches (see CLAUDE.md debugging order). |
| `XDG_CONFIG_HOME` | path | `~/.config` | Standard XDG base dir for `edit/config.toml`. |
| `XDG_STATE_HOME` | path | `~/.local/state` | Standard XDG base dir for logs and crash reports. |
| `XDG_RUNTIME_DIR` | path | (falls back to `$TMPDIR/edit-recovery`) | Standard XDG base dir for recovery/lock files. |

CLI flags always take precedence over the corresponding environment variable, which in turn
takes precedence over the config file.

## Behavior Notes

- When multiple `FILE` arguments are given, all files are opened as separate buffers.
  The first file is the active buffer on startup.
- `--encoding` applies to ALL files in the session. To open files with different encodings,
  use separate `edit` invocations (or the in-editor Options > Encoding menu per buffer).
- If `--readonly` is combined with write-protected files, no special warning is given
  (already read-only).
- **`--readonly` + `--no-autosave`**: a valid combination. Read-only already implies no
  buffer modification, so autosave would never fire; `--no-autosave` is redundant but accepted
  without warning. No lock or recovery file is created in read-only mode regardless.
- **`--no-highlight` + `--theme`**: highlighting-off does not disable the theme; the theme's
  non-syntax colors (background, menu bar, status bar) still apply.
- `--debug` writes verbose output to the log file AND to stderr if a TTY is not attached.

## Examples

```bash
# Open a file
edit file.txt

# Open a CP437-encoded DOS text file
edit --encoding=cp437 dosfile.txt

# Open two files in split view
edit left.txt right.txt

# Open a system file read-only
edit --readonly /etc/hosts

# Open with line numbers and high-contrast theme
edit --line-numbers --theme=high-contrast notes.txt

# Debug locale and terminal capability issues
edit --debug file.txt
```
