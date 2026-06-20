# Implementation Plan: Caret-on-click in dialog text fields

**Branch**: `031-dialog-field-caret-click` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/031-dialog-field-caret-click/spec.md`

## Summary

Close #58: click inside a dialog text field to position the caret. Add one shared, pure helper
`field_caret_at(value, field_w, click_offset)` (reverses the renderer's visible-window logic via
`ui::width`), then wire it to the three field families. Find/Replace already has a caret model (just set
`d.caret`); the file-browser Name field and the Go-to-Line input are append-only today, so this feature
also gives them a caret model (Left/Right/Home/End, insert/delete at caret, mid-string render) so
click-to-position is meaningful. No new dependencies.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74)

**Primary Dependencies**: `ratatui` + `crossterm`, `unicode-width`, `unicode-segmentation`. No new
dependencies (Principle IV).

**Storage**: N/A.

**Testing**: `cargo test` — inline unit tests + integration tests under `tests/integration/`.

**Target Platform**: Linux TUI; headless VT100-compatible.

**Project Type**: Single-project Rust terminal application.

**Performance Goals**: Within `make perf-check` (input-path only).

**Constraints**: DOS-faithful look; drawn==clickable geometry; no regression; no panic on any value/width.

**Scale/Scope**: `src/ui/width.rs` (the helper), `src/ui/mod.rs` (Find/Replace field rects + click,
Go-to-Line render), `src/app.rs` (Find/Replace + Go-to-Line click + caret keys; Go-to-Line caret state),
`src/ui/file_browser.rs` (filename caret model + field rect + render), `src/ui/dialog.rs`
(FindReplaceDialog already has `caret`).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. DOS-Faithful UI**: PASS — click-to-position and arrow caret movement are standard input-field
  behavior; rendering unchanged except the caret can sit mid-string.
- **II. UTF-8 First**: PASS — caret indices are grapheme indices; click mapping uses `ui::width`
  (combining=0, wide=2); no byte slicing of values.
- **III. Portable Build**: PASS.
- **IV. No New Crates**: PASS.
- **V. Test-Gated (NON-NEGOTIABLE)**: PASS — `field_caret_at` is unit-tested in isolation; each field's
  caret movement/insert/delete/click is tested.
- **VI. Simplicity/YAGNI**: PASS — one shared helper; the two append-only inputs gain the minimal caret
  model needed; no new widgets.
- **VII. Security Hardening**: PASS — input handling only; no network/plugin/file-format surface.

**Result**: No violations. Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/031-dialog-field-caret-click/
├── plan.md · research.md · data-model.md · quickstart.md
├── contracts/behavior.md
└── tasks.md  (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── ui/
│   ├── width.rs          # NEW: field_caret_at(value, field_w, click_offset) -> grapheme index
│   ├── mod.rs            # US1: find_replace_field_rects() (text rects, shared with render_find_field);
│   │                     #   US3: Go-to-Line render uses a mid-string caret
│   ├── dialog.rs         # FindReplaceDialog already has `caret` (no model change)
│   └── file_browser.rs   # US2: add `caret` to the filename input — insert/delete at caret,
│                         #   move_left/right/home/end, field text rect, mid-string render
└── app.rs                # US1 click → set d.caret; US3 Go-to-Line caret state + keys + click;
                          #   route field clicks in handle_mouse_event (after buttons, before fall-through)
tests/
└── integration/
    └── field_caret.rs    # NEW: end-to-end click→caret per field (registered in Cargo.toml)
```

**Structure Decision**: Single-project Rust. One tiny new pure function in `ui::width`; the rest extends
existing field code. Geometry helpers are shared with each renderer so drawn==clickable.

## Field geometry (drawn == clickable)

- **Find/Replace** (`render_find_field`): text at `(dx+2, text_row)`, width `dw-4`; query `text_row = dy+3`,
  replacement `text_row = dy+7` (label + 3-row box per field). A `find_replace_field_rects` helper returns
  these so the click handler and render agree.
- **File browser Name** (`compute_layout`): text at `(field_box.x+1, field_box.y+1)`, width
  `field_box.width-2`. Expose the field text rect from the layout.
- **Go-to-Line**: body `"Go to line: {entry}"` in a bordered box; the value starts at `dx + 1 + len("Go to line: ")`
  on row `dy+1`, width `dw - 2 - 12`.

## Phasing & independence

US1 (Find/Replace) is the MVP — the field already edits mid-string. US2 (file browser) and US3
(Go-to-Line) each add a caret model and are independent of US1 and each other. Foundational: the shared
`field_caret_at` helper (used by all three).

## Complexity Tracking

> No Constitution violations — section intentionally empty.
