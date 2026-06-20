# Feature Specification: Interactive Find and Replace dialogs

**Feature Branch**: `015-find-replace-dialog`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "Interactive Find and Replace dialogs. Today the Search ▸ Find and Search ▸ Find Replace menu items are stubs … Build a working interactive Find dialog (Ctrl+F): type a search term, Enter to find, matches highlighted, view jumps to current match; F3/F2 next/prev with wrap and an 'X of Y' indicator; Esc closes. Also a working Replace dialog (Ctrl+H): find + replace-with fields, Replace-current and Replace-All, reporting how many replacements. The SearchEngine, navigation, replace_all, and highlight styles already exist; missing are the interactive dialog UI, input routing, wiring, and rendering match highlights. Options to consider: case-sensitive, whole-word, regex, wrap. Must be UTF-8 correct."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Find text and jump to it (Priority: P1)

A user opens the Find dialog (Ctrl+F or Search ▸ Find), types a search term into an input box, and
presses Enter. The editor finds all occurrences, highlights them in the document, and moves the view
and cursor to the first match at/after the cursor. A counter shows which match is current and how many
there are (e.g. "3 of 12"). If there are no matches, the dialog says so and the document is unchanged.
Pressing Esc closes the dialog.

**Why this priority**: This is the core missing capability — there is currently no way to type a search
term at all. Without it, Find is unusable. It delivers the primary value on its own.

**Independent Test**: Open a document, Ctrl+F, type a word that occurs, Enter → matches highlight and
the view jumps to the first one with a correct "X of Y" count. Fully testable end-to-end.

**Acceptance Scenarios**:

1. **Given** a document containing the term, **When** the user opens Find, types it, and presses Enter,
   **Then** all occurrences are highlighted, the view jumps to the first match at/after the cursor, and
   the current/total count is shown.
2. **Given** a search term with no occurrences, **When** the user presses Enter, **Then** a "not found"
   message is shown and the document/cursor are unchanged.
3. **Given** the Find dialog is open, **When** the user presses Esc, **Then** the dialog closes and the
   user returns to editing.
4. **Given** a multi-byte/Unicode search term, **When** the user types and searches, **Then** the input
   field and the match positions are correct (no split characters, correct highlight spans).

### User Story 2 - Cycle through matches (Priority: P1)

After a search, the user presses Find Next (F3) and Find Previous (F2) to move to the next/previous
occurrence, wrapping around the ends of the document. The "X of Y" indicator and the highlighted
"current" match update each step.

**Why this priority**: Finding the first match is rarely enough; stepping through results is the other
half of a usable Find. Independently testable once US1 exists.

**Acceptance Scenarios**:

1. **Given** several matches with one current, **When** the user presses Find Next, **Then** the current
   match advances to the next occurrence and the view follows.
2. **Given** the current match is the last one, **When** the user presses Find Next, **Then** it wraps to
   the first match (and the indicator reflects the wrap).
3. **Given** the current match is the first one, **When** the user presses Find Previous, **Then** it
   wraps to the last match.
4. **Given** matches exist, **When** the current match changes, **Then** the "current" highlight is
   visually distinct from the other match highlights.

### User Story 3 - Replace occurrences (Priority: P2)

The user opens the Replace dialog (Ctrl+H or Search ▸ Find Replace), enters a search term and a
replacement, and can move focus between the two fields. **Replace** replaces the current match and
advances to the next; **Replace All** replaces every occurrence at once. The editor reports how many
replacements were made. The buffer becomes modified accordingly and the change is undoable.

**Why this priority**: Replace builds on Find and is the explicit second half of the request, but Find
alone is already valuable, so this is P2.

**Acceptance Scenarios**:

1. **Given** the Replace dialog with a term and replacement, **When** the user chooses Replace All,
   **Then** every occurrence is replaced and the count of replacements is reported.
2. **Given** a current match, **When** the user chooses Replace, **Then** that occurrence is replaced,
   the view advances to the next match, and remaining matches/indicator update.
3. **Given** replacements were made, **When** the user undoes, **Then** the document returns to its
   prior state (replace is undoable, consistent with other edits).
4. **Given** a term with no occurrences, **When** the user chooses Replace/Replace All, **Then** nothing
   changes and the editor reports zero replacements.
5. **Given** the Replace dialog, **When** the user moves focus between the find and replace fields and
   types, **Then** each field edits independently and UTF-8 input is correct.

### User Story 4 - Search options (Priority: P3)

The user can toggle search options in the dialog — at minimum case-sensitive matching and wrap-around —
and the option states are reflected in the results. (Additional options such as whole-word and regex are
exposed per clarification.)

**Why this priority**: Options refine search but a usable default (case-insensitive, wrap on) already
covers most needs; lower priority than getting Find/Replace working.

**Acceptance Scenarios**:

1. **Given** case-sensitive is off, **When** the user searches "the", **Then** "The" and "the" both
   match; **and** with case-sensitive on, only exact-case occurrences match.
2. **Given** wrap-around is on, **When** Find Next passes the last match, **Then** it returns to the
   first; with wrap off (if exposed), it stops at the last with an indication.
3. **Given** an option is toggled while the dialog is open, **When** the user re-runs the search,
   **Then** the matches reflect the new option.

### Edge Cases

- **Empty search term**: pressing Enter with an empty field does nothing (no matches, no error spam).
- **No matches**: clearly indicated; navigation keys are inert; nothing in the document changes.
- **Search term longer than any line / spanning behavior**: matches are plain substrings (or regex if
  enabled); multi-line behavior follows the engine's defined semantics.
- **Invalid regex (if regex enabled)**: the dialog reports the pattern is invalid instead of crashing;
  no matches are produced.
- **Match highlighting vs. selection**: match highlights are visually distinct and do not corrupt normal
  cursor/selection rendering.
- **Document edited after a search** (incl. via Replace): stale match positions must not be applied to
  wrong offsets — matches are recomputed so highlights/counts stay correct.
- **Replace overlapping or adjacent matches**: replacing does not corrupt later match offsets (offsets
  recomputed after each replace / handled by Replace All in one pass).
- **Unicode/wide characters** in either field and in the document: input editing and match offsets are
  character-correct, never splitting a multi-byte character.
- **Dialog modality**: while a Find/Replace dialog is open, typing edits the dialog field(s), not the
  document; closing returns to normal editing.

## Clarifications

### Session 2026-06-20

- Q: Which search-option toggles should the dialog expose? → A: All four — **case-sensitive,
  wrap-around, regex, and whole-word**. Case/wrap/regex are already supported by the engine; **whole-word
  requires new engine support** (word-boundary matching) and is in scope for this feature.
- Q: How should the Replace dialog's keys work? → A: **Tab** switches between the Find and Replace-with
  fields; **Enter** replaces the current match and advances to the next; **Ctrl+A** replaces all; **F3/F2**
  still step matches; **Esc** closes. (Ctrl+A maps to Replace-All only while the Replace dialog is open;
  it remains Select-All in normal editing.)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor MUST provide an interactive **Find** dialog (Search ▸ Find and `Ctrl+F`) with a
  text-input field in which the user can type, edit (insert/backspace), and clear a search term.
- **FR-002**: On confirming a Find, the editor MUST compute all matches, highlight them in the document,
  move the view/cursor to the current match (first at/after the cursor), and show a current/total
  indicator (e.g. "X of Y").
- **FR-003**: When a search yields no matches, the editor MUST indicate "not found" and leave the
  document and cursor unchanged.
- **FR-004**: Find Next (`F3`) and Find Previous (`F2`) MUST move the current match forward/backward
  through the result set with wrap-around, updating the indicator and the current-match highlight.
- **FR-005**: The current match MUST be highlighted distinctly from the other (non-current) matches.
- **FR-006**: The editor MUST provide an interactive **Replace** dialog (Search ▸ Find Replace and
  `Ctrl+H`) with two editable fields — search term and replacement — and a way to move focus between
  them.
- **FR-007**: **Replace** MUST replace the current match and advance to the next; **Replace All** MUST
  replace every occurrence in one operation. Both MUST report the number of replacements made.
- **FR-008**: Replacements MUST mark the buffer modified and MUST be undoable as normal edits.
- **FR-009**: `Esc` MUST close the active Find/Replace dialog and return to editing without applying any
  pending change; while a dialog is open, keyboard input MUST edit the dialog field(s), not the buffer.
- **FR-010**: The dialog MUST expose toggles for **case-sensitive**, **wrap-around**, **regex**, and
  **whole-word** matching; re-running the search MUST reflect the current option states. Whole-word
  matching (word-boundary aware) MUST be added to the search engine as part of this feature.
- **FR-010a**: The Replace dialog MUST use `Tab` to switch between the find and replace fields, `Enter`
  to replace the current match and advance, and `Ctrl+A` to replace all (Ctrl+A is Replace-All only
  while the Replace dialog is open; it stays Select-All in normal editing). `F3`/`F2` step matches and
  `Esc` closes.
- **FR-011**: All text entry in the dialog fields and all match offsets/highlight spans MUST be
  UTF-8/Unicode-correct — never splitting a multi-byte character and never misaligning a highlight.
- **FR-012**: Match positions MUST stay correct after the document changes (including after a replace) —
  matches are recomputed so highlights and the indicator are never applied to stale offsets.
- **FR-013**: These additions MUST NOT regress ordinary editing or other dialogs; outside an open
  Find/Replace dialog, all keys behave exactly as before.

### Key Entities *(include if feature involves data)*

- **Find/Replace dialog state**: which dialog is open (Find or Replace), the current text of the search
  field and (for Replace) the replacement field, which field has focus, the cursor position within the
  focused field, and the active option toggles (case-sensitive, wrap, …).
- **Match set**: the ordered list of match ranges in the document, plus which one is "current", driving
  the highlights and the "X of Y" indicator. Recomputed when the term, options, or document change.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From the menu or `Ctrl+F`, a user can type a term and reach the first matching occurrence
  in the document in a single Enter, with the view scrolled to it — in 100% of cases where the term
  exists.
- **SC-002**: The "X of Y" indicator matches the actual number of occurrences and the actual current
  position for every search and every next/prev step (no off-by-one, no stale counts).
- **SC-003**: Find Next/Previous visit every occurrence and wrap correctly (visiting N distinct matches
  over N steps before repeating) in 100% of cases.
- **SC-004**: Replace All replaces exactly the number of occurrences reported, and the document contains
  zero remaining occurrences of the (case/option-matched) term afterward.
- **SC-005**: A single Undo after a Replace/Replace All restores the document to its pre-replace state in
  100% of cases.
- **SC-006**: Case-sensitive and wrap toggles change results as specified in 100% of tested cases.
- **SC-007**: No regression: existing editing, navigation, and other dialogs behave unchanged when no
  Find/Replace dialog is open.

## Assumptions

- The existing search engine, match navigation (`find_next`/`find_prev`/`scroll_to_match`), `replace_all`,
  and highlight styles are reused; this feature adds the interactive UI, input routing, wiring, and the
  rendering of highlights — it does not re-implement matching.
- Default options are case-insensitive matching with wrap-around on, matching common editor behavior; the
  user can change them in the dialog.
- The dialog reuses the editor's existing modal-dialog conventions (centered overlay, Esc to cancel) and
  text-field editing conventions established by other input fields.
- Toggles exposed: case-sensitive, wrap, regex, whole-word (clarified); whole-word is new engine work.
  Replace keys: Tab (switch field) / Enter (replace+advance) / Ctrl+A (replace all) (clarified). In-dialog
  option-toggle keys (e.g. Alt+letter) are settled in planning.
- Match highlighting is rendered in the editor view for the active search; clearing the search (closing
  Find) removes the highlights.
