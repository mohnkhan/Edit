# Phase 1 Data Model: Interactive Find and Replace

## `FindReplaceDialog` (new) — `src/app.rs` (or `src/ui/dialog.rs` model)

| Field | Type | Notes |
|---|---|---|
| `mode` | `DialogMode` (`Find` \| `Replace`) | which dialog is open |
| `query` | `String` | the search-term field text (UTF-8) |
| `replacement` | `String` | the replace-with field text (Replace mode) |
| `focus` | `Field` (`Query` \| `Replacement`) | which field the caret edits (Tab toggles; always `Query` in Find) |
| `caret` | `usize` | grapheme index of the caret within the focused field |
| `case_sensitive` | `bool` | option toggle (default false) |
| `wrap` | `bool` | option toggle (default true) |
| `regex` | `bool` | option toggle (default false) |
| `whole_word` | `bool` | option toggle (default false) |

Held by `App` as `pending_find_replace: Option<FindReplaceDialog>` (`None` = closed). Field editing is
grapheme-aware (insert/backspace/left/right operate on grapheme boundaries — reuse the file-browser
field conventions).

## `SearchState` (extended) — `src/search/mod.rs`

| Field | Type | Notes |
|---|---|---|
| existing fields | … | `query, replacement, regex_mode, case_sensitive, wrap, direction, matches, active_match` unchanged |
| `whole_word` | `bool` | **new** — restrict matches to whole words (word-boundary aware) |

`SearchEngine::find_all` gains whole-word filtering: a candidate match `[s, e)` is kept only when the
char before `s` and the char at `e` are both non-word characters (`!(is_alphanumeric() || '_')`), with
document boundaries treated as non-word. Applied to both plain-text and regex candidates. Char-index
based → UTF-8 safe.

## Derived: match highlight feed to the editor

`EditorWidget` gains (optional) `match_ranges: &[CharRange]` and `active_match: Option<usize>`. The
renderer overlays the match background on cells within each range (current match uses the distinct
active style from `collect_match_spans`). Empty/absent → no overlay (no behavior change when no search
is active).

## State transitions

- **Open Find** (`Ctrl+F` / Search ▸ Find): set `pending_find_replace = Some(Find{...})`, seed `query`
  from the last search if any, caret at end.
- **Open Replace** (`Ctrl+H` / Search ▸ Find Replace): `Some(Replace{...})`.
- **Enter (Find)**: copy dialog query/options into `SearchState`, run `find_all`, set `active_match` to
  the first at/after cursor, `scroll_to_match`; if empty → "not found", no document change.
- **Enter (Replace)**: replace current match (undoable edit), recompute, advance.
- **Ctrl+A (Replace)**: `replace_all`, report count, recompute.
- **F3 / F2**: `find_next` / `find_prev` (respect `wrap`).
- **Esc**: `pending_find_replace = None`; clear active highlights (search results no longer shown).
- **Any document change** (incl. replace): recompute matches against current content (FR-012).

## Invariants

- While `pending_find_replace.is_some()`, keystrokes edit the dialog, never the buffer (FR-009/FR-013).
- `active_match`, when `Some`, is always a valid index into `matches`; the "X of Y" indicator =
  (`active_match + 1`, `matches.len()`).
- All field edits and match offsets respect grapheme/char boundaries (FR-011).
- Replace operations go through the normal edit/undo path (FR-008; integrates with feature 014 clean
  tracking).
