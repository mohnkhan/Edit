# Research: Linux EDIT.COM Clone

**Feature**: `specs/001-linux-editcom-clone`
**Phase**: 0 — Pre-design research
**Date**: 2026-06-18

---

## Decision 1: Implementation Language

**Decision**: Rust (stable toolchain, edition 2021, MSRV 1.74.0)

**Rationale**: The project constitution (Principle IV, updated v1.1.0) mandates Rust as
the preferred language for memory safety and performance. Rust's ownership model eliminates
buffer overflows and use-after-free — both relevant to a terminal app that reads arbitrary
file content. The Cargo ecosystem provides first-class crates for every required subsystem
(TUI, Unicode, encoding, regex, config). A `musl` static binary is achievable with no
changes to application code.

**Alternatives considered**:
- C: Would match the original EDIT.COM heritage, but requires manual memory management
  and increases security risk for file-content parsing. Rejected per constitution.
- Python: Disqualified by Principle IV (no Python runtime dependency).
- Go: Good TUI crates exist (`bubbletea`), but the constitution explicitly names Rust.

---

## Decision 2: TUI Rendering Library

**Decision**: `ratatui` 0.26+ with `crossterm` 0.27+ backend

**Rationale**: `ratatui` is the actively maintained fork of `tui-rs` with wide-character
support via `unicode-width`. It provides a retained-mode widget model (Canvas, Paragraph,
Block, Table) that maps cleanly to a menu-bar + editor-area + status-bar layout. `crossterm`
provides the backend: it handles raw mode, alternate screen, mouse events, and key input
on Linux, BSD, and macOS without any native C calls in the critical path.

**Alternatives considered**:
- Raw ncursesw via `-sys` FFI: Would satisfy the constitution's ncursesw mention, but
  requires unsafe Rust FFI blocks throughout the rendering path. Ratatui + crossterm
  achieves the same terminal compatibility in safe Rust. Constitution allows this: "or
  equivalent wide-character API."
- `cursive` (Rust TUI): Opinionated widget model that fights against the DOS-style
  custom layout; harder to reproduce pull-down menus precisely.
- Direct ANSI escape sequences: Fragile, requires reimplementing terminfo detection.

---

## Decision 3: Text Buffer Data Structure

**Decision**: `ropey` 0.6+ rope implementation

**Rationale**: A rope provides O(log n) insert and delete at arbitrary positions, which
is essential for the 100 MB file requirement (SC-004). A gap buffer is simpler but degrades
to O(n) for random-access edits. `ropey` is grapheme-cluster-aware and provides char-index
↔ byte-index ↔ line-index conversion, which is exactly what the cursor model needs.

The undo stack stores `(position, deleted_text, inserted_text)` tuples referencing the
rope's slice API — no full-buffer snapshots.

**Alternatives considered**:
- `Vec<u8>` with gap buffer: Simple, but O(n) worst-case inserts and no built-in line
  indexing. Acceptable for a <1 MB editor; not acceptable for 100 MB requirement.
- `xi-rope` (Xi editor rope): More complex API, less maintained. Ropey covers the use case.
- Line-array model (`Vec<String>`): Standard for small editors; catastrophic for large
  files and mid-line edits. Rejected.

---

## Decision 4: Configuration File Format

**Decision**: TOML (`config.toml` via `serde` + `toml` crates)

**Rationale**: The constitution allows YAML or INI; TOML is the Rust ecosystem standard
(Cargo itself uses TOML) and is more human-readable than YAML for the simple key-value
and nested-table structures in the editor config. The `serde` + `toml` crates are
production-grade. TOML parses with clear error messages, which is important for the
"log error and use default on invalid config" requirement (FR-020).

Config location: `$XDG_CONFIG_HOME/edit/config.toml`
(falls back to `~/.config/edit/config.toml` if `XDG_CONFIG_HOME` is unset)

**Alternatives considered**:
- INI via `configparser`: No type validation; all values are strings requiring manual
  coercion. Rejected for poor error reporting.
- YAML via `serde_yaml`: More complex spec, indentation-sensitive, larger crate. TOML
  is sufficient for the config schema needed.

---

## Decision 5: Legacy Encoding Transcoding

**Decision**: `encoding_rs` (ISO-8859-1, Windows-1252) + `oem-cp` (CP437, CP850)

**Rationale**: `encoding_rs` is the Mozilla-maintained implementation of the WHATWG
encoding standard. It covers ISO-8859-1 and Windows-1252 (which are in the WHATWG spec).
However, `encoding_rs` does NOT cover IBM OEM code pages (CP437, CP850) because they are
not part of the WHATWG standard. `oem-cp` provides a minimal, dependency-light lookup-table
implementation for OEM code pages. Together they cover all four legacy encodings required.

The transcoding pipeline on file open:
  raw bytes → detect encoding (BOM or heuristic or user override)
              → decode to `Vec<char>` in memory
              → store in rope as UTF-8

On file save with legacy encoding: rope UTF-8 → encode back → write bytes.

**Alternatives considered**:
- `iconv` via FFI: Available on all targets, but requires libiconv linkage (breaks static
  build on some platforms). Rejected as a primary approach; kept as a fallback suggestion
  in user documentation for unsupported encodings.
- `chardetng`: Mozilla's chardet library; useful for encoding detection heuristics, added
  as an optional dependency for detection (not transcoding).
- Manual lookup tables: Correct for CP437 but fragile to maintain. Delegated to `oem-cp`.

---

## Decision 6: Unicode Column Width Computation

**Decision**: `unicode-width` + `unicode-segmentation` crates

**Rationale**: `unicode-width` provides `UnicodeWidthChar::width()` and
`UnicodeWidthStr::width()` following UAX #11 (Unicode east-asian-width tables). This
correctly reports 2 for fullwidth/wide characters, 0 for combining characters, and 1
for everything else. `unicode-segmentation` splits strings at grapheme cluster boundaries
so the cursor advances one cluster at a time (correctly handling `e + combining acute` as
one cursor step). Both are already transitive dependencies of `ratatui`, so they add no
binary size overhead.

---

## Decision 7: Regular Expression Engine

**Decision**: `regex` crate (Rust standard)

**Rationale**: The `regex` crate provides linear-time, Unicode-aware matching with no
exponential backtracking. It supports the same syntax as PCRE for common patterns without
the catastrophic backtracking risk. It handles UTF-8 strings natively. Search wraps at
end of file using a two-pass scan (forward from cursor + forward from start if wrap enabled).

**Alternatives considered**:
- `fancy_regex`: Supports lookahead/lookbehind; adds backtracking risk. Unnecessary for
  an editor search feature. Rejected.
- `pcre2` via FFI: Breaks static musl build. Rejected.

---

## Decision 8: Syntax Highlighting Approach

**Decision**: Inline regex-based highlighter per language (no tree-sitter)

**Rationale**: A regex-based highlighter for 5 languages is ~200 lines of Rust with no
additional dependencies. Tree-sitter would produce more accurate highlighting for nested
constructs but adds a C FFI dependency (breaking musl static build) and significant binary
size. For the v1.x baseline of 5 languages, regex patterns for keywords, string literals,
and comments are sufficient and fast (< 1 ms per screen redraw for typical files).

Each language highlighter implements a `Highlighter` trait returning a `Vec<Span>` (byte
ranges + style attributes). The rendering layer merges these spans into ratatui cells.

---

## Decision 9: Auto-Save and Recovery File Location

**Decision**: `$XDG_RUNTIME_DIR/edit/<sha256-of-abs-path>.recovery`

**Rationale**: `XDG_RUNTIME_DIR` is a session-scoped tmpfs on systemd-based Linux distros,
which means recovery files are automatically cleaned on logout. If `XDG_RUNTIME_DIR` is
unset (non-systemd, BSD, macOS), fall back to `$TMPDIR/edit-recovery/` created with 0700
permissions. The filename is the SHA-256 of the absolute file path (hex encoded), which
creates a stable, collision-resistant association between buffer and recovery file without
a separate database.

A lock file (`<hash>.lock`) is created on open and deleted on clean exit. Its presence
at the next open triggers the recovery offer dialog.

---

## Decision 10: Packaging

**Decision**: `cargo-deb` for DEB, a hand-authored `.spec` file for RPM

**Rationale**: `cargo-deb` reads `Cargo.toml` metadata and generates a standards-compliant
`.deb` from the release binary + man page + shell completions. For RPM, a hand-authored
`packaging/edit.spec` gives full control over `%files`, `%post`, and `%preun` sections
(needed for man page registration). Both outputs are produced by `make package`.

**Man page**: Authored in groff (`man/edit.1`); installed to `/usr/share/man/man1/`.
