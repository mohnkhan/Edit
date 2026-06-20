# Implementation Plan: Focusable dialog buttons (borders, tab order, mouse)

**Branch**: `016-dialog-buttons` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/016-dialog-buttons/spec.md`

## Summary

Add a single reusable **boxed-button bar** (`src/ui/buttons.rs`) — each button drawn in its own box
border, one focused — and wire it into every modal so dialogs are mouse-clickable and tab-navigable.
The bar owns three things, all from one shared geometry so drawn position == clickable region:
`button_rects(area, labels)` (layout), `render_buttons(...)` (draw, focused distinct), and
`hit_test_buttons(rects, col, row)` (click → index). `App` gains a `dialog_focus: usize` (only one
modal is open at a time) reset when a dialog opens. A `Shift+Tab`/`BackTab` action is added.

Integration is uniform: each dialog declares an ordered button list; `Tab`/`Shift+Tab` move
`dialog_focus` (wrap), `Enter`/`Space` activate the focused button, a left-click activates the clicked
button, and the mouse handler stops ignoring modals — it hit-tests the open dialog's buttons (and an
outside click cancels where a safe cancel exists). Existing per-dialog letter shortcuts, list
navigation (Up/Down), and `Esc` all keep working.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74).

**Primary Dependencies**: ratatui (Block/Borders for boxed buttons, Modifier), crossterm (BackTab key),
existing `src/ui/mod.rs` overlay renders, `src/app.rs` modal guards + `handle_mouse_event`,
`src/ui/theme.rs`. No new crates.

**Storage**: N/A.

**Testing**: `cargo test` — unit (`buttons.rs` layout/hit-test/focus), integration (dialog activation by
button via `handle_action` + `handle_mouse_event`), smoke. TDD per Constitution Principle V.

**Target Platform**: Linux + portable; terminal TUI.

**Performance**: trivial (a handful of buttons per dialog).

**Constraints**: width/UTF-8-correct button layout; no panic on small terminals (clamp/wrap); dialogs
stay modal; no regression to editing or the file-browser/menu mouse paths. Boxed buttons add ~3 rows, so
dialog heights grow.

**Scope & staging (clarified: all dialogs, boxed style).** Implemented in this feature: the shared
component + **confirm/dismiss dialogs** (unsaved-changes Save/Discard/Cancel, session restore, revert,
external-change, plugin consent, Help/About) and the **list dialogs** (encoding select, plugin manager)
which gain OK/Cancel buttons coexisting with list nav. **Deferred** (GitHub issue + ROADMAP row): the
**Find/Replace** dialog (just shipped in feat 015; needs a field+button focus ring) and the **file
browser** (already fully mouse- and keyboard-navigable) — both already navigable, lower risk to defer.

## Constitution Check

| Principle | Assessment |
|---|---|
| **I. DOS-Faithful UI** | ✅ Boxed buttons with a focused highlight and tab order are the DOS dialog idiom; Esc still backs out. |
| **II. UTF-8 First** | ✅ Button width + hit-test use display-width over graphemes; labels are ASCII but the code is width-correct. |
| **III. Portable Build** | ✅ Pure Rust, no new deps/platform code. |
| **IV. Minimal Footprint** | ✅ One small module + a focus field. |
| **V. Test-Gated (NON-NEGOTIABLE)** | ✅ TDD: layout/hit-test/focus units; per-dialog activation integration tests. |
| **VI. Simplicity / YAGNI** | ✅ One shared component; uniform integration; deferrals filed per the deferral rule. |
| **VII. Security Hardening** | ✅ No new input/attack surface; buttons map to existing gated actions. |

**Gate result: PASS.** Deferrals tracked via issue + ROADMAP (Constitution / project rule).

## Project Structure

```text
specs/016-dialog-buttons/  → plan.md research.md data-model.md quickstart.md
                             contracts/dialog-buttons.md checklists/{requirements,quality}.md
src/ui/buttons.rs   → ButtonBar: button_rects / render_buttons / hit_test_buttons (+ focus helpers)
src/ui/mod.rs       → render a button bar in each in-scope dialog (heights grown)
src/app.rs          → dialog_focus field; per-dialog button lists + activate; Tab/BackTab/Enter/Space
                      in modal guards; handle_mouse_event hit-tests dialog buttons (+ outside cancel)
src/input/keymap.rs → BackTab (Shift+Tab) → Action::FocusPrevField
tests/integration/dialog_buttons.rs → activation by Tab+Enter and by click for representative dialogs
```

## Phase 0/1 outputs

- [research.md](./research.md) — boxed-button geometry, focus-style choice, mouse-handler restructuring,
  Tab disambiguation (vs feat-015 field switch), staging/deferral rationale.
- [data-model.md](./data-model.md) — `ButtonBar` layout/render/hit-test; `App.dialog_focus`; per-dialog
  button tables + activation mapping.
- [contracts/dialog-buttons.md](./contracts/dialog-buttons.md) — keyboard/mouse/render contract.
- [quickstart.md](./quickstart.md).
- Agent context: point `CLAUDE.md` SPECKIT block at this plan.

## Complexity Tracking

Deferred dialogs (Find/Replace, file browser) → one GitHub `follow-up` issue + a `ROADMAP.md` row before
merge (project deferral rule).
