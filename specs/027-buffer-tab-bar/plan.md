# Implementation Plan: Buffer tab bar

**Branch**: `027-buffer-tab-bar` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/027-buffer-tab-bar/spec.md`

## Summary

Add a one-row tab bar below the menu bar, shown only when `buffers.len() > 1`. Each tab shows the
buffer's filename + a modified marker, the active tab highlighted; clicking a tab's label switches to it,
clicking its `[x]` closes it. Because the bar consumes one editor row, the editor's top/height become a
single shared, tab-bar-aware computation used by rendering, `viewport_height`, click→cursor mapping, and
the feature-023 wheel / feature-021+024 scrollbar editor-area logic.

Two sub-parts:
1. **Tab bar** — new `src/ui/tabbar.rs` owning one geometry source (`tab_hit_regions(area, …)` →
   per-tab label rect + `[x]` rect) shared by the renderer and mouse hit-testing (the buttons.rs
   pattern). Layout adds a conditional 1-row chunk; `editor_top()` becomes `1 + tab_rows`.
2. **`[x]` close with unsaved guard** — the existing `close_active_buffer` doesn't prompt, so add
   `close_buffer_at(idx)` plus a `CloseConfirm` confirm dialog (a new `ButtonDialog` variant backed by
   `pending_close_confirm: Option<usize>`) reusing the feature-016 boxed-button machinery: clean buffer →
   close immediately; modified buffer → Save / Discard / Cancel confirm, reusing the existing save logic.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74).

**Primary Dependencies**: `src/app.rs` (`buffers`/`active_idx`, `next/prev_buffer`, `close_active_buffer`,
`handle_mouse_event`, `viewport_height`, `handle_mouse_click`, the wheel/scrollbar editor-area logic, the
feature-016 `ButtonDialog` infra), `src/ui/mod.rs` (layout + render), `src/ui/buttons.rs` (reused for the
confirm dialog), `src/buffer` (`path`, `modified`), `unicode-width`/`unicode-segmentation` for tab labels.
No new crates.

**Storage**: N/A.

**Testing**: `cargo test` — unit (tab geometry: label/`[x]` rects, overflow keeps active visible, no panic
tiny width; `editor_top`/`viewport_height` with vs without the bar; `close_buffer_at` index adjustment),
integration (click a tab switches; click `[x]` on a clean buffer closes; `[x]` on a modified buffer opens
the close-confirm and Save/Discard/Cancel behave; click in text places the cursor accounting for the tab
row; tab bar hidden at 1 buffer). TDD per Constitution V.

**Target Platform**: Linux + portable; terminal TUI.

**Project Type**: Single-project Rust desktop TUI application.

**Performance Goals**: O(buffers) per render/hit-test — trivial.

**Constraints**: single source of truth for the editor top/height (tab-bar-aware); drawn tab geometry ==
clickable geometry; no silent data loss on close; UTF-8/width-correct labels + overflow that keeps the
active tab visible; no panic on any size/buffer count; single-buffer layout unchanged; keyboard switching
unchanged.

**Scale/Scope**: 1 new UI module + a confirm-dialog variant + the shared editor-geometry refactor in
`src/app.rs`.

## Constitution Check

| Principle | Assessment |
|---|---|
| **I. DOS-Faithful UI** | ✅ A labeled tab/file row is standard; keyboard switching preserved; Esc cancels the close-confirm. |
| **II. UTF-8 First** | ✅ Tab labels truncated by display width (grapheme-aware); names already valid UTF-8. |
| **III. Portable Build** | ✅ Pure Rust, no new deps. |
| **IV. Minimal Footprint** | ✅ One small module + reuse of buttons.rs + the existing save logic. |
| **V. Test-Gated (NON-NEGOTIABLE)** | ✅ TDD: geometry + close + no-regression tests before implementation. |
| **VI. Simplicity / YAGNI** | ✅ Tab bar only when 2+ buffers; no drag-reorder; reuse confirm infra. |
| **VII. Security Hardening** | ✅ Close-confirm prevents silent data loss; no new I/O/attack surface. |

**Result**: PASS.

## Project Structure

```text
src/ui/tabbar.rs   # NEW — tab geometry (label + [x] rects, overflow keeping active visible) + render
src/ui/mod.rs      # add conditional tab-bar chunk to the layout; render the tab bar; pass the shrunk
                   #   editor area to the editor (using the shared editor-area computation)
src/app.rs         # editor_top()/editor_area helpers (tab-bar-aware) used by viewport_height,
                   #   handle_mouse_click, the wheel block, and scrollbar_regions; tab-row click handling
                   #   in handle_mouse_event (switch / [x] close); close_buffer_at(idx); CloseConfirm
                   #   ButtonDialog variant + pending_close_confirm + its labels/activate/view-text +
                   #   keyboard intercept (Save/Discard/Cancel, Esc)
tests/integration/buffer_tab_bar.rs  # NEW
```

**Structure Decision**: Single-project. The tab bar mirrors the `buttons.rs`/scrollbar "shared geometry"
pattern so drawn == clickable. The editor top/height is centralized into one helper to keep every
geometry consumer (render, viewport_height, click mapping, wheel, scrollbar) in lockstep — the key risk.

## Phase 0 — Research

See [research.md](./research.md). Key decisions: tab bar only when 2+ buffers; shared `editor_top()`;
shared tab hit-geometry; `[x]` close adds a `CloseConfirm` dialog (existing close path has no prompt);
overflow keeps the active tab visible; keyboard switching untouched.

## Phase 1 — Design & Contracts

- [data-model.md](./data-model.md) — tab/region model, `editor_top` computation, close-confirm state.
- [contracts/tab-bar.md](./contracts/tab-bar.md) — visibility, click/switch/close, geometry-sync, and the
  no-regression contract the tests assert.
- [quickstart.md](./quickstart.md) — build/test + manual walkthrough.

Agent context updated to point at this plan.

## Complexity Tracking

No constitution violations — table omitted.
