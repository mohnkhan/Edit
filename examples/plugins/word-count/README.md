# word-count — reference menu plugin

Adds a **Tools > Word Count** menu item that reports the active buffer's word count in the
status bar. Demonstrates a `menu` plugin for the `edit` Plugin API (Feature 008).

## Install

```sh
mkdir -p ~/.config/edit/plugins/word-count
cp plugin.toml plugin.rhai ~/.config/edit/plugins/word-count/
```

## Manifest

```toml
id       = "word-count"
types    = ["menu"]
host_api = "^1"

[[menu_items]]
menu    = "Tools"        # top-level menu (created if absent)
item    = "Word Count"   # item label
item_id = "wc"           # passed to menu_action()
```

## Script contract

A menu plugin defines:

```rust
fn menu_action(item_id, buf_content) {
    // ... compute something from buf_content ...
    status_bar("Word count: 42");          // show a status-bar message, and/or
    #{ status: "ok", message: "Word count: 42" }   // return a result map
}
```

`status` is `"ok"` or `"error"`; `message` (≤120 chars) is shown in the status bar.

See `../lua-syntax/README.md` for the full manifest schema and sandbox details.
