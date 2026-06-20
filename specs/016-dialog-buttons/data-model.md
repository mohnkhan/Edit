# Phase 1 Data Model: Focusable dialog buttons

## `src/ui/buttons.rs` (new shared module)

Stateless layout/render/hit-test for a horizontal row of boxed buttons.

- `pub fn button_rects(area: Rect, labels: &[&str]) -> Vec<Rect>`
  - Each rect is 3 rows tall, `width(label)+4` wide (2 borders + 2 padding), 1-col gaps, row centered
    horizontally on the bottom interior rows of `area`. Width computed with display width (wide-char
    safe). Buttons that would overflow the dialog are dropped from the layout (caller keeps them
    keyboard-reachable); never panics on tiny `area`.
- `pub fn render_buttons(buf: &mut Buffer, rects: &[Rect], labels: &[&str], focused: usize, theme: &Theme)`
  - Draws each label in a bordered box. The `focused` button uses the selection style (inverted) and a
    `▶` marker; others use the normal dialog style.
- `pub fn hit_test_buttons(rects: &[Rect], col: u16, row: u16) -> Option<usize>`
  - Index of the button whose rect contains `(col,row)`, else `None`.
- Focus helpers (pure): `next(focus, n) -> (focus+1)%n`, `prev(focus, n) -> (focus+n-1)%n` (n>0).

## `App` (state) — `src/app.rs`

| Field | Type | Notes |
|---|---|---|
| `dialog_focus` | `usize` | **new** — index of the focused button in the currently open dialog; reset to the dialog's default when it opens. |

Helpers:
- `dialog_button_labels(&self) -> Vec<&'static str>` — the ordered buttons for whichever in-scope dialog
  is open, else empty.
- `dialog_default_focus(&self) -> usize` — default focused index for the open dialog (R6).
- `activate_dialog_button(&mut self, idx: usize)` — run the choice for `(open dialog, idx)`, reusing the
  existing handlers (e.g. save-prompt idx→ save/discard/cancel).
- `dialog_supports_outside_cancel(&self) -> bool` — whether an outside click cancels.

## Per-dialog button tables (in scope)

| Dialog | Buttons (tab order) | Default focus | Outside-click |
|---|---|---|---|
| Unsaved-changes (quit) | Save · Discard · Cancel | Cancel | cancel |
| Session restore | Restore · Decline | Restore | decline |
| Revert confirm | Revert · Cancel | Cancel | cancel |
| External change | Reload · Keep | Keep | keep |
| Plugin consent | Allow · Deny | Deny | deny |
| Help / About | Close | Close | close |

Each button maps to the **existing** action path (e.g. Save → `prompt_save_and_quit`, Discard →
`prompt_discard_and_quit`, Revert → `reload_from_disk`, Allow → `consent_decide(true)`, OK in encoding →
confirm the highlighted encoding, …). Letter shortcuts and list Up/Down remain.

## Keyboard / mouse integration

- Keymap: add `"BackTab" -> Action::FocusPrevField` (Shift+Tab); `Tab -> FocusNextField` already exists.
- In each in-scope modal guard (`handle_action`): `FocusNextField`→`dialog_focus = next(..)`,
  `FocusPrevField`→`prev(..)`, `InsertNewline`/`InsertChar(' ')`→`activate_dialog_button(dialog_focus)`
  (Space excluded in plugin-manager). Existing letter shortcuts, Up/Down, and Esc unchanged.
- `handle_mouse_event`: when an in-scope dialog is open, hit-test its `button_rects`; a hit →
  `activate_dialog_button(i)`; outside the dialog box → cancel if supported; inside-not-on-button → inert.
  (File-browser/menu mouse paths unchanged.)

## Deferred (follow-up issue + ROADMAP)

- The **interactive/list dialogs** — encoding select, plugin manager, Find/Replace, and the file browser
  — keep their current navigation for now. They each have list/field-specific `Enter`/`Space`/`Tab`
  semantics that need a combined focus-ring (list/field + buttons) design; folding them in cleanly is a
  follow-up. (File browser is already fully mouse+keyboard navigable; Find/Replace shipped in feat 015.)

## Invariants

- Drawn button position == clickable region (shared `button_rects`).
- Exactly one focused button; focus wraps; only one modal open at a time.
- Activating by click or Tab+Enter == the existing letter/key shortcut result.
- Dialogs stay modal; Esc cancels; no panic at any terminal size.
