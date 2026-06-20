# Implementation Plan: Mouse-wheel scrolling (app-wide)

**Branch**: `023-mouse-wheel-scroll` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/023-mouse-wheel-scroll/spec.md`

## Summary

`handle_mouse_event` (`src/app.rs`) normalizes wheel events (`NormalizedMouseKind::ScrollUp/ScrollDown`,
with cursor col/row) but then early-returns because it only acts on a left-button **press**. Add a wheel
block **before** that guard (after the existing drag block) that routes a `ScrollUp/ScrollDown` to the
surface under the cursor / the open modal and adjusts that surface's existing scroll offset by a fixed
step (3), clamped to bounds:

- **Open modal/overlay wins**: Help/About → `help_scroll`; encoding select → its cursor index; plugin
  manager → its cursor; file browser → move selection by the step (its existing `move_up/move_down`,
  which scroll via `ensure_visible`); Find/Replace → ignored (nothing to scroll).
- **Otherwise the editor**: adjust the pane-under-the-cursor buffer's `scroll_offset.0` by ±step,
  clamped to `[0, content_rows-1]` (content = `line_count`, or `wrap_cache.total_visual_rows()` in
  soft-wrap) — viewport only, cursor untouched. The feature-021 scrollbar reads `scroll_offset` so it
  tracks automatically.

No new actions, no config, no change to click/drag/keyboard paths. All in `src/app.rs`.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74).

**Primary Dependencies**: existing `src/input/mouse.rs` (`normalize_mouse`, `NormalizedMouseKind::Scroll*`),
`src/app.rs` (`handle_mouse_event`, `viewport_height`, `buffers[].scroll_offset`, `file_browser`,
`pending_help`/`help_scroll`, `pending_encoding_select`, `pending_plugin_manager`), `wrap_cache`
(`total_visual_rows`). No new crates.

**Storage**: N/A.

**Testing**: `cargo test` — unit (editor viewport scroll clamps at top/bottom; soft-wrap bound), and an
inline/integration drive of `handle_mouse_event` with synthesized `MouseEventKind::ScrollUp/ScrollDown`
asserting per-surface scroll + no cursor move + bounds + no-regression of click/drag. TDD per
Constitution V.

**Target Platform**: Linux + portable; terminal TUI.

**Project Type**: Single-project Rust desktop TUI application.

**Performance Goals**: O(1) per wheel event — trivial.

**Constraints**: no over-scroll/underflow/panic at limits, empty content, split view, soft-wrap, or any
terminal size; zero change to existing click/drag/keyboard behavior; modal-wins routing so the wheel
never scrolls the editor under an open dialog.

**Scale/Scope**: one new block + one small helper in `src/app.rs`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Assessment |
|---|---|
| **I. DOS-Faithful UI** | ✅ Wheel scrolling is the expected modern terminal behavior; keyboard nav unchanged. |
| **II. UTF-8 First** | ✅ No text handling change; scrolls by rows/lines. |
| **III. Portable Build** | ✅ Pure Rust, no new deps; wheel events already cross-platform via crossterm. |
| **IV. Minimal Footprint** | ✅ One block + one helper; no new module/crate. |
| **V. Test-Gated (NON-NEGOTIABLE)** | ✅ TDD: per-surface scroll + bounds + no-regression tests first. |
| **VI. Simplicity / YAGNI** | ✅ Reuse each surface's existing scroll state; fixed step constant (no config). |
| **VII. Security Hardening** | ✅ No new input surface beyond already-received wheel events; no I/O. |

**Result**: PASS (no violations).

## Project Structure

### Documentation (this feature)

```text
specs/023-mouse-wheel-scroll/
├── plan.md, research.md, data-model.md, quickstart.md
├── contracts/wheel.md
├── checklists/requirements.md
└── tasks.md   # /speckit-tasks output
```

### Source Code (repository root)

```text
src/app.rs   # handle_mouse_event: new ScrollUp/ScrollDown block before the Press/Left guard,
             #   routing to modal/overlay or the editor pane under the cursor; small helper
             #   `wheel_scroll_editor(buf_idx, down, step)` clamping scroll_offset.0.
tests/integration/mouse_wheel.rs  # NEW — per-surface wheel scroll + bounds + no-regression
```

**Structure Decision**: Single-project; the entire change is in `src/app.rs` (mouse dispatch + a clamp
helper) plus one integration test file. The feature-021 scrollbars need no change — they read the same
scroll offsets the wheel adjusts.

## Phase 0 — Research

See [research.md](./research.md). Key decisions: place the wheel block before the Press/Left guard;
modal-wins routing; editor scrolls viewport-only with a clamped offset; reuse `move_up/move_down` for the
file browser and the existing cursor/scroll fields for the other surfaces; fixed step = 3.

## Phase 1 — Design & Contracts

- [data-model.md](./data-model.md) — the wheel event, routing target table, and per-surface offset math.
- [contracts/wheel.md](./contracts/wheel.md) — per-surface wheel behavior + bounds + no-regression
  contract the tests assert.
- [quickstart.md](./quickstart.md) — build/test + manual walkthrough.

Agent context (`CLAUDE.md`/`CLAUDE.MD` SPECKIT markers) updated to point at this plan.

## Complexity Tracking

No constitution violations — table intentionally omitted.
