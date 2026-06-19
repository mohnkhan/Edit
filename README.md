# edit

> A faithful MS-DOS **EDIT.COM** for the modern Linux terminal — DOS look-and-feel, Unicode soul.

[![Build](https://img.shields.io/badge/build-passing-brightgreen)](docs/STATUS.md)
[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)](Cargo.toml)
[![MSRV](https://img.shields.io/badge/MSRV-1.74-blue?logo=rust)](docs/STATUS.md)
[![License](https://img.shields.io/badge/license-MPL--2.0-green)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.3.0-informational)](CHANGELOG.md)

`edit` recreates the unmistakable blue-screen experience of Microsoft's MS-DOS text editor —
pull-down menus, F-key bindings, and a status bar — as a single, fast, native Linux binary.
Unlike the original, it is **UTF-8/Unicode native from the ground up**, written in **Rust** with
[`ratatui`](https://ratatui.rs) and [`crossterm`](https://github.com/crossterm-rs/crossterm), and
extensible through a **sandboxed plugin API**.

```text
┌ File  Edit  Search  View  Options  Help ─────────────────────────────────────┐
│                                                                               │
│  Welcome to edit — the DOS editor, reborn for Linux.                          │
│                                                                               │
│  • Arrow keys to move, F10 for the menu bar, Alt+F to drop down File.         │
│  • Ctrl+S saves, Ctrl+F finds, Ctrl+Q quits.                                  │
│  • UTF-8 everywhere: 日本語, café, emoji 🚀 — all just work.                   │
│                                                                               │
│                                                                               │
├───────────────────────────────────────────────────────────────────────────┤
│ readme.txt    UTF-8    Ln 6, Col 12    [WRAP]                  F10=Menu  ◄┘   │
└───────────────────────────────────────────────────────────────────────────┘
```

---

## ✨ Why `edit`?

- **DOS-faithful TUI** — classic blue background, pull-down menu bar (File / Edit / Search / View /
  Options / Help), F-key bindings, and a live status bar. Full keyboard menu navigation
  (`F10`, `Alt+<letter>`, arrows, `Enter`, `Esc`).
- **Unicode native** — UTF-8 everywhere, grapheme-aware cursor movement and editing.
- **Legacy codepage transcoding** — read and write CP437, CP850, ISO-8859-1, Windows-1252, and
  UTF-16 LE/BE with BOM auto-detection and byte-identical round-trips.
- **Syntax highlighting** — C/C++, Python, Shell, YAML, and Markdown out of the box.
- **Soft-wrap mode** — optional visual line wrapping with a `»` continuation marker (`Alt+Z`).
- **Multi-buffer & split view** — open many files, cycle buffers, split the screen.
- **Session restore & crash recovery** — pick up where you left off; auto-save protects against
  crashes via the `EDIT-RECOVERY-V1` format.
- **External file-change watching** — get prompted to reload when another process rewrites the
  file under you; a notice if it's deleted.
- **Sandboxed plugin API** — extend the editor with **Rhai** scripts: custom syntax highlighters,
  keybindings, and top-level menus, all in a default-deny sandbox.
- **Single binary** — no X11/Wayland, no glibc lock-in; static musl builds available.

---

## 🧩 Part of MyOS

`edit` is developed as a **standalone terminal editor** that is also destined to ship as the
built-in text editor of **[MyOS](https://github.com/)** — a Linux-based operating system project
that this repository lives within (`/MyOS-2026/`). It is designed to stand on its own anywhere a
Linux terminal runs, while fitting cleanly into MyOS as a dependency-light, self-contained
component. You can use it today; MyOS will adopt it as a first-class part of its base userland.

---

## 📦 Installation

### Prerequisites

- **Rust 1.74+** (MSRV) and `cargo` — see [rustup.rs](https://rustup.rs)
- A terminal emulator with UTF-8 support

### Build from source

```bash
git clone <repo-url> edit && cd edit

make build      # debug binary   → target/debug/edit
make release    # optimized, LTO, stripped → target/release/edit
make static     # static musl binary (requires the musl target + nightly)
```

The static build produces a fully self-contained binary with no glibc dependency at
`target/x86_64-unknown-linux-musl/release-static/edit`.

### Packaging

```bash
make package-deb    # .deb via cargo-deb
make package-rpm    # .rpm via rpmbuild + packaging/edit.spec
```

### Supported targets

| Target | Toolchain | Profile |
|---|---|---|
| `x86_64-unknown-linux-gnu` | stable 1.74+ | debug, release |
| `aarch64-unknown-linux-gnu` | stable 1.74+ (cross) | debug, release |
| `x86_64-unknown-linux-musl` | nightly | `release-static` |

---

## 🚀 Usage

```bash
edit [OPTIONS] [FILE...]
```

Open one or more files, or launch with no arguments for a blank buffer (or a session-restore
prompt). A man page is installed at `man/edit.1`.

### Common options

| Flag | Description |
|---|---|
| `--encoding <ENC>` | Force file encoding (`utf-8`, `cp437`, `cp850`, `iso-8859-1`, `windows-1252`, `utf-16-le`, `utf-16-be`, `utf-16`) |
| `--legacy-cp437` | Enable CP437 → UTF-8 transcoding on open |
| `--theme <NAME>` | `classic` (default), `high-contrast`, or `plain` |
| `--line-numbers` | Show the line-number gutter |
| `--no-highlight` | Disable syntax highlighting |
| `--readonly` | Open all files read-only |
| `--no-session` | Skip the session-restore prompt |
| `--no-watch` | Disable external file-change watching |
| `--no-plugins` | Disable plugin loading for this session |
| `--no-autosave` | Disable auto-save / crash recovery |
| `--locale <LOC>` | Override locale detection (e.g. `C.UTF-8`) |
| `--version` / `--help` | Print version / help and exit |

See [`docs/CAPABILITIES.md`](docs/CAPABILITIES.md) for the complete flag reference.

### Quickstart

```bash
edit notes.md                       # open a file
edit --theme high-contrast log.txt  # accessible color scheme
edit --encoding cp437 README.DOC    # open a legacy DOS file
edit --line-numbers src/main.rs     # show line numbers
```

---

## ⌨️ Keybindings

A curated selection of the defaults — see [`docs/CAPABILITIES.md`](docs/CAPABILITIES.md) or
[`wiki/Keybindings.md`](wiki/Keybindings.md) for the full table.

| Category | Key | Action |
|---|---|---|
| **File** | `Ctrl+S` / `Ctrl+Shift+S` | Save / Save As |
| | `F12` | Save As with encoding selection |
| | `Ctrl+O` / `Ctrl+N` | Open / New |
| | `Ctrl+Q` | Quit (prompts if unsaved) |
| **Edit** | `Ctrl+Z` / `Ctrl+Y` | Undo / Redo |
| | `Ctrl+X` / `Ctrl+C` / `Ctrl+V` | Cut / Copy / Paste |
| | `Ctrl+A` | Select all |
| **Navigate** | `Ctrl+←` / `Ctrl+→` | Word left / right |
| | `Ctrl+Home` / `Ctrl+End` | Start / end of file |
| | `PgUp` / `PgDn` | Page up / down |
| **Search** | `Ctrl+F` / `Ctrl+H` | Find / Find & Replace |
| | `F3` / `Shift+F3` | Find next / previous |
| **Buffers** | `Ctrl+Tab` / `Ctrl+Shift+Tab` | Next / previous buffer |
| | `Ctrl+W` / `F6` | Close buffer / next split pane |
| **View** | `Alt+Z` | Toggle soft-wrap |
| **Menus** | `F10` | Activate menu bar |
| | `Alt+F/E/S/V/O/H` | Open File / Edit / Search / View / Options / Help |

---

## 🔌 Plugins

`edit` ships a **Rhai-based plugin API** ([feature 008](CHANGELOG.md)) so you can extend the editor
without touching its source. Plugins live in `$XDG_CONFIG_HOME/edit/plugins/<id>/` as a
`plugin.toml` manifest plus an optional `plugin.rhai` script, and can provide:

- **Syntax highlighters** — take precedence over the built-in highlighter for their file types.
- **Keybindings** — merge into the keymap (Save and Quit cannot be overridden).
- **Menu items** — contribute top-level menus rendered between *Options* and *Help*, activatable
  from the keyboard ([feature 009](CHANGELOG.md)).

Every newly installed plugin requires a **one-time consent dialog** before it runs, and executes in
a **default-deny sandbox**: no filesystem, process, or network access except a permission-gated
`read_file`. Each call is bounded to **50 ms**, and a misbehaving plugin is disabled for the session
so the editor stays responsive. Manage plugins via **Options › Plugins**, or disable all of them
with `--no-plugins`.

Reference plugins live in [`examples/plugins/`](examples/plugins/): `word-count`, `custom-keys`,
`lua-syntax`, plus the `fs-violation` and `infinite-loop` sandbox test fixtures. See
[`wiki/Plugin-Development.md`](wiki/Plugin-Development.md) to write your own.

---

## ⚙️ Configuration

| What | Location |
|---|---|
| Config file | `$XDG_CONFIG_HOME/edit/config.toml` (default `~/.config/edit/config.toml`) |
| Plugin consent | `$XDG_CONFIG_HOME/edit/plugins.toml` |
| Recovery files | `$XDG_STATE_HOME/edit/recovery/` |
| Logs | `$XDG_STATE_HOME/edit/logs/edit-<date>.log` |
| Crash reports | `$XDG_STATE_HOME/edit/crash-<timestamp>.log` |

**Themes:** `classic` (DOS blue, default), `high-contrast` (accessibility), and `plain` (terminal
default colors). Set via `--theme`, the Options menu, or `config.toml`. The full schema lives in
`src/config/schema.rs`.

---

## 🛠️ Development

`edit` is built with a [Spec Kit](specs/) driven workflow — each feature has a numbered directory
under [`specs/`](specs/) with its spec, plan, and tasks.

```bash
make check        # unit + integration tests (cargo test)
make smoke        # expect-based TUI smoke tests (needs expect + tmux)
make perf-check   # Criterion benchmarks
make stress-test  # 5-minute continuous-editing stress test
make ci-local     # full gate: fmt → clippy → test → smoke → bench
```

> 💾 **Save your SSD**: `make tmpfs-setup` redirects `target/` (the only large gitignored output
> tree) into `/tmp/edit/<hash>/` so Cargo's write-heavy build cycle hits RAM instead of the SSD.
> Reversible (`make tmpfs-teardown`), idempotent, opt-in, no-op on CI. See
> [`docs/dev-tmpfs.md`](docs/dev-tmpfs.md).

### Module layout (`src/`)

| Module | Responsibility |
|---|---|
| `buffer/` | Rope-backed text buffer, undo/redo, auto-save |
| `ui/` | `ratatui` widgets: menu bar, dialogs, status bar, soft-wrap |
| `encoding/` | Detection + transcoding (UTF-8/16, CP437/850, Latin-1, CP1252) |
| `highlight/` | Syntax highlighting pipeline |
| `plugin/` | Rhai engine, sandbox, manifest loading, consent |
| `watcher/` | External file-modification detection (inotify via `notify`) |
| `session/` | Session save/restore |
| `search/` | Find & replace with regex |
| `input/` | Keymap and key handling |
| `config/` | TOML config schema and persistence |
| `security/` | Path-traversal guards, sandbox helpers |
| `diagnostics/` | Logging, crash handler |

---

## 🗺️ Roadmap

Features 001–009 are complete (see [`CHANGELOG.md`](CHANGELOG.md) and [`docs/STATUS.md`](docs/STATUS.md)).
Planned and deferred work — along with the issues tracking it — lives in [`ROADMAP.md`](ROADMAP.md).

---

## 📚 Documentation

- **[Project Wiki](wiki/Home.md)** — start here
- [Installation](wiki/Installation.md)
- [Keybindings](wiki/Keybindings.md)
- [Plugin Development](wiki/Plugin-Development.md)
- [Architecture](wiki/Architecture.md)
- [Capabilities reference](docs/CAPABILITIES.md) · [Status](docs/STATUS.md) · [Changelog](CHANGELOG.md)

---

## 🤝 Contributing

Contributions are welcome! A few project conventions:

- Branch per change, named `NNN-short-description` (e.g. `010-line-numbers-gutter`), branched from
  `origin/master`.
- PRs target `master` and are merged via GitHub — never commit directly to `master`.
- Feature PRs must update `CHANGELOG.md` and `docs/STATUS.md` (and `docs/CAPABILITIES.md` for any
  user-visible change). Run `make ci-local` before opening a PR.

---

## 📄 License

Released under the **Mozilla Public License 2.0**, matching the parent
[MyOS](../MyOS2026/) project. See [`LICENSE`](LICENSE).
