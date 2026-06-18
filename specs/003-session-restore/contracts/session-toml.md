# Contract: session.toml Format

**Version**: 1 | **Feature**: 003-session-restore | **Date**: 2026-06-18

---

## Location

`$XDG_STATE_HOME/edit/session.toml`

Default path when `XDG_STATE_HOME` is unset: `$HOME/.local/state/edit/session.toml`

---

## Schema (version 1)

```toml
version        = <integer>        # Required. Must be 1. Unknown versions → file ignored.
active_buffer  = <integer>        # Required. 0-based index of the active buffer in [[buffers]].
split_layout   = <string>         # Required. One of: "none", "horizontal", "vertical".
active_pane    = <integer>        # Required. 0 (left/only pane) or 1 (right pane).

[[buffers]]                       # Required. Zero or more entries.
path           = <string>         # Required. UTF-8 file path.
cursor_line    = <integer>        # Required. 1-based line number (≥ 1).
cursor_col     = <integer>        # Required. 1-based grapheme column (≥ 1).
```

---

## Constraints

| Field | Type | Valid Range | On violation |
|---|---|---|---|
| `version` | integer | exactly `1` | Treat whole file as absent |
| `active_buffer` | integer | `0 ≤ n < len(buffers)` | Treat whole file as absent |
| `split_layout` | string | `"none"`, `"horizontal"`, `"vertical"` | Treat whole file as absent |
| `active_pane` | integer | `0` or `1` | Clamp to `1` on restore |
| `path` | string | Non-empty UTF-8; passes `validate_path` | Skip this buffer, show warning |
| `cursor_line` | integer | `≥ 1` | Clamp to `1` |
| `cursor_col` | integer | `≥ 1` | Clamp to `1` |

---

## Versioning Policy

The `version` field gates all schema readers. When this field is incremented (i.e., a
future schema change), readers that only know version 1 will treat the file as absent and
open a blank buffer. This prevents silent data corruption from schema mismatches.

Breaking changes (field removal, rename, type change) require a `version` bump.
Additive changes (new optional fields) MAY retain version 1 if readers ignore unknown keys.

---

## Write Contract

- **Who writes**: The editor on every clean user-initiated exit.
- **When NOT written**: On crash exits, signal kills, or IO errors during write.
- **Atomic write**: The file is written via a `.session.toml.tmp` sibling, then renamed.
  Readers must never see a partial file.
- **Encoding**: UTF-8, no BOM.

## Read Contract

- **Who reads**: The editor on startup, when no file arguments are passed and `--no-session`
  is not set.
- **Errors**: Any read error, parse error, or constraint violation causes the file to be
  treated as absent. The editor never panics on a malformed session file.
- **Stale paths**: Each `path` in `[[buffers]]` is existence-checked and sanitized before
  opening. Non-existent or invalid paths are skipped with a status-bar warning.

---

## CLI Integration

| Flag | Effect |
|---|---|
| `--no-session` | Skip session load on startup; suppress restore prompt entirely |
| (no flag, no file args) | Normal startup: load session, show restore dialog if file present |
| (explicit file args) | File args take precedence; session not loaded, no prompt |
