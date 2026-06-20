# Contract: Focusable dialog buttons

Applies to the in-scope dialogs (confirm/dismiss + list dialogs). "Dialog open" = its `pending_*` state
is set.

## Rendering

- Each discrete choice is drawn as a boxed button (3 rows: top border / `│ label │` / bottom border)
  in a horizontally-centered row on the dialog's bottom interior rows.
- Exactly one button is focused, rendered distinctly (inverted style + `▶` marker); others plain.
- Drawn button rectangles come from the shared `button_rects` used for hit-testing → drawn == clickable.
- List dialogs (encoding, plugin manager) keep their list/body above and show the button row below.
- Degrades gracefully (label readable) where box-drawing/colour is unavailable; never panics on small
  terminals (overflowing buttons are dropped from the drawn row but remain keyboard-reachable).

## Keyboard (while an in-scope dialog is open)

| Key | Effect |
|---|---|
| `Tab` | Move button focus forward (wrap). |
| `Shift+Tab` | Move button focus backward (wrap). |
| `Enter` / `Space` | Activate the focused button (Space excluded in plugin-manager, where it toggles the list item). |
| letter shortcuts (S/D/C, Y/N, …) | Still run their choice directly (unchanged). |
| `Up`/`Down` (list dialogs) | Still navigate the list (unchanged). |
| `Esc` | Cancel/close the dialog (unchanged). |

A dialog focuses its safe default button on open (Cancel/No/Keep for destructive prompts; the
affirmative for non-destructive ones).

## Mouse

| Click target | Effect |
|---|---|
| A button's drawn box | Activate that button (same as Enter on it). |
| Inside the dialog, not on a button | Inert (dialog stays open). |
| Outside the dialog box | Cancel where a safe cancel exists (Cancel/No/Keep/Close); inert otherwise. |

The file-browser and menu mouse behavior is unchanged.

## Equivalence & non-regression

- Activating a choice by click or by Tab+Enter yields the identical result to the pre-existing letter/key
  shortcut.
- All dialogs remain modal (input never reaches the buffer); editing and the file-browser/menu mouse
  paths are unchanged; no panic at any terminal size.

## Out of scope (deferred — follow-up issue + ROADMAP)

- Find/Replace dialog buttons (field+button focus ring) and file-browser Open/Cancel buttons; both are
  already mouse/keyboard navigable.
