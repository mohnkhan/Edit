# Implementation Plan: Interaction completeness

**Branch**: `030-interaction-completeness` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/030-interaction-completeness/spec.md`

## Summary

Close the four deferred feature-029 follow-ups as four independent user stories: in-dialog mouse
content hit-testing (#53), double/triple-click selection (#54), a right-click context menu (#55), and
additional DOS F-key accelerators (#56). All reuse existing infrastructure — the shared `ui::width`,
the dialog geometry helpers, the `last_browser_click` click-tracking pattern, the menu/button render +
hit-test, and the existing edit actions. No new crates.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74)

**Primary Dependencies**: `ratatui` + `crossterm`, `ropey`, `unicode-width`, `unicode-segmentation`. No
new dependencies (Principle IV).

**Storage**: N/A.

**Testing**: `cargo test` — inline unit tests + integration tests under `tests/integration/`.

**Target Platform**: Linux TUI (x86_64/aarch64), headless VT100-compatible.

**Project Type**: Single-project Rust terminal application.

**Performance Goals**: Within the existing `make perf-check` budget (these are input-path additions, not
render-hot-path changes).

**Constraints**: DOS-faithful look preserved; no regression to existing keys/mouse/editing; modal
precedence respected; no panic on any geometry/content.

**Scale/Scope**: ~4 stories across `src/input/mouse.rs` (already normalizes Right), `src/app.rs`
(mouse routing, click-tracker, context-menu state, dialog content hit-testing), `src/ui/dialog.rs` +
`src/ui/plugin_manager.rs` (list-row + field geometry helpers), `src/ui/contextmenu.rs` (new small
overlay), `src/ui/mod.rs` (render the context menu), `src/input/keymap.rs` (F-keys).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. DOS-Faithful UI**: PASS — mouse selection, a context menu, and F-keys are all classic
  text-editor affordances; the context menu reuses the existing DOS-style menu rendering.
- **II. UTF-8 First**: PASS — word selection uses Unicode word boundaries; click→caret mapping uses the
  shared display-width function (combining=0, wide=2). No raw-byte handling.
- **III. Portable Build**: PASS — no platform-specific code.
- **IV. No New Crates**: PASS — reuses existing deps and helpers.
- **V. Test-Gated (NON-NEGOTIABLE)**: PASS — each story gets failing-first tests (per-dialog click
  mapping, word/line selection incl. multibyte, context-menu activate/dismiss, F-key bindings).
- **VI. Simplicity/YAGNI**: PASS — one small new overlay (context menu) modelled on existing menus;
  everything else extends current code paths. Four independent, separately-shippable stories.
- **VII. Security Hardening**: PASS — no network/plugin/file-format surface touched; context-menu
  actions are the existing, already-guarded edit actions.

**Result**: No violations. Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/030-interaction-completeness/
├── plan.md · research.md · data-model.md · quickstart.md
├── contracts/behavior.md
└── tasks.md  (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── input/
│   ├── mouse.rs          # (already normalizes Right/Middle — no change expected)
│   └── keymap.rs         # US4: F6/Shift+F6/F8/F9/F11 accelerators
├── ui/
│   ├── contextmenu.rs    # NEW (US3): small popup — items, focus, anchor; render + hit-test
│   ├── dialog.rs         # US1: encoding list-row rects + field-interior rect/caret mapping helpers
│   ├── plugin_manager.rs # US1: plugin list-row rect helper
│   ├── file_browser.rs   # US1: Name/path field caret mapping (list rows already clickable)
│   └── mod.rs            # US3: render the context-menu overlay
└── app.rs                # US1 routing (dialog content hit-test) ; US2 click-tracker + word/line
                          #   selection ; US3 right-click → open menu, key/mouse handling, modal guard
tests/
└── integration/
    └── interaction.rs    # NEW: cross-cutting end-to-end tests (registered in Cargo.toml)
```

**Structure Decision**: Single-project Rust. One new small module `src/ui/contextmenu.rs` (US3),
modelled on the existing menu/button widgets; everything else extends existing files.

## Phasing & independence

- **US4 (F-keys)** is the smallest and fully independent → implement first as a quick win.
- **US2 (double/triple-click)** is self-contained in the editor click path.
- **US1 (in-dialog mouse)** spans several dialogs but each dialog is independent; share a column→grapheme
  helper built on `ui::width`.
- **US3 (context menu)** adds the one new widget; depends on nothing else here.

Each story is independently testable and shippable; the MVP is US4+US2 (no new widget), then US1, then US3.

## Complexity Tracking

> No Constitution violations — section intentionally empty.
