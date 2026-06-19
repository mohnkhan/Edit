# Quickstart: Plugin API Validation Guide (Feature 008)

**Prerequisites**: A built `./edit` binary (run `make` or `cargo build --release`). Plugins are plain Rhai source scripts — no compilation toolchain is required.

---

## Scenario 1: Install and activate a syntax highlighter plugin (US1 / P1)

### Setup

```bash
# Install the reference example plugin (Rhai source — no build step)
mkdir -p ~/.config/edit/plugins/lua-syntax
cp examples/plugins/lua-syntax/plugin.toml ~/.config/edit/plugins/lua-syntax/
cp examples/plugins/lua-syntax/plugin.rhai ~/.config/edit/plugins/lua-syntax/
```

### Run

```bash
# Launch editor on a Lua file (first run → consent dialog appears)
./edit examples/hello.lua
# In consent dialog: press Enter (allow)
```

### Expected outcome

- `--` comment tokens appear in the theme's comment colour.
- Status bar shows the file's encoding (unchanged).

### Verify file-level

```bash
# If plugin NOT installed/active, run editor headlessly (future --dump-tokens flag):
./edit --dump-tokens examples/hello.lua | grep comment
# Should output at least one "Comment" token for each -- line
```

---

## Scenario 2: Cancel plugin consent (US4 / P4 — consent flow)

### Run

```bash
# Remove consent record to re-trigger consent dialog
grep -v lua-syntax ~/.config/edit/plugins.toml > /tmp/p.toml && mv /tmp/p.toml ~/.config/edit/plugins.toml
./edit examples/hello.lua
# In consent dialog: press Escape (deny)
```

### Expected outcome

- Editor opens normally.
- No Lua syntax highlighting is applied.
- `plugins.toml` has `[plugins.lua-syntax] allowed = false`.

---

## Scenario 3: Disable a plugin from the plugin manager (US4 / P4)

### Run

```bash
./edit examples/hello.lua
# In editor: Alt+O → Plugins → highlight lua-syntax → press Space to toggle off → press Esc
# Restart editor
./edit examples/hello.lua
```

### Expected outcome

- Lua syntax highlighting is absent after restart.
- `plugins.toml` has `allowed = false` for `lua-syntax`.

---

## Scenario 4: Plugin keybinding (US2 / P2)

### Setup

```bash
# Install the example keybinding plugin (manifest-only — no plugin.rhai needed)
mkdir -p ~/.config/edit/plugins/custom-keys
cp examples/plugins/custom-keys/plugin.toml ~/.config/edit/plugins/custom-keys/
# plugin.toml declares: [keybindings] "F9" = "save"
# Keybinding-only plugins carry no script; the manifest is sufficient.
```

### Run

```bash
./edit /tmp/test.txt
# Edit text, then press F9
```

### Expected outcome

- File is saved (mtime updated on `/tmp/test.txt`).
- Status bar shows "Saved".

```bash
stat /tmp/test.txt    # mtime should be recent
```

---

## Scenario 5: Plugin menu item (US3 / P3)

### Setup

```bash
mkdir -p ~/.config/edit/plugins/word-count
cp examples/plugins/word-count/plugin.toml ~/.config/edit/plugins/word-count/
cp examples/plugins/word-count/plugin.rhai ~/.config/edit/plugins/word-count/
# plugin.toml declares: menu_items = [{menu="Tools", item="Word Count", item_id="wc"}]
```

### Run

```bash
./edit /tmp/essay.txt
# Press Alt+T (Tools menu) → Word Count
```

### Expected outcome

- Status bar displays "Word count: N" where N matches `wc -w /tmp/essay.txt`.

---

## Scenario 6: Misbehaving plugin is isolated (US5 / P5)

### Setup

```bash
# Test plugin whose highlight() never returns
mkdir -p ~/.config/edit/plugins/infinite-loop
cp examples/plugins/infinite-loop/plugin.toml ~/.config/edit/plugins/infinite-loop/
cp examples/plugins/infinite-loop/plugin.rhai ~/.config/edit/plugins/infinite-loop/
# plugin.rhai contains: fn highlight(line, ext) { loop {} }
```

### Run

```bash
./edit examples/hello.lua
# Plugin is loaded → first highlight call triggers the loop
```

### Expected outcome

- Within 200 ms of opening the file, a status-bar warning appears:
  `"Plugin 'infinite-loop' timed out and has been disabled"`
- The editor remains fully responsive.
- The file opens with built-in highlighting (or no highlighting).
- `~/.local/state/edit/logs/edit-*.log` contains a `WARN` entry for the plugin.

---

## Scenario 7: `--no-plugins` flag (FR-008)

```bash
./edit --no-plugins examples/hello.lua
```

### Expected outcome

- No consent dialog.
- No syntax highlighting from plugins.
- `plugins.toml` is not modified.
- Options > Plugins shows empty list with notice "Plugins disabled (--no-plugins)".

---

## Verification Checklist

| Check | Command / Method | Pass criterion |
|---|---|---|
| Plugin loaded | Open file in editor with plugin installed | No error; extension takes effect |
| Consent persisted | Check `plugins.toml` | Entry present with correct `allowed` value |
| Time-limit enforced | Install infinite-loop plugin | Warning in ≤ 200 ms; editor responsive |
| Crash isolation | Install crash-on-call plugin | Editor survives; plugin disabled message shown |
| `--no-plugins` | Launch with flag | No plugins active; `plugins.toml` unchanged |
| UTF-8 validation | Plugin returning non-UTF-8 string | Plugin disabled; error logged; editor intact (Rhai scripts and returned strings are validated UTF-8 by the host) |
| Round-trip token | `--dump-tokens` on a Lua file | `Comment` tokens present for `--` lines (host runs the plugin's `plugin.rhai` and collects returned tokens) |
