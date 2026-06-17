<!--
SYNC IMPACT REPORT
==================
Version change: 1.0.0 → 1.1.0
Bump rationale: MINOR — new Principle VII (Security Hardening) added; material expansion of
  Platform & Encoding Standards (Rust language, large-file spec, XDG paths, distro targets,
  WSL); YAGNI principle corrected to align with Thoughts.MD baseline scope; performance
  baseline corrected (200ms → 2s startup, 100 MB large-file threshold added);
  Observability requirements added to Development Workflow section.

Modified principles:
  - III. Portable Build — Linux, BSD, macOS: added explicit distro targets, WSL terminals
  - IV. Minimal, Static-Linkable Footprint: updated preferred language C11 → Rust
  - VI. Simplicity and YAGNI: removed syntax highlighting + auto-save from deferred list
    (Thoughts.MD marks them High/Medium in-scope baseline features)

Added sections / principles:
  - Principle VII: Security Hardening (new — sourced from Thoughts.MD security section)
  - Platform & Encoding Standards: Rust toolchain, large-file spec (100 MB), XDG paths,
    distro targets (Ubuntu/Fedora/Arch/Debian/CentOS), DEB/RPM packaging
  - Development Workflow: Observability requirements (logging, --debug, crash reports)

Removed sections: none

Templates reviewed:
  - .specify/templates/plan-template.md    ✅ aligned — Constitution Check section present;
      update "Language/Version" guidance to mention Rust as preferred language
  - .specify/templates/spec-template.md    ✅ aligned — FR/SC structure compatible; no edits
  - .specify/templates/tasks-template.md   ✅ aligned — security and observability tasks now
      expected in Phase 2 Foundational per Principle VII
  - .specify/templates/commands/ (dir)     ✅ N/A — directory does not exist

Deferred TODOs: none
-->

# edit — Linux EDIT.COM Port Constitution

## Core Principles

### I. Platform-Native, DOS-Faithful UI

The editor MUST reproduce the MS-DOS EDIT.COM user experience: blue TUI background,
pull-down menu bar (File / Edit / Search / View / Options / Help), F-key bindings
(F1 Help, F3 Repeat Search, F5 Save, F6 Switch Window, F10 Menu), mouse support where
the terminal provides it, and a bottom status bar showing row/col, filename, encoding, and mode.

Rendering MUST use ncursesw (wide-character ncurses) or PDCurses — no X11, Wayland,
Electron, or browser runtime. The binary MUST run correctly on a headless SSH session
with a VT100-compatible terminal. The editor MUST degrade gracefully on terminals without
color support, using reverse-video for the menu bar.

**Rationale**: Faithfulness to the original UI is the core value proposition. Deviating
from EDIT.COM's look-and-feel without an explicit spec and accepted user story is prohibited.

### II. UTF-8 First, Always (NON-NEGOTIABLE)

All internal buffers, file I/O, clipboard data, and terminal rendering MUST be UTF-8.
Legacy byte encodings (CP437, CP850, ISO-8859-1, Windows-1252) MUST transcode to UTF-8
on read via built-in iconv wrappers; back-conversion to legacy encoding on save MUST
be explicit and user-confirmed.

The binary MUST call `setlocale(LC_ALL, "")` at startup, verify the resolved locale is
UTF-8 capable, and fall back to `LC_ALL=C.UTF-8` with a logged warning if not. If `LANG`
is not a UTF-8 locale, the editor MUST display a clear warning and suggest the fix.
CLI invocations in tests and CI MUST set `LC_ALL=C.UTF-8 LANG=C.UTF-8`.

The editor MUST correctly compute display column widths for combining characters, East
Asian wide characters, double-width characters, and emoji (per Unicode east-asian-width
tables) without misalignment.

Violations of this principle are blocking — no code path that widens raw bytes into the
editor buffer without UTF-8 validation may merge.

**Rationale**: The entire motivation for this project is a modern, Unicode-correct EDIT.COM
replacement. A UTF-8 regression is a project-identity failure, not a cosmetic bug.

### III. Portable Build — Linux, BSD, macOS

Every commit MUST build and pass tests on:
- Linux x86_64 and ARM64/aarch64 (glibc ≥ 2.17 or musl ≥ 1.2); canonical distros:
  Ubuntu, Fedora, Arch, Debian, CentOS
- FreeBSD ≥ 13 (x86_64)
- macOS ≥ 12 Monterey (x86_64 + Apple Silicon / arm64)

Supported terminals: xterm, xterm-256color, gnome-terminal, konsole, Alacritty, screen,
tmux, linux (console), and Windows Subsystem for Linux (WSL) terminals.

Platform-specific code MUST be isolated behind `#ifdef __linux__`, `#ifdef __FreeBSD__`,
or `#ifdef __APPLE__` guards. POSIX.1-2008 APIs are the baseline; glibc-only extensions
are prohibited in shared code paths.

The build system MUST be GNU Make + Cargo (for Rust components); a CMake wrapper for IDE
integration is optional. No platform-specific IDE project files are the canonical build.

**Rationale**: A text editor that only runs on one OS has limited value as a cross-platform
EDIT.COM replacement for BSD and macOS users who remember the original.

### IV. Minimal, Static-Linkable Footprint

The released binary MUST depend only on: libc, libncursesw (or PDCurses wide), and
optionally libiconv/libintl. No JVM, Python interpreter, scripting engine, D-Bus, or GUI
toolkit runtime.

A statically linked build (`make static`) MUST be achievable and produce a single
self-contained binary. Dynamic builds are the default for distribution packages. Distribution
packaging MUST provide DEB (Debian/Ubuntu) and RPM (Fedora/CentOS/Arch via alien) formats,
installing the binary to `/usr/bin/edit` and config schema to `$XDG_CONFIG_HOME/edit/`.

**Rationale**: EDIT.COM was a ~70 KB binary. The Linux port must be deployable on a
minimal container or embedded Linux system without installing a runtime ecosystem.

### V. Test-Gated Merges (NON-NEGOTIABLE)

Every user-visible behavior MUST have at least one automated test before the implementing
PR merges. Required test levels:
- **Unit**: buffer operations, encoding transcoding, key-binding dispatch, column-width
  computation for wide/combining characters
- **Integration**: file open/save/edit round-trips (including encoding conversion),
  auto-save and crash-recovery, large-file (≥ 10 MB) edit performance
- **Smoke/UI**: headless terminal capture via `expect` or tmux scripting for menu
  navigation, rendering regression, and keybinding behavior
- **Compatibility**: test matrix across declared terminal emulators (CI scripts)

Tests MUST be written before implementation (TDD). CI MUST enforce: no green tests → no
merge. This gate has no exceptions — not for "trivial" patches, not for doc-only changes
that touch a code path.

**Rationale**: Terminal UI regressions and encoding corner cases are invisible in diffs.
Only automated headless capture and real-file round-trips reliably catch them.

### VI. Simplicity and YAGNI

The v1.x baseline includes the following features — all others require a spec and user
story before implementation:

**In baseline (v1.x — no extra justification needed)**:
- DOS-faithful UI, F-key bindings, pull-down menus (Principle I)
- UTF-8 and legacy encoding transcoding (Principle II)
- Basic syntax highlighting (≥ 5 languages: C, Python, Shell, YAML, Markdown)
- Auto-save (30-second interval, crash recovery — FR-012)
- Multi-file via tabs or two-window split view (FR-011)
- Configurable keybindings and color themes via INI/YAML config at `$XDG_CONFIG_HOME/edit/`
- Search and replace with regex (FR-010)
- Mouse support in compatible terminals (FR-016)

**Deferred — require spec + accepted user story before work begins**:
- Plugin/extension API (FR-015, Low priority)
- Built-in version control beyond auto-save recovery
- Network/remote file access
- GUI beyond terminal emulators
- Additional syntax-highlighting languages beyond the baseline 5

No speculative abstractions may be added without a filed spec: no plugin framework, no
embedded scripting engine, no config format more complex than YAML/INI.

**Rationale**: Scope creep killed more editor projects than any technical challenge.
Explicitly naming the baseline prevents the opposite problem: under-scoping that ignores
what Thoughts.MD marks as High/Medium priority.

### VII. Security Hardening

The editor MUST NOT transmit user file contents to any external service without explicit
user consent and a visible confirmation dialog.

The following mitigations MUST be implemented and covered by tests:
- **Privilege**: never escalate privileges; respect file ownership and permissions on save.
- **Path traversal**: sanitize all file-dialog and relative-path inputs; reject `../`
  sequences that escape the working directory.
- **Escape injection**: sanitize all terminal control sequences read from file content
  or clipboard before rendering, to prevent terminal escape injection.
- **Plugin sandboxing**: if/when a plugin API is implemented, plugins MUST run in a
  restricted sandbox and MUST require explicit user consent before first execution.

Telemetry, if any, MUST be opt-in, documented, and disabled by default.

**Rationale**: An editor that opens arbitrary files on a shared server is a natural
attack surface. Escape injection and path-traversal are real CVE classes for terminal apps.

## Platform & Encoding Standards

**Supported OS targets**:
- Linux kernel ≥ 4.4 (glibc or musl); Ubuntu, Fedora, Arch, Debian, CentOS
- FreeBSD ≥ 13.0
- macOS ≥ 12.0 (Monterey)
- WSL (Windows Subsystem for Linux) on Windows 10/11

**Supported terminals**: VT100+, xterm, xterm-256color, gnome-terminal, konsole, Alacritty,
screen, tmux, linux (console), WSL default terminal. Graceful degradation to reverse-video
on no-color terminals.

**Preferred implementation language**: Rust (edition 2021, MSRV to be set per toolchain
support matrix). Low-level terminal control and encoding logic MUST be written in Rust for
memory safety and performance. C/C++ interop via FFI is permitted only for libncursesw
bindings and platform-specific terminal detection code where no safe Rust crate exists.

**Key toolchain**:
- Rust stable toolchain (Cargo); gcc/clang for C FFI layers only
- Build: GNU Make + Cargo; optional CMake wrapper for IDE integration
- ncursesw ≥ 6.1 or PDCurses ≥ 3.9 (wide-character build); bound via `-sys` crate
- Test runner: `cargo test` for unit/integration; `expect` + tmux for UI smoke tests

**Encoding runtime contract**:
- Source files: MUST be UTF-8 (`file -i <src>` MUST report `charset=utf-8`)
- Runtime locale: MUST be a UTF-8 locale; binary logs warning and falls back to `C.UTF-8`
- File I/O default: UTF-8 with BOM detection; BOM consumed, not stored internally
- Legacy encoding support: opt-in via `--encoding=<enc>` CLI flag or editor Options menu;
  built-in conversion for CP437, CP850, ISO-8859-1, Windows-1252 via iconv wrappers

**XDG base-directory compliance**:
- Config: `$XDG_CONFIG_HOME/edit/config` (YAML or INI; keys include `default_encoding`,
  `theme`, `keybindings`, `autosave_interval`, `recent_files_limit`, `plugins_enabled`)
- Logs: `$XDG_STATE_HOME/edit/logs` (configurable level; `--debug` enables verbose output)
- Recovery files: `$XDG_RUNTIME_DIR/edit/` or OS temp as fallback

**Performance baselines** (enforced by CI smoke + perf tests):
- Cold start to interactive: ≤ 2 seconds on typical modern hardware
- Open a 100 MB UTF-8 file: ≤ 3 seconds, UI remains responsive
- Cursor/keystroke latency: ≤ 50 ms in responsive terminals
- Memory: ≤ 50 MB for small files; scales linearly with file size for large files
- No memory leaks; editor MUST pass 72-hour continuous-editing stress test

**Packaging**:
- DEB package (Debian/Ubuntu) and RPM package (Fedora/CentOS) MUST be produced as
  release artifacts. Binary installed to `/usr/bin/edit`.

## Development Workflow & Quality Gates

**Branch model**: `NNN-short-description` branched from `origin/master`; PR targeting
`master`; merge only after CI passes and at least one review.

**CI gate** (`make ci-local` — MUST pass before every push):
1. `cargo fmt --check` — formatting diff must be empty
2. `cargo clippy -- -D warnings` — zero warnings
3. `cargo test` — all unit + integration tests green
4. `make smoke` — headless terminal smoke suite green
5. `make perf-check` — large-file open ≤ 3 s, startup ≤ 2 s
6. `make docs-gate` — CHANGELOG.md + docs/STATUS.md updated (bypassed with `[no-docs]`)

**Docs gate**: Every feature PR MUST update `CHANGELOG.md` and `docs/STATUS.md`. Update
`docs/CAPABILITIES.md` when a user-visible capability (menu item, CLI flag, file format,
keybinding) is added or removed.

**Deferrals**: Descoped stories MUST have a GitHub issue (label: `follow-up`) AND a
ROADMAP.md row before the PR merges.

**Observability requirements**:
- Logging: structured log output to `$XDG_STATE_HOME/edit/logs`; log level configurable
  (`error`, `warn`, `info`, `debug`). Default level: `warn`.
- Debug mode: `--debug` flag enables verbose logging and diagnostic terminal output.
- Crash reports: on SIGSEGV/panic, the editor MUST write a crash dump file to
  `$XDG_STATE_HOME/edit/crash-<timestamp>.log` for developer analysis.
- Startup locale: `--debug` MUST log the resolved locale and ncurses capabilities on init.

**Security gates**: Every PR touching file I/O, CLI parsing, or terminal rendering MUST
include a self-certification that Principle VII mitigations are preserved or updated.

## Governance

This constitution supersedes all other practices. Conflicts between this document and any
README, wiki page, or informal agreement resolve in favor of this constitution.

**Amendment procedure**:
1. Open a PR with the proposed constitution change and a version bump (see below).
2. PR description MUST explain: what changed, why, and what the migration impact is.
3. At least one maintainer review + CI green before merge.
4. After merge, sync `CLAUDE.md`/`CLAUDE.MD` if any principle or workflow rule changed.

**Versioning policy** (semantic):
- MAJOR: Principle removed, renamed, or governance redefined in a backward-incompatible way.
- MINOR: New principle or section added, or materially expanded guidance.
- PATCH: Clarifications, wording, typo fixes, non-semantic refinements.

**Compliance review**: Every PR author MUST self-certify against the Constitution Check in
`plan.md` before requesting review. Reviewers are expected to call out violations; a
violation of Principle II (UTF-8), Principle V (Test-Gated), or Principle VII (Security)
is always blocking.

**Version**: 1.1.0 | **Ratified**: 2026-06-18 | **Last Amended**: 2026-06-18
