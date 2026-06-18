# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased] — feature 003: Session Restore

### Added

- `src/session/mod.rs` — new module: `BufferEntry`, `SplitLayoutKind`, `SessionData` types with
  serde round-trip support; `session_path()`, `save_session()`, `load_session()` functions
- Session file written atomically (`.session.toml.tmp` → rename) to
  `$XDG_STATE_HOME/edit/session.toml` on every clean exit (FR-001, FR-002)
- Session restore dialog: a TUI overlay rendered at startup when a valid session file exists and
  no explicit file arguments or `--no-session` flag were supplied (FR-003, FR-007)
- `Y`/`y`/`Enter` confirms restore; `N`/`n`/`Escape`/`Ctrl+Q` declines (FR-003, FR-007)
- Missing or unreadable files are silently skipped during restore with a status-bar warning;
  the editor falls back to a blank buffer when all files fail (FR-004, FR-005, FR-006)
- Corrupt or invalid session files are treated as absent and overwritten on next clean exit using
  the same atomic sequence; a status-bar warning is shown on startup (FR-010)
- Path traversal guard via `security::sanitize::validate_path` on every path loaded from the
  session file (FR-005, Constitution Principle VII)
- `--no-session` CLI flag suppresses the restore prompt entirely; editor opens a blank buffer
  regardless of session file state (FR-008)
- Explicit `FILE` arguments on the CLI bypass session restore completely (FR-009)
- `active_idx` is clamped when the active buffer was among the skipped/missing files to prevent
  out-of-bounds panics (remediation I1)
- Orphaned `.session.toml.tmp` files from a previous crash are silently removed at startup
- 6 unit tests in `src/session/mod.rs` (`#[cfg(test)]` block)
- 8 integration tests in `tests/integration/session.rs` registered as `[[test]] name = "session"`
- `no_session: bool` field added to `Config` (runtime-only, `#[serde(skip)]`)
- `pending_session_restore`, `default_encoding` fields added to `App` struct
- `App::new` signature extended with `session: Option<SessionData>` and
  `session_warning: Option<String>` parameters

### Changed

- `App::new` now accepts two additional arguments; callers (`src/main.rs`) pass the
  session data resolved at startup

---


## [0.1.0] - 2026-06-18

### Added

- DOS-faithful blue background UI with pull-down menus (US1)
- Full UTF-8/Unicode support with CP437/CP850/ISO-8859-1/Windows-1252 transcoding (US2)
- DOS-style pull-down menu bar with keyboard and mouse navigation (US3)
- Find and Replace with regex support and match highlighting (US4)
- Auto-save and crash recovery with EDIT-RECOVERY-V1 format (US5)
- Multi-file editing with split-view and buffer cycling (US6)
- Syntax highlighting for C, Python, Shell, YAML, Markdown (US7)
- Configurable themes: classic (DOS blue), high-contrast, plain (US8)
- Grapheme-aware cursor movement and text editing
- Undo/redo with composite operation support
- XDG-compliant config, log, and state directories
- Crash handler with panic hook and SIGSEGV recovery
- Man page (`man/edit.1`)
- RPM and Debian packaging configs
- Static musl binary build profile (`make static`, `profile.release-static`)
- Criterion benchmark suite (`benches/startup.rs`, `benches/large_file.rs`, `benches/keystroke.rs`)
- Stress test suite (`tests/integration/stress.rs`, opt-in with `--ignored`)
