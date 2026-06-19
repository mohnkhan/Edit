# custom-keys — reference keybinding plugin

Binds **F9** to **Save**. Demonstrates a `keybinding` plugin for the `edit` Plugin API
(Feature 008). Keybinding plugins are **manifest-only** — there is no `plugin.rhai`.

## Install

```sh
mkdir -p ~/.config/edit/plugins/custom-keys
cp plugin.toml ~/.config/edit/plugins/custom-keys/
```

## Manifest

```toml
id       = "custom-keys"
types    = ["keybinding"]
host_api = "^1"

[keybindings]
"F9" = "save"        # key chord -> built-in action name
```

Action names accept the lowercase form (`"save"`, `"quit"`, `"find"`, …) or the editor's
PascalCase form (`"Save"`). Plugin bindings take precedence over built-in bindings, **except**
safety-critical actions (Save, Quit) which a plugin may not override.

See `../lua-syntax/README.md` for the full manifest schema.
