# Feature Specification: Soft-Wrap Mode

**Feature Branch**: `005-soft-wrap-mode`

**Created**: 2026-06-19

**Status**: Draft

**Input**: User description: "Feature 005 — Soft-Wrap Mode: An optional soft-wrap rendering mode
for the editor that wraps long lines at the terminal width instead of scrolling horizontally. The
mode is toggled via a View > Soft Wrap menu item and/or a keyboard shortcut (Alt+Z or similar).
When enabled, lines wider than the visible area are visually broken at word boundaries (or at the
column boundary if no word break fits) and the continuation lines are indicated by a visual marker
(e.g. a chevron '»' in the leftmost gutter column). The cursor moves through the logical line, not
the visual lines. Ctrl+S, Find, and all existing editor operations continue to work on the underlying
logical text unchanged. The setting is persisted in the user config file. DOS EDIT.COM does not have
soft-wrap; this is an extension, so it should be clearly labeled as a non-DOS extension in the UI
(e.g. 'Soft Wrap (ext)' in the menu). The feature branch is 005-soft-wrap-mode."

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Enable Soft-Wrap to Read Long Lines Without Horizontal Scrolling (Priority: P1)

A user opens a log file or minified source file with very long lines. Horizontal scrolling makes
reading painful. They press Alt+Z (or open View > Soft Wrap (ext)) and the editor immediately
reflows all visible content to fit the terminal width. Lines break at the last word boundary that
fits the column limit; a '»' marker appears in the leftmost gutter column on each continuation
line. The user can read the full content without scrolling left/right.

**Why this priority**: This is the entire value proposition of the feature. Without the visual
reflow, nothing else matters.

**Independent Test**: Open any file containing a line longer than the terminal width. Toggle
soft-wrap on — the long line must visually wrap and a continuation marker must appear. Toggle off
— horizontal scroll returns. No file content changes.

**Acceptance Scenarios**:

1. **Given** a file is open containing at least one line wider than the viewport, **When** the user
   presses Alt+Z, **Then** all long lines are visually broken at word boundaries to fit the viewport
   and '»' markers appear on continuation lines.
2. **Given** soft-wrap is active and a line has no word-break opportunity within the viewport width,
   **When** the editor renders that line, **Then** it breaks at the column boundary (hard wrap at
   column limit) rather than overflowing or hiding content.
3. **Given** soft-wrap is active, **When** the terminal is resized, **Then** all visual wrap points
   are recalculated immediately to match the new width.

---

### User Story 2 — Edit Text Normally While Soft-Wrap Is Active (Priority: P1)

A user enables soft-wrap and types, deletes, and pastes text. The cursor moves through the logical
line with arrow keys; pressing End jumps to the end of the logical line, not the visual wrap point.
Ctrl+S saves the file with exactly the same logical bytes as if soft-wrap were off. Find and Replace
locates matches that span visual wrap boundaries without issue.

**Why this priority**: The feature is useless — or actively harmful — if it corrupts editing
semantics. Logical-line operations must be transparent to wrap state.

**Independent Test**: Enable soft-wrap, edit a long line (add/delete text mid-line), save with
Ctrl+S, then disable soft-wrap and verify the file on disk is byte-for-byte identical to what was
typed (no extra newlines, no truncation at the wrap point).

**Acceptance Scenarios**:

1. **Given** soft-wrap is active and the cursor is on a visually wrapped line, **When** the user
   presses End, **Then** the cursor moves to the end of the logical line (past the visual wrap
   point), not the end of the visual segment.
2. **Given** soft-wrap is active, **When** the user saves with Ctrl+S, **Then** the file on disk
   contains no extra newlines inserted at visual wrap boundaries.
3. **Given** soft-wrap is active and a search term spans a visual wrap boundary, **When** Find
   executes, **Then** the match is found and highlighted correctly.
4. **Given** soft-wrap is active, **When** the user presses ↑ or ↓, **Then** the cursor moves
   between logical lines (skipping visual continuation lines).

---

### User Story 3 — Toggle Soft-Wrap Off and Return to Horizontal Scroll (Priority: P2)

A user who enabled soft-wrap wants to compare columns in a CSV or table. They press Alt+Z again
(or uncheck View > Soft Wrap (ext)) and the editor immediately returns to horizontal-scroll mode.
The cursor position, buffer content, and encoding are unaffected by the toggle.

**Why this priority**: The toggle must be reversible without side effects, or users will fear using it.

**Independent Test**: Enable soft-wrap, move the cursor to a specific logical position, then
disable soft-wrap — the cursor must be at the same logical position, and horizontal scrolling
must work as before.

**Acceptance Scenarios**:

1. **Given** soft-wrap is active, **When** the user presses Alt+Z, **Then** the editor returns to
   horizontal-scroll mode and all '»' markers disappear.
2. **Given** soft-wrap was toggled on and off, **When** the user inspects the buffer, **Then** the
   content is byte-for-byte identical to its state before the first toggle.

---

### User Story 4 — Persist Soft-Wrap Preference Across Sessions (Priority: P3)

A user sets soft-wrap to their preferred state and quits. On next launch the editor reopens with
soft-wrap in the same state, without requiring the user to re-toggle.

**Why this priority**: Persistence is a quality-of-life improvement. Acceptable to ship without it
if time-constrained, but required for a polished experience.

**Independent Test**: Enable soft-wrap, quit the editor, relaunch — the View > Soft Wrap (ext)
item must be checked and lines must be visually wrapped without any user action.

**Acceptance Scenarios**:

1. **Given** soft-wrap is enabled and the user quits cleanly, **When** the editor is relaunched
   without arguments, **Then** soft-wrap is active from the first frame rendered.
2. **Given** soft-wrap is disabled and the user quits cleanly, **When** the editor is relaunched,
   **Then** soft-wrap is off (horizontal scroll mode).

---

### Edge Cases

- What happens when a line consists entirely of a single token longer than the viewport? The editor
  must hard-wrap at the column boundary (character boundary respecting grapheme clusters) — no
  content may be hidden.
- How does the editor handle wide (CJK double-width) characters and emoji at a wrap boundary? The
  wrap point must land at a grapheme-cluster boundary; a double-width character that straddles the
  column limit must be pushed to the next visual line in its entirety.
- What happens when the terminal is extremely narrow? Soft-wrap functions normally at viewport widths
  ≥ 10 display columns. Below 10 columns, soft-wrap is automatically disabled with a status-bar
  warning "Terminal too narrow for soft wrap (min 10 columns)". At 10–19 columns, soft-wrap is active
  with aggressive hard-breaking but produces correct output.
- How are empty lines rendered in soft-wrap mode? Identically to non-wrap mode — an empty logical
  line produces a single empty visual line, no '»' marker.
- What happens when the user pastes a very long line? The pasted content is inserted into the
  logical buffer; the visual reflow updates immediately after insertion.
- How does the status bar behave? It must display a "[WRAP]" mode indicator when soft-wrap is active.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor MUST provide a "Soft Wrap (ext)" toggle in the View pull-down menu,
  visually indicated as checked/unchecked to reflect the current state.
- **FR-002**: The editor MUST bind Alt+Z as the keyboard shortcut for toggling soft-wrap on and off.
- **FR-003**: When soft-wrap is enabled, any logical line wider than the current viewport column
  count MUST be rendered as multiple visual lines, breaking at the last word-boundary opportunity
  that fits within the viewport. Word-boundary opportunities are: space (U+0020), tab (U+0009),
  comma (U+002C), period (U+002E), semicolon (U+003B), colon (U+003A), hyphen (U+002D), slash
  (U+002F). The break is placed immediately AFTER the boundary character (the boundary character
  appears at the end of the preceding visual line, not at the start of the continuation line).
- **FR-004**: When no word-boundary fits within the viewport, the editor MUST break the line at
  the column boundary, respecting grapheme-cluster boundaries (no partial multi-byte sequences or
  split double-width characters).
- **FR-005**: Each continuation visual line MUST display a '»' marker (U+00BB RIGHT-POINTING
  DOUBLE ANGLE QUOTATION MARK) in the leftmost gutter position to distinguish it from a new
  logical line.
- **FR-006**: Cursor-movement commands (↑ ↓ ← → Home End PgUp PgDn) MUST operate on logical
  lines; ↑/↓ navigate between logical lines, not visual wrap lines.
- **FR-007**: All file I/O operations (Ctrl+S, Save As, Save As Encoding, and the 30-second
  auto-save) MUST write the logical text with no extra newlines inserted at visual wrap boundaries.
  Soft-wrap state MUST NOT alter the bytes written to disk by any save path.
- **FR-008**: Find and Replace MUST search the logical text, correctly matching patterns that
  span visual wrap boundaries. Match highlights MUST be rendered on all visual rows that the match
  occupies — if a match spans a visual wrap boundary, the highlight must appear on both the closing
  segment of the first visual row and the opening segment of the continuation row.
- **FR-009**: When soft-wrap is enabled, horizontal scroll MUST be suppressed; the horizontal
  scroll offset is reset to zero on toggle-on and restored on toggle-off.
- **FR-010**: The status bar MUST display a "[WRAP]" indicator while soft-wrap is active;
  the indicator MUST disappear immediately when soft-wrap is toggled off.
- **FR-011**: The soft-wrap setting MUST be written to the user's config file
  (`$XDG_CONFIG_HOME/edit/config.toml`) under the key `soft_wrap` and loaded on every startup.
  The write MUST use an atomic tmp-rename pattern (write to `.config.toml.tmp` then rename) to
  prevent config file corruption. If the write fails (e.g., disk full, read-only directory), the
  editor MUST log the error at warn level, show a status bar message "Config save failed: [reason]",
  and continue operating with the toggled in-memory state — the toggle itself MUST NOT be reverted.
- **FR-012**: When the terminal is resized, all visual wrap points MUST be recalculated and the
  display updated within one render cycle without requiring user action.
- **FR-013**: Double-width characters (CJK, emoji) at a wrap boundary MUST be moved entirely
  to the next visual line; no partial rendering of double-width cells is permitted.
- **FR-014**: The feature MUST be labeled as a non-DOS extension in all user-facing named labels —
  the View menu item MUST read "Soft Wrap (ext)". The status bar uses the abbreviated `[WRAP]`
  indicator (consistent with `[Modified]`/`[Read Only]` style) and does not repeat the "(ext)"
  suffix. No user-facing text may refer to the feature as simply "Soft Wrap" without the
  "(ext)" suffix in any named/titled context.

### Key Entities

- **Logical Line**: A single newline-terminated (or EOF-terminated) string in the editor buffer.
  The authoritative unit for all text operations. Unchanged by soft-wrap state.
- **Visual Line**: A screen row produced by the renderer. One logical line may map to one or
  more visual lines when soft-wrap is active.
- **Wrap Point**: The column position within a logical line where a visual break is inserted.
  Computed per logical line from the current viewport width.
- **Continuation Marker**: The '»' glyph rendered in the leftmost column of every visual line
  that continues a previous logical line. Occupies one display column.
- **Viewport Width**: The number of usable editor columns, excluding gutter and scroll-bar columns.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can toggle soft-wrap on and off in a single keystroke (Alt+Z) with visual
  reflow completing within one screen refresh (imperceptible delay).
- **SC-002**: Files saved while soft-wrap is active are byte-for-byte identical to files saved
  with soft-wrap off, for the same logical content — verified by automated round-trip tests.
- **SC-003**: Cursor navigation through a 500-column logical line with soft-wrap active produces
  zero off-by-one positioning errors, verified by automated unit tests covering all movement keys.
- **SC-004**: The soft-wrap preference is correctly restored after a clean quit-and-relaunch cycle,
  verified by integration tests.
- **SC-005**: Lines containing CJK double-width characters and emoji wrap at legal grapheme-cluster
  boundaries with no display corruption, verified by targeted unit tests with fixture files.
- **SC-006**: The "(ext)" suffix appears in all user-facing named labels for this feature (the View
  menu item reads exactly "Soft Wrap (ext)"). The abbreviated status bar indicator `[WRAP]` is
  exempt from the "(ext)" suffix requirement — it follows the terse `[Modified]`/`[Read Only]`
  convention and is not a "named label". No full-word occurrence of "Soft Wrap" without "(ext)"
  may appear in any menu label, dialog title, or help text.

---

## Assumptions

- Soft-wrap is a global editor-wide toggle, not per-buffer. A per-buffer toggle adds complexity
  without clear user benefit for a first implementation.
- The '»' continuation marker occupies exactly one display column. Terminals that cannot render
  U+00BB fall back to the '>' ASCII character.
- Word-boundary detection uses Unicode whitespace characters (space U+0020, tab U+0009) and the
  standard punctuation set (comma, period, semicolon, colon, hyphen, slash) as break opportunities.
  No locale-specific word-break tables are required for v1.
- The minimum supported viewport width for soft-wrap is 10 display columns. Narrower terminals
  display a status-bar warning and disable soft-wrap automatically.
- Soft-wrap does not affect the line-number gutter display; if line numbers are shown, they reflect
  logical line numbers (not visual row numbers).
- The feature is implemented in the Rust rendering layer (ratatui widget) and does not require
  changes to the buffer, rope, or encoding subsystems.
- The `soft_wrap` config key defaults to `false` (opt-in); existing users who have no config entry
  experience no change in behavior.
- Mouse click-to-position while soft-wrap is active maps the clicked visual row/column back to the
  correct logical position; this mapping is included in scope.
