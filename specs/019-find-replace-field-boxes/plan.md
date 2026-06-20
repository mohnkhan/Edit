# Implementation Plan: Bordered-box styling for Find/Replace fields

**Branch**: `019-find-replace-field-boxes` | **Date**: 2026-06-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/019-find-replace-field-boxes/spec.md`

## Summary

Restyle the Find and Replace dialog text fields so each editable field is drawn as a labeled,
bordered, single-line input box with a visible caret — matching the file-browser input box shipped
in feature 018 — instead of the current inline `Find:    text│` lines. All existing behavior (field
editing, `Tab` field switch, option toggles, match count, find/replace/replace-all, `Esc`) is
preserved. This is a render-layer change concentrated in `src/ui/mod.rs`, reusing the box-drawing
and the right-anchored caret/`truncate_to_width` horizontal-scroll approach already in
`src/ui/file_browser.rs`.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74)

**Primary Dependencies**: `ratatui` + `crossterm` (TUI rendering), `unicode-segmentation` (grapheme
iteration), `unicode-width` (display-column width). No new dependencies.

**Storage**: N/A (no persisted state; in-memory dialog state only).

**Testing**: `cargo test` — unit tests in `src/ui/dialog.rs` (field-editing state) plus new render
tests using `ratatui` `TestBackend` (mirroring the existing file-browser render tests).

**Target Platform**: Linux x86_64 / aarch64 terminal (VT100-compatible, including headless SSH).

**Project Type**: Single-project desktop TUI application.

**Performance Goals**: No measurable change — rendering one extra overlay per frame is negligible;
`make perf-check` baseline must hold.

**Constraints**: Must render without panic or corruption on the existing minimum terminal size
(graceful clamping of width and height). UTF-8 / wide-grapheme correct caret placement.

**Scale/Scope**: Two dialogs (Find, Replace), one rendering function (~80 lines in `src/ui/mod.rs`),
plus reuse of one helper from `src/ui/file_browser.rs`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Platform-Native, DOS-Faithful UI** — PASS. Bordered single-line input boxes are consistent
  with the DOS dialog aesthetic and with the file browser already shipped; no deviation from the
  established look. (Constitution names ncursesw/PDCurses; the project standardized on
  `ratatui`/`crossterm` per CLAUDE.md — this feature follows the established renderer, no new tech.)
- **II. UTF-8 First** — PASS. Caret placement uses grapheme segmentation and display-column width
  (`unicode-width`); no raw-byte buffer construction. Reuses existing UTF-8-correct helpers.
- **III. Portable Build** — PASS. No platform-specific code; pure `ratatui` render logic.
- **IV. Minimal Footprint** — PASS. No new dependencies.
- **V. Test-Gated Merges** — PASS. New render tests assert the bordered boxes + caret; existing
  field-editing and search tests continue to cover behavior. Tests written before/with implementation.
- **VI. Simplicity / YAGNI** — PASS. Reuses existing box/caret helpers; no new abstraction beyond a
  small shared input-box render helper. No focus-ring/buttons (deferred to issue #38).
- **VII. Security Hardening** — N/A (pure presentation change; no external input handling beyond the
  already-validated keystroke path).

**Result**: PASS — no violations, Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/019-find-replace-field-boxes/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── render-contract.md   # Visual/render contract for the dialogs
├── checklists/
│   └── requirements.md  # Spec quality checklist (from /speckit-specify)
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
src/ui/
├── mod.rs           # PRIMARY CHANGE: render_find_replace overlay (lines ~205-287) reworked to
│                    #   draw each field as a labeled, bordered 3-row input box with a caret.
├── file_browser.rs  # REUSE: truncate_to_width() helper + right-anchored caret pattern; possibly
│                    #   promote a shared input-box render helper.
└── dialog.rs        # FindReplaceDialog state (unchanged behavior; tests live here). The field
                     #   focus/caret/option fields already exist; no state changes expected.
```

**Structure Decision**: Single-project layout (existing). The change is localized to the UI render
layer. The dialog *state* model in `src/ui/dialog.rs` (`FindReplaceDialog`) is already sufficient and
will not change; only how `src/ui/mod.rs` paints it changes. Where the file-browser caret/scroll
logic is reused, factor a small private helper (e.g. `render_input_box`) to avoid duplication — keep
it in `src/ui/` (either a shared spot in `mod.rs` or reused from `file_browser.rs`).

## Complexity Tracking

No constitution violations; no entries required.
