# Project Status

**Project**: Linux EDIT.COM Clone (`edit`)
**Version**: 0.2.0 (dev — feature 002 in progress)
**Last updated**: 2026-06-18

## Implementation Status

| User Story | Description | Status |
|---|---|---|
| US1 | Basic File Editing (open, navigate, edit, save, quit) | Complete |
| US2 | UTF-8/Unicode support, CP437/CP850/ISO-8859-1/Windows-1252 transcoding | Complete |
| F002-US1 | UTF-16 LE/BE auto-detect (BOM sniffing) | Complete |
| F002-US2 | UTF-16 LE/BE decode/encode with full round-trip and surrogate-pair support | Complete |
| F002-US3 | `--encoding utf-16-le/be` CLI aliases via `encoding_from_str()` | Complete |
| F002-US4 | Save-As encoding selection UI (interactive dialog) | Deferred (#9) |
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

- Soft-wrap mode not implemented (deferred; see `ROADMAP.md`)
- External file modification detection (`inotify`) not implemented (deferred; see `ROADMAP.md`)
- Plugin API not implemented (deferred; see `ROADMAP.md`)
- Mouse support requires a terminal emulator that reports mouse events in crossterm's supported protocol
