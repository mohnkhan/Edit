# Implementation Plan: Visible text selection (highlight, Shift-select, mouse-drag)

**Branch**: `017-selection-highlight` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

## Summary

Make `buffer.selection` visible and easy to create. (1) **Render**: the editor overlays a selection
highlight (reverse video, distinct from the yellow search-match highlight) over the selected character
range, in both plain and soft-wrap render paths, mirroring the feat-015 match-highlight overlay.
(2) **Keyboard**: add `Shift+Arrow`/`Shift+Home`/`Shift+End` selecting movement — anchor the selection
at the cursor on the first shifted move, move the cursor, set `active`; any non-shift move or edit clears
it. (3) **Mouse**: a left press in the editor anchors a selection and moves the cursor; `Drag` extends
`active`; a press+release at one spot (no drag) clears the selection. Copy/Cut already read
`buffer.selection`; this only makes selections visible and creatable.

## Technical Context

**Language/Version**: Rust 2021, MSRV 1.74. **Deps**: ratatui (Modifier::REVERSED), crossterm (Drag
events), existing `src/ui/editor.rs`, `src/app.rs`, `src/input/{keymap,mouse}.rs`. No new crates.
**Testing**: `cargo test` unit (selection range math, editor render) + integration (shift-select, copy,
mouse drag via `handle_mouse_event`). TDD per Principle V. **Constraints**: UTF-8/wide-correct, scroll/
soft-wrap-correct highlight; no regression to movement/editing/search-highlight/menu mouse.

## Constitution Check

| Principle | Assessment |
|---|---|
| I. DOS-Faithful UI | ✅ Visible selection + Shift-select + mouse drag are standard editor behavior. |
| II. UTF-8 First | ✅ Highlight spans grapheme cells via the existing wide-char-aware render loop. |
| III/IV. Portable/Minimal | ✅ Pure Rust, no new deps. |
| V. Test-Gated (NON-NEGOTIABLE) | ✅ Unit + integration, TDD. |
| VI. Simplicity | ✅ Reuses match-highlight overlay pattern, existing move + click→cursor logic. |
| VII. Security | ✅ No new external input/attack surface. |

**Gate: PASS.**

## Project Structure

```text
specs/017-selection-highlight/ → plan/research/data-model/quickstart + contracts + checklists
src/ui/editor.rs   → selection-highlight overlay (plain + soft-wrap paths)
src/app.rs         → move_cursor_selecting(dir); selecting Home/End; clear-on-plain-move/edit;
                     mouse press=anchor, Drag=extend, single-click=clear
src/input/keymap.rs→ Shift+Left/Right/Up/Down/Home/End → Select* actions
src/input/mouse.rs → (already normalizes Drag) — used by the app
tests/integration/selection.rs → shift-select + copy, mouse drag, clear-on-move
```

## Phases

- Phase 0 [research.md]: highlight style (reverse video), selection range model (ordered (line,col)
  span → per-line visible spans), reuse of `handle_mouse_click` for drag endpoints, clear-on-move rules.
- Phase 1 [data-model.md / contracts/selection.md / quickstart.md]: Select* actions, selection
  bookkeeping helpers, render overlay contract, keyboard/mouse contract.
- Agent context: point `CLAUDE.md` SPECKIT block at this plan.

## Complexity Tracking

No violations.
