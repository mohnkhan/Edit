# Implementation Plan: Live Menu-Bar Activation

**Branch**: `009-menu-bar-activation` | **Date**: 2026-06-19 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/009-menu-bar-activation/spec.md`

## Summary

Wire keyboard navigation and activation into the editor's pull-down menu bar for **both**
built-in and plugin-contributed menus, and render plugin-declared top-level menus. The
feature-008 plugin engine (registry, sandboxed `dispatch_menu_action` → `Action::PluginMenuActivated`,
consent flow, Plugins manager) is reused unchanged. The technical core is introducing a single
**resolved menu model** — the ordered composite of the six built-in menus plus plugin menus
(merged by name; new ones inserted between Options and Help) — that drives both
`MenuBarState` navigation/selection and `MenuBarWidget` rendering, so indices stay consistent.
When no plugin contributes menu items, the resolved model reproduces the existing layout
byte-for-byte, preserving all current geometry tests.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV stable 1.74.0)

**Primary Dependencies**: ratatui 0.26 (TUI widgets), crossterm (key events), existing
`src/plugin` (Rhai engine, registry), existing `src/input` keymap. No new crates.

**Storage**: N/A (no persistence change). Plugin consent/enablement already persisted to
`plugins.toml` by feature 008.

**Testing**: `cargo test` (unit + integration under `tests/integration/`), `expect`+tmux smoke
under `tests/smoke/`. TDD per Constitution Principle V.

**Target Platform**: Linux x86_64/aarch64 (+ FreeBSD/macOS per constitution); headless VT100
terminals via ncurses/crossterm.

**Project Type**: Single-project desktop TUI application (Rust binary `edit`).

**Performance Goals**: Keystroke→render latency ≤ 50 ms (Constitution). Menu resolution is an
in-memory build over a handful of menus per frame — negligible.

**Constraints**: Existing menu-bar geometry and all current menu tests MUST remain green
(FR-011, SC-003). UTF-8/wide-character-correct rendering of all labels (FR-014, Principle II).
No new CLI flags or config keys.

**Scale/Scope**: 6 built-in menus; realistically 0–5 plugin menus. Touch points:
`src/ui/menubar.rs` (model + nav + render), `src/app.rs` (event-loop wiring + dispatch),
`src/ui/mod.rs` (pass plugin items to widget). No change to `src/plugin/*` or `src/input/keymap.rs`
beyond reuse (no new `Action` variants needed — `PluginMenuActivated` already exists).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Assessment |
|---|---|
| **I. Platform-Native, DOS-Faithful UI** | ✅ Directly advances it: arrow/Enter/Esc menu navigation is core EDIT.COM behavior. Help stays rightmost (clarified placement). Graceful no-color degradation unchanged. |
| **II. UTF-8 First** | ✅ All menu/item strings already UTF-8 (manifest+registry validated in feat 008); rendering reuses existing wide-char-aware menu renderer. No raw-byte paths introduced. |
| **III. Portable Build** | ✅ Pure Rust, no platform-specific code, no new deps. Builds on all targets unchanged. |
| **IV. Minimal Footprint** | ✅ No new dependencies; static build unaffected. |
| **V. Test-Gated Merges (NON-NEGOTIABLE)** | ✅ TDD: unit tests for the resolved-model builder and nav methods, integration tests for end-to-end activation, smoke test for plugin-menu keyboard activation. No behavior ships without a test. |
| **VI. Simplicity / YAGNI** | ✅ This is the accepted user story for the deferred plugin-menu activation (issue #19, ROADMAP). No speculative abstraction — one model type + nav wiring. Mouse activation explicitly out of scope. |
| **VII. Security Hardening** | ✅ Plugin execution still goes through the existing sandbox/consent path; this feature only routes a UI selection to the already-gated `dispatch_menu_action`. No new attack surface; plugin labels are validated UTF-8 and rendered through the escape-safe menu renderer. |

**Gate result: PASS.** No violations; Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/009-menu-bar-activation/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── menu-interaction.md   # keyboard + dispatch contract
├── checklists/
│   └── requirements.md  # spec quality checklist (from /speckit-specify)
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── ui/
│   ├── menubar.rs        # MODIFY: add ResolvedMenu/ResolvedItem model + resolve_menus();
│   │                     #         refactor MenuBarState nav/select to operate on the model;
│   │                     #         add navigate_left/navigate_right; render from resolved model
│   └── mod.rs            # MODIFY: build resolved menus from plugin registry; pass to widget
├── app.rs                # MODIFY: in handle_action, add a menu-active guard that routes
│                         #         MoveUp/Down/Left/Right, InsertNewline(select), MenuClose
│                         #         to MenuBarState; dispatch the resolved Action
└── plugin/               # REUSE unchanged (registry().menu_items(), dispatch_menu_action)

tests/
├── integration/
│   └── menu_activation.rs   # NEW: end-to-end built-in + plugin menu activation
└── smoke/
    └── plugin_menu_activate.exp  # NEW: headless keyboard activation of a plugin menu
```

**Structure Decision**: Single-project layout (existing). All changes are localized to the UI
menu module and the app event loop; the plugin subsystem is consumed, not modified.

## Architecture Decisions (detail)

1. **Single resolved menu model as source of truth.** A new `resolve_menus(&[PluginMenuItem])
   -> Vec<ResolvedMenu>` builds the composite, ordered top-level list. Both `MenuBarWidget`
   (render) and the `App` event loop (navigation + action resolution) consume the *same* model,
   guaranteeing index agreement between what is drawn and what Enter activates.

2. **Refactor `MenuBarState` nav/select to take the model.** `open_menu`, `navigate_up/down`,
   `navigate_left/right`, and `select_item` currently hardcode `ALL_MENUS`. They will instead
   accept the resolved menu slice (counts + item actions) so plugin menus and merged items
   participate. State stays a pure index machine (Inactive / TopActive / DropDown).

3. **No-plugin parity is a hard invariant.** `resolve_menus(&[])` MUST yield exactly the six
   built-in menus with identical labels, order, items, and column geometry. Guarded by the
   existing geometry tests plus a new explicit parity test.

4. **Event-loop guard mirrors the existing modal-dialog pattern.** `handle_action` already
   short-circuits when `pending_encoding_select` / `pending_plugin_manager` etc. are set. A new
   `if self.menu_bar.is_active() { ... }` guard is added *after* those modal guards (modals win,
   FR-012) and *before* the normal action match, routing menu-control actions to the state machine.

5. **Enter = `Action::InsertNewline` while a dropdown is open** (the editor's existing confirm
   key, consistent with how other dialogs are confirmed). No new keybinding/Action is added.

## Complexity Tracking

No constitution violations; section intentionally empty.
