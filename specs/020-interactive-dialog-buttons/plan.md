# Implementation Plan: Boxed buttons + focus ring for the interactive/list dialogs

**Branch**: `020-interactive-dialog-buttons` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/020-interactive-dialog-buttons/spec.md`

## Summary

Feature 016 shipped a reusable boxed-button bar (`src/ui/buttons.rs`) and wired it into the five
confirm/dismiss dialogs via a single `dialog_focus: usize` index. The four **interactive/list**
dialogs — encoding select, plugin manager, Find/Replace, file browser — were deferred (issue #38)
because each has a primary control (list or field group) with its own `Up/Down`/`Space`/`Tab`/typing
semantics, so they need a **combined focus ring** rather than a plain button bar.

This feature generalizes the focus ring so that, for each interactive dialog, focus stop `0` (and, for
Find/Replace in replace mode, stop `1`) is the **primary control** and the remaining stops are its
**buttons**. `Tab`/`Shift+Tab` cycle the whole ring (wrapping); `Enter`/`Space` activate the focused
button; a left-click activates the clicked button. While focus is on the primary control, every
pre-existing key (`Up`/`Down`, `Space` toggle, field typing, option toggles `Alt+C/A/R/W`, match nav
`F3`/`F2`) behaves exactly as before. Buttons added: encoding **OK/Cancel**, plugin manager **Close**,
Find/Replace **Find / Replace / Replace All / Close** (mode-dependent), file browser **Open/Save +
Cancel**. No new actions are introduced — each button maps onto an action the dialog already performs.

The work is uniform but per-dialog: each dialog gains (a) one shared outer-`Rect` computation used by
both its renderer and its mouse handler (mirroring `button_dialog_rect()`), (b) a button-row render via
`render_buttons`, (c) ring-aware key handling in its existing `handle_action` intercept block, and (d)
button hit-testing in `handle_mouse_event`.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74).

**Primary Dependencies**: existing `src/ui/buttons.rs` (`button_rects` / `render_buttons` /
`hit_test_buttons` / `next` / `prev`), `ratatui` (Block/Borders, Clear, Modifier), `unicode_width`,
`unicode-segmentation`. `src/ui/dialog.rs` (EncodingSelectDialog, FindReplaceDialog),
`src/ui/file_browser.rs` (FileBrowser + layout + hit_test), `src/ui/plugin_manager.rs`, `src/ui/mod.rs`
(overlay renderers), `src/app.rs` (`dialog_focus`, per-dialog intercepts, `handle_mouse_event`). No new
crates.

**Storage**: N/A.

**Testing**: `cargo test` — unit (ring math, per-dialog rect/button-label helpers, button hit-test),
integration (each dialog: activate each button via `handle_action`/`handle_mouse_event`; confirm every
legacy key still works), smoke (headless). TDD per Constitution Principle V.

**Target Platform**: Linux + portable (FreeBSD, macOS); terminal TUI.

**Project Type**: Single-project Rust desktop TUI application.

**Performance Goals**: trivial — a handful of buttons + a single focus index per open dialog; no change
to render or keystroke latency budgets (≤ 50 ms).

**Constraints**: width/UTF-8-correct button layout and hit-testing; no panic on a terminal too small to
fit buttons (reuse the existing drop-overflow behavior); dialogs stay modal; **zero behavioral
regression** to any existing dialog key; boxed buttons add ~4 rows so each dialog grows in height and the
primary control's content area shrinks accordingly.

**Scale/Scope**: 4 dialogs; ~1 small focus-ring abstraction reused across them; no data-model or config
changes.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Assessment |
|---|---|
| **I. DOS-Faithful UI** | ✅ Boxed buttons + a single visible focus highlight + Tab order are the DOS dialog idiom; `Esc` still backs out of every dialog; `Up/Down` list nav preserved. |
| **II. UTF-8 First (NON-NEGOTIABLE)** | ✅ Button width, hit-testing, and field/list rendering already use display-width over graphemes (`unicode_width`/`unicode-segmentation`); no new byte→buffer path. |
| **III. Portable Build** | ✅ Pure Rust, no new deps, no platform-specific code. |
| **IV. Minimal Footprint** | ✅ Reuses `buttons.rs` + the existing `dialog_focus` field; adds only small per-dialog helpers. |
| **V. Test-Gated (NON-NEGOTIABLE)** | ✅ TDD: ring/label/rect units + per-dialog button-activation and no-regression integration tests before implementation. |
| **VI. Simplicity / YAGNI** | ✅ One uniform focus-ring model across all four dialogs; no speculative abstraction; no new actions. |
| **VII. Security Hardening** | ✅ No new input or attack surface; buttons dispatch existing, already-gated actions (file browser still goes through its existing path-sanitization on activate). |

**Result**: PASS (no violations; Complexity Tracking not required).

## Project Structure

### Documentation (this feature)

```text
specs/020-interactive-dialog-buttons/
├── plan.md              # This file
├── research.md          # Phase 0 output — design decisions
├── data-model.md        # Phase 1 output — focus-ring model
├── quickstart.md        # Phase 1 output — manual + automated validation
├── contracts/
│   └── focus-ring.md    # Phase 1 output — per-dialog ring & button contract
├── checklists/
│   └── requirements.md  # /speckit-specify output (spec quality)
└── tasks.md             # /speckit-tasks output (NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
src/
├── app.rs                 # dialog_focus → unified ring; per-dialog intercepts (encoding,
│                          #   plugin mgr, find/replace, file browser); handle_mouse_event button hit-test;
│                          #   ensure_dialog_focus extended to interactive dialogs;
│                          #   per-dialog button-label + outer-Rect + activate helpers
└── ui/
    ├── buttons.rs         # reused as-is (no change expected)
    ├── dialog.rs          # EncodingSelectDialog / FindReplaceDialog: expose mode→labels + sizing if needed
    ├── file_browser.rs    # add Open/Save + Cancel button rects into layout; button hit-test
    ├── plugin_manager.rs  # Close button (label/body sizing)
    └── mod.rs             # overlay renderers: grow each dialog, render its button row, focus-aware highlight

tests/
└── (integration)          # per-dialog button activation + legacy-key no-regression tests
```

**Structure Decision**: Single-project layout (Constitution III/IV). All changes live under `src/`
(primarily `src/app.rs` for state/dispatch and `src/ui/` for geometry+render), with unit tests inline
(`#[cfg(test)]`) and integration tests under `tests/`. No new modules are required; `buttons.rs` is
reused unchanged.

## Phase 0 — Research

See [research.md](./research.md). Key decisions:

1. **Unified focus-ring index.** Reuse `dialog_focus: usize` as the ring index for the interactive
   dialogs too. Define, per open dialog, `field_stops` (count of primary-control focus stops: 1 for the
   three list/browser dialogs; 1 in Find mode and 2 in Replace mode for Find/Replace) and the ordered
   button labels. Ring length = `field_stops + labels.len()`. `dialog_focus < field_stops` ⇒ the primary
   control is focused; otherwise button index = `dialog_focus - field_stops`.
2. **Find/Replace Tab now drives the whole ring.** Today `Tab` toggles Query↔Replacement; it becomes the
   ring advance (Query → [Replacement] → Find → [Replace → Replace All] → Close → wrap). This subsumes —
   not removes — field switching, satisfying the issue's "combined field+button focus ring".
3. **Button focused in a list dialog ⇒ `Up/Down` is a no-op** (non-destructive, predictable per the
   spec Assumptions). List nav is active only while the primary control is focused.
4. **File-browser confirm button label follows the mode**: "Open" in open mode, "Save" in save-as mode;
   Cancel always. (Spec/issue said "Open/Cancel"; "Save" is the mode-correct verb for the shared widget.)
5. **Shared geometry.** Each dialog computes its outer `Rect` once in a helper used by both the renderer
   and the mouse handler, so the drawn button position always equals the clickable region (the invariant
   `buttons.rs` was built around).

## Phase 1 — Design & Contracts

- [data-model.md](./data-model.md) — the focus-ring model, per-dialog stop tables, and state fields
  touched.
- [contracts/focus-ring.md](./contracts/focus-ring.md) — the per-dialog button list, activation mapping,
  default focus, and the key/mouse behavior contract that integration tests assert against.
- [quickstart.md](./quickstart.md) — how to build, the automated test commands, and a manual
  click/Tab walkthrough for each of the four dialogs.

Agent context (`CLAUDE.md` SPECKIT markers) updated to point at this plan.

## Complexity Tracking

No constitution violations — table intentionally omitted.
