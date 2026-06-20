# Phase 0 Research: Boxed buttons + focus ring for the interactive/list dialogs

All decisions resolve the "combined field/list + button focus ring" requirement of issue #38 while
reusing the feature-016 machinery and preserving every existing dialog key.

## Existing machinery (from code survey)

- `src/ui/buttons.rs`: `button_rects(area, labels)` lays a centered row of 3-row boxed buttons along the
  bottom interior of `area` (dropping any that overflow), `render_buttons(...)` draws them with the
  focused one inverted + `▶`, `hit_test_buttons(rects, col, row)` maps a click to an index, and
  `next`/`prev` wrap a focus index. **Reused unchanged.**
- `src/app.rs`: `dialog_focus: usize` + `dialog_focus_init: bool`; `ensure_dialog_focus()` resets focus
  when a dialog opens. The confirm-dialog dispatch (`handle_action`) intercepts `Action::FocusNextField`/
  `FocusPrevField` → `buttons::next/prev`, and `InsertNewline`/`InsertChar(' ')` → `activate_dialog_button`
  — but only when `dialog_button_labels()` is non-empty, which today is true only for the five
  `ButtonDialog` confirm dialogs. The four interactive dialogs have their own earlier intercept blocks
  that `return Ok(())`, so they never reach the confirm dispatch.
- Confirm dialogs share one outer-`Rect` source, `button_dialog_rect()`, used by both the renderer
  (`src/ui/mod.rs`) and the mouse handler (`handle_mouse_event`) — guaranteeing drawn == clickable.

## Decision 1 — Unified focus-ring index over `dialog_focus`

**Decision**: Treat `dialog_focus` as the ring index for the interactive dialogs as well. For the open
dialog define `field_stops` (number of primary-control focus stops) and an ordered `Vec<&str>` of button
labels. Ring length = `field_stops + labels.len()`. If `dialog_focus < field_stops` the **primary
control** is focused (and a `field_stops`-specific sub-state, e.g. which Find/Replace field, is derived
from `dialog_focus`); otherwise the **button** at `dialog_focus - field_stops` is focused.

**Rationale**: One index, one wrap helper (`buttons::next/prev`), one mental model across all dialogs;
mirrors the confirm-dialog design so reviewers see a consistent pattern. Avoids a second focus field.

**Alternatives considered**:
- *A separate `Focus` enum per dialog.* More types, more match arms, no real benefit over an index plus a
  small per-dialog `field_stops`/labels descriptor.
- *Buttons-only ring (list always keeps focus).* Rejected: the issue explicitly wants the list/field to
  be one focus stop in a single ring reachable by Tab.

## Decision 2 — Find/Replace `Tab` drives the whole ring

**Decision**: `Tab`/`Shift+Tab` advance/retreat the full ring. Replace mode ring:
`Query → Replacement → Find → Replace → Replace All → Close → (wrap)`. Find mode ring:
`Query → Find → Close → (wrap)`. The first 1–2 stops are the fields (so `field_stops` = 1 in Find mode,
2 in Replace mode); the existing `switch_focus()` field toggle is replaced by ring movement that lands on
the field stops.

**Rationale**: The issue asks for a "combined field+button focus ring (Find / Replace / Replace All /
Close)". Tab already moved between fields, so extending it to continue into the buttons is the least
surprising generalization. Field switching is *subsumed*, not lost.

**Preserved behavior**: while a field stop is focused, typing, `Backspace`, `Left`/`Right`, option
toggles (`Alt+C/A/R/W`), and match nav (`F3`/`F2`) behave exactly as today; `Enter` on a field stop runs
the current per-mode action (Find mode → run find; Replace mode → replace current), matching today's
`InsertNewline` behavior.

**Alternatives considered**: keep `Tab` as field-only and add a different key for buttons — rejected as
inconsistent with the other three dialogs and with the issue text.

## Decision 3 — `Up`/`Down` while a button is focused (list dialogs)

**Decision**: In the list dialogs (encoding, plugin manager) and the file browser, `Up`/`Down` are a
**no-op** when a button stop is focused. List/entry navigation is active only while the primary control
is focused.

**Rationale**: Predictable and non-destructive (spec Assumptions / FR-010 / US3 scenario 5). Keeps the
list-cursor and the focus-ring concerns from interfering. (An alternative — `Up`/`Down` snap focus back
to the list — was considered but adds a second meaning to arrow keys; the no-op is simpler and still
lets `Tab` return to the list.)

## Decision 4 — File-browser confirm button label follows mode

**Decision**: The file browser's confirm button is labeled **"Open"** in open mode and **"Save"** in
save-as mode; the second button is always **"Cancel"**. Activation maps to the browser's existing
`activate()` outcome (same as `Enter`) and to closing the browser, respectively.

**Rationale**: The widget is shared between File›Open and File›Save As (title already switches "Open
File"/"Save As"). The issue named "Open/Cancel" generically; using the mode-correct verb is more
DOS-faithful and avoids a misleading "Open" on a save dialog. Cancel mirrors the existing outside-click /
`Esc` close.

## Decision 5 — Shared per-dialog outer `Rect` (drawn == clickable)

**Decision**: Each interactive dialog computes its outer `Rect` in exactly one place, called by both its
renderer (`src/ui/mod.rs` / widget) and its mouse handler (`handle_mouse_event`). The dialog height is
grown by the button row (gap + 3 rows) and the primary control's content area is shrunk by the same
amount, mirroring `button_dialog_rect()`/confirm rendering. Buttons are laid out by passing this `Rect`
to `buttons::button_rects`.

**Rationale**: `buttons.rs` is built on the invariant that the same geometry feeds render and hit-test;
sharing the `Rect` is the only way to keep clicks landing on the drawn buttons across resizes. The file
browser already centralizes geometry in `compute_layout()`/`hit_test()`, so its buttons fold into that;
Find/Replace currently computes its rect inline in `mod.rs` and will have that extracted into a shared
helper so the mouse handler can reproduce it.

## Decision 6 — Default focus on open

**Decision**: Every interactive dialog opens with `dialog_focus = 0` (the primary control / first field).

**Rationale**: Preserves existing keyboard muscle memory — open the encoding/plugin list and press
`Up`/`Down`, or open Find and start typing immediately — exactly as before this feature (spec FR-006,
US2 scenario 3). Buttons are opt-in via `Tab` or mouse.

## Testing strategy (Constitution V — TDD)

- **Unit**: ring length & wrap for each dialog/mode; `field_stops` and button-label tables; per-dialog
  outer-`Rect` sizing (grows for buttons, no panic on tiny terminals); button hit-test maps clicks.
- **Integration**: for each dialog, drive `Tab` around the full ring and assert focus visits each stop
  once and wraps; activate each button by `Enter`/`Space` and by simulated click and assert the existing
  action ran; drive every legacy key with focus on the primary control and assert unchanged behavior;
  `Esc` closes from any focus position.
- **Smoke**: headless render of each dialog shows the boxed button row and exactly one focused control.

## No open clarifications

All spec items have reasonable, documented defaults; no `NEEDS CLARIFICATION` remains.
