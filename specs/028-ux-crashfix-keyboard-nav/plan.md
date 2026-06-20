# Implementation Plan: UX crash-safety and keyboard navigation hardening

**Branch**: `028-ux-crashfix-keyboard-nav` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/028-ux-crashfix-keyboard-nav/spec.md`

## Summary

Fix a cluster of confirmed crash-safety and keyboard-UX defects in one pass. The headline crash is a
panic in the soft-wrap renderer (`src/ui/editor.rs`) when the active buffer's content changes (session
restore, buffer switch/open/close) without the wrap cache being invalidated — the renderer slices the
new buffer's lines using stale byte offsets. The fix is two-layer: make the renderer **never panic**
(clamp every runtime slice to the current line length) **and** invalidate the wrap cache on every
active-buffer change (the existing `wrap_text_gen` generation mechanism). Independently, the panic hook
must restore the terminal so a crash leaves the shell usable. The keyboard-UX fixes reset interactive
dialog focus to the input field on open (so Save-As typing works), add arrow-key movement between
dialog buttons, add keyboard scrolling + dismissal to Help/About, bind Home/End, and add PageUp/PageDown
to lists. All via existing helpers; no new crates.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74)

**Primary Dependencies**: `ratatui` + `crossterm` (TUI/terminal), `ropey` (rope buffer),
`unicode-segmentation` / `unicode-width` (grapheme + display width). No new dependencies (Principle IV).

**Storage**: N/A (session file already exists; no schema change).

**Testing**: `cargo test` — inline `#[cfg(test)]` unit tests in `src/`, integration tests under
`tests/integration/` registered in `Cargo.toml`.

**Target Platform**: Linux (x86_64, aarch64) TUI; headless VT100-compatible terminals.

**Project Type**: Single-project Rust desktop/terminal application.

**Performance Goals**: Editor render stays within the existing `make perf-check` budget; wrap-cache
rebuild on buffer switch is the same cost as the existing edit-driven rebuild.

**Constraints**: No panic on any terminal size, buffer count, or malformed/partial session; DOS-faithful
look-and-feel preserved; existing keys/mouse/behaviors unchanged.

**Scale/Scope**: ~8 focused fixes across `src/ui/editor.rs`, `src/app.rs`,
`src/diagnostics/crash.rs`, `src/ui/buttons.rs`, `src/ui/file_browser.rs`, `src/input/keymap.rs`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Platform-Native, DOS-Faithful UI**: PASS — keyboard navigation (arrows/PageUp-Down/Home/End)
  and a usable-after-crash terminal are *more* DOS-faithful, not less. No new UI surfaces; styling
  unchanged.
- **II. UTF-8 First**: PASS — the slice-clamping fix operates on UTF-8 byte offsets but must clamp to
  char/grapheme boundaries already used by the renderer; no raw-byte buffer construction. Display-width
  logic is unchanged.
- **III. Portable Build**: PASS — uses existing crossterm terminal calls already used on normal exit;
  no platform-specific additions beyond what already exists.
- **IV. Minimal Footprint / No New Crates**: PASS — reuses `ratatui`/`crossterm`/`ropey` and existing
  helpers; zero new dependencies.
- **V. Test-Gated Merges (NON-NEGOTIABLE)**: PASS — every fix gets a regression test written first
  (renderer no-panic with stale cache, wrap-cache invalidation, panic-hook terminal restore,
  interactive focus reset, arrow-key button movement, help keyboard scroll, Home/End, list paging).
- **VI. Simplicity / YAGNI**: PASS — no new abstractions; smallest change that removes each defect.
- **VII. Security Hardening**: PASS — strictly improves robustness (no panics, no data loss on
  restore); no network or plugin surface touched.

**Result**: No violations. Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/028-ux-crashfix-keyboard-nav/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (behavioral contracts)
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── app.rs                 # wrap-cache invalidation on buffer switch/open/close/restore;
│                          #   interactive-dialog focus reset on open; arrow-key button movement
│                          #   (016 + 020 rings); help keyboard scroll; list PageUp/Down; copy guard
├── ui/
│   ├── editor.rs          # clamp soft-wrap (and non-wrap) line slices — never panic
│   ├── buttons.rs         # next/prev focus helpers (reused for arrow keys)
│   └── file_browser.rs    # saturating_sub in scroll arithmetic; PageUp/Down paging
├── input/
│   └── keymap.rs          # bind Home→MoveLineStart, End→MoveLineEnd
└── diagnostics/
    └── crash.rs           # panic hook restores terminal before printing the report

tests/
└── integration/
    └── ux_hardening.rs     # new: cross-cutting regression tests (registered in Cargo.toml)
```

**Structure Decision**: Single-project Rust layout (existing). Changes are surgical edits to the files
above plus one new integration test file; unit tests are added inline next to each fix.

## Complexity Tracking

> No Constitution violations — section intentionally empty.
