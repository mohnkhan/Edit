# Contract: File dialog — glob filtering + richer entry details

Behavioral contract the tests assert against.

## Filter interpretation (`apply_filter`)

| Field text | Effect on listing |
|---|---|
| empty | full listing (no filter) |
| absolute path (`/…`) | **no filter**; remains a jump/open target on confirm (feature-012 behavior) |
| contains `*` or `?` | glob-match file names (case-insensitive); dirs + `..` always kept |
| other (plain text) | substring-match file names (case-insensitive); dirs + `..` always kept |

- Live: applied on every field edit (`push_char`/`backspace`), not only on Enter.
- After filtering, `selected` is clamped to a visible row and `scroll` reset/clamped.
- A no-match filter still lists `..` and sub-directories.

## `glob_match(pattern, name)`

- `*` matches any run (incl. empty); `?` matches exactly one character; other chars match literally.
- Case-insensitive; whole-name anchored (`*.log` matches `a.log`, not a substring like `xlogy`).
- No character classes / no regex.

## Detail columns

- File row: human-readable **size** + **modified date**, aligned in fixed right-hand columns.
- Dir / `..` row: `<DIR>` in the size column (no byte size); a date may still be shown.
- Name column takes remaining width and truncates with `…` (grapheme/width-correct); detail columns are
  not truncated. On a too-narrow row, detail columns are dropped (name-only) without corruption.
- `human_size`: `0..1024` → `NB`; `K`/`M`/`G` above, ≤1 decimal for sub-10 magnitudes.
- `format_mtime`: `YYYY-MM-DD HH:MM` (UTC); blank when metadata is unavailable.

## No-regression (feature-012/020/021)

- Arrow keys, parent/enter, typing into the field, mouse entry click/double-click all behave as before
  (now over the filtered `entries`).
- Open confirm: absolute path jumps/opens; otherwise opens the selected (filtered) entry.
- Save confirm: saves the typed filename (unchanged) — filtering is a display aid only.
- Feature-020 Open/Save/Cancel buttons + focus ring and feature-021 scrollbar keep working with a filter
  active; the scrollbar reflects the **filtered** count.
- No panic across empty dirs, no-match filters, unreadable metadata, multi-byte names, tiny terminals.
