# Keybindings

The complete default keyboard reference for `edit`. Keybindings can be customized in
[`config.toml`](Configuration.md) and extended by [plugins](Plugin-Development.md) — note that the
safety-critical **Save** and **Quit** actions cannot be overridden by a plugin.

## File operations

| Key | Action |
|---|---|
| `Ctrl+S` / `F5` | Save current file |
| `F12` | Save As Encoding dialog (select output encoding) |
| `Ctrl+O` | Open file browser |
| `Ctrl+N` | New buffer |
| `Ctrl+W` | Close current buffer |
| `Ctrl+Q` | Quit (prompts if unsaved changes) |

> Save As… is reached from the **File** menu (it opens the Save file browser); it has no default
> key binding.

## Editing

| Key | Action |
|---|---|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+X` / `F8` | Cut selection |
| `Ctrl+C` / `F9` | Copy selection |
| `Ctrl+V` / `F11` | Paste |
| `Ctrl+Backspace` | Delete previous word (Feature 032) |
| `Ctrl+Delete` | Delete next word (Feature 032) |
| `Delete` | Delete character at cursor |
| `Backspace` | Delete character before cursor |
| `Tab` | Insert tab / indent selection |
| `Shift+Tab` | Dedent selection |

## Navigation

| Key | Action |
|---|---|
| Arrow keys | Move cursor |
| `Ctrl+Left` / `Ctrl+Right` | Move cursor by word — word left / word right (Feature 032) |
| `Home` | Beginning of line |
| `End` | End of line |
| `Ctrl+Home` | Beginning of file |
| `Ctrl+End` | End of file |
| `PgUp` / `PgDn` | Page up / page down |

## Selection

Selected text is shown highlighted (reverse video). Copy/Cut act on the selection; typing or pasting
replaces it; `Backspace`/`Delete` delete it; moving without `Shift` (or a single click) clears it.

| Key / action | Action |
|---|---|
| `Shift+Arrow` | Extend selection by a character / line |
| `Shift+Home` / `Shift+End` | Extend selection to line start / end |
| `Ctrl+Shift+Left` / `Ctrl+Shift+Right` | Extend selection by word (Feature 032) |
| `Ctrl+A` | Select all |

## Search and replace

| Key | Action |
|---|---|
| `Ctrl+F` | Open the interactive Find dialog (type the term in a bordered, labeled input with a caret; `Enter` searches; matches are highlighted, the view jumps to the current match, and "X of Y" is shown) |
| `Ctrl+H` | Open the interactive Replace dialog (find + replace-with fields, each a bordered, labeled input) |
| `F3` | Find next (wraps) |
| `F2` | Find previous (wraps) |
| `Ctrl+G` | Go to Line — type a 1-based line number and `Enter` to jump (out-of-range clamps; `Esc` cancels) |
| `Tab` | Replace dialog: switch between the find and replace fields |
| `Enter` | Find: search / advance · Replace: replace the current match and advance |
| `Ctrl+A` | Replace dialog: Replace All |
| `Alt+C` / `Alt+A` / `Alt+R` / `Alt+W` | Toggle case-sensitive / wrap-around / regex / whole-word (while a Find/Replace dialog is open) |
| `Esc` | Close the Find/Replace dialog and clear match highlights |

## Multi-file / buffers

| Key / action | Action |
|---|---|
| `F6` | Next buffer |
| `Shift+F6` | Previous buffer |
| `Ctrl+W` | Close current buffer |
| Mouse left-click on a tab | Switch to that buffer (tab bar, Feature 027) |
| Mouse left-click on a tab's `✕` | Close that buffer; a modified buffer prompts Save/Discard/Cancel |

With **2 or more buffers open**, a one-row **tab bar** appears below the menu bar listing each open
file (active tab highlighted, a `●` marks unsaved buffers). It is hidden with a single buffer.
Overflowing tabs truncate/scroll to keep the active tab visible.

## Mouse

| Action | Effect |
|---|---|
| Single click | Position the caret, clear selection |
| Press-drag-release | Select the dragged range |
| Double-click | Select the word under the pointer (Feature 030) |
| Triple-click | Select the whole line (Feature 030) |
| Right-click | Open the **Cut / Copy / Paste / Select All** context menu (Feature 030) |
| Click a list row | In dialogs (encoding select, plugin manager, file browser) selects that row (Feature 030) |
| Click in a text field | Position the caret at the clicked character (Find/Replace, file-browser Name, Go-to-Line — Feature 031) |
| Mouse wheel | Scroll whatever is under the cursor (editor viewport, file browser, Help/About, lists) — ~3 lines per notch; cursor unchanged (Feature 023) |
| Scrollbar click / drag | Click the track to page by a viewport; drag the thumb to scroll proportionally (Feature 024) |

The right-click context menu is operable by mouse or keyboard (`↑`/`↓`, `Enter`/`Space`, `Esc`).

## View

| Key | Action |
|---|---|
| `Alt+Z` | Toggle soft-wrap mode (non-DOS extension) |

## Menu activation

The pull-down menus are fully operable from the keyboard (Feature 009) and the mouse (Feature 011).

| Key | Action |
|---|---|
| `F10` | Activate menu bar (highlight first menu, no dropdown) |
| `Alt` (tapped alone) | Activate menu bar like `F10` — terminal-permitting (Feature 013) |
| `Alt+F` | Open File menu (dropdown) |
| `Alt+E` | Open Edit menu (dropdown) |
| `Alt+S` | Open Search menu (dropdown) |
| `Alt+V` | Open View menu (dropdown) |
| `Alt+O` | Open Options menu (dropdown) |
| `Alt+H` | Open Help menu (dropdown) |
| accelerator letter | Each menu title and item shows one **underlined** accelerator letter; while the bar is active it opens that menu, while a dropdown is open it activates that item (case-insensitive) |
| `←` / `→` | Move between top-level menus (wraps; opens the adjacent dropdown) |
| `↑` / `↓` | Move between items within the open dropdown (wraps) |
| `Enter` | Activate the highlighted menu item |
| `Esc` | Close menu / cancel dialog |
| Mouse left-click | Click a top-level title to open it; click a dropdown item to activate it; click outside to close |

Plugin-contributed top-level menus appear in the menu bar **between Options and Help** and are
navigable and activatable with the same keys and the mouse.

## Help

| Key | Action |
|---|---|
| `F1` | Open the Help cheat sheet |

## Dialogs (confirm / dismiss prompts)

| Key / action | Effect |
|---|---|
| `Tab` / `Shift+Tab` | Move focus between the dialog's boxed buttons (wraps) |
| `←` / `→` / `↑` / `↓` | Move focus between buttons (same as `Tab`/`Shift+Tab`) (Feature 028) |
| `Enter` / `Space` | Activate the focused button |
| letter shortcuts (e.g. `S`/`D`/`C`, `Y`/`N`) | Choose directly |
| Mouse left-click | Click a button to activate it; click outside to cancel (where a safe cancel exists) |
| `Esc` | Cancel / close the dialog |

Each dialog opens focused on its safe default (Cancel/No/Keep for destructive prompts). The
encoding selector, plugin manager, Find/Replace, and file browser use a combined focus ring (the
list/field group is the first stop, then each button).

## Plugin manager dialog (Options › Plugins)

| Key | Action |
|---|---|
| `Up` / `Down` | Navigate the plugin list |
| `Space` | Toggle the selected plugin on/off |
| `Esc` | Close the dialog |

## Consent dialog (first run of a new plugin)

| Key | Action |
|---|---|
| `Enter` | Allow the plugin |
| `Esc` | Deny the plugin |
