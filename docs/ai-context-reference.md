# AI Context Reference

Deep, per-subsystem implementation detail for `edit` — the on-demand companion to the always-loaded
[`CLAUDE.md`](../CLAUDE.md) behavioral summary. Open the relevant section here when working on a
specific subsystem; this file is intentionally **not** loaded into every turn.

> **Stack:** `edit` is a native **Rust** terminal application (cargo + rustc, MSRV 1.74,
> edition 2021). The TUI is built on [`ratatui`](https://ratatui.rs) + `crossterm`; the text buffer
> is a [`ropey`](https://docs.rs/ropey) rope; transcoding uses `encoding_rs` + `oem_cp`; plugins run
> on an embedded [`rhai`](https://rhai.rs) engine. There is **no C/ncurses code** anywhere in the
> tree — earlier docs that mentioned gcc/clang/PDCurses were inaccurate.

For the user-facing capability list (keys, flags, encodings, themes) see
[`CAPABILITIES.md`](CAPABILITIES.md); for status/CI/build matrix see [`STATUS.md`](STATUS.md). Each
feature also has a `specs/NNN-*/` directory (spec → plan → tasks) linked from the relevant section
below.

---

## Source layout

All code lives under `src/`. Modules are re-exported for integration tests from `src/lib.rs`; the
binary entry point is `src/main.rs`.

| Module | Path | Responsibility |
|---|---|---|
| App / event loop | `src/app.rs` | Owns all editor state; drives the main draw/input loop |
| Buffer | `src/buffer/` | `ropey`-backed text model, undo/redo, autosave |
| Encoding | `src/encoding/` | Detection (BOM + heuristics) and decode/encode transcoding |
| Highlight | `src/highlight/` | `Highlighter` trait + built-in language highlighters |
| Input | `src/input/` | `Action` enum, `KeybindingMap`, event dispatch, mouse |
| UI | `src/ui/` | ratatui widgets: editor, menubar, statusbar, dialogs, theme, wrap |
| Plugin | `src/plugin/` | Rhai host, manifest parsing, sandbox, consent, registry |
| Watcher | `src/watcher/` | External-file-change detection via `notify` |
| Session | `src/session/` | Session save/restore (`session.toml`) |
| Search | `src/search/` | Regex/literal search + match highlighting |
| Config | `src/config/` | `config.toml` schema + load/merge |
| Security | `src/security/` | Path sanitization, input safety helpers |
| Diagnostics | `src/diagnostics/` | Logging + crash-report capture |

---

## Rendering pipeline (`src/app.rs`, `src/ui/`)

Rendering is **ratatui over a `CrosstermBackend`** — *not* ncurses. The flow:

1. `App` sets up the terminal in `app.rs`: `enable_raw_mode()`, `EnterAlternateScreen`, and
   (when mouse is enabled) `EnableMouseCapture` via `crossterm::execute!`. A
   `Terminal<CrosstermBackend<Stdout>>` is constructed.
2. The main loop alternates **draw** and **input**:
   - **Draw:** `terminal.draw(|f| …)` paints, top to bottom, the menu bar
     (`ui::menubar::MenuBarWidget`), the editor area (`ui::editor`), and the status bar
     (`ui::statusbar`). Any active modal (`ui::dialog`, `ui::plugin_manager`) is drawn last with a
     `Clear` so it sits on top. Open dropdowns are also rendered after the editor area so they are
     never painted under content.
   - **Input:** `crossterm` events are read and translated to an [`Action`](#keybindings-srcinputkeymaprs)
     via `input::dispatch_event`, then applied to `App` state.
3. On exit (clean or panic), the terminal is restored: `LeaveAlternateScreen`, `DisableMouseCapture`,
   `disable_raw_mode()`.

Styling is centralized in `ui::theme::Theme` (see [Themes](#themes-srcuithemers)); every widget looks
up the relevant `Theme` field when it paints. Syntax/search styling is expressed as
`ratatui::style::Style` spans (see [Highlighting](#syntax-highlighting-srchighlight)).

**Debugging rendering:** run with `--debug` (optionally `RUST_LOG=debug`) and read the log under
`$XDG_STATE_HOME/edit/logs/edit-<date>.log`. There is no `NCURSES_TRACE`.

---

## Buffer internals (`src/buffer/`)

The buffer is a thin, ergonomic wrapper over a `ropey::Rope`.

- `buffer/rope.rs` — `EditorRope`, wrapping `ropey::Rope`. Ropes give O(log n) edits/indexing on
  large files, which is why startup/large-file/keystroke benchmarks stay flat (see `benches/`).
- `buffer/mod.rs` — the core data model:
  - `CursorPos` — cursor position in **grapheme** + visual (display-column) coordinates. Cursor math
    uses `unicode-segmentation` (grapheme clusters) and `unicode-width` (display width), so wide
    CJK glyphs and combining marks move the cursor correctly.
  - `Selection` — an `(anchor, active)` pair.
  - `LineEnding` — `Lf` vs `Crlf`; detected on open and **preserved** on save.
  - `Buffer` — open/save, dirty tracking, encoding association, undo stack.
  - `BufferError` — file-I/O + encoding error type.
- `buffer/undo.rs` — `UndoStack` with linear undo/redo over `EditOp` records.
- `buffer/autosave.rs` — `AutosaveState`; periodic crash-recovery snapshots
  (`EDIT-RECOVERY-V1` envelope; see [Recovery](#recovery--crash-reports)).

**Invariant:** everything in the rope is valid UTF-8. External bytes never enter the rope directly —
they are decoded through `src/encoding/` first (see below).

---

## Encoding pipeline (`src/encoding/`)

This is the enforcement point for the project's UTF-8-everywhere rule.

### Detection — `encoding/detect.rs`

`detect_encoding(&[u8]) -> EncodingId` runs in priority order:

1. **BOM sniffing** — entries in `ENCODING_REGISTRY` that carry a `bom` are checked first
   (UTF-16 LE `FF FE`, UTF-16 BE `FE FF`, UTF-8 BOM).
2. **UTF-8 validation** — if the bytes are valid UTF-8, `Utf8` wins.
3. **Heuristics** — otherwise `chardetng` picks the most likely legacy codepage.

`EncodingId` variants and their transcoding backend:

| `EncodingId` | Name | BOM | Backend |
|---|---|---|---|
| `Utf8` | UTF-8 | optional (stripped on read) | native |
| `Utf16Le` | UTF-16 LE | `FF FE` (written on encode) | `encoding_rs` |
| `Utf16Be` | UTF-16 BE | `FE FF` (written on encode) | `encoding_rs` |
| `Cp437` | DOS CP437 | — | `oem_cp` |
| `Cp850` | DOS CP850 | — | `oem_cp` |
| `Iso8859_1` | Latin-1 | — | `encoding_rs` |
| `Windows1252` | Windows Western | — | `encoding_rs` |

An `EncodingProfile` (static descriptor: `name`, `id`, optional `bom`) exists for each variant in
`ENCODING_REGISTRY`. `profile_for(id)` and `encoding_from_str(name)` (case-insensitive; unknown →
`Utf8`) are the public lookups. `encoding_from_str` is what backs the `--encoding` CLI aliases
(`utf-8`, `cp437`, `cp850`, `iso-8859-1`, `windows-1252`, `utf-16-le`, `utf-16-be`, `utf-16`).

### Transcoding — `encoding/transcode.rs`

`decode(&[u8], EncodingId) -> Result<String, TranscodeError>` and
`encode(&str, EncodingId) -> Result<Vec<u8>, TranscodeError>`. UTF-16 round-trips full surrogate
pairs; legacy codepages map through `oem_cp`/`encoding_rs` tables.

**Rule of thumb:** any function that reads external bytes must call `decode` (or validate as UTF-8)
before the text reaches a `Buffer`. Never build buffer text from raw `&[u8]`.

Specs: `specs/002-utf16-transcoding/`, `specs/004-save-as-encoding-ui/`.

---

## Keybindings (`src/input/keymap.rs`)

- `Action` — the enum of every bindable editor action (File: `Save`, `SaveAs`, `SaveAsEncoding`,
  `Open`, `Close`, `Quit`; Edit: `Cut`, `Copy`, `Paste`, `Undo`, `Redo`, `SelectAll`;
  Search: `Find`, `FindNext`, …; plus navigation, menu, and plugin actions).
- `KeybindingMap` — a `HashMap` from key chord → `Action`, seeded with the DOS-faithful defaults and
  layered with user/plugin overrides. **Safety-critical actions (`Save`, `Quit`) cannot be
  overridden** by plugins.
- `input/mod.rs` — `dispatch_event` translates a `crossterm` key/mouse event into an `Action` given
  the current `KeybindingMap` and modal state.
- `input/mouse.rs` — mouse event mapping (gated on terminal support).

Menu activation (`F10` top-level, `Alt+<letter>` direct dropdown, arrows to navigate, `Enter`
activate, `Esc` close) is handled in `App` against the resolved menu model from
`ui::menubar::resolve_menus`. Specs: `specs/009-menu-bar-activation/`.

**Key-binding regression?** Start at `src/input/keymap.rs` and the DOS scan-code mapping table.

---

## Menus (`src/ui/menubar.rs`)

A single resolved menu model drives both rendering and navigation:

- `MenuItem` — `{ label, action }`. Static built-in menus (File/Edit/Search/View/Options/Help) are
  defined here.
- `MenuState` / `MenuBarState` — the open/close/highlight state machine stored in `App`.
- `MenuBarWidget` — the ratatui `Widget` that paints the bar and the active dropdown (with `Clear`
  so the dropdown is on top of editor content).
- `resolve_menus()` — merges built-in menus with plugin-contributed menus. Plugin top-level menus
  render **between Options and Help** (Help stays rightmost); a plugin menu whose name collides with
  a built-in is merged into that built-in dropdown. Activating a plugin item dispatches
  `Action::PluginMenuActivated` and shows the sandboxed `menu_action` result in the status bar.

---

## Syntax highlighting (`src/highlight/`)

- `highlight/mod.rs` — the `Highlighter` trait (`Send + Sync`), the `Span` struct (`{ start, end,
  style }` with **byte** offsets, half-open `[start, end)`, `ratatui::style::Style`), and
  `detect_highlighter(path)` which picks a highlighter by file extension.
- `highlight/languages/` — built-ins: `c.rs` (C/C++ `.c/.h/.cpp/.hpp`), `python.rs` (`.py`),
  `shell.rs` (`.sh/.bash`), `yaml.rs` (`.yml/.yaml`), `markdown.rs` (`.md`).

Plugin highlighters (Feature 008) integrate via the same trait and **take precedence** over the
built-in for their declared extensions (see [Plugins](#plugin-host-srcplugin)).

---

## Plugin host (`src/plugin/`)

Plugins extend the editor with syntax highlighters, keybindings, and menu items, with no native
code — the engine is **Rhai** (pure-Rust, statically linkable, builds on every target).

- `plugin/manifest.rs` — parses & **fully validates** `plugin.toml` *before* any script is compiled
  (so identity + requested permissions are known ahead of the consent flow). `PluginLoadError`
  covers parse errors, invalid id/version, `host_api` version mismatch, non-UTF-8, script parse
  errors, and consent denial.
- `plugin/types.rs` — `Plugin`, `PluginType`, `Permission`, `PluginMenuItem`,
  `HOST_PLUGIN_API_VERSION`.
- `plugin/sandbox.rs` — builds the sandboxed `rhai::Engine`:
  - **Per-call wall-clock deadline** `PLUGIN_CALL_TIMEOUT_MS = 50` ms, enforced via Rhai
    `on_progress` against a shared `Deadline` (`Arc<Mutex<Instant>>`) the host sets before each call.
  - Resource caps: `MAX_OPERATIONS = 5_000_000`, `MAX_CALL_LEVELS = 32`,
    `MAX_STRING_SIZE = 256 KiB`, `MAX_ARRAY_SIZE`/`MAX_MAP_SIZE = 100_000`; expression depths pinned
    so debug/release parse identically.
  - Module imports disabled — the **only** host FS capability is the permission-gated `read_file`.
- `plugin/api.rs` — registers host functions (`register_host_functions`, `HostState`).
- `plugin/consent.rs` — one-time consent dialog on first run of a newly-installed plugin; decisions
  persisted to `$XDG_CONFIG_HOME/edit/plugins.toml`.
- `plugin/registry.rs` — discovery/load of `$XDG_CONFIG_HOME/edit/plugins/<id>/` and the
  Options → Plugins manager (`src/ui/plugin_manager.rs`).
- `plugin/highlighter.rs` — adapts a plugin script to the `Highlighter` trait.

`--no-plugins` disables all plugin loading for a session without changing saved consent. Reference
plugins live in `examples/plugins/` (`word-count`, `custom-keys`, `lua-syntax`, plus `fs-violation`
and `infinite-loop` which exercise the sandbox denials). User-facing detail:
[`wiki/Plugin-Development.md`](../wiki/Plugin-Development.md). Specs: `specs/008-plugin-api/`,
`specs/009-menu-bar-activation/`.

---

## File watching (`src/watcher/`)

Wraps the `notify` crate's platform-native watcher (inotify on Linux). The editor drains events
non-blocking from the main loop.

- `SELF_WRITE_GRACE = 2s` — suppresses the inotify event caused by the editor's own write.
- `DEBOUNCE_SECS = 1s` — coalesces rapid external writes into a single reload prompt.
- On external modify → Y/N reload dialog (with an unsaved-changes warning if the buffer is dirty);
  on delete → status-bar notice, buffer kept in memory.
- `--no-watch` / `no_watch` config disables watching for the session.

Specs: `specs/007-external-file-watch/`.

---

## Session & recovery

- **Session** (`src/session/`) — on clean exit writes `$XDG_STATE_HOME/edit/session.toml`
  (`BufferEntry` per open file: path + 1-based cursor line/col, plus split layout). On the next
  no-arg startup a TUI restore dialog offers to reopen. `--no-session` skips it.
  Specs: `specs/003-session-restore/`.
- **Recovery / crash reports** — `buffer/autosave.rs` writes `EDIT-RECOVERY-V1` snapshots (TOML
  envelope wrapping content + metadata) under `$XDG_STATE_HOME/edit/recovery/`. On startup, if a
  recovery file exists for an opened file, the user is prompted to restore or discard.
  `--no-autosave` disables it. Crash reports: `diagnostics/crash.rs` →
  `$XDG_STATE_HOME/edit/crash-<timestamp>.log`.

---

## Themes (`src/ui/theme.rs`)

`Theme` is a complete color scheme; each field maps to a logical UI role (editor background/
foreground, menu bar, status bar, syntax roles). Built-ins selectable via `--theme` or
Options → Theme:

| Name | Description |
|---|---|
| `classic` | DOS-faithful blue background, white text (default) |
| `high-contrast` | Black background, bright text |
| `plain` | Terminal default colors; no custom background |

Soft-wrap rendering (`src/ui/wrap.rs`) draws the `»` continuation marker and a `[WRAP]` status-bar
indicator; toggle via `Alt+Z` / View → "Soft Wrap (ext)" (a non-DOS extension). Check-state menu
items (e.g. `✓ Soft Wrap (ext)`) are driven by `toggle_states`. Specs: `specs/005-soft-wrap-mode/`,
`specs/006-menu-check-state-indicator/`.

---

## Configuration (`src/config/`)

`config/schema.rs` defines the full schema + defaults (serde + `toml`); `config/mod.rs` loads and
merges. Location: `$XDG_CONFIG_HOME/edit/config.toml` (default `~/.config/edit/config.toml`). CLI
flags override config values for the session. See `schema.rs` for the authoritative default set.

---

## Build flag matrix

Driven by `Cargo.toml` profiles and the `Makefile` (cargo wrappers).

| Profile | Command | Output | Notes |
|---|---|---|---|
| `debug` | `make` / `cargo build` | `target/debug/edit` | symbols, no opt |
| `release` | `make release` / `cargo build --release` | `target/release/edit` | LTO, `-O3`, stripped |
| `release-static` | `make static` | `target/x86_64-unknown-linux-musl/release-static/edit` | musl static, needs musl target + nightly |

Targets: `x86_64-unknown-linux-gnu` (primary), `aarch64-unknown-linux-gnu` (cross), and the musl
static build. MSRV stable 1.74.0 (required by `ratatui` 0.26 / `clap` 4); nightly only for the musl
`release-static` profile. Packaging: `make package-deb` (cargo-deb) and `make package-rpm`
(`packaging/edit.spec`). Full matrix: [`STATUS.md`](STATUS.md).

---

## Test & benchmark harness

| Suite | Command | What |
|---|---|---|
| Unit | `cargo test` (`make check`) | `#[cfg(test)]` modules in `src/` |
| Integration | `cargo test --test '*'` | `tests/integration/*.rs` (one `[[test]]` per file in `Cargo.toml`) |
| Smoke | `make smoke` | `expect` scripts in `tests/smoke/*.exp` (needs `expect` + `tmux`) |
| Stress | `make stress-test` | continuous-editing / encoding stress (slow, opt-in) |
| Benchmarks | `make perf-check` | criterion benches in `benches/` (`startup`, `large_file`, `keystroke`) |

Integration tests cover each feature: `encoding_roundtrip`, `encoding_select`, `file_io`,
`file_watch`, `menu_activation`, `plugin_api`, `recovery`, `session`, `soft_wrap`, `stress`. Smoke
scripts drive the real TUI (`basic_edit`, `menu_nav`, `unicode_display`, `search_replace`,
`plugin_highlighter`, `plugin_menu_activate`, …). **All integration/smoke tests launch with
`LC_ALL=C.UTF-8 LANG=C.UTF-8`.**

`make ci-local` runs the full gate in order: `cargo fmt --check` → `cargo clippy -D warnings` →
`cargo test` → `make smoke` → `make perf-check`.

---

## Spec Kit map

Each feature was built spec-first under `specs/NNN-*/` (spec → plan → tasks, with `contracts/` and
`checklists/`). Quick index:

| # | Feature | Spec dir |
|---|---|---|
| 001 | Linux EDIT.COM clone (core editing, UTF-8) | `specs/001-linux-editcom-clone/` |
| 002 | UTF-16 transcoding | `specs/002-utf16-transcoding/` |
| 003 | Session restore | `specs/003-session-restore/` |
| 004 | Save-As encoding UI | `specs/004-save-as-encoding-ui/` |
| 005 | Soft-wrap mode | `specs/005-soft-wrap-mode/` |
| 006 | Menu check-state indicator | `specs/006-menu-check-state-indicator/` |
| 007 | External file watch | `specs/007-external-file-watch/` |
| 008 | Plugin API (Rhai) | `specs/008-plugin-api/` |
| 009 | Menu-bar activation | `specs/009-menu-bar-activation/` |
