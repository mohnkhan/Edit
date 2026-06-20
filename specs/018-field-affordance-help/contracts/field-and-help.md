# Contract: Input-field affordance + Help table

## File-dialog input box
- Both Open and Save modes render a **bordered input box** with a label and an always-visible caret.
- Save: label "Name:", text = the filename being typed. Open: label "Go to path:", text = the jump path
  (this field is now visible; previously it was not drawn).
- Typing/backspace update the box text + caret; the confirm key uses the typed value exactly as before.
- Long text is shown within the box without breaking layout (truncate/scroll inside, grapheme-correct).
- Entry list + navigation/selection are unchanged aside from the list being shorter to fit the box.

## Help screen
- Shortcuts render as a grouped, aligned two-column **Key | Action** table (sections: File, Edit, Search,
  Selection, View, Menus, Dialogs).
- The box fits the terminal; when there are more rows than fit, the screen is **scrollable**
  (`Up`/`Down`/`PageUp`/`PageDown`) with a visible "more" cue; no row is silently truncated.
- `Esc` (and existing dismiss keys) close Help; it stays modal.
- About is unchanged in content.

## Non-regression
- File-dialog browse/select, search/selection highlighting, and all other dialogs behave as before; no
  panic at the minimum terminal size or smaller.
