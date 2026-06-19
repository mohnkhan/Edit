# Project Status

**Project**: Linux EDIT.COM Clone (`edit`)
**Version**: 0.2.0 (dev — feature 007 complete)
**Last updated**: 2026-06-19

## Implementation Status

| User Story | Description | Status |
|---|---|---|
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
- Plugin API not implemented (deferred; see `ROADMAP.md`)
- Mouse support requires a terminal emulator that reports mouse events in crossterm's supported protocol
