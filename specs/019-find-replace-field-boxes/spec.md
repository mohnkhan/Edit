# Feature Specification: Bordered-box styling for Find/Replace fields

**Feature Branch**: `019-find-replace-field-boxes`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "Bordered-box styling for the Find/Replace input fields (feature 018 follow-up, issue #41). Feature 018 gave the file-browser Open/Save dialog its editable fields a bordered, labeled input box with a visible caret. The Find/Replace dialog fields (from feature 015) currently render a label plus an inline │ caret but are NOT drawn as bordered boxes. Apply the same bordered-input-box treatment to the Find field and the Replace-with field for visual consistency, preserving all existing behavior. Scope: visual/affordance only — no focus-ring/button changes (issue #38)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Find field appears as a bordered input box (Priority: P1)

A user opens the Find dialog (`Ctrl+F` / Search ▸ Find). The text-entry area for the search
term is drawn as a clearly bordered, labeled box — visually identical in style to the file-browser
input box introduced in feature 018 — instead of a bare label-and-caret line. The user immediately
recognizes where to type.

**Why this priority**: The Find dialog is the most-used of the two search dialogs and is the
direct subject of the consistency complaint. Delivering just this story already makes the search
UI consistent with the file browser and is independently shippable.

**Independent Test**: Open the Find dialog and confirm the search term is entered inside a bordered
box with a visible caret; type, edit, and run a search; verify matches and the "X of Y" indicator
still work.

**Acceptance Scenarios**:

1. **Given** the editor is open, **When** the user presses `Ctrl+F`, **Then** the Find dialog shows
   the search term inside a bordered, labeled input box with a visible caret at the insertion point.
2. **Given** the Find dialog is open, **When** the user types and edits text (insert, Backspace,
   Delete, Home/End, Left/Right), **Then** the text and caret update correctly inside the box.
3. **Given** a search term is entered, **When** the user runs the search, **Then** matches are found
   and the match-count ("X of Y") indicator displays as before.

---

### User Story 2 - Replace dialog fields both appear as bordered boxes (Priority: P1)

A user opens the Replace dialog (`Ctrl+H` / Search ▸ Find Replace). Both the "Find what" field and
the "Replace with" field are drawn as bordered, labeled boxes in the same style. `Tab` moves focus
between the two fields, and the currently focused box is visually distinguishable from the unfocused
one.

**Why this priority**: The Replace dialog has two fields and is where inconsistency is most visible;
it shares the same rendering path as Find, so completing it finishes the consistency goal.

**Independent Test**: Open the Replace dialog, confirm both fields are bordered boxes, `Tab` between
them, type in each, and perform a replace / replace-all.

**Acceptance Scenarios**:

1. **Given** the editor is open, **When** the user presses `Ctrl+H`, **Then** both the Find-what and
   Replace-with fields render as bordered, labeled input boxes.
2. **Given** the Replace dialog is open, **When** the user presses `Tab`, **Then** focus moves
   between the two boxes and the focused box is visually indicated (e.g., the caret appears only in
   the focused box and/or its border is emphasized).
3. **Given** both fields contain text, **When** the user triggers replace and replace-all, **Then**
   the replacement behavior is unchanged from before this feature.

---

### User Story 3 - Search options and behavior remain intact (Priority: P2)

All existing search controls continue to work unchanged after the visual restyle: case-sensitive
toggle (`Alt+C`), wrap-around toggle (`Alt+A`), regex toggle, the match-count indicator, and dialog
dismissal (`Esc`).

**Why this priority**: This is a regression-guard story rather than new value; it ensures the
visual change does not break interaction, but it is verified as part of stories 1 and 2.

**Independent Test**: With either dialog open, toggle each option and confirm the toggle state is
reflected and affects the search; confirm `Esc` closes the dialog.

**Acceptance Scenarios**:

1. **Given** a search dialog is open, **When** the user presses `Alt+C`, `Alt+A`, or the regex
   toggle, **Then** the corresponding option toggles and its state is visible.
2. **Given** a search dialog is open, **When** the user presses `Esc`, **Then** the dialog closes and
   the editor returns to its prior state.

---

### Edge Cases

- **Small terminal**: Each field becoming a 3-row box makes the dialog taller. On a terminal too
  short to show the full taller dialog, the dialog MUST still render without panicking or corrupting
  the screen, degrading gracefully (e.g., clamping to the available height) the same way other
  dialogs do today.
- **Long entry text**: When the typed term is wider than the inner width of the box, the text MUST
  scroll horizontally within the box so the caret stays visible, as it does today.
- **Empty field**: An empty box renders correctly with the caret at the start; running a search with
  an empty term behaves exactly as it does today.
- **Narrow terminal**: When the terminal is narrower than the dialog's preferred width, the boxes
  MUST clamp to the available width without overflowing the dialog border.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Find dialog MUST render its search-term entry as a bordered, labeled input box
  matching the visual style of the file-browser input box introduced in feature 018.
- **FR-002**: The Replace dialog MUST render both the "Find what" and "Replace with" entries as
  bordered, labeled input boxes in the same style.
- **FR-003**: Each input box MUST show a visible caret at the insertion point within the focused box.
- **FR-004**: All existing text-editing operations within a field (character insertion, Backspace,
  Delete, Home, End, Left, Right, and horizontal scrolling for long text) MUST continue to work.
- **FR-005**: `Tab` (and `Shift+Tab` where currently supported) MUST continue to switch focus
  between the Replace dialog's two fields, and the focused field MUST be visually distinguishable
  from the unfocused field.
- **FR-006**: The search-option toggles (case-sensitive `Alt+C`, wrap-around `Alt+A`, regex) MUST
  continue to function and display their state.
- **FR-007**: The match-count ("X of Y") indicator MUST continue to display and update as before.
- **FR-008**: `Esc` MUST continue to dismiss the dialog and the search/replace actions
  (find next/previous, replace, replace all) MUST continue to function unchanged.
- **FR-009**: The taller dialog MUST render correctly on small terminals, degrading gracefully
  (clamping to available height/width) without panicking or corrupting the display.
- **FR-010**: This feature MUST NOT introduce focus-ring or boxed-button behavior for the dialog
  (that work is tracked separately by issue #38); the change is limited to field box styling.

### Key Entities

- **Find dialog**: The modal search dialog with one text field (search term), option toggles, and a
  match-count indicator.
- **Replace dialog**: The modal find-and-replace dialog with two text fields (find / replace), a
  field-focus selector, option toggles, and a match-count indicator.
- **Input box**: A bordered, labeled, single-line editable field with a caret and horizontal
  scrolling — the shared visual treatment being applied.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user looking at the Find dialog and the file-browser Open dialog side by side
  perceives the editable fields as the same visual component (bordered, labeled box with caret).
- **SC-002**: 100% of pre-existing Find/Replace interactions (typing, editing, `Tab` field switch,
  all three option toggles, match count, find/replace/replace-all, `Esc`) continue to work after the
  restyle, verified by the existing and new tests.
- **SC-003**: The Find and Replace dialogs render without panic or visual corruption on terminals as
  small as the existing minimum supported size.
- **SC-004**: No new keybinding, menu item, or option is introduced or removed; the change is purely
  visual/affordance.

## Assumptions

- The bordered-input-box visual treatment from feature 018 (file browser) is the canonical style to
  reuse; the box-drawing approach used there / by feature-016 buttons is available for reuse.
- "Visible caret" means the same caret affordance already used by the file-browser field, not a
  hardware terminal cursor.
- Focus indication between the Replace dialog's two boxes can reuse a lightweight emphasis (caret
  presence and/or border emphasis); a full focus-ring spanning fields and buttons is out of scope
  (issue #38).
- The dialog's existing minimum-terminal handling (introduced when the crash on very small terminals
  was fixed in feature 015) is the baseline graceful-degradation behavior to preserve.
