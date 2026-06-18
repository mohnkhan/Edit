# Data Model: Linux EDIT.COM Clone

**Feature**: `specs/001-linux-editcom-clone`
**Phase**: 1 — Design
**Date**: 2026-06-18

---

## Entity: Buffer

The central in-memory representation of an open file.

| Field | Type | Description |
|-------|------|-------------|
| `path` | `Option<PathBuf>` | Absolute path on disk; `None` for a new unsaved buffer |
| `rope` | `Rope` | File contents as a rope (UTF-8 grapheme clusters) |
| `encoding` | `EncodingProfile` | Encoding used when reading/writing to disk |
| `line_ending` | `LineEnding` | Dominant line ending detected on open (`LF` or `CRLF`) |
| `modified` | `bool` | True if buffer has unsaved changes |
| `readonly` | `bool` | True if opened with `--readonly` or file is not writable |
| `cursor` | `CursorPos` | Current cursor position (line index, grapheme column) |
| `scroll_offset` | `(usize, usize)` | Top-left visible position `(line, visual_col)`. The `.1` element is the **horizontal scroll column** used for long lines wider than the terminal (see "Long-line handling" below) |
| `selection` | `Option<Selection>` | Active text selection (anchor + cursor) |
| `undo_stack` | `UndoStack` | Ordered list of reversible edit operations |
| `autosave` | `AutosaveState` | Countdown timer, recovery file path, lock file state |
| `syntax` | `Option<Box<dyn Highlighter>>` | Active syntax highlighter, `None` if disabled |

**Validation rules**:
- `rope` content MUST be valid UTF-8 at all times; raw bytes are transcoded before insertion.
- `cursor.line` MUST be < number of lines in `rope`.
- `cursor.col` MUST be ≤ grapheme length of the current line.

**State transitions**:
```
new (unmodified) → edit → modified
modified → save → unmodified
modified → autosave timer → recovery file written (buffer remains modified)
any → clean quit → recovery file deleted
any → abnormal exit → lock file + recovery file remain (detected on next open)
```

**Long-line handling**: Lines wider than the terminal use **horizontal scrolling**, not
soft-wrap — matching MS-DOS EDIT.COM behavior. When the cursor moves past the right edge
of the viewport, `scroll_offset.1` advances so the cursor remains visible. No glyph is ever
wrapped to a second screen row. (Rationale: EDIT.COM never soft-wrapped; horizontal scroll
preserves the 1:1 line-to-row mapping that the status bar row/col indicator depends on.)

---

## Enum: LineEnding

| Variant | On-disk bytes | Description |
|---------|---------------|-------------|
| `Lf`    | `\n` (0x0A)   | Unix line ending (default for new buffers) |
| `Crlf`  | `\r\n` (0x0D 0x0A) | DOS/Windows line ending |

**Detection**: On `Buffer::open`, the first 512 bytes are scanned. If any `\r\n` pair is
found, `line_ending = Crlf`; otherwise `Lf`. The internal rope ALWAYS stores `\n`-only
lines (CR is stripped on load).

**Persistence**: On `Buffer::save`, if `line_ending == Crlf`, each `\n` is re-emitted as
`\r\n`. A new buffer defaults to `Lf`. The user may override via Options > Line Endings;
the override updates this field and is honored on the next save.

---

## Enum: BufferError

Errors returned by `Buffer::open` / `Buffer::save`, surfaced to the user via dialogs:

| Variant | Trigger | User-visible behavior |
|---------|---------|-----------------------|
| `BinaryContent` | A null byte (`0x00`) found in the first 512 bytes on open | `ErrorDialog` "Binary file — cannot edit"; buffer not opened |
| `DecodeError { byte_offset }` | Byte sequence invalid for the declared encoding | `ErrorDialog` naming the offset; offending bytes replaced with U+FFFD if user chooses "Open anyway" |
| `Io(std::io::Error)` | OS error on read/write (permission denied, disk full, ENOSPC) | `SaveErrorDialog` "Cannot save: <error>. Retry / Cancel"; never falls back to privilege elevation |
| `EncodeError` | Active buffer contains characters not representable in the target legacy encoding on save | `ErrorDialog` offering "Save as UTF-8 instead / Cancel" |

---

## Entity: CursorPos

| Field | Type | Description |
|-------|------|-------------|
| `line` | `usize` | Zero-based line index in the rope |
| `grapheme_col` | `usize` | Zero-based grapheme cluster index within the line |
| `visual_col` | `usize` | Zero-based visual (display) column, accounting for wide chars |

`visual_col` is derived from `grapheme_col` by summing `unicode_width` of each cluster
up to `grapheme_col`. It is cached and invalidated on every edit.

---

## Entity: Selection

| Field | Type | Description |
|-------|------|-------------|
| `anchor` | `CursorPos` | Fixed end of selection (set when selection starts) |
| `active` | `CursorPos` | Moving end of selection (tracks cursor movement) |

The selected region spans from `min(anchor, active)` to `max(anchor, active)`. Cut/copy
reads this region; paste replaces it if active.

---

## Entity: UndoStack

| Field | Type | Description |
|-------|------|-------------|
| `ops` | `Vec<EditOp>` | Chronological list of reversible operations |
| `cursor` | `usize` | Index of next undo position (points after last committed op) |

**EditOp** (enum):
- `Insert { at: CharIdx, text: String }` — inserted `text` starting at `at`
- `Delete { at: CharIdx, text: String }` — deleted `text` starting at `at`
- `Composite(Vec<EditOp>)` — grouped ops (e.g., replace-all) that undo as one unit

Undo: decrement `cursor`, apply inverse of `ops[cursor]`.
Redo: apply `ops[cursor]`, increment `cursor`.
Any new edit truncates `ops` at `cursor` (redo history discarded).

---

## Entity: AutosaveState

| Field | Type | Description |
|-------|------|-------------|
| `interval_secs` | `u32` | Seconds between auto-saves (default: 30; user-configurable) |
| `last_save_at` | `Instant` | Monotonic timestamp of last auto-save write |
| `recovery_path` | `PathBuf` | Derived from SHA-256 of buffer's absolute file path |
| `lock_path` | `PathBuf` | Lock file path (`recovery_path` + `.lock`) |
| `enabled` | `bool` | False when `--no-autosave` flag set |

**Invariant**: `lock_path` exists iff the buffer is currently open in this session.
Its absence at startup with a present `recovery_path` signals an abnormal previous exit.

---

## Entity: EncodingProfile

| Field | Type | Description |
|-------|------|-------------|
| `name` | `&'static str` | Human-readable name (`"UTF-8"`, `"CP437"`, `"ISO-8859-1"`, …) |
| `id` | `EncodingId` | Enum variant used to select the transcoder |
| `bom` | `Option<&'static [u8]>` | BOM byte sequence, if any (UTF-8: `[0xEF, 0xBB, 0xBF]`) |

Supported `EncodingId` values: `Utf8`, `Cp437`, `Cp850`, `Iso8859_1`, `Windows1252`.

**Detection priority**: explicit `--encoding` flag > BOM > heuristic (chardetng) > UTF-8 default.

---

## Entity: Configuration

Loaded from `$XDG_CONFIG_HOME/edit/config.toml` at startup; merged with CLI flags
(flags take precedence).

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `default_encoding` | `String` | `"utf-8"` | Encoding assumed for new files |
| `theme` | `String` | `"classic"` | Color theme (`classic`, `high-contrast`, `plain`) |
| `autosave_interval` | `u32` | `30` | Seconds between recovery file writes |
| `line_numbers` | `bool` | `false` | Show line numbers in gutter |
| `highlight` | `bool` | `true` | Syntax highlighting enabled |
| `mouse` | `bool` | `true` | Mouse event handling enabled |
| `log_level` | `String` | `"warn"` | Log verbosity (`error`, `warn`, `info`, `debug`) |
| `keybindings` | `KeybindingMap` | EDIT.COM defaults | See `contracts/config.md` |

**Validation**: Unknown keys log a warning and are ignored. Type mismatches log an error
and the field reverts to its default. The editor never refuses to start due to a bad config.

---

## Entity: KeybindingMap

A flat mapping from key sequence string to editor action name. Loaded from the
`[keybindings]` section of the config file and merged over the compiled-in defaults.

| Key (string) | Action (string) |
|-------------|-----------------|
| `"Ctrl+S"` | `"save"` |
| `"Ctrl+Q"` | `"quit"` |
| `"Ctrl+X"` | `"cut"` |
| `"Ctrl+C"` | `"copy"` |
| `"Ctrl+V"` | `"paste"` |
| `"Ctrl+Z"` | `"undo"` |
| `"Ctrl+Y"` | `"redo"` |
| `"Ctrl+F"` | `"find"` |
| `"F1"` | `"help"` |
| `"F3"` | `"find_next"` |
| `"F5"` | `"save"` |
| `"F10"` | `"menu"` |
| `"Alt+F"` | `"menu_file"` |
| `"Alt+E"` | `"menu_edit"` |
| `"Alt+S"` | `"menu_search"` |
| `"Alt+V"` | `"menu_view"` |
| `"Alt+O"` | `"menu_options"` |
| `"Alt+H"` | `"menu_help"` |

**Conflict rule**: If two entries map to the same key, the user entry wins; a warning is
logged naming both the key and the shadowed action.

---

## Entity: SearchState

| Field | Type | Description |
|-------|------|-------------|
| `query` | `String` | Current search string or regex pattern |
| `replacement` | `Option<String>` | Replacement string; `None` when in find-only mode |
| `regex_mode` | `bool` | Interpret `query` as a regex pattern |
| `case_sensitive` | `bool` | Case-sensitive matching |
| `wrap` | `bool` | Wrap search from end of file back to start |
| `direction` | `Direction` | `Forward` or `Backward` |
| `matches` | `Vec<CharRange>` | Cached match positions in the current buffer |
| `active_match` | `Option<usize>` | Index into `matches` of the highlighted match |

**Invariant**: `matches` is invalidated (cleared) on every buffer edit. It is lazily
recomputed on the next `find_next` or `find_prev` call.

---

## Entity: RecoveryFile (on-disk format)

See `contracts/recovery.md` for the binary/text format specification.

Key fields stored in the recovery file header:
- Original file path (UTF-8)
- Encoding profile name
- Buffer content length (bytes)
- Auto-save timestamp (Unix epoch seconds)
- Buffer content (UTF-8 text of the rope at auto-save time)

---

## Entity: Theme

| Field | Type | Description |
|-------|------|-------------|
| `name` | `&'static str` | Theme identifier |
| `background` | `Color` | Main editing area background |
| `foreground` | `Color` | Main editing area text |
| `menubar_bg` | `Color` | Menu bar background |
| `menubar_fg` | `Color` | Menu bar text |
| `menu_selected_bg` | `Color` | Selected menu item background |
| `status_bg` | `Color` | Status bar background |
| `status_fg` | `Color` | Status bar text |
| `highlight_keyword` | `Color` | Syntax: keywords |
| `highlight_string` | `Color` | Syntax: string literals |
| `highlight_comment` | `Color` | Syntax: comments |
| `highlight_number` | `Color` | Syntax: numeric literals |

Built-in themes: `classic` (blue background, white text — DOS EDIT.COM faithful),
`high-contrast` (black background, bright white text, yellow keywords),
`plain` (terminal defaults, no color override).
