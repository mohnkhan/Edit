# Implementation Plan: Scroll affordances + dialog button polish

**Branch**: `021-scroll-affordances` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/021-scroll-affordances/spec.md`

## Summary

Every scrollable view scrolls but never draws a scrollbar, Help/About has no Close button, and dialog
buttons don't advertise their key. Add, using ratatui 0.26's built-in `Scrollbar`/`ScrollbarState`:

- **Editor**: vertical (line) + horizontal (column) scrollbars on the right/bottom edges in non-wrap
  mode; vertical only in soft-wrap.
- **File browser**: vertical scrollbar in the list when entries overflow.
- **Help/About**: vertical scrollbar on overflow + a boxed **Close (Esc)** button (clickable).
- **Encoding-select / plugin-manager**: vertical scrollbar when their list overflows.
- **All dialog buttons**: append the activating key to the label (`Close (Esc)`, `Cancel (Esc)`,
  `OK (Enter)`, `Save (Enter)`, …) across the feature-016 confirm dialogs, feature-020 interactive
  dialogs, and the new Help Close.

A small shared helper (`src/ui/scrollbar.rs`) wraps `Scrollbar` so symbols/threshold are consistent and
a bar is drawn only when content overflows. The riskiest piece is the editor: the bars **reserve** the
rightmost column and bottom row of the editor area, so the reservation is done once at the layout level
(`src/ui/mod.rs`) and the three places that compute the editor text geometry — `viewport_height()`
(`src/app.rs:487`), the horizontal content-width helper (`src/app.rs:~3657`), and `handle_mouse_click`
(`src/app.rs:3424`) — are updated to match, keeping cursor-visibility, paging, and mouse mapping in sync.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74).

**Primary Dependencies**: `ratatui` 0.26 `widgets::{Scrollbar, ScrollbarState, ScrollbarOrientation}`
(already in tree), existing `src/ui/buttons.rs`, `src/ui/wrap.rs` (`WrapCache::total_visual_rows`,
`visual_to_logical`), `src/ui/editor.rs`, `src/ui/file_browser.rs`, `src/ui/dialog.rs`,
`src/ui/plugin_manager.rs`, `src/ui/mod.rs`, `src/app.rs`. No new crates.

**Storage**: N/A.

**Testing**: `cargo test` — unit (scrollbar thumb/visibility math, editor viewport height with bars,
file-browser bar visibility, key-hint label mapping), integration (Help Close by click + Esc; key-hint
labels per dialog; no-regression of scroll/nav/actions), smoke. TDD per Constitution V.

**Target Platform**: Linux + portable; terminal TUI.

**Project Type**: Single-project Rust desktop TUI application.

**Performance Goals**: editor render stays within the existing budget (≤ 2 s startup, responsive
keystrokes); the horizontal-extent measure is over visible lines only (O(viewport)), not the whole file.

**Constraints**: scrollbars must not hide content (reserve their edge); no panic on tiny terminals;
zero behavioral regression to scrolling, navigation, dialog actions, or dismissal keys; correct with
line-number gutter and split view; UTF-8/width-correct button labels and hit-testing.

**Scale/Scope**: ~6 UI surfaces; 1 shared helper; editor geometry kept in a single source of truth.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Assessment |
|---|---|
| **I. DOS-Faithful UI** | ✅ DOS EDIT.COM showed scrollbars on the edit window and list boxes and key hints on buttons; this restores that. |
| **II. UTF-8 First (NON-NEGOTIABLE)** | ✅ Button labels and hit-testing stay display-width correct; scrollbars draw box-drawing glyphs in dedicated cells. |
| **III. Portable Build** | ✅ Pure Rust, no new deps/platform code. |
| **IV. Minimal Footprint** | ✅ One small helper + reuse of `buttons.rs`/`wrap.rs`; no new crates. |
| **V. Test-Gated (NON-NEGOTIABLE)** | ✅ TDD: scrollbar math + Help Close + key-hint + no-regression tests before implementation. |
| **VI. Simplicity / YAGNI** | ✅ Reuse the toolkit's Scrollbar; one shared wrapper; documented horizontal-extent simplification. |
| **VII. Security Hardening** | ✅ No new input/attack surface; affordance-only. |

**Result**: PASS (no violations; Complexity Tracking not required).

## Project Structure

### Documentation (this feature)

```text
specs/021-scroll-affordances/
├── plan.md, research.md, data-model.md, quickstart.md
├── contracts/scroll-affordances.md
├── checklists/requirements.md
└── tasks.md   # /speckit-tasks output
```

### Source Code (repository root)

```text
src/ui/
├── scrollbar.rs     # NEW — wraps ratatui Scrollbar: vertical/horizontal, draw-only-on-overflow
├── editor.rs        # reserve right col + bottom row; render V (+H non-wrap) scrollbars
├── file_browser.rs  # vertical scrollbar in the list area when entries overflow
├── dialog.rs        # encoding-select scrollbar on overflow; key-hint labels
├── plugin_manager.rs# plugin-manager scrollbar on overflow
├── buttons.rs       # reused (labels now carry key hints from the callers)
└── mod.rs           # editor-area reservation (single + split); Help/About scrollbar + Close button
src/app.rs           # viewport_height() (−h-bar row); h-scroll content-width helper; handle_mouse_click
                     #   (ignore reserved bar cells); pending_help mouse intercept (Close click);
                     #   button-label builders append key hints (confirm + interactive + Help)
tests/integration/   # help_close_button.rs, dialog_key_hints.rs (+ extend existing where natural)
```

**Structure Decision**: Single-project layout. New `src/ui/scrollbar.rs` is the only new module; all
other changes are localized to the views above plus the editor-geometry helpers in `src/app.rs`. Unit
tests inline; integration under `tests/`.

## Phase 0 — Research

See [research.md](./research.md). Key decisions:

1. **ratatui `Scrollbar` via a thin wrapper** — `scrollbar::vertical(area, content_len, viewport, pos)`
   and `horizontal(...)`, each a no-op when `content_len <= viewport` (FR-007). One place sets the
   begin/end arrow symbols and orientation.
2. **Editor space reserved at the layout level** (`src/ui/mod.rs`) — shrink each editor pane by 1 col
   (right) and, in non-wrap mode, 1 row (bottom); render bars in the reserved strip; pass the shrunk
   area to `EditorWidget`. Keeps `viewport_height`, paging, cursor-visibility, and mouse mapping aligned.
3. **Editor horizontal extent = max visual width of visible lines** — cheap, computed during the render
   walk; horizontal bar hidden when nothing overflows and `scroll_offset.1 == 0`; no horizontal bar in
   soft-wrap.
4. **Help/About Close button** reuses `buttons.rs` (mirrors feature-020 plugin-manager wiring); a new
   `pending_help` mouse intercept activates it; `Esc`/Enter/printable dismissal unchanged.
5. **Key-hint labels** built in the existing label functions; click/focus mapping stays keyed on button
   index/identity, not the display text (FR-010).

## Phase 1 — Design & Contracts

- [data-model.md](./data-model.md) — the scrollbar indicator model + per-view (content/viewport/offset)
  sources and the button-label composition.
- [contracts/scroll-affordances.md](./contracts/scroll-affordances.md) — per-surface scrollbar inputs,
  reservation rules, Close-button behavior, and the key-hint label table the tests assert against.
- [quickstart.md](./quickstart.md) — build/test commands and a manual walkthrough.

Agent context (`CLAUDE.md`/`CLAUDE.MD` SPECKIT markers) updated to point at this plan.

## Complexity Tracking

No constitution violations — table intentionally omitted.
