# Implementation Plan: Linux EDIT.COM Clone

**Branch**: `001-linux-editcom-clone` | **Date**: 2026-06-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-linux-editcom-clone/spec.md`

## Summary

Build a full-screen terminal text editor for Linux, BSD, and macOS that faithfully
reproduces the MS-DOS EDIT.COM user experience (blue background, pull-down menus, F-key
bindings) while adding first-class UTF-8/Unicode support, syntax highlighting for five
languages, auto-save crash recovery, and multi-file editing. Implemented in Rust using
`ratatui` + `crossterm` for rendering, a rope data structure for the text buffer,
`encoding_rs` + `oem-cp` for legacy encoding transcoding, and a TOML configuration file
stored under XDG base directories. See `research.md` for full decision rationale.

## Technical Context

**Language/Version**: Rust stable, edition 2021; MSRV 1.74.0

**Primary Dependencies**:
- `ratatui` 0.26+ — TUI widget framework (wide-char-aware rendering)
- `crossterm` 0.27+ — cross-platform terminal input/output, mouse events
- `unicode-width` 0.1+ — East Asian width / double-width column computation
- `unicode-segmentation` 1.11+ — grapheme cluster boundary splitting
- `encoding_rs` 0.8+ — ISO-8859-1, Windows-1252 transcoding
- `oem-cp` 0.8+ — CP437, CP850 OEM code page transcoding
- `ropey` 0.6+ — rope data structure for O(log n) insert/delete on large files
- `regex` 1.x — incremental search and replace engine
- `serde` + `toml` 0.8 — TOML configuration file parsing
- `clap` 4.x — CLI argument parsing with `--help` / `--version` auto-generation
- `dirs` 5.x — XDG base directory resolution
- `log` + `env_logger` — structured logging; level controlled by env var
- `signal-hook` — SIGTERM/SIGINT graceful shutdown + crash dump on SIGSEGV

**Storage**:
- Text buffers: in-memory rope; flushed to original file path on save
- Recovery files: `$XDG_RUNTIME_DIR/edit/<hash-of-path>.recovery` (temp on fallback)
- Config: `$XDG_CONFIG_HOME/edit/config.toml`
- Logs: `$XDG_STATE_HOME/edit/logs/edit-<date>.log`
- Crash reports: `$XDG_STATE_HOME/edit/crash-<timestamp>.log`

**Testing**:
- Unit: `cargo test` — buffer, encoding, search, security, config modules
- Integration: `cargo test --test '*'` — file I/O round-trips, recovery, encoding
- Smoke/UI: `expect` scripts in `tests/smoke/` — menu navigation, unicode rendering,
  keybinding dispatch; run with `make smoke`
- Performance: `cargo bench` (criterion) — startup time, 100 MB open, keystroke latency
- Compatibility: CI matrix across Linux x86_64, Linux ARM64, macOS, FreeBSD

**Target Platform**: Linux ≥ 4.4 (glibc or musl), FreeBSD ≥ 13, macOS ≥ 12;
x86_64 and ARM64/aarch64

**Project Type**: CLI terminal application (single binary, `edit [file...]`)

**Performance Goals**:
- Cold start to interactive: ≤ 2 s
- 100 MB UTF-8 file open: ≤ 3 s
- Keystroke-to-screen latency: ≤ 50 ms
- Session memory for 1 MB file: ≤ 50 MB

**Constraints**:
- No X11, Wayland, JVM, Python, D-Bus, or GUI toolkit
- Static build must be achievable: `cargo build --target x86_64-unknown-linux-musl`
- All source files UTF-8; no raw byte widening without validation
- No privilege escalation; respect file permissions on every save

**Scale/Scope**: Single-user interactive editor; no server/daemon component

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Gate | Status | Notes |
|-----------|------|--------|-------|
| I. DOS-Faithful UI | ratatui reproduces blue background, pull-down menus, F-keys, status bar | ✅ PASS | crossterm handles VT100/xterm/tmux; reverse-video fallback on no-color |
| II. UTF-8 First | unicode-width + unicode-segmentation; encoding_rs + oem-cp; setlocale fallback | ✅ PASS | All raw bytes validated before entering rope buffer |
| III. Portable Build | crossterm + ratatui are pure Rust (no OS-specific C calls in shared paths) | ✅ PASS | `#[cfg(target_os)]` guards only for signal handling |
| IV. Minimal Footprint | No runtime beyond libc; musl static target; binary ~8–15 MB estimated | ✅ PASS | DEB via `cargo-deb`; RPM via hand-authored `.spec` (research.md §10) |
| V. Test-Gated | Unit + integration via cargo test; UI smoke via expect; perf via criterion | ✅ PASS | All 29 FRs (FR-001–FR-029, incl. FR-007a) mapped to ≥ 1 test; per-phase TDD gates |
| VI. YAGNI | No plugin API, no scripting; only baseline features per spec | ✅ PASS | Plugin API filed as follow-up in ROADMAP.md |
| VII. Security | sanitize module (escape injection); path validation (traversal); no privilege escalation | ✅ PASS | signal-hook for SIGSEGV crash dumps |

**No violations.** No complexity justification required.

## Project Structure

### Documentation (this feature)

```text
specs/001-linux-editcom-clone/
├── plan.md              # This file
├── research.md          # Phase 0 decisions
├── data-model.md        # Phase 1 entity model
├── quickstart.md        # Phase 1 validation guide
├── contracts/
│   ├── cli.md           # CLI flags and arguments contract
│   ├── config.md        # TOML config file schema
│   └── recovery.md      # Recovery file format
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── main.rs                  # Entry point: CLI parse → bootstrap → event loop
├── app.rs                   # Application state machine, top-level event dispatch
├── buffer/
│   ├── mod.rs               # Buffer type: rope + metadata
│   ├── rope.rs              # Thin wrapper around ropey; grapheme-aware ops
│   ├── undo.rs              # Undo/redo command stack
│   └── autosave.rs          # 30-second timer, recovery file write/delete
├── ui/
│   ├── mod.rs               # Rendering coordinator: frame → widgets
│   ├── editor.rs            # Main editing area (scroll, cursor, selection)
│   ├── menubar.rs           # Pull-down menu state and rendering
│   ├── statusbar.rs         # Bottom status bar (row, col, encoding, mode)
│   ├── dialog.rs            # Modal dialogs: save-prompt, find, replace, open
│   └── theme.rs             # Color palettes: classic-dos, high-contrast, plain
├── input/
│   ├── mod.rs               # Event → Action dispatch
│   ├── keymap.rs            # Default EDIT.COM bindings + user override layer
│   └── mouse.rs             # Click-to-cursor, menu click handling
├── encoding/
│   ├── mod.rs               # EncodingProfile registry + detect/transcode API
│   ├── detect.rs            # BOM detection, heuristic (chardetng fallback)
│   └── transcode.rs         # encoding_rs (ISO-8859-1, Win-1252) + oem-cp (CP437, CP850)
├── search/
│   ├── mod.rs               # SearchState: query, regex, case, direction
│   └── highlight.rs         # Match span collection for rendering
├── highlight/
│   ├── mod.rs               # Highlighter trait + file-type detection by extension
│   └── languages/
│       ├── c.rs             # C keyword/string/comment patterns
│       ├── python.rs
│       ├── shell.rs
│       ├── yaml.rs
│       └── markdown.rs
├── config/
│   ├── mod.rs               # Config load → validate → merge with CLI flags
│   └── schema.rs            # Serde structs for config.toml
├── security/
│   └── sanitize.rs          # Escape sequence stripping, path traversal guard
└── diagnostics/
    ├── logging.rs           # env_logger init; XDG log path setup
    └── crash.rs             # Panic hook + SIGSEGV handler → crash report

tests/
├── unit/                    # cargo test (inline or in files)
├── integration/
│   ├── file_io.rs           # Open / save / encoding round-trips
│   ├── recovery.rs          # Auto-save and crash recovery flow
│   └── security.rs          # Escape injection, path traversal rejection
└── smoke/
    ├── basic_edit.exp       # expect: open, type, save, quit
    ├── unicode_display.exp  # expect: Japanese/emoji display alignment
    ├── menu_nav.exp         # expect: Alt+F, arrow nav, Escape
    └── search_replace.exp   # expect: Ctrl+F, F3, replace-all

benches/
├── startup.rs               # Cold-start benchmark
├── large_file.rs            # 100 MB open + scroll benchmark
└── keystroke.rs             # Keystroke latency benchmark

docs/
├── ai-context-reference.md  # Per-subsystem deep reference
├── STATUS.md
└── CAPABILITIES.md

man/
└── edit.1                   # Man page source (groff)

packaging/
├── edit.deb/                # cargo-deb metadata
└── edit.spec                # RPM spec file
```

**Structure Decision**: Single Rust project at repository root. Modular crate layout with
clear subsystem boundaries (buffer / ui / input / encoding / search / highlight / config /
security / diagnostics) to satisfy Principle VII (Security Hardening) isolation requirement
and support independent unit testing of each subsystem.

## Complexity Tracking

*No constitution violations — this section is empty.*
