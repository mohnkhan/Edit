# Quickstart: Live Menu-Bar Activation

Validation guide proving feature 009 works end-to-end. See [contracts/menu-interaction.md](./contracts/menu-interaction.md)
for the full interaction contract and [data-model.md](./data-model.md) for the menu model.

## Prerequisites

- Rust stable ≥ 1.74.0, `cargo`.
- `expect` + `tmux` for the smoke test (`make smoke`).
- Locale: tests run under `LC_ALL=C.UTF-8 LANG=C.UTF-8`.

## Build & test

```bash
# Unit + integration
cargo test menu_activation
cargo test --test '*' menu          # integration suite
cargo test -p edit menubar          # menubar unit tests (resolve_menus, navigation)

# Full local gate
make ci-local                       # fmt → clippy -D warnings → tests → smoke → perf-check
make static                         # confirm static musl build still links
```

## Scenario 1 — Built-in menu activation by keyboard (US1)

```bash
./edit /tmp/demo.txt
```

1. Press `Alt+F` → the File dropdown opens (first item highlighted).
2. Press `Down` until "Save" is highlighted; press `Enter`.
3. **Expect**: the buffer is saved (status bar confirms); the menu closes.
4. Re-open with `Alt+E`, press `Down`, then `Esc`.
5. **Expect**: the menu closes, buffer and cursor unchanged, no action performed.
6. Open any menu, press `Right`/`Left` repeatedly.
7. **Expect**: focus cycles through all top-level menus (wrapping), each opening its dropdown.

## Scenario 2 — Plugin menu activation by keyboard (US2)

Install + consent the reference `word-count` plugin (from `examples/plugins/word-count/`):

```bash
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/edit/plugins/word-count"
cp examples/plugins/word-count/plugin.toml examples/plugins/word-count/plugin.rhai \
   "${XDG_CONFIG_HOME:-$HOME/.config}/edit/plugins/word-count/"
# Launch once and Allow at the consent prompt (or pre-seed plugins.toml with allowed = true).
printf 'hello world from the editor\n' > /tmp/wc.txt
./edit /tmp/wc.txt
```

1. **Expect**: a **Tools** top-level menu appears in the bar, **between Options and Help**.
2. Navigate to Tools (Left/Right), press `Down` to "Word Count", press `Enter`.
3. **Expect**: the status bar shows the word count (e.g. `Word count: 5`).

## Scenario 3 — No-plugin parity (US1 / FR-011 / SC-003)

```bash
./edit --no-plugins /tmp/demo.txt
```

1. **Expect**: the menu bar looks exactly as before this feature — File, Edit, Search, View,
   Options, Help at their existing positions; no extra menus.
2. Run `cargo test menubar` — all pre-existing geometry tests pass unchanged, plus
   `test_resolve_menus_empty_matches_builtin`.

## Scenario 4 — Misbehaving plugin stays contained (SC-006)

With a menu plugin whose `menu_action` errors or loops (e.g. the `fs-violation` fixture):

1. Activate its menu item via the keyboard.
2. **Expect**: the editor remains responsive, a warning appears in the status bar, the buffer is
   intact, and the plugin is disabled (per the existing sandbox/dispatch layer).

## Pass criteria

- Scenarios 1–4 behave as described.
- `cargo test` green (new unit + integration tests, all existing tests unchanged).
- `make smoke` green including `plugin_menu_activate.exp`.
- `make ci-local` and `make static` succeed.
