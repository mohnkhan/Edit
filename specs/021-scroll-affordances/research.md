# Phase 0 Research: Scroll affordances + dialog button polish

Resolves the approach for adding scrollbars and dialog-button polish, reusing existing machinery.

## Existing machinery (from code survey)

- **Editor** (`src/ui/editor.rs`): `EditorWidget::render` draws into the given `area`. Gutter is
  `const GUTTER_WIDTH: u16 = 4` (line numbers). Vertical scroll = `buffer.scroll_offset.0` (logical
  line, or visual row in soft-wrap); horizontal scroll = `scroll_offset.1` (visual column). Visible rows
  = `area.height`. Total lines = `rope.line_count()`; soft-wrap total = `WrapCache::total_visual_rows()`
  (`src/ui/wrap.rs:73`).
- **Editor geometry consumers in `src/app.rs`**: `viewport_height()` = `terminal_size.1 - 2` (menubar +
  statusbar) at line 487; horizontal content-width helper at ~3657 = `terminal_size.0 - gutter`;
  `handle_mouse_click` (3424) maps `row-1`→editor row and guards `row==0` / `row>=rows-1`.
- **File browser** (`src/ui/file_browser.rs`): `compute_layout` gives `list_top`/`list_rows`/`inner_left`
  /`inner_width`; render loop draws `entries[scroll + vis]`; `scroll`/`selected` drive the window;
  `hit_test` uses the same layout.
- **Help/About** (`src/ui/mod.rs::render_help_overlay`): scrolls via `app.help_scroll`, footer shows a
  "▼ more" cue and "Esc close"; no button; `pending_help` has no mouse intercept today.
- **Buttons** (`src/ui/buttons.rs`): `button_rects`/`render_buttons`/`hit_test_buttons` — reused as-is.
- **Dialog labels**: confirm dialogs via `App::dialog_button_labels` (~2667); interactive dialogs via
  `App::interactive_button_labels` (feature 020).
- **ratatui 0.26** provides `widgets::{Scrollbar, ScrollbarState, ScrollbarOrientation}`
  (`StatefulWidget`), renderable into a buffer via `StatefulWidget::render`.

## Decision 1 — Thin wrapper over ratatui `Scrollbar`

**Decision**: Add `src/ui/scrollbar.rs` with `vertical(buf, area, content_len, viewport_len, pos)` and
`horizontal(...)`. Each builds a `ScrollbarState::new(content_len).position(pos)` and renders a
`Scrollbar` with `VerticalRight`/`HorizontalBottom` orientation; **returns early (draws nothing) when
`content_len <= viewport_len`** (FR-007). Centralizes the begin/end arrow symbols and theme styling.

**Rationale**: One consistent look and one overflow rule for all six surfaces; minimal code; uses the
toolkit's tested widget (user's decision).

**Alternatives**: hand-drawn DOS bar (rejected per user decision); per-call inline Scrollbar (rejected —
duplicates the overflow/threshold logic).

## Decision 2 — Reserve editor scrollbar space at the layout level

**Decision**: In `src/ui/mod.rs`, before constructing each `EditorWidget` (single and both split panes),
shrink the pane area by 1 column on the right (vertical bar) and, in non-wrap mode, 1 row at the bottom
(horizontal bar). Render the bars into the reserved strip. Update the three editor-geometry helpers in
`src/app.rs` to subtract the same reservation so scroll math and mouse mapping match what's drawn:
`viewport_height()` subtracts the horizontal-bar row (non-wrap), the horizontal content-width helper
subtracts the vertical-bar column, and `handle_mouse_click` ignores clicks on the reserved cells.

**Rationale**: A scrollbar drawn inside the widget without telling the rest of the app about the lost
column/row would desync cursor visibility, paging, and click-to-position. Reserving once at the layout
boundary keeps a single source of truth.

**Alternatives**: draw the bar over the last content column (rejected — hides text, violates FR-006);
recompute geometry independently in each consumer (rejected — drift risk).

## Decision 3 — Editor horizontal extent from visible lines only

**Decision**: The horizontal scrollbar's `content_len` is the maximum visual width among the currently
visible lines (computed during the existing render walk), with `pos = scroll_offset.1` and viewport =
content width. The bar is hidden when no visible line exceeds the content width and `scroll_offset.1 == 0`.
No horizontal bar in soft-wrap mode.

**Rationale**: Scanning the whole file for the longest line each frame would break the large-file
performance budget. A visible-lines measure is O(viewport), already iterated, and gives a correct,
useful local indication. Documented as an intentional simplification in the spec.

**Alternatives**: track a global max line width incrementally on edit (more state + invalidation, not
worth it for an affordance); full-file scan per frame (too slow).

## Decision 4 — Help/About Close button + mouse intercept

**Decision**: Render a boxed Close button via `buttons.rs` in `render_help_overlay`, growing the box for
the button row (mirrors the feature-020 plugin-manager integration). Add a `pending_help` branch in
`handle_mouse_event` that hit-tests the Close button and dismisses on click. Keyboard dismissal
(`Esc`/Enter/printable) unchanged. Also add the vertical scrollbar to the overflowed cheat sheet.

**Rationale**: Directly fixes "no way to close with the mouse / unclear how to close"; reuses the proven
button component and the established mouse-hit-test pattern.

## Decision 5 — Key-hint button labels keyed on identity, not text

**Decision**: Build labels with the key appended (e.g. `Cancel (Esc)`) inside the existing label
functions. Activation, focus, and click mapping continue to use the button **index/identity**, never the
displayed string, so the longer labels can't break dispatch. `button_rects` already measures display
width, so wider labels lay out correctly.

**Rationale**: Satisfies FR-009/FR-010 with no risk to existing click/focus behavior.

## Testing strategy (Constitution V — TDD)

- **Unit**: scrollbar wrapper hides on fit / shows on overflow and computes thumb from (len, viewport,
  pos); `viewport_height()` reflects the reserved h-bar row; file-browser bar visibility threshold;
  key-hint label strings.
- **Integration**: Help and About close via a click on the Close button and via `Esc`; each dialog's
  button labels include the activating key; scrolling/navigation/dialog actions unchanged (no regression).
- **Smoke**: headless render shows a scrollbar on an overflowing editor/list and a Close button on Help.

## No open clarifications

All three product decisions are fixed; remaining choices have documented defaults. No `NEEDS
CLARIFICATION` remains.
