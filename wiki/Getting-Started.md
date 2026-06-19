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
  menus appear between `Options` and `Help`.
- **Editor area** (middle) — your text, on the classic blue background under the `classic` theme.
  An optional line-number gutter can be enabled.
- **Status bar** (bottom) — filename, current encoding (e.g. `UTF-8`), line-ending style (`LF` /
  `CRLF`), cursor position, and transient indicators such as `[WRAP]` (soft-wrap on) or a `[+]`/dirty
  marker for unsaved changes. Search results, save confirmations, and plugin messages also flash
  here.

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
| Indent / Dedent selection | `Tab` / `Shift+Tab` |

### Navigate

Arrow keys move the cursor. `Home`/`End` jump to line start/end; `Ctrl+Home`/`Ctrl+End` to
file start/end; `Ctrl+Left`/`Ctrl+Right` move by word; `PgUp`/`PgDn` page through the buffer.

### Find & replace

- `Ctrl+F` — Find (regex supported, matches highlighted)
- `F3` / `Shift+F3` — Find next / previous
- `Ctrl+H` — Find and Replace

### Save

- `Ctrl+S` — Save
- `Ctrl+Shift+S` — Save As
- `F12` — Save As Encoding (choose the output encoding from a dialog; see [Encodings](Encodings.md))

### Quit

- `Ctrl+Q` — Quit. If the buffer has unsaved changes you'll be prompted before exiting.

## Next steps

- The complete key map: [Keybindings](Keybindings.md)
- Customize themes and behavior: [Configuration](Configuration.md)
- Work with legacy code pages and UTF-16: [Encodings](Encodings.md)
