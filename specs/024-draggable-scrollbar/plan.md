# Implementation Plan: Interactive (clickable + draggable) scrollbars

**Branch**: `024-draggable-scrollbar` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/024-draggable-scrollbar/spec.md`

## Summary

The feature-021 scrollbars are display-only. Make them interactive in `handle_mouse_event`:

- **Track click → page** toward the click (one viewport up/back if above/left of the thumb, down/forward
  if below/right).
- **Thumb drag → proportional scroll** (cursor fraction along the track → same fraction of the scroll
  range); release ends the drag.
- Applies to the editor vertical bar, the editor horizontal bar (non-wrap), and the file browser /
  Help/About / encoding / plugin vertical bars.
- Editor interaction is **viewport-only** (cursor not moved) and a press starting on a scrollbar
  **suppresses** the feature-017 text drag-selection for that gesture.

Two pieces: (1) pure geometry/mapping helpers in `src/ui/scrollbar.rs` (`thumb_span`, `pos_to_offset`,
hit classification) so hit-testing matches what ratatui draws; (2) App wiring — a `scrollbar_drag`
state field, a single "scrollbar press" check placed **before** the existing click/selection/entry
handlers, and a branch in the Drag handler that scrolls while a drag is active.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74).

**Primary Dependencies**: `src/ui/scrollbar.rs` (feature 021 — extend with pure hit/map helpers),
`src/ui/mod.rs` (`editor_panes` geometry — expose as `pub(crate)`), `src/ui/file_browser.rs`
(list-area geometry), `src/app.rs` (`handle_mouse_event`, `wheel_scroll_editor`, per-surface scroll
state: `buffers[].scroll_offset`, `file_browser.scroll`, `help_scroll`, `pending_encoding_select`,
`plugin_manager_cursor`, `drag_anchor`). No new crates.

**Storage**: N/A.

**Testing**: `cargo test` — unit (`thumb_span`/`pos_to_offset` math: monotonic, clamped, fills track when
content fits; classification above/on/below thumb); integration (track click pages; thumb drag scrolls
proportionally + cursor unchanged; press-in-text still selects; press-on-bar does not select; modal bar
interactive, editor beneath not). TDD per Constitution V.

**Target Platform**: Linux + portable; terminal TUI.

**Project Type**: Single-project Rust desktop TUI application.

**Performance Goals**: O(1) per mouse event.

**Constraints**: the interactive region MUST equal the drawn region (shared geometry); bounded (no
over-scroll); no panic on resize/release-outside/no-bar/tiny-thumb; **must not regress feature-017 text
drag-selection** (the highest risk) or the wheel/buttons/keyboard; editor drag is viewport-only.

**Scale/Scope**: a few pure helpers + one drag-state field + one press-check + one drag branch in
`src/app.rs`.

## Constitution Check

| Principle | Assessment |
|---|---|
| **I. DOS-Faithful UI** | ✅ Click/drag scrollbars are the expected desktop idiom; keyboard nav unchanged. |
| **II. UTF-8 First** | ✅ No text handling; scrolls by rows/cols. |
| **III. Portable Build** | ✅ Pure Rust, no new deps; mouse events already cross-platform. |
| **IV. Minimal Footprint** | ✅ Extend the existing scrollbar module + one App field. |
| **V. Test-Gated (NON-NEGOTIABLE)** | ✅ TDD: mapping-math units + interaction/no-regression integration first. |
| **VI. Simplicity / YAGNI** | ✅ Reuse feature-021 geometry; no new actions/config. |
| **VII. Security Hardening** | ✅ No new input/attack surface. |

**Result**: PASS.

## Project Structure

### Documentation (this feature)

```text
specs/024-draggable-scrollbar/
├── plan.md, research.md, data-model.md, quickstart.md
├── contracts/scrollbar-interaction.md
├── checklists/requirements.md
└── tasks.md   # /speckit-tasks output
```

### Source Code (repository root)

```text
src/ui/scrollbar.rs   # NEW pure helpers: thumb_span(track_len, content, viewport, pos) -> (start,len);
                      #   pos_to_offset(track_len, content, viewport, click) -> offset; hit class
                      #   (Above/Thumb/Below) — match ratatui's rendered thumb.
src/ui/mod.rs         # make editor pane/bar geometry reachable (pub(crate) editor_panes or a small
                      #   `editor_bar_rects` accessor) so app.rs can hit-test the editor bars.
src/ui/file_browser.rs# pub accessor for the list-bar rect + metrics (entries/list_rows/scroll).
src/app.rs            # `scrollbar_drag: Option<ScrollbarDrag>` state; `scrollbar_regions()` building the
                      #   active surface's interactive bars (rect, axis, content, viewport, get/set offset);
                      #   in handle_mouse_event: (a) Drag branch — if a scrollbar_drag is active, map the
                      #   cursor to an offset and apply, return (before feature-017 selection); (b) a press
                      #   check before the click/selection/entry handlers — track click pages, thumb press
                      #   starts a drag; Release clears scrollbar_drag.
tests/integration/scrollbar_interaction.rs  # NEW
```

**Structure Decision**: Single-project; pure math in `scrollbar.rs`, geometry accessors exposed from the
renderers (so drawn == interactive), and the state machine in `src/app.rs`. The bars occupy reserved
columns/rows that don't overlap buttons or text, so a single scrollbar press-check placed first is safe.

## Phase 0 — Research

See [research.md](./research.md). Key decisions: own thumb/offset math (ratatui's Scrollbar exposes none);
unified `scrollbar_regions()` for the active surface; press-check ordered before selection/entry/click;
`scrollbar_drag` state guards feature-017; editor viewport-only via the feature-023 helper.

## Phase 1 — Design & Contracts

- [data-model.md](./data-model.md) — `ScrollbarDrag` state, per-surface region descriptor, mapping math.
- [contracts/scrollbar-interaction.md](./contracts/scrollbar-interaction.md) — track-click/thumb-drag
  behavior, ordering vs feature-017, bounds, and no-regression contract.
- [quickstart.md](./quickstart.md) — build/test + manual walkthrough.

Agent context updated to point at this plan.

## Complexity Tracking

No constitution violations — table omitted.
