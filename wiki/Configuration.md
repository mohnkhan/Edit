# Configuration

`edit` reads a TOML configuration file at startup and writes its state files under XDG-compliant
directories. This page documents the config file, themes, and where to find recovery, session, log,
and crash files.

## Config file location

```
$XDG_CONFIG_HOME/edit/config.toml
```

Defaulting to `~/.config/edit/config.toml`. The file is optional — every key has a default, and a
missing or partial file is filled in from the defaults. The authoritative schema lives in
`src/config/schema.rs`.

## config.toml options

```toml
# Encoding used for files that lack a BOM or declaration.
default_encoding = "utf-8"

# Color theme: "classic", "high-contrast", or "plain".
theme = "classic"

# Auto-save interval in seconds; 0 disables auto-save.
autosave_interval = 30

# Show line numbers in the left gutter.
line_numbers = false

# Enable syntax highlighting.
highlight = true

# Enable mouse support (click-to-position, scroll wheel).
mouse = true

# Minimum log severity: off | error | warn | info | debug | trace
log_level = "warn"

# Soft-wrap rendering (non-DOS extension).
soft_wrap = false

# Disable external file-watching (no reload prompts / deletion notices).
no_watch = false

# Custom key bindings: trigger -> command id.
[keybindings]
"ctrl+s" = "save"
"ctrl+q" = "quit"
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `default_encoding` | string | `"utf-8"` | Fallback encoding for files without a BOM/declaration |
| `theme` | string | `"classic"` | Color theme |
| `autosave_interval` | integer (s) | `30` | Auto-save cadence; `0` disables |
| `line_numbers` | bool | `false` | Line-number gutter |
| `highlight` | bool | `true` | Syntax highlighting |
| `mouse` | bool | `true` | Mouse support |
| `log_level` | string | `"warn"` | Log verbosity |
| `soft_wrap` | bool | `false` | Soft-wrap mode (non-DOS extension) |
| `no_watch` | bool | `false` | Persistently disable file-watching |
| `[keybindings]` | table | empty | Override/add bindings (`trigger = "command"`) |

> Some options are runtime-only (set by CLI flags, not persisted): `--no-autosave`, `--readonly`,
> `--locale`, `--no-session`, `--no-plugins`. See the [CLI Reference](CLI-Reference.md).

The `soft_wrap` setting is persisted automatically when you toggle it with `Alt+Z` — `edit` writes
`config.toml` atomically (write-to-temp then rename) so the file is never left half-written.

## Themes

| Name | Description |
|---|---|
| `classic` | DOS-faithful blue background, white text (default) |
| `high-contrast` | Black background, bright text for accessibility |
| `plain` | Terminal default colors; no custom background |

Set the theme in `config.toml` (`theme = "..."`), via the **Options › Theme** menu, or per-launch
with `--theme <name>`.

## Recovery files

```
$XDG_STATE_HOME/edit/recovery/
```

Defaulting to `~/.local/state/edit/recovery/`. Auto-save writes a recovery snapshot in the
**`EDIT-RECOVERY-V1`** format — a TOML envelope wrapping the buffer content and metadata. On startup,
if a recovery file exists for a file you open, `edit` prompts you to **restore** or **discard** it.
Disable auto-save and recovery for a session with `--no-autosave`.

## Session restore

```
$XDG_STATE_HOME/edit/session.toml
```

On a clean exit `edit` writes the open buffer set to `session.toml` (atomically). On the next launch
*without* explicit file arguments, it offers a restore dialog. Decline with `N`/`Esc`, suppress the
prompt with `--no-session`. Missing or corrupt session files are handled gracefully (skipped with a
status-bar warning, then overwritten on the next clean exit).

## Logs and crash reports

| File | Path |
|---|---|
| Log | `$XDG_STATE_HOME/edit/logs/edit-<date>.log` |
| Crash report | `$XDG_STATE_HOME/edit/crash-<timestamp>.log` |

Raise log verbosity with `log_level` in `config.toml` or the `--debug` flag. A panic hook and
SIGSEGV handler write a crash report so a hard failure leaves a diagnostic trail.

## Plugin consent

```
$XDG_CONFIG_HOME/edit/plugins.toml
```

Per-plugin allow/deny decisions made through the one-time consent dialog are stored here. Installed
plugins themselves live under `$XDG_CONFIG_HOME/edit/plugins/<id>/`. See
[Plugin Development](Plugin-Development.md).
