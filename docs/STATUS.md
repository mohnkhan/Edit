# Project Status

**Project**: Linux EDIT.COM Clone (`edit`)
**Version**: 0.3.0 (features 008 + 009 complete; 010 + 011 + 012 unreleased)
**Last updated**: 2026-06-20

## Implementation Status

| User Story | Description | Status |
|---|---|---|
| F012-US1 | Open a file by browsing the directory tree (keyboard + mouse), no path typing required | Complete |
| F012-US2 | Save to a chosen folder/name by browsing; `Ctrl+S` on an unnamed buffer opens the Save browser | Complete |
| F012-US3 | Consistent keyboard & mouse navigation; long listings scroll; UTF-8-safe name display | Complete |
| F011-US1 | Mouse: click top-level menus and dropdown items; click-outside closes; editor click repositions cursor | Complete |
| F011-US2 | Edit/File/View menu actions wired (Undo/Redo/Cut/Copy/Paste/Select All/New/Save As/Toggle Line Nos) | Complete |
| F011-US3 | Help ▸ Help (key bindings) and Help ▸ About (version + copyright) screens | Complete |
| F010-US1 | File ▸ Open (and `Ctrl+O`) opens a modal path dialog that loads the file into a new buffer | Complete |
| F010-US2 | `Esc` bound to `MenuClose`: closes menus and cancels all modal dialogs (was a no-op) | Complete |
| F008-US1 | Syntax highlighter plugins (Rhai); plugin highlighter takes precedence over built-in | Complete |
| F008-US2 | Custom keybinding plugins; merged into keymap; Save/Quit non-overridable | Complete |
| F008-US3 | Menu item plugins; `menu_action` dispatched in sandbox; live menu-bar activation wired in feature 009 | Complete |
| F009-US1 | Keyboard navigation/activation of built-in pull-down menus (arrows/Enter/Esc; F10 + Alt+letter entry) | Complete |
| F009-US2 | Plugin-contributed top-level menus render (between Options and Help) and activate via keyboard | Complete |
| F009-US3 | DOS-faithful navigation semantics: wrap-around, Left/Right ring, modal precedence | Complete |
| F008-US4 | Plugin manager (Options > Plugins) + one-time consent dialog; decisions persisted | Complete |
| F008-US5 | Default-deny sandbox: 50 ms timeout, FS-violation denial, crash isolation | Complete |
| F007-US1 | Detect external file modification, prompt Y/N reload dialog | Complete |
| F007-US2 | Unsaved-changes warning shown in reload dialog when buffer is dirty | Complete |
| F007-US3 | File-deleted notice in status bar; buffer kept in memory | Complete |
| F007-US4 | `--no-watch` CLI flag / `no_watch` config option to disable watching | Complete |
| US1 | Basic File Editing (open, navigate, edit, save, quit) | Complete |
| US2 | UTF-8/Unicode support, CP437/CP850/ISO-8859-1/Windows-1252 transcoding | Complete |
| F002-US1 | UTF-16 LE/BE auto-detect (BOM sniffing) | Complete |
| F002-US2 | UTF-16 LE/BE decode/encode with full round-trip and surrogate-pair support | Complete |
| F002-US3 | `--encoding utf-16-le/be` CLI aliases via `encoding_from_str()` | Complete |
| F002-US4 | Save-As encoding selection UI (interactive dialog) | Complete (feat 004) |
| F006-US1 | View menu "Soft Wrap (ext)" shows `✓` prefix when soft-wrap is ON; no prefix when OFF | Complete |
| F006-US2 | Check-state mechanism general: any action/bool pair in `toggle_states` shows `✓` | Complete |
| F006-US3 | Check-state reflects config-persisted `soft_wrap=true` on first render (no toggle needed) | Complete |
| F005-US1 | Soft-wrap visual rendering with `»` continuation marker; Alt+Z / View menu | Complete |
| F005-US2 | Cursor, Home/End, mouse click work on logical lines in wrap mode | Complete |
| F005-US3 | Soft-wrap setting persisted to `config.toml` via atomic write | Complete |
| F005-US4 | `[WRAP]` status-bar indicator; "Soft Wrap (ext)" in View menu | Complete |
| F004-US1 | Save active buffer in chosen encoding via dialog (F12 / File menu) | Complete |
| F004-US2 | Cancel encoding dialog — file and encoding unchanged | Complete |
| F004-US3 | Selected encoding persists for subsequent Ctrl+S saves | Complete |
| F004-US4 | Unnamed buffer triggers filename prompt after encoding selection | Complete |
| F003-US1 | Session restore: write session on clean exit; TUI restore dialog on relaunch | Complete |
| F003-US2 | Handle missing/unreadable session files gracefully; status-bar warning | Complete |
| F003-US3 | `--no-session` CLI flag suppresses restore prompt | Complete |
| F003-US4 | Explicit file arguments bypass session restore | Complete |
| US3 | DOS-style pull-down menu bar, keyboard and mouse navigation | Complete |
| US4 | Find and Replace with regex support and match highlighting | Complete |
| US5 | Auto-save and crash recovery (EDIT-RECOVERY-V1 format) | Complete |
| US6 | Multi-file editing with split-view and buffer cycling | Complete |
| US7 | Syntax highlighting for C, Python, Shell, YAML, Markdown | Complete |
| US8 | Configurable themes: classic (DOS blue), high-contrast, plain | Complete |

## Feature Summary

- Grapheme-aware cursor movement and text editing
- Undo/redo with composite operation support
- XDG-compliant config, log, and state directories
- Crash handler with panic hook and SIGSEGV recovery via `signal-hook`
- Man page at `man/edit.1`
- RPM and Debian packaging configs
- Static musl binary support (`make static`)

## CI Matrix

### Target Platforms

| Target | Toolchain | Profile | Notes |
|---|---|---|---|
| `x86_64-unknown-linux-gnu` | stable 1.74.0+ | debug, release | Primary development target |
| `aarch64-unknown-linux-gnu` | stable 1.74.0+ | debug, release | Cross-compiled via cross |
| `x86_64-unknown-linux-musl` | nightly | release-static | Static binary, no glibc dependency |

### Rust Toolchain

- **Minimum supported**: stable 1.74.0 (required for `ratatui` 0.26 and `clap` 4)
- **Nightly**: used only for the `release-static` musl profile; not required for development
- **Edition**: 2021

### Test Suite

| Suite | Command | Description |
|---|---|---|
| Unit tests | `cargo test` | All `#[cfg(test)]` modules in `src/` |
| Integration tests | `cargo test --test '*'` | Files under `tests/integration/` |
| Smoke tests | `make smoke` | `expect`-based scripts in `tests/smoke/` (requires `expect` + `tmux`) |
| Stress tests | `cargo test --test stress -- --ignored` | Continuous-editing and encoding stress tests (slow, opt-in) |
| Benchmarks | `make perf-check` | Criterion benchmarks in `benches/` |

### Build Profiles

| Profile | Command | Output | Notes |
|---|---|---|---|
| `debug` | `make build` / `cargo build` | `target/debug/edit` | Debug symbols, no optimizations |
| `release` | `make release` / `cargo build --release` | `target/release/edit` | LTO, stripped, `-O3` |
| `release-static` | `make static` | `target/x86_64-unknown-linux-musl/release-static/edit` | musl, static linkage, requires musl target + nightly |

### CI Gate (`make ci-local`)

Runs in order:
1. `cargo fmt --check` — formatting
2. `cargo clippy -- -D warnings` — lints
3. `cargo test` — unit + integration tests
4. `make smoke` — expect smoke tests
5. `make perf-check` — benchmarks (non-regressing, results logged)

## Known Limitations
- Menu activation is keyboard-driven; mouse-click menu selection is not yet wired (keyboard
  navigation covers all menus). General mouse support requires a terminal emulator that reports
  mouse events in crossterm's supported protocol.
