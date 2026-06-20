# Phase 0 Research: Interactive Find and Replace

## R1. Dialog state shape and focus model

**Decision**: A single `App` field `pending_find_replace: Option<FindReplaceDialog>` holding: `mode`
(Find | Replace), `query: String`, `replacement: String`, `focus` (QueryField | ReplaceField),
`caret` (grapheme index within the focused field), and the option toggles (`case_sensitive`, `wrap`,
`regex`, `whole_word`). One struct serves both dialogs (Replace just also shows/edits `replacement` and
the second field/focus). `None` = no dialog open.

**Rationale**: Mirrors the existing single-modal pattern (`pending_encoding_select`, file browser). One
struct avoids duplicated intercept/render logic and keeps Find a strict subset of Replace.

**Alternatives**: Two separate `Option` fields — more duplicated routing. Rejected.

## R2. Input routing and modal precedence

**Decision**: Add an intercept block in `handle_action` alongside the other modal guards (after the
higher-priority confirm modals, consistent with file-browser placement). While open it consumes input:
printable chars insert into the focused field at the caret; Backspace deletes; Left/Right move the
caret; Tab switches focus (Replace only); Enter runs the action (Find: run/advance search; Replace:
replace current + advance); Ctrl+A = Replace All (Replace mode); F3/F2 = next/prev; Esc closes and
clears highlights; option toggles via Alt+C (case), Alt+W (wrap), Alt+G (regex), Alt+O (whole-word).
Mouse is ignored while open (added to the mouse modal guard list).

**Rationale**: Reuses the established modal-intercept-then-`return Ok(())` pattern; keeps all dialog keys
from leaking into the buffer (FR-009/FR-013). Alt+letter toggles are safe because the dialog consumes
input before menu handling.

## R3. Whole-word matching in the engine

**Decision**: Add `whole_word: bool` to `SearchState` and a `whole_word` parameter to the find path. In
`SearchEngine::find_all`, after computing candidate matches (plain or regex), filter to those whose
immediately preceding and following characters are not word characters (`char::is_alphanumeric` or `_`).
Boundaries at start/end of document count as non-word. Operates on char indices so it is UTF-8 safe.

**Rationale**: Word-boundary post-filtering is simple, correct for both plain and regex candidates, and
keeps the engine's existing match-finding intact. The only option not already supported.

**Alternatives**: Wrapping the query in `\b…\b` regex — only works in regex mode and changes plain-text
semantics; rejected in favor of a uniform post-filter.

## R4. Match-highlight rendering integration

**Decision**: Pass the active match ranges and current-match index to `EditorWidget` (new optional
fields, e.g. `match_ranges: &[CharRange]`, `active_match: Option<usize>`). During the per-line render,
the widget computes each visible line's char span and, for cells inside a match, overlays the match
background (current match uses the distinct "active" style from `collect_match_spans`). Highlights are
shown only while a search is active (dialog open or results present) and cleared when Find closes.

**Rationale**: Reuses `collect_match_spans` styles; the editor already iterates cells per line for
syntax highlighting, so adding a background overlay is a localized change. Char-index spans keep it
UTF-8-correct.

**Alternatives**: Precomputing per-line spans in `App` and passing them down — also viable; chosen
approach keeps the data minimal and the conversion in one place (the renderer that already walks cells).

## R5. Recompute strategy (no stale offsets)

**Decision**: Recompute matches (`find_all` + re-clamp `active_match`) whenever the query, any option,
or the document changes — specifically after running a search, after each Replace, and after Replace
All. After a Replace, recompute against the edited document and set the current match to the next
occurrence at/after the replaced position. This guarantees highlights/counts never reference stale
positions (FR-012).

**Rationale**: `find_all` is linear and documents are modest; recomputing is simpler and safer than
incrementally shifting offsets, eliminating an entire class of off-by-offset bugs.

## R6. Replace semantics and undo

**Decision**: "Replace current" replaces the current match range via the normal edit path (so it is
recorded in the undo history and marks the buffer modified — consistent with feature 014), then
recomputes and advances. "Replace All" uses the existing `replace_all`, reports the count, and records
the change as a single undoable edit (composite) where the engine/edit layer supports it; otherwise as
the existing implementation does. Both report the number replaced in the status bar.

**Rationale**: Replace must be undoable as normal edits (FR-008) and update the clean/dirty state
correctly (feature 014 integration). Reusing the edit/undo path gives this for free.

## R7. In-dialog option toggle keys

**Decision**: Bind four new toggle actions to **free** Alt keys: `Alt+C` = case-sensitive, `Alt+A` =
wrap-around, `Alt+R` = regex, `Alt+W` = whole-word. They must be added to the keymap as distinct
`Action` variants (`ToggleSearch*`) because an unbound `Alt+<letter>` produces **no** action in
`dispatch_key` (the char fallback only fires for no-modifier / Shift), so without a binding the dialog
would never receive them. The dialog intercept handles these actions (toggle + re-search); outside an
open dialog they are inert no-ops. The dialog renders each option's on/off state (e.g. `[x] Case`).

**Rationale**: `Alt+C/A/R/W` are all currently unbound (bound Alt letters are E/F/H/O/S/V/Z), so they
avoid collisions — notably `Alt+O` is already Options and is **not** used. Routing through real `Action`
variants is the only way the keystrokes reach the modal intercept.
