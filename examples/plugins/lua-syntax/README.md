# lua-syntax — reference highlighter plugin

A minimal Lua syntax highlighter demonstrating the `edit` Plugin API (Feature 008).

Plugins for `edit` are **Rhai scripts** — plain text, no compilation, no build toolchain.
A plugin is a directory containing two files:

| File | Purpose |
|------|---------|
| `plugin.toml` | Manifest: identity, version, capabilities, permissions |
| `plugin.rhai` | The script the editor runs in a sandbox |

## Install

```sh
mkdir -p ~/.config/edit/plugins/lua-syntax
cp plugin.toml plugin.rhai ~/.config/edit/plugins/lua-syntax/
edit somefile.lua          # first run shows a one-time consent prompt → press Enter
```

No `cargo`, no `wasm`, no targets to add — just copy the two files.

## Manifest schema (`plugin.toml`)

```toml
id          = "lua-syntax"        # required; kebab-case, unique
name        = "Lua Syntax Highlighter"
version     = "1.0.0"             # required; semver
host_api    = "^1"               # required; host API range (current host = v1)
types       = ["highlighter"]    # one or more of: highlighter | keybinding | menu
extensions  = [".lua", ".luac"]  # required for highlighter; extensions handled
publisher   = "..."              # optional; shown in the consent dialog
description = "..."              # optional; shown in the plugin manager

# [keybindings]                  # keybinding plugins only — manifest-only, no script
# "F9" = "save"

# [[menu_items]]                 # menu plugins only
# menu = "Tools"
# item = "Word Count"
# item_id = "wc"

# [permissions]                  # optional; default is no filesystem access
# read_paths = []                # absolute paths/prefixes the plugin may read
# write_dirs = []
```

## Script contract (`plugin.rhai`)

A highlighter defines:

```rust
fn highlight(line, ext) {
    // return an array of token maps; start/end are BYTE offsets into `line`
    [ #{ start: 0, end: 2, kind: "comment" } ]
}
```

Valid `kind` values: `"default"`, `"keyword"`, `"string"`, `"comment"`, `"number"`,
`"operator"`, `"type"`. The host maps each kind to a theme colour.

The host **validates every token** (in-bounds, non-overlapping, on char boundaries) and
silently discards the whole array if any token is invalid — so a buggy plugin degrades to
no highlighting rather than corrupting the display.

## Sandbox

Scripts run in a default-deny Rhai sandbox:

- No filesystem, process, or network access by default. The only file access is the
  permission-gated `read_file(path)` host function (paths must be declared in `[permissions]`).
- A 50 ms per-call wall-clock limit; a script that loops or errors is disabled for the session.
- Host functions available to scripts: `log(level, msg)`, `status_bar(msg)`, `read_file(path)`.

See `examples/plugins/word-count/` (menu plugin) and `examples/plugins/custom-keys/`
(keybinding plugin, manifest-only) for the other plugin types.
