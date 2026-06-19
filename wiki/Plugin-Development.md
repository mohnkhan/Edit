# Plugin Development

`edit` has a small, sandboxed plugin system (Feature 008, extended in Feature 009) that lets you add
**syntax highlighters**, **custom keybindings**, and **menu items** without touching the editor's
source. Plugins are written in [**Rhai**](https://rhai.rs) — a pure-Rust embedded scripting language,
chosen because it needs no C/C++ runtime, links statically on every target, and keeps the footprint
tiny (a key requirement for the MyOS image).

## Directory layout

Each plugin is a directory under your config dir, named by its `id`:

```
$XDG_CONFIG_HOME/edit/plugins/<id>/
├── plugin.toml     # manifest (required)
└── plugin.rhai     # script (required for highlighter & menu plugins)
```

Keybinding-only plugins are **manifest-only** and need no script. Reference plugins ship under
`examples/plugins/` in the repository (`word-count`, `custom-keys`, `lua-syntax`, plus the
`fs-violation` and `infinite-loop` test fixtures).

## The three plugin types

A plugin declares one or more `types` in its manifest:

| Type | Purpose | Script entry point |
|---|---|---|
| `highlighter` | Tokenize lines for a set of file extensions; takes precedence over the built-in highlighter for those extensions | `highlight(line, ext)` |
| `keybinding` | Merge key → command bindings into the keymap (manifest-only) | — |
| `menu` | Contribute top-level menu items; activation runs sandboxed code | `menu_action(item_id, buf_content)` |

Plugin keybindings take precedence over built-ins, **except** the safety-critical `save` and `quit`
actions, which can never be overridden.

## Manifest fields (`plugin.toml`)

These are the fields the manifest parser actually recognizes (`src/plugin/manifest.rs`):

| Field | Required | Notes |
|---|---|---|
| `id` | yes | kebab-case: `[a-z0-9-]`, no leading/trailing hyphen |
| `name` | yes | display name, max 64 characters |
| `version` | yes | semver (e.g. `1.0.0`) |
| `host_api` | yes | semver requirement against the host API (e.g. `"^1"`) |
| `types` | yes | at least one of `highlighter`, `keybinding`, `menu` |
| `extensions` | required for highlighters | e.g. `[".lua", ".luac"]` |
| `publisher` | optional | free text |
| `description` | optional | free text |
| `[keybindings]` | optional | `"<key>" = "<command>"` table |
| `[[menu_items]]` | optional | `menu`, `item`, `item_id` (all non-empty); optional `position` |
| `[permissions]` | optional | `read_paths = [...]`, `write_dirs = [...]` (gates host `read_file`) |

If `host_api` does not match the host's API version the plugin is rejected at load time.

## The consent model

The first time a newly installed plugin is loaded, `edit` shows a **one-time consent dialog**:

- `Enter` — allow
- `Esc` — deny

Your decision is persisted to `$XDG_CONFIG_HOME/edit/plugins.toml`, so you are only asked once per
plugin. The **plugin manager** at **Options › Plugins** lists installed plugins and lets you toggle
them on and off afterwards (`Up`/`Down` to navigate, `Space` to toggle, `Esc` to close).

## The sandbox

Plugins run **default-deny**:

- **No filesystem, network, or process access** — the only host capability is a permission-gated
  `read_file`, available only for paths declared under `[permissions].read_paths`.
- **50 ms per-call wall-clock limit** — enforced via Rhai's progress hook; a script that runs too
  long is interrupted.
- **Crash / misbehavior isolation** — a plugin that loops, errors, or repeatedly violates the
  sandbox is **disabled for the session** while the editor stays responsive.
- **`--no-plugins`** disables all plugin loading for a session without changing saved consent.

Highlighter output is additionally validated by the host: every returned token is bounds-checked, must
not overlap, must fall on char boundaries, and must use a known kind — any invalid token causes the
whole array for that line to be silently discarded (the plugin is **not** disabled for bad output).

## Worked example 1: a menu plugin (Word Count)

This is the real `examples/plugins/word-count` plugin. It adds **Tools › Word Count** and reports
the active buffer's word count in the status bar.

`plugin.toml`:

```toml
id = "word-count"
name = "Word Count"
version = "1.0.0"
host_api = "^1"
types = ["menu"]
publisher = "edit reference plugins"
description = "Adds Tools > Word Count; reports the active buffer's word count in the status bar."

[[menu_items]]
menu = "Tools"
item = "Word Count"
item_id = "wc"
```

`plugin.rhai`:

```rust
// The host calls menu_action(item_id, buf_content) when the user activates the menu item.
// It returns #{ status, message } and/or calls the host status_bar(msg) function.

fn is_space(ch) {
    ch == " " || ch == "\t" || ch == "\n" || ch == "\r"
}

fn menu_action(item_id, buf_content) {
    let count = 0;
    let in_word = false;
    let n = buf_content.len();
    let i = 0;
    while i < n {
        let ch = buf_content.sub_string(i, 1);
        if is_space(ch) {
            in_word = false;
        } else if !in_word {
            in_word = true;
            count += 1;
        }
        i += 1;
    }
    let msg = "Word count: " + count;
    status_bar(msg);
    #{ status: "ok", message: msg }
}
```

When the user opens the menu (it renders **between Options and Help**) and activates *Word Count*,
the host dispatches `menu_action("wc", <buffer text>)` in the sandbox and shows the returned message
in the status bar. A menu plugin whose `menu` name matches a built-in menu is merged into that
built-in dropdown.

## Worked example 2: a keybinding plugin (Custom Keys)

This is the real `examples/plugins/custom-keys` plugin — manifest-only, no script:

```toml
id = "custom-keys"
name = "Custom Keys"
version = "1.0.0"
host_api = "^1"
types = ["keybinding"]
publisher = "edit reference plugins"
description = "Adds an F9 = Save keybinding. Keybinding plugins are manifest-only (no script)."

[keybindings]
"F9" = "save"
```

The `[keybindings]` entries merge into the keymap on load (subject to the Save/Quit override rule).

## Worked example 3: a highlighter plugin

A highlighter declares its `extensions` and implements `highlight(line, ext)`, returning an array of
token maps `#{ start, end, kind }` where `start`/`end` are **byte offsets** into `line` (end
exclusive) and `kind` is one of: `default`, `keyword`, `string`, `comment`, `number`, `operator`,
`type`. See `examples/plugins/lua-syntax/plugin.rhai` for a complete, working implementation that
highlights Lua comments, keywords, and numbers.

## See also

- [Architecture](Architecture.md) — where the plugin host sits in the system
- [Configuration](Configuration.md) — plugin and consent file locations
- [Keybindings](Keybindings.md) — the menu and manager dialog keys
