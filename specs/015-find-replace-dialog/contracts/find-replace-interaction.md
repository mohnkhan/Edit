# Contract: Find / Replace interaction

"Dialog open" means `App.pending_find_replace.is_some()`.

## Entry

| Trigger | Result |
|---|---|
| `Ctrl+F` / Search ▸ Find | Open the Find dialog (single query field). |
| `Ctrl+H` / Search ▸ Find Replace | Open the Replace dialog (query + replacement fields). |

## Keyboard contract (while a dialog is open; all other input consumed)

| Key | Effect |
|---|---|
| printable char | Insert at the caret of the focused field (UTF-8/grapheme-correct). |
| `Backspace` | Delete the grapheme before the caret in the focused field. |
| `Left` / `Right` | Move the caret within the focused field. |
| `Tab` | Replace mode: switch focus between query and replacement fields. |
| `Enter` | Find: run the search / advance to the current match. Replace: replace the current match and advance. |
| `Ctrl+A` | Replace mode: **Replace All** (Select-All only applies outside the dialog). |
| `F3` / `F2` | Find Next / Find Previous (respect wrap). |
| `Alt+C` / `Alt+A` / `Alt+R` / `Alt+W` | Toggle case-sensitive / wrap-around / regex / whole-word; re-running reflects new state. (New toggle actions bound to these free Alt keys; inert outside the dialog.) |
| `Esc` | Close the dialog; clear active match highlights; return to editing. |

## Search / result contract

- On a successful Find: all matches highlighted; view+cursor moved to the current match (first at/after
  the cursor); the dialog shows an "X of Y" indicator.
- No matches: "not found" shown; document and cursor unchanged; navigation keys inert.
- The current match is highlighted distinctly from non-current matches.
- Empty query: Enter is a no-op (no matches, no error).
- Invalid regex (regex on): reported; no matches; no crash.
- Matches are recomputed after any document change (including each Replace) so highlights/counts are
  never applied to stale offsets.

## Replace contract

- **Replace (Enter)**: replaces the current match via the normal edit path (undoable; marks buffer
  modified), recomputes, advances to the next match.
- **Replace All (Ctrl+A)**: replaces every occurrence in one operation; reports the count; recomputes
  (zero remaining occurrences of the option-matched term).
- A single Undo restores the pre-replace document.

## Modality / non-regression

- While open, the dialog is modal: keyboard edits the fields, mouse is ignored, the buffer is untouched
  except by an explicit Replace/Replace-All.
- With no dialog open, all keys (including `Ctrl+A` = Select All) behave exactly as before.
- Highlights appear only for an active search and are cleared when Find closes.
