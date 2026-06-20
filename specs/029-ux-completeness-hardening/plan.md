# Implementation Plan: UX completeness hardening (round 2)

**Branch**: `029-ux-completeness-hardening` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/029-ux-completeness-hardening/spec.md`

## Summary

Fix a verified batch of crash, correctness, consistency, and silent-failure defects found in a full UX
audit, in one feature. Highest priority: never-panic on real content (multibyte delete/cut, Unicode
recovery path, byte→char, oversized file), and never silently lose a save. Then dialog/encoding/click
correctness, a single unified display-width function, and consistent action feedback. Larger parity
enhancements (in-dialog mouse editing, double/triple-click, context menu, extra F-keys) are out of scope
and tracked as issues + ROADMAP rows.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74)

**Primary Dependencies**: `ratatui` + `crossterm`, `ropey`, `unicode-width`, `unicode-segmentation`,
`encoding_rs`/`oem_cp`, `arboard`. No new dependencies (Principle IV).

**Storage**: N/A (no schema change).

**Testing**: `cargo test` — inline unit tests + integration tests under `tests/integration/`.

**Target Platform**: Linux TUI (x86_64/aarch64), headless VT100-compatible.

**Project Type**: Single-project Rust terminal application.

**Performance Goals**: Within the existing `make perf-check` budget; the unified width function must be
no slower than the current per-grapheme width calls (it replaces them).

**Constraints**: No panic on any content/size; DOS-faithful look preserved; existing keys/mouse/behaviors
unchanged except where correcting a defect; no silent data loss (Principle VII).

**Scale/Scope**: ~17 fixes across `src/ui/dialog.rs`, `src/app.rs`, `src/buffer/{rope,mod}.rs`,
`src/ui/{file_browser,tabbar,mod,theme}.rs`, `src/ui/width.rs` (new shared helper), `src/input/keymap.rs`,
`src/ui/menubar.rs`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. DOS-Faithful UI**: PASS — fixes (legible selected menu item, reachable Close, correct clicks) make
  the UI more faithful; no new surfaces.
- **II. UTF-8 First (NON-NEGOTIABLE)**: PASS — directly strengthens UTF-8 correctness (char-safe slices,
  unified display width for combining/wide/emoji). No raw-byte buffer construction added.
- **III. Portable Build**: PASS — no platform-specific additions.
- **IV. No New Crates**: PASS — reuses `unicode-width`/`unicode-segmentation`/existing deps.
- **V. Test-Gated (NON-NEGOTIABLE)**: PASS — each defect gets a failing-first regression test.
- **VI. Simplicity/YAGNI**: PASS — consolidates two divergent width helpers into one; smallest change per
  defect; larger enhancements deferred.
- **VII. Security Hardening / no silent data loss**: PASS — the save-failure and autosave-failure
  surfacing and the file-size guard directly serve this principle.

**Result**: No violations. Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/029-ux-completeness-hardening/
├── plan.md · research.md · data-model.md · quickstart.md
├── contracts/behavior.md
└── tasks.md  (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── ui/
│   ├── width.rs        # NEW: single display_width(grapheme)/str_width(&str) helper (unicode-width based)
│   ├── dialog.rs       # recovery-path char-safe truncation; dialog sizing by display width
│   ├── file_browser.rs # use shared width; remove local grapheme_width
│   ├── tabbar.rs       # use shared width
│   ├── mod.rs          # field caret/truncation use shared width
│   ├── menubar.rs      # File ▸ Close menu item
│   └── theme.rs        # legible selected menu item in the light theme
├── app.rs              # delete_selection char-safe; Ctrl+S feedback+failure; SavePrompt Esc;
│                       #   browser Save-As encoding; click gutter+hscroll mapping; copy/cut/paste &
│                       #   read-only & file-open feedback; autosave-failure notice; GoToLine modal guard;
│                       #   editor click uses shared width
├── buffer/
│   ├── rope.rs         # byte_to_char char-boundary-safe
│   └── mod.rs          # file-size guard on open (clear error)
└── input/keymap.rs     # bind Ctrl+W → Close

tests/
└── integration/
    └── ux_round2.rs    # NEW: cross-cutting regression tests (registered in Cargo.toml)
```

**Structure Decision**: Single-project Rust. The one structural addition is `src/ui/width.rs` — a tiny
shared module so display-width has exactly one implementation (resolves the divergent-helpers defect).

## Complexity Tracking

> No Constitution violations — section intentionally empty.
