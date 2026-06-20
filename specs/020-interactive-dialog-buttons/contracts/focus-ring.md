# Contract: Interactive-dialog focus ring & buttons

This is the behavioral contract integration tests assert against. "Primary control" = the dialog's
list/field group; "button stop" = a focus stop that is a boxed button.

## Common contract (all four dialogs)

| Input | Precondition | Effect |
|---|---|---|
| `Tab` | dialog open | `dialog_focus = next(dialog_focus, ring_len)` (wraps). |
| `Shift+Tab` / `BackTab` | dialog open | `dialog_focus = prev(dialog_focus, ring_len)` (wraps). |
| `Enter` / `Space` | a **button** stop focused | activate that button (see per-dialog mapping). |
| left click on a drawn button | dialog open | activate that button directly (regardless of focus). |
| `Esc` | dialog open | close/cancel the dialog (unchanged), from any focus. |
| any legacy key | a **primary-control** stop focused | identical behavior to before this feature. |
| on open | — | `dialog_focus = 0` (primary control focused). |
| render | dialog open | exactly one stop shown focused; button row drawn boxed; geometry drawn == hit-tested. |

`ring_len`, `field_stops`, and button labels per dialog/mode are defined in `data-model.md`.

## Encoding select

- Buttons: `OK`, `Cancel`. `field_stops = 1`.
- Activation: `OK` ⇒ apply `ENCODING_OPTIONS[selected]` (== current `Enter` on the list) and close;
  `Cancel` ⇒ close with no change (== `Esc`).
- Primary-control keys preserved: `Up`/`Down` move the selection **only while the list is focused**;
  `Enter` on the list applies the selected encoding (unchanged).
- Button-focused `Up`/`Down`: no-op.

## Plugin manager

- Buttons: `Close`. `field_stops = 1`.
- Activation: `Close` ⇒ close the manager (== `Esc`).
- Primary-control keys preserved: `Up`/`Down` move the cursor; `Space`/`Enter` toggle the highlighted
  plugin — **only while the list is focused**.
- Button-focused `Up`/`Down`/`Space-as-toggle`: `Space`/`Enter` activate `Close`; `Up`/`Down` no-op.
- Empty plugin list: ring is still `[List, Close]`; `Close` reachable and works.

## Find/Replace

- Buttons (Find mode): `Find`, `Close`; `field_stops = 1`.
- Buttons (Replace mode): `Find`, `Replace`, `Replace All`, `Close`; `field_stops = 2`.
- Activation:
  - `Find` ⇒ run find from dialog (== current Find-mode `Enter`).
  - `Replace` ⇒ replace current match from dialog (== current Replace-mode `Enter`).
  - `Replace All` ⇒ replace all from dialog (== current `Ctrl+A` / `Action::SelectAll` in replace mode).
  - `Close` ⇒ close the dialog (== `Esc`).
- Primary-control keys preserved (while a field stop is focused): typing inserts, `Backspace`,
  `Left`/`Right` move caret, `Alt+C/A/R/W` toggle case/regex/.../whole-word, `F3`/`F2` next/prev match,
  `Enter` runs the per-mode action.
- `Tab` from the last field stop moves to the first button; from `Close` wraps to `Query`.
- Field/ring sync: when `dialog_focus` is on a field stop, `FindReplaceDialog.focus` matches it so the
  focused field is the one edited and rendered with a caret.

## File browser

- Buttons: `Open` (open mode) or `Save` (save-as mode), then `Cancel`. `field_stops = 1`.
- Activation: `Open`/`Save` ⇒ apply the browser's `activate()` outcome (== `Enter`/double-click);
  `Cancel` ⇒ close the browser (== `Esc` / outside click).
- Primary-control keys preserved (while the browser is focused): `Up`/`Down` move selection,
  `Left` parent dir, `Right`/`Enter` activate, `Backspace`/typing edit the path field, mouse entry
  click/double-click unchanged.
- Mouse precedence: a click is hit-tested against the **buttons first**, then existing entry/inside/
  outside handling.
- Button-focused `Up`/`Down`: no-op.

## Edge cases (contract)

- Terminal too narrow for all buttons: overflow buttons are dropped from the drawn row (existing
  `button_rects` behavior) but remain reachable via `Tab`; no panic.
- Resize while open: outer `Rect` recomputed by the shared helper; render and hit-test stay consistent.
- Wide/UTF-8 labels & field/list content: widths via `unicode_width`; no misalignment.
- Click inside the dialog but not on a button or the primary control: no state change; dialog stays open.
