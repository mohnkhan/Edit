# Roadmap

This page summarizes `ROADMAP.md`. Notably, every feature that was once deferred has now **shipped**
as of v0.3.0 (2026-06-19) — the roadmap is currently a record of completed work with pointers to the
issues that tracked each item. New work is added per the [Development](Development.md) deferral
process (GitHub issue + a `ROADMAP.md` row).

## Shipped (formerly deferred)

| Item | Issue | Shipped in | Notes |
|---|---|---|---|
| **Plugin API (Rhai)** | #2 | feature 008 | Sandboxed highlighter/keybinding/menu plugins; default-deny; one-time consent; `plugins.toml`; manager at Options › Plugins; `--no-plugins` |
| **Plugin top-level menu activation** | #19 (`follow-up`) | feature 009 | Live keyboard activation of plugin menus + the broader menu-interaction pass; plugin menus render between Options and Help |
| **External file modification detection** | #3 | feature 007 | inotify via `notify`; reload prompt; deletion notice; self-write suppression; debounce; `--no-watch` |
| **Soft-wrap mode** | #4 | feature 005 | `»` continuation marker; `Alt+Z` / View menu; `soft_wrap` config; `[WRAP]` indicator |
| **Menu checked-state indicator** | #13 | feature 006 | `✓` prefix on toggleable items; general `toggle_states` mechanism (follow-up of #4) |
| **UTF-16 transcoding** | #5 | feature 002 (v0.2.0) | Auto-detect LE/BE by BOM; full round-trip; surrogate pairs; `--encoding` aliases |
| **Save-As encoding selection UI** | #9 | feature 004 | F12 / File › Save As Encoding… modal listbox (follow-up of #5) |
| **Session restore** | #6 | feature 003 | `session.toml` on clean exit; TUI restore dialog; `--no-session`; explicit-file bypass |

## Open follow-ups

There are no open deferred features in `ROADMAP.md` at this time — all tracked items are complete.

## Known limitations (from `docs/STATUS.md`)

- **Mouse menu selection** is not yet wired; menu navigation is fully keyboard-driven (all menus are
  reachable via the keyboard). General mouse support also depends on a terminal that reports mouse
  events in crossterm's supported protocol.

For the authoritative, always-current list see `ROADMAP.md` and the project's GitHub issue tracker
(items use the `follow-up` label). New deferrals require both an issue and a `ROADMAP.md` row — see
[Development](Development.md).
