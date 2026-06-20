# Getting Started

This page walks you through your first session with `edit`: launching it, touring the DOS-style
interface, and running the open → edit → save → quit loop.

## First launch

```sh
# A blank buffer
edit

# Open an existing file
edit notes.txt
```

If you ran `edit` previously and exited cleanly, you may be greeted by a **session restore** prompt
offering to reopen your last set of files. Press `Y`/`Enter` to restore or `N`/`Esc` to start fresh.
To skip this prompt entirely, launch with `--no-session`. (Passing explicit file arguments also
bypasses session restore.)

## The UI tour

`edit` recreates the classic EDIT.COM screen. From top to bottom:

```
┌──────────────────────────────────────────────────────────────────────────┐
│ File  Edit  Search  View  Options  Help                                    │  ← Menu bar
├──────────────────────────────────────────────────────────────────────────┤
│ Hello, world!                                                              │
│ This is the editor area on a DOS-blue background.                          │
│ Unicode works everywhere: café, αβγ, 日本語, 🙂                             │
│ █                                                                          │  ← Cursor
│                                                                            │
│                                                                            │
│                                                                            │
│                                                                            │
├──────────────────────────────────────────────────────────────────────────┤
│ notes.txt   UTF-8   LF   Ln 4, Col 1   [+]                                  │  ← Status bar
└──────────────────────────────────────────────────────────────────────────┘
```

- **Menu bar** (top) — `File`, `Edit`, `Search`, `View`, `Options`, `Help`. Press `F10` to activate
  it, or `Alt+<first letter>` to drop a menu open directly (e.g. `Alt+F` for File). Plugin-provided
  menus appear between `Options` and `Help`. Menus are fully mouse-operable — click a title to open
  it and click an item to run it.
- **Buffer tab bar** (below the menu bar) — appears when **two or more buffers are open**: a one-row
  strip listing each open file (active tab highlighted, a `●` marks unsaved buffers). Click a tab to
  switch to it, or click its `✕` to close it. With a single buffer there is no tab bar.
- **Editor area** (middle) — your text, on the classic blue background under the `classic` theme.
  An optional line-number gutter can be enabled. The current selection is shown highlighted (reverse
  video), and scrollbars appear when the content overflows.
- **Status bar** (bottom) — filename, current encoding (e.g. `UTF-8`), line-ending style (`LF` /
  `CRLF`), cursor position, and transient indicators such as `[WRAP]` (soft-wrap on) or a `[+]`/dirty
  marker for unsaved changes. Search results, save confirmations, and plugin messages also flash
  here.

## Mouse support

`edit` is fully mouse-operable (on any terminal that reports mouse events; everything also works from
the keyboard). Click to position the cursor, **press-drag-release to select**, double-click to select a
word, triple-click to select a line, and right-click for a Cut / Copy / Paste / Select All context menu.
The wheel scrolls the editor, file browser, Help/About, and list dialogs, and the scrollbars are
clickable and draggable. Mouse support can be turned off with `mouse = false` in `config.toml`.

## Basic workflow

### Open a file

- `Ctrl+O` opens the file dialog, or
- pass the filename on the command line: `edit path/to/file`.

### Edit text

Type to insert. Editing is grapheme-aware, so multi-codepoint characters behave as single units.

| Action | Key |
|---|---|
| Undo / Redo | `Ctrl+Z` / `Ctrl+Y` |
| Cut / Copy / Paste | `Ctrl+X` / `Ctrl+C` / `Ctrl+V` |
| Select all | `Ctrl+A` |
| Extend selection | `Shift+Arrow` |
| Extend selection by word | `Ctrl+Shift+Left` / `Ctrl+Shift+Right` |
| Delete previous / next word | `Ctrl+Backspace` / `Ctrl+Delete` |
| Indent / Dedent selection | `Tab` / `Shift+Tab` |

### Navigate

Arrow keys move the cursor. `Home`/`End` jump to line start/end; `Ctrl+Home`/`Ctrl+End` to
file start/end; `Ctrl+Left`/`Ctrl+Right` move by word; `PgUp`/`PgDn` page through the buffer.

### Find & replace

- `Ctrl+F` — Find (interactive dialog; regex supported, matches highlighted, "X of Y" indicator)
- `F3` / `F2` — Find next / previous (wraps)
- `Ctrl+H` — Find and Replace (interactive dialog)
- `Ctrl+G` — Go to Line (type a 1-based line number and press `Enter` to jump)

### Save

- `Ctrl+S` / `F5` — Save
- **File › Save As…** — save to a new path (via the file browser)
- `F12` — Save As Encoding (choose the output encoding from a dialog; see [Encodings](Encodings.md))

### Quit

- `Ctrl+Q` — Quit. If the buffer has unsaved changes you'll be prompted before exiting.

## Next steps

- The complete key map: [Keybindings](Keybindings.md)
- Customize themes and behavior: [Configuration](Configuration.md)
- Work with legacy code pages and UTF-16: [Encodings](Encodings.md)
