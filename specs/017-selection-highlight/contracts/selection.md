# Contract: Text selection

## Render
- The active buffer's non-empty selection is drawn with reverse-video over exactly the ordered selected
  range (direction-independent), in plain and soft-wrap modes, grapheme/scroll-correct, only visible
  cells. Distinct from the search-match (yellow) highlight; the cursor cell keeps its own style.
- No selection (or empty) → nothing highlighted.

## Keyboard
| Key | Effect |
|---|---|
| `Shift+Left/Right/Up/Down` | Extend/shrink selection from the anchor as the cursor moves. |
| `Shift+Home` / `Shift+End` | Extend selection to line start / end. |
| `Ctrl+A` | Select all (whole buffer, highlighted). |
| arrows / Home / End (no Shift) | Move cursor and clear the selection. |
| typing / paste | Replace the selection (then insert); selection cleared. |
| `Ctrl+C` / `Ctrl+X` | Copy / Cut the selection (Cut removes it); undoable. |

## Mouse
| Action | Effect |
|---|---|
| Left press in editor | Move cursor to the point; set the selection anchor (no selection yet). |
| Left drag in editor | Extend the selection from the anchor to the drag point (highlighted live). |
| Single click (press+release, no drag) | Move cursor; clear any selection. |

Menu/dialog/file-browser mouse behavior is unchanged.

## Non-regression
- Cursor movement, editing, search-match highlighting, and clipboard mechanics behave as before; this
  feature only makes selections visible and adds ways to create them.
