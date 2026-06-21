# Internal API Contract: Modal accessors & layer precedence

This feature exposes no external/public API. The "contract" is the internal accessor surface on `App`
that replaces direct field access, so call-sites (and tests) migrate mechanically and behavior is
preserved (FR-009).

## Accessors on `App` (new)

```rust
impl App {
    /// The current foreground overlay (read-only).
    fn modal(&self) -> &Modal;

    /// True iff a foreground overlay is open (≠ Modal::None).
    /// Replaces the consolidated OR-guard in handle_mouse_event.
    fn modal_is_open(&self) -> bool;

    /// Close any open overlay (set to Modal::None). Idempotent.
    /// Used wherever an overlay was previously cleared by `self.pending_X = None`.
    fn close_modal(&mut self);

    // Inner-payload access where a call-site needs the open dialog (mutating editing/scroll state).
    // Each returns Some only when that specific variant is open.
    fn find_replace(&self) -> Option<&FindReplaceDialog>;
    fn find_replace_mut(&mut self) -> Option<&mut FindReplaceDialog>;
    fn file_browser(&self) -> Option<&FileBrowser>;
    fn file_browser_mut(&mut self) -> Option<&mut FileBrowser>;
    fn context_menu(&self) -> Option<&crate::ui::contextmenu::ContextMenu>;
    // (goto-line digits/caret, help scroll, encoding row, plugin cursor accessed via the
    //  variant pattern in the dispatcher matches; helpers added only where ≥2 call-sites read them.)
}
```

**Contract rules**:
- C-1: `modal()` is the *only* read path for overlay state; no code reads removed fields.
- C-2: Every former `self.pending_X = Some(v)` becomes `self.modal = Modal::X(v)`; every
  `self.pending_X = None`/`false` becomes `self.close_modal()` (or a direct re-assign when transitioning
  to another overlay).
- C-3: Existing helper signatures are unchanged — `open_button_dialog()`, `interactive_dialog()`,
  `button_dialog_rect()`, `interactive_dialog_rect()` keep their return types; only their bodies read
  `self.modal`. Dependent code and tests that call these are untouched.

## Geometry contract (FR-006)

```rust
impl App {
    /// Single source of the Go-to-Line dialog rect, used by BOTH render and hit-test.
    fn goto_line_rect(&self) -> ratatui::layout::Rect;
}
```

- G-1: `goto_line_rect()` replaces the duplicated inline math at app.rs ~4424 (mouse) and
  ui/mod.rs:403–407 (render). Both call it.
- G-2: Find/Replace field rects: the render path is routed through the existing
  `crate::ui::find_replace_field_rects(d, area)` so render and hit-test share one function.

## Layer-precedence contract (FR-004/FR-005)

```rust
impl App {
    /// Active layers given current state, BOTTOM→TOP. Render iterates as-is; mouse iterates reversed.
    fn active_layers(&self) -> impl Iterator<Item = Layer>;
}
```

- L-1: `Ui::render` paints by iterating `active_layers()` ascending.
- L-2: `handle_mouse_event` resolves a press by iterating `active_layers()` reversed and dispatching to
  the first layer whose rect contains `(col,row)`.
- L-3: No per-layer-pair reconciliation guard exists (the `!dropdown_open` special-case is deleted).

## Behavioral equivalence contract (the real gate)

- B-1: All 33 integration test files pass with only mechanical field→accessor edits
  (`app.pending_find_replace.is_some()` → `app.find_replace().is_some()` or
  `matches!(app.modal(), Modal::FindReplace(_))`). No assertion's expected value changes.
- B-2: All 87 inline `app.rs` tests pass; those reading removed fields switch to accessors only.
- B-3: All 9 `.exp` smoke tests pass unchanged (they drive the real binary; no source-level coupling).
- B-4: `make ci-local` is clean (fmt, clippy -D warnings, test, smoke, perf-check).
