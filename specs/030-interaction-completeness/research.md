# Research: Interaction completeness

All four stories are deferred issues with confirmed code anchors from the feature-029 audit; no NEEDS
CLARIFICATION remained. This records the approach per story.

## D1 (US1) — In-dialog mouse content hit-testing

**Finding**: `App::handle_mouse_event` routes a left press to the feature-020 boxed buttons (via
`interactive_dialog_rect` + `buttons::hit_test_buttons`) and, for the file browser, to list rows
(`FileBrowser::hit_test` → `BrowserHit::Entry`). The encoding-select and plugin-manager **list rows**,
and the text **fields** of Find/Replace, Go-to-Line, and the file-browser Name/path, are not hit-tested
— content clicks fall through to the modal guard and are dropped.

**Decision**:
- **List rows**: add geometry helpers next to each list's renderer — `dialog::encoding_row_hit(rect,
  col, row) -> Option<usize>` and an equivalent for the plugin manager — using the same inner-list
  origin the renderer draws at. In `handle_mouse_event`, after the button hit-test, map a click inside
  the dialog to a row index → set `pending_encoding_select`/`plugin_manager_cursor` and set
  `dialog_focus = 0` (primary control).
- **Text fields**: add a helper `field_caret_at(field_rect, visible_text, click_col) -> caret_grapheme`
  that walks the visible (right-anchored) field text by `ui::width::display_width`, clamped to the value
  length. Apply per field (Find query/replacement, Go-to-Line, file-browser Name) and set the field's
  caret + focus. Share the helper.

**Rationale**: Mirrors the renderer geometry so drawn==clickable; reuses the shared width function (D from
029). Keeps each dialog independent.

**Alternatives**: A generic dialog "content rect → element" registry — rejected as over-engineering for
four known dialogs.

## D2 (US2) — Double-click word / triple-click line

**Finding**: editor clicks go through `handle_mouse_click` (single position) + the feature-017 drag
anchor; the file browser already tracks double-clicks via `last_browser_click: Option<(usize, Instant)>`
and `DOUBLE_CLICK_MS`.

**Decision**: Add an editor click-tracker `last_editor_click: Option<(u16, u16, u8, Instant)>` (col,
row, count, time). On a left press in the editor: if within `DOUBLE_CLICK_MS` and the same cell,
increment the count (2 = double, 3 = triple, wraps back to 1); else count = 1. Count 1 → existing
position+anchor behavior; 2 → select the word under the cursor; 3 → select the whole logical line.
Word boundaries: classify each grapheme's first scalar as word (`char::is_alphanumeric` or `_`) vs
non-word; double-click extends left/right over the run of the clicked class (space runs select the
space run). Build the selection as `Selection { anchor, active }` on grapheme columns.

**Rationale**: Reuses the proven click-timing approach; word logic is local and panic-free at line
ends/empty lines.

**Alternatives**: Use `unicode-segmentation` word boundaries (`unicode_words`) — heavier and splits on
punctuation differently; the alphanumeric+`_` run rule matches common editors and the existing tests'
expectations. (We already depend on `unicode-segmentation` for graphemes; word-run classification is
simpler and sufficient.)

## D3 (US3) — Right-click context menu

**Finding**: `normalize_mouse` already maps `MouseButton::Right`; `handle_mouse_event` ignores
non-Left presses (`if ev.kind != Press || ev.button != Left { return }`). There is no popup widget.

**Decision**: Add `src/ui/contextmenu.rs` with a `ContextMenu { items: &'static [(&str, Action)],
focus: usize, anchor: (u16,u16) }` and `render`/`hit_test`/rect helpers modelled on the menubar
dropdown + `buttons`. Add `pending_context_menu: Option<ContextMenu>` to `App`. On a Right press in the
editor (and only when no other modal/menu is active — FR-010), open it anchored at the click, clamped
on-screen. While open: Up/Down move focus, Enter/Space activate the focused item, Esc/outside-click
dismiss; a left click on an item activates it. Activation routes to the existing
`Cut/Copy/Paste/SelectAll` handlers and closes the menu. Items: Cut, Copy, Paste, Select All.

**Rationale**: Smallest new surface, reusing existing render/hit-test/actions; modal precedence keeps it
from fighting other dialogs.

**Alternatives**: Reuse the menubar `MenuState` machinery directly — rejected; it's bound to the top
menu bar geometry. A dedicated tiny overlay is simpler and self-contained.

## D4 (US4) — DOS F-key accelerators

**Finding**: `default_map` binds F1/F2/F3/F5/F10/F12; F6/F8/F9/F11 are free. Actions exist:
`NextBuffer`/`PrevBuffer`, `Cut`/`Copy`/`Paste`.

**Decision**: Add `F6 → NextBuffer`, `Shift+F6 → PrevBuffer`, `F8 → Cut`, `F9 → Copy`, `F11 → Paste`.
These are additive; existing Ctrl bindings remain. Verify no collision and that existing F-keys are
unchanged (the keymap warns on conflicts; tests assert the mapping and the unchanged set).

**Rationale**: Cheap, familiar, low-risk; all actions already implemented.

**Alternatives**: A larger DOS F-key set (F4, F7, …) — deferred; only the genuinely useful,
non-conflicting, action-backed keys are added now.

## Testing approach

TDD per story. Unit: encoding/plugin row-hit helpers; `field_caret_at` (incl. multibyte + clamp);
word/line selection over ASCII and multibyte and at boundaries; context-menu open/focus/activate/
dismiss + on-screen clamping; keymap F-key mappings + existing-F-key regression. Integration
(`tests/integration/interaction.rs`): click a dialog list row → selection; click a field → caret;
double/triple-click → Copy returns the word/line; right-click → menu → Copy runs; F-keys drive actions.
