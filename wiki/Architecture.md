# Architecture

`edit` is a single-binary terminal application written in **Rust**. This page gives a high-level map
of the design and the modules under `src/`. It describes only modules that exist in the codebase.

## Technology stack

| Concern | Crate |
|---|---|
| Terminal UI / widgets | `ratatui` 0.26 |
| Terminal backend / input events | `crossterm` 0.27 |
| Text buffer | `ropey` 0.6 (rope data structure) |
| Encoding / transcoding | `encoding_rs`, `oem_cp`, `chardetng` |
| Unicode width / segmentation | `unicode-width`, `unicode-segmentation` |
| Search | `regex` |
| Plugin scripting | `rhai` (with `sync`), `semver` |
| File watching | `notify` 6 (inotify on Linux) |
| Config / serialization | `serde`, `toml` |
| CLI | `clap` 4 |
| Clipboard | `arboard` |
| Logging / signals | `log`, `env_logger`, `signal-hook` |

The crate exposes both a binary (`src/main.rs`) and a library (`src/lib.rs`) so integration tests can
drive internals directly.

## Module breakdown

```
src/
├── main.rs            # binary entry: CLI parse, startup, terminal setup, event loop
├── lib.rs             # library entry: re-exports modules for tests
├── app.rs             # App state + handle_action: the central state machine
├── buffer/            # text storage
│   ├── rope.rs        #   ropey-backed text buffer
│   ├── undo.rs        #   undo/redo with composite operations
│   ├── autosave.rs    #   auto-save + EDIT-RECOVERY-V1 snapshots (FNV-1a change detection)
│   └── mod.rs
├── encoding/          # encoding pipeline
│   ├── detect.rs      #   BOM/heuristic detection (UTF-8/16, code pages)
│   ├── transcode.rs   #   decode-to-UTF-8 / encode-from-UTF-8
│   └── mod.rs         #   encoding_from_str() aliases
├── highlight/         # syntax highlighting engine
│   ├── languages/     #   c, python, shell, yaml, markdown, rust, json, toml
│   └── mod.rs         #   Highlighter trait + dispatch
├── search/            # regex find/replace + match highlighting
│   ├── highlight.rs   #   match-range highlighting overlay
│   └── mod.rs         #   find/replace engine
├── input/             # keymap.rs, mouse.rs — key/scan handling → Action
│   ├── keymap.rs      #   DOS scan-code keymap → Action (merged with plugin bindings)
│   ├── mouse.rs       #   click/drag/wheel → Action
│   └── mod.rs
├── ui/                # ratatui rendering
│   ├── mod.rs         #   Ui::render composes the frame
│   ├── editor.rs      #   editor area widget (+ gutter, selection highlight)
│   ├── menubar.rs     #   menu bar + resolve_menus() model
│   ├── contextmenu.rs #   right-click context menu
│   ├── tabbar.rs      #   buffer tab bar (multi-buffer switching)
│   ├── statusbar.rs   #   status line (filename, encoding, EOL, position, notices)
│   ├── dialog.rs      #   modal dialogs (open, encoding select, find/replace, go-to-line, …)
│   ├── file_browser.rs #  navigable file browser for Open / Save (glob filter, entry details)
│   ├── buttons.rs     #   boxed buttons + focus ring shared by dialogs
│   ├── scrollbar.rs   #   interactive (clickable + draggable) scrollbars
│   ├── plugin_manager.rs # Options › Plugins dialog
│   ├── wrap.rs        #   soft-wrap WrapCache (visual ↔ logical mapping)
│   ├── width.rs       #   Unicode display-width helpers
│   └── theme.rs       #   classic / high-contrast / plain
├── plugin/            # Rhai plugin host
│   ├── manifest.rs    #   plugin.toml parsing + validation
│   ├── types.rs       #   Plugin, PluginType, Permission, tokens
│   ├── sandbox.rs     #   engine build + 50ms deadline
│   ├── api.rs         #   host functions exposed to scripts (status_bar, read_file…)
│   ├── consent.rs     #   consent records (plugins.toml)
│   ├── registry.rs    #   loaded-plugin registry
│   ├── highlighter.rs #   plugin highlighter integration
│   └── mod.rs         #   PluginHost: scan → validate → consent → compile → dispatch
├── watcher/           # notify-based external file watcher (debounce, refcounted dir watch)
├── session/           # session.toml save/load + restore types
├── config/            # config.toml schema (schema.rs) + loading
├── security/          # sanitize.rs — path traversal guard (validate_path)
└── diagnostics/       # logging.rs, crash.rs — panic hook + SIGSEGV crash reports
```

## Major subsystems (through v0.4.0)

The 0.1.0 foundation was a keyboard-only menu prototype; features 010–035 grew it into a
mouse-driven editor. The notable subsystems layered on since then:

- **Buffer tab bar** (`ui/tabbar.rs`) — multiple open buffers with a clickable DOS-style tab strip.
- **Mouse input** (`input/mouse.rs`) — clickable/draggable menus, dialog buttons, scrollbars,
  tab bar, caret-on-click in text fields, and app-wide mouse-wheel scrolling.
- **Selection model** — visible text selection with Shift-select and mouse-drag, rendered as a
  highlight by `ui/editor.rs`; word-wise navigation/selection/deletion (feature 032).
- **Find / Replace** (`search/`, `ui/dialog.rs`) — interactive regex find and replace dialogs with
  live match highlighting (`search/highlight.rs`).
- **Syntax highlighting** (`highlight/`) — pluggable highlighters for C, Python, shell, YAML,
  Markdown, Rust, JSON, and TOML; third-party highlighters can be contributed by plugins.
- **Crash-safety & diagnostics** (`diagnostics/`, `buffer/autosave.rs`) — panic/SIGSEGV crash
  reports plus `EDIT-RECOVERY-V1` autosave snapshots and crash-safe line access (feature 034).
- **Security / plugin sandbox** (`security/`, `plugin/`) — path-traversal guard and a default-deny
  Rhai plugin host (see below).

## How rendering and input flow

`edit` runs a single-threaded event loop driven by `crossterm` events.

1. **Input** — `crossterm` delivers a key/mouse/resize event. `src/input/` maps it through the
   keymap (merged with any plugin keybindings) to an `Action`.
2. **Update** — `app.rs` applies the `Action` in `App::handle_action`, the central state machine:
   editing buffer mutations, navigation, menu navigation, dialog state, save/quit prompts, plugin
   menu dispatch, etc.
3. **Render** — `ui::Ui::render` composes the whole frame from `App` state every tick: the menu bar
   (drawn *on top* of the editor so dropdowns are visible), the editor area (with optional gutter and
   soft-wrap via `WrapCache`), and the status bar. Rendering is stateless — it reads authoritative
   state fresh each frame (e.g. menu check-states, soft-wrap indicator).

A single resolved menu model — `resolve_menus()` in `src/ui/menubar.rs` — drives both rendering and
keyboard navigation, and is where plugin-contributed menus are positioned (between Options and Help,
merging into a built-in dropdown on name collision).

## The encoding pipeline

External bytes never enter the buffer untranslated. On read, `encoding/detect.rs` sniffs the BOM and
heuristics to pick a codec, then `encoding/transcode.rs` decodes to a UTF-8 rope. On write, the rope
is encoded back to the buffer's target encoding (with BOM where applicable) and written atomically.
This is the mechanism behind the UTF-8 hygiene guarantee described in [Encodings](Encodings.md).

## Security / sandbox model

Two layers of defense:

- **Path sanitation** (`src/security/sanitize.rs`) — `validate_path` guards against path-traversal,
  applied to paths loaded from session files and elsewhere.
- **Plugin sandbox** (`src/plugin/sandbox.rs`) — default-deny Rhai engine: no FS/network/process
  access except the permission-gated `read_file`; a 50 ms per-call wall-clock deadline; and session
  disabling for plugins that loop, error, or violate the sandbox. See
  [Plugin Development](Plugin-Development.md).

## Resilience

`src/diagnostics/` installs a panic hook and SIGSEGV handler (via `signal-hook`) that write a crash
report, and `buffer/autosave.rs` periodically writes `EDIT-RECOVERY-V1` snapshots so work can be
recovered after an unexpected exit.
