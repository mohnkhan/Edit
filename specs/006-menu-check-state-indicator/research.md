# Research: Menu Check-State Indicator

**Feature**: 006 — Menu Check-State Indicator
**Date**: 2026-06-19

---

## Decision 1: How to carry runtime toggle state into the static menu definitions

**Question**: `MenuItem` items are declared as `static` Rust constants (`static VIEW_MENU: &[MenuItem]`). They cannot hold mutable runtime state. How should the checked state reach the renderer?

**Decision**: Pass `toggle_states: &'a [(Action, bool)]` as an additional lifetime-bound field on `MenuBarWidget<'a>`. At render time the widget looks up each item's `action` in this slice to determine its prefix (`"✓ "`, `"  "`, or nothing). The static `MenuItem` definitions remain unchanged.

**Rationale**: The widget already holds a lifetime-bound `&'a MenuBarState`; adding a parallel `&'a [(Action, bool)]` is idiomatic and zero-cost. `Action` derives `PartialEq + Eq` so slice scanning is correct and avoids any map allocation on the hot render path.

**Alternatives considered**:
- **Add `checked: Option<bool>` field to `MenuItem` static structs** — rejected; static items can't carry runtime state. The field would always be `None` or a compile-time constant, providing no live toggle feedback.
- **Add `toggle_state_fn: Box<dyn Fn(&Action) -> Option<bool>>` closure field** — rejected; `Widget` must be `Send` and closures capturing `App` state introduce lifetime and ownership complexity far exceeding the feature scope.
- **Add `soft_wrap: bool` directly to `MenuBarWidget`** — functionally correct but not general. FR-007 requires the mechanism to apply to any future toggleable item without further refactoring. A generic `&'a [(Action, bool)]` satisfies this without complexity overhead.

---

## Decision 2: Width and alignment for the checkmark prefix column

**Question**: How should the 2-character prefix column (`"✓ "` or `"  "`) affect dropdown geometry?

**Decision**: When the current dropdown's items include at least one action that appears in `toggle_states` ("checkable-aware menu"), `content_width` is increased by 2 and every item in the dropdown gets a 2-char prefix written before the label text. Non-checkable menus are unaffected.

**Rationale**: The existing layout formula is `content_width = max_label_len + 4` with label starting at `start_col + 1`. Extending to `+ 6` and shifting the label start to `start_col + 3` preserves the same padding structure (1 leading space, label area, ≥ 1 trailing space) while accommodating the prefix column. All items in a checkable menu receive a prefix (filled or space) to guarantee horizontal label alignment (FR-008).

**Alternatives considered**:
- **Prefix only the checked item, indent all others** — same visual outcome, but the width calculation must still account for the widest possible prefix, so behavior is identical. The chosen approach is simpler.
- **Render `✓` in a different color/style** — out of scope; constitution Principle I warns against deviating from DOS-faithful UI without an explicit spec.

---

## Decision 3: Which Unicode code point for the checkmark

**Question**: Use U+2713 `✓` (CHECK MARK) or a fallback ASCII character?

**Decision**: Use U+2713 `✓` as the primary character. No runtime fallback is required: the constitution Principle II mandates UTF-8-capable terminals and the CI matrix uses `xterm-256color` which supports U+2713. The spec Assumptions section documents this.

**Rationale**: The existing codebase already renders U+00BB `»` as the soft-wrap continuation marker (feature 005) using the same `.set_char(ch)` ratatui API, confirming that non-ASCII code points work correctly in the render path. U+2713 has Unicode east-asian-width "Neutral" (display width 1), so it occupies exactly one terminal cell — the same as the 2 spaces it replaces when unchecked.

**Alternatives considered**:
- **`*` (ASCII asterisk)** — clearly readable but visually inconsistent with the project's use of Unicode characters in the UI.
- **Runtime detection via `$TERM` capability** — complex, fragile, and unnecessary given the constitution's UTF-8 mandate.

---

## Decision 4: Call-site change in `src/ui/mod.rs`

**Question**: `MenuBarWidget::new()` is called in `src/ui/mod.rs` at line 73 with `(app.theme, &app.menu_bar)`. How does `app.soft_wrap` reach the widget?

**Decision**: Update the call to `MenuBarWidget::new(app.theme, &app.menu_bar, &[(Action::ToggleSoftWrap, app.soft_wrap)])`. The `app` reference available in `Ui::render()` gives direct access to both `app.menu_bar` and `app.soft_wrap` at the same scope, so no additional plumbing is needed.

**Rationale**: `Ui::render()` already accesses `app.soft_wrap` (it passes it to `EditorWidget::new()` on line 83). The incremental change is minimal and contained to two files (`menubar.rs` + `ui/mod.rs`).

---

## Decision 5: Test approach

**Question**: Unit tests exist in `app.rs` (`#[cfg(test)]`) but `menubar.rs` has no test module. Where should check-state tests live?

**Decision**: Add a `#[cfg(test)]` module at the bottom of `src/ui/menubar.rs` with unit tests that directly call `MenuBarWidget::render()` against a ratatui `Buffer` and inspect cell contents. This follows the pattern used by `src/ui/editor.rs` and other widget modules. No integration tests are required for this feature — the unit tests cover the complete user-visible behavior.

**Rationale**: The check-state feature is a pure rendering concern. All state inputs are trivially controllable in a widget unit test (pass any `toggle_states` slice). Integration tests would add setup complexity for minimal extra coverage.
