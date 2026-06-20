# edit — the MS-DOS EDIT.COM editor, reborn for Linux

`edit` is a Linux reimplementation of Microsoft's MS-DOS **EDIT.COM** text editor. It faithfully
recreates the DOS look-and-feel — the blue background, the pull-down menu bar, the F-key bindings,
the status line — while being fully **UTF-8 / Unicode native** under the hood. It is written in
**Rust** using the [`ratatui`](https://ratatui.rs) + [`crossterm`](https://docs.rs/crossterm)
terminal stack.

**Current version: 0.4.0.**

## Where `edit` is headed: the MyOS text editor

`edit` is developed as a standalone editor today, but its destiny is to ship as the **built-in text
editor component of MyOS** — a Linux-based operating-system project (this repository lives under the
`MyOS-2026/` tree). Everything in `edit` is built with that future in mind: a single, minimally
dependent binary; no X11/Wayland requirement; UTF-8 correctness everywhere; and a small, auditable
sandboxed plugin surface. When MyOS ships, `edit` is the editor you'll reach for at the terminal.

## Feature overview

- **DOS-faithful TUI** — blue background, pull-down menus, F-key bindings, status bar; three themes
  (`classic`, `high-contrast`, `plain`).
- **Full keyboard menu navigation** — `F10` / `Alt+<letter>` to enter, arrows + `Enter` + `Esc`
  to drive every menu (Feature 009).
- **UTF-8 / Unicode native** with legacy code-page transcoding: CP437, CP850, ISO-8859-1,
  Windows-1252, plus UTF-16 LE/BE BOM auto-detection.
- **Grapheme-aware editing** — cursor movement and editing respect grapheme clusters.
- **Find & Replace** with regex support and live match highlighting.
- **Syntax highlighting** for C/C++, Python, Shell, YAML, and Markdown.
- **Multi-file editing** with split view and buffer cycling.
- **Auto-save & crash recovery** (`EDIT-RECOVERY-V1` format) plus session restore.
- **External-file watching** — prompts to reload when a file changes on disk.
- **Soft-wrap mode** (non-DOS extension) toggled with `Alt+Z`.
- **Rhai plugin system** — sandboxed syntax highlighters, keybindings, and menu items with a
  one-time consent model.

## Table of contents

| Page | What's inside |
|---|---|
| [Installation](Installation.md) | Prerequisites, building from source, packaging, supported targets |
| [Getting Started](Getting-Started.md) | First launch, the UI tour, basic editing workflow |
| [Keybindings](Keybindings.md) | The complete keyboard reference |
| [CLI Reference](CLI-Reference.md) | Every command-line flag |
| [Encodings](Encodings.md) | UTF-8, legacy code pages, UTF-16, line endings, the hygiene philosophy |
| [Configuration](Configuration.md) | `config.toml`, themes, recovery/session/log file locations |
| [Plugin Development](Plugin-Development.md) | The Rhai plugin API, manifest format, sandbox, worked example |
| [Architecture](Architecture.md) | High-level design and module breakdown |
| [Development](Development.md) | Contributor guide: workflow, branches, build & test targets |
| [Roadmap](Roadmap.md) | Shipped features and open follow-ups |
| [FAQ](FAQ.md) | Practical questions and answers |

## Quick start

```sh
# Build the release binary
make release

# Open a file
./target/release/edit notes.txt
```

See [Installation](Installation.md) and [Getting Started](Getting-Started.md) for the full path.
