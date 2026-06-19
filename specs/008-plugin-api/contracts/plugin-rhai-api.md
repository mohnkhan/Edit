# Plugin Host ↔ Plugin Interface (Rhai)

**Type:** Interface contract
**Stability:** Draft (pre-1.0)
**Date:** 2026-06-19

This contract defines the boundary between the editor host (Rust) and a plugin
authored as a [Rhai](https://rhai.rs) script. It replaces the previous
WASM-based plugin contract, which has been removed.

## Overview

A plugin is a directory:

```
$XDG_CONFIG_HOME/edit/plugins/<id>/
├── plugin.toml    # manifest
└── plugin.rhai    # Rhai source script
```

At load time the host parses `plugin.rhai` into a `rhai::AST`. There is **no
compilation step and no binaries** — the script is interpreted source. Rhai is
pinned to `rhai = "1"` (pure Rust). There is **no MessagePack** in this design:
Rhai `Array` / `Map` values convert directly to and from their Rust
counterparts at the call boundary.

The Rhai base language exposes **no `io`, `fs`, `process`, or `network`
access**, and module `import` is disabled (`set_max_modules(0)`). The only
filesystem reach available to a script is the host-registered `read_file`
function, which is permission-gated (see below).

## API Versioning

The manifest field `host_api` is a semver requirement (e.g. `host_api = "^1"`)
and is **authoritative**. The host evaluates it against:

```rust
const HOST_PLUGIN_API_VERSION: i32 = 1;
```

If the requirement does not match `HOST_PLUGIN_API_VERSION`, the plugin is
rejected at load.

No in-script version export is required. Optionally, the host injects a
`HOST_API_VERSION` constant into the script's scope for runtime inspection.

## Plugin Types

| Type          | Requires script | Entry point(s)        |
|---------------|-----------------|-----------------------|
| `highlighter` | yes             | `highlight`           |
| `menu`        | yes             | `menu_action`         |
| `keybinding`  | **no**          | manifest-only         |

A plugin may declare more than one type via the manifest `types` array.

## Plugin → Host: Entry-Point Functions

Entry-point functions are defined in `plugin.rhai` and invoked by the host via
`engine.call_fn`. Only the functions matching the plugin's declared `types` are
called.

### `fn highlight(line, ext)` — Highlighter plugins

- `line`: the UTF-8 line string with **no trailing newline**.
- `ext`: the file extension **including the leading dot** (e.g. `".lua"`).

Returns a Rhai `Array` of `Map`s, each token shaped as:

```rust
#{ start: <int>, end: <int>, kind: <string> }
```

- `start` / `end`: **byte offsets** into `line`, `end` exclusive.
- `kind`: one of
  `"default"`, `"keyword"`, `"string"`, `"comment"`, `"number"`, `"operator"`, `"type"`.

**Host validation** (a violation discards the **whole array** but does **not**
disable the plugin):

- `start < end`
- both `start` and `end` within `[0, line.len()]`
- tokens **must not overlap**
- `kind` must be a known string — **an unknown kind discards the array**

When the array is discarded, the host falls back to built-in highlighting (or
no highlighting) for that line.

**Example** — highlighting `let x = 42` in a `.rhai` file:

```rhai
fn highlight(line, ext) {
    [
        #{ start: 0, end: 3, kind: "keyword" },   // let
        #{ start: 8, end: 10, kind: "number" },   // 42
    ]
}
```

### `fn menu_action(item_id, buf_content)` — Menu plugins

- `item_id`: the manifest-declared id string of the invoked menu item.
- `buf_content`: the active buffer's full UTF-8 text (may be empty).

Returns a Rhai `Map`:

```rust
#{ status: <string>, message: <string?> }
```

- `status`: `"ok"` or `"error"`.
- `message`: optional status-bar text (**≤ 120 chars, truncated**).

Alternatively, the script may call the host `status_bar(msg)` function directly
instead of (or in addition to) returning a `message`.

**Example** — a "count lines" menu action:

```rhai
fn menu_action(item_id, buf_content) {
    if item_id == "count_lines" {
        let n = buf_content.split("\n").len();
        #{ status: "ok", message: `Lines: ${n}` }
    } else {
        #{ status: "error", message: "Unknown action" }
    }
}
```

### Keybinding plugins — manifest-only

Keybinding plugins define **no script entry point**. They are declared entirely
in the manifest `[keybindings]` table and bind keys to existing editor
commands. A keybinding-only plugin need not ship a `plugin.rhai` at all.

## Host → Plugin: Registered Functions

These functions are registered into the Rhai engine via
`Engine::register_fn` and are callable from any script.

### `log(level, msg)`

- `level`: int — `0` = debug, `1` = info, `2` = warn, `3` = error.
- `msg`: string.

Writes a structured entry to the editor log.

### `status_bar(msg)`

- `msg`: string.

Queues a status-bar message (**≤ 120 chars, truncated**).

### `read_file(path)`

- `path`: string filesystem path.
- Returns: the file contents as a string.

**Gated:** throws a Rhai error (returns `Err`) if `path` is not within the
plugin's declared `read_paths` (`ReadPath`) permissions; the violation is
logged. This is the **only filesystem access** available to scripts.

## Manifest Schema (`plugin.toml`)

```toml
# Unique plugin id; must match the containing directory name.
id = "rhai-highlighter"

# Human-readable display name.
name = "Rhai Syntax Highlighter"

# Plugin's own version (semver, informational).
version = "0.1.0"

# Semver requirement against the host API version (authoritative).
host_api = "^1"

# Declared plugin types. Drives which entry points the host calls.
types = ["highlighter", "menu", "keybinding"]

# File extensions (incl. leading dot) this plugin handles.
extensions = [".rhai"]

# Optional metadata.
publisher = "Example Org"
description = "Highlights Rhai source and adds a line-count tool."

# Keybinding plugins: bind a key to an existing editor command.
[keybindings]
"F9" = "save"

# Menu plugins: one table per declared menu item.
[[menu_items]]
menu = "Tools"          # top-level menu to attach to
item = "Count Lines"    # display label
item_id = "count_lines" # id passed to menu_action(item_id, ..)
position = 3            # optional ordering hint within the menu

# Sandbox permissions (default-deny).
[permissions]
read_paths = []   # paths read_file() may access; empty = none
write_dirs = []   # reserved for future write access; empty = none
```

## Sandbox & Resource Limits

- **Per-call wall-clock limit:** default **50 ms**, enforced via
  `Engine::on_progress` checking a deadline; on expiry Rhai aborts with
  `ErrorTerminated`.
- **Resource caps:** `max_operations`, `max_call_levels`, `max_string_size`,
  `max_array_size`, `max_map_size`.
- **Filesystem:** default-deny. Only `read_file` against declared `read_paths`
  is permitted; `import` / modules disabled (`set_max_modules(0)`); no
  `io`/`fs`/`process`/`network` in the base language.

## Error-Handling Contract

| Scenario | Host behaviour |
|---|---|
| Manifest `host_api` incompatible with `HOST_PLUGIN_API_VERSION` | Plugin rejected at load; error logged; editor continues. |
| `plugin.toml` parse error | Plugin rejected; warning logged; editor continues. |
| `plugin.rhai` parse/compile error | Plugin rejected; warning logged; editor continues. |
| `highlight()` returns invalid tokens (overlap / out-of-bounds / unknown kind) | Token array discarded; built-in / no highlighting used; plugin **not** disabled. |
| `highlight()` / `menu_action()` exceeds 50 ms time limit | Rhai aborts (`ErrorTerminated`); plugin **disabled for session**; status-bar warning; editor continues. |
| Script runtime error / panic in dispatch | Caught (`catch_unwind`); plugin **disabled for session**; status-bar warning; editor continues. |
| `read_file()` for undeclared path | Rhai error thrown to script; violation logged; plugin **not** disabled on first offence; **disabled after 3 violations**. |
| `menu_action()` returns `status = "error"` | `message` shown in status bar; plugin remains enabled. |
