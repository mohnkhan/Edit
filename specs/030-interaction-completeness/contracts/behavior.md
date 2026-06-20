# Contract: Interaction completeness

Behavioral contracts the tests assert against.

## US1 — in-dialog mouse (closes #53)

| Surface | Click | Effect |
|---|---|---|
| Encoding / plugin list row | left click on a visible row | that row becomes selected; list gains focus |
| Find query/replacement, Go-to-Line, file-browser Name field | left click in the field | caret moves to the clicked grapheme (clamped to value end); field gains focus |
| Dialog button | left click | unchanged (feature-020 behavior) |
| Inside dialog, not on row/field/button | left click | no-op |
| Outside dialog | left click | unchanged |

## US2 — double/triple-click (closes #54)

- Double-click selects the word under the pointer (run of alphanumeric/`_`, or the adjacent non-word /
  whitespace run); Copy on that selection returns exactly the word.
- Triple-click selects the whole logical line; Copy returns the line text.
- A single click after a multi-click clears the selection and positions the cursor.
- Boundaries (line end, empty line, multibyte/combining) select safely with no panic.

## US3 — context menu (closes #55)

- Right-click in the editor opens a Cut / Copy / Paste / Select All menu near the click, on-screen.
- Mouse click on an item, or Up/Down + Enter/Space, runs the item's action and closes the menu.
- Esc or an outside click dismisses without acting.
- A non-applicable item (Cut/Copy w/o selection, Paste w/ empty clipboard) is a safe no-op with existing
  feedback.
- The menu does not open while another modal/dialog/menu is active.

## US4 — F-keys (closes #56)

| Key | Action |
|---|---|
| F6 | Next buffer |
| Shift+F6 | Previous buffer |
| F8 | Cut |
| F9 | Copy |
| F11 | Paste |

- F1 (Help), F2 (Find Prev), F3 (Find Next), F5 (Save), F10 (Menu), F12 (Save As Encoding), and all
  existing Ctrl/Alt bindings are unchanged.

## No-regression

- Single-click positioning, drag-select, wheel, scrollbars, dialog buttons, editing semantics, file
  formats, and dialog flows are all unchanged except for the additions above. No new dependencies.
