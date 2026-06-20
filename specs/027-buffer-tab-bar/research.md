# Phase 0 Research: Buffer tab bar

## Existing machinery (from code survey)

- Buffers: `App::buffers: Vec<Buffer>`, `active_idx`; `next_buffer`/`prev_buffer` (wrap); `Buffer.path:
  Option<PathBuf>`, `Buffer.modified: bool`. The status bar shows a `[n/m]` indicator.
- `close_active_buffer()` removes the active buffer (or resets to empty when it's the last) — **no
  unsaved-changes prompt**.
- Layout (`src/ui/mod.rs`): vertical `[menubar(1), editor(Min 1), status(1)]`. Editor area = `chunks[1]`.
- Editor-geometry consumers in `src/app.rs` (must stay in lockstep): `viewport_height()` =
  `terminal-2-hbar`; `handle_mouse_click` maps `row-1`→editor row and guards `row==0`/`row>=rows-1`; the
  feature-023 wheel block and feature-024 `scrollbar_regions` compute the editor area as
  `Rect::new(0, 1, w, h-2)` and guard `ev.row >= 1`.
- Feature-016 confirm-dialog infra (`ButtonDialog`, `open_button_dialog`, `dialog_button_labels`,
  `dialog_default_focus`, `dialog_cancel_index`, `activate_dialog_button`, `dialog_view_text`,
  `button_dialog_rect`/`button_dialog_render`) — boxed buttons + focus ring, reusable for a new confirm.
- Save logic: `Buffer::save()`; quit prompt helpers `prompt_save_and_quit`/`_discard_`/`_cancel_`.

## Decision 1 — Tab bar only when 2+ buffers; shared `editor_top()`

**Decision**: Show the bar iff `buffers.len() > 1`. Add `editor_top(&self) -> u16` = `1 + tab_rows`
(`tab_rows = 1` when shown else 0) and use it everywhere the editor area/height is computed:
`viewport_height` subtracts `tab_rows` too; `handle_mouse_click` uses `clicked_row = row - editor_top`;
the wheel/scrollbar editor-area becomes `Rect::new(0, editor_top, w, h - editor_top - 1)`; the menu/status
guards become `ev.row >= editor_top`.

**Rationale**: One helper = no drift; single-buffer layout is byte-for-byte unchanged (tab_rows 0).

## Decision 2 — Shared tab hit-geometry (buttons.rs pattern)

**Decision**: `src/ui/tabbar.rs` exposes `tab_hit_regions(area, buffers, active) -> Vec<TabRegion {
idx, label_rect, close_rect }>` used by both the renderer and the mouse hit-test, so a click always lands
on the drawn tab/`[x]`. Overflow: lay out tabs left→right; if they exceed the width, scroll the start so
the **active** tab is visible (drop/condense others), never corrupting the row.

**Rationale**: Same "drawn == clickable" guarantee as the feature-016 buttons and feature-024 scrollbars.

## Decision 3 — `[x]` close adds a CloseConfirm dialog (no silent data loss)

**Decision**: Add `close_buffer_at(idx)` (generalize `close_active_buffer`: remove `buffers[idx]`, fix
`active_idx`). On `[x]`: clean buffer → `close_buffer_at` immediately; **modified** buffer → open a
`CloseConfirm` confirm (new `ButtonDialog` variant + `pending_close_confirm: Option<usize>`) with
**Save / Discard / Cancel** reusing `Buffer::save()` + `close_buffer_at`. Wire it through the existing
016 label/default-focus/cancel-index/activate/view-text functions + a keyboard intercept (like the revert
confirm) so Tab/Enter/click/Esc all work.

**Rationale**: The spec/decision requires no silent data loss, but the existing close path doesn't prompt;
this adds the smallest faithful prompt by reusing the proven confirm-dialog infra. (Analyze note: this is
new behavior beyond `close_active_buffer`, justified by the "save prompt on close" decision.)

## Decision 4 — Switch vs close on click

**Decision**: A click in a tab's `label_rect` switches (`active_idx = idx`); a click in its `close_rect`
(`[x]`) closes (Decision 3). A click on the tab row outside any region is a no-op. The tab-row check runs
in `handle_mouse_event` **before** the editor click path so it never places the cursor.

## Decision 5 — Keyboard switching unchanged; modified marker; labels

**Decision**: `Ctrl+Tab`/`Ctrl+Shift+Tab` (NextBuffer/PrevBuffer) untouched. Modified buffers show a
marker (e.g. `●`); labels use the file name (or "[No Name]"), truncated by display width.

## Testing strategy (Constitution V — TDD)

- **Unit**: `editor_top`/`viewport_height` with 1 vs 2+ buffers; `tab_hit_regions` (label/`[x]` rects,
  overflow keeps active visible, no panic at tiny width); `close_buffer_at` active-index adjustment.
- **Integration**: 2 buffers → tab bar present, active highlighted; click other tab → switches; click
  `[x]` on a clean buffer → closes (bar hides at 1 left); `[x]` on a modified buffer → CloseConfirm shown,
  Save closes+saves, Discard closes, Cancel keeps; a text click with the bar shown lands on the right
  line (tab row accounted for); a tab-row click never moves the cursor.

## No open clarifications

Both product decisions are fixed; the close-confirm is the documented resolution of the "save prompt on
close" requirement vs the prompt-less existing path. No NEEDS CLARIFICATION remains.
