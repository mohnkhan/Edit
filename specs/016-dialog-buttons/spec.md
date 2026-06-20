# Feature Specification: Focusable dialog buttons (borders, tab order, mouse)

**Feature Branch**: `016-dialog-buttons`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "All the dialog boxes open but they cannot be navigated by mouse, we need to
have a button boundary on them. like other Dos/linux programs do. it makes the window and buttons look
nicer, helps bring in focus and navigation cues. All buttons should also have tab order."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Click a dialog button with the mouse (Priority: P1)

When a dialog (e.g. the unsaved-changes prompt, the Revert confirm, the external-change prompt, a plugin
consent prompt) is open, the user can click its on-screen buttons (Save / Discard / Cancel, Yes / No,
etc.) with the mouse to choose, instead of being forced to remember a letter key. Clicking outside the
dialog does the conventional thing for that dialog (cancel where a cancel exists).

**Why this priority**: Today every dialog ignores the mouse entirely, so mouse users are stuck. This is
the core of the request and immediately unblocks mouse-driven use of all dialogs.

**Independent Test**: Open the unsaved-changes prompt, click the "Cancel" button → the prompt closes and
nothing is saved/discarded. Click "Save"/"Discard" → the corresponding action runs. Testable by
hit-testing a click at the button's drawn position.

**Acceptance Scenarios**:

1. **Given** a dialog with buttons is open, **When** the user clicks a button, **Then** that button's
   action runs (identical to choosing it by keyboard) and the dialog closes if that choice closes it.
2. **Given** a dialog is open, **When** the user clicks inside the dialog but not on a button, **Then**
   nothing happens (the dialog stays open).
3. **Given** a dialog with a Cancel/No option is open, **When** the user clicks outside the dialog box,
   **Then** the dialog cancels (no destructive action).

### User Story 2 - Buttons have visible boundaries and a focused look (Priority: P1)

Each dialog button is drawn with a visible boundary (a bordered/bracketed button) like DOS/Linux
programs, and the currently focused button is visually distinct (highlighted). This makes the dialog
read as a real window with buttons and shows where keyboard focus is.

**Why this priority**: The visible button boundary + focus highlight is explicitly requested and is what
makes the dialog navigable and legible; it pairs with US1 (you can see what you're clicking) and US3
(you can see what Tab moved to).

**Independent Test**: Open a dialog; each choice renders inside a button boundary; exactly one button is
shown highlighted as focused. Verifiable by inspecting the rendered cells.

**Acceptance Scenarios**:

1. **Given** any dialog with buttons, **When** it is shown, **Then** each button is drawn with a visible
   boundary around its label.
2. **Given** a dialog with buttons, **When** it is shown, **Then** exactly one button is rendered as
   focused (visually distinct from the others).
3. **Given** a button is focused, **When** focus moves to another button, **Then** the highlight moves
   with it and only one button is focused at a time.

### User Story 3 - Tab order across buttons (Priority: P1)

The user can move focus between a dialog's buttons with `Tab` (forward) and `Shift+Tab` (backward) in a
predictable order, and activate the focused button with `Enter` (and `Space`). Focus wraps around the
ends. The dialog's existing letter shortcuts keep working.

**Why this priority**: Keyboard-only users need to reach every button without a dedicated letter, and
"all buttons should have tab order" is explicit in the request.

**Independent Test**: Open a 3-button dialog; `Tab` cycles focus 1→2→3→1; `Shift+Tab` reverses; `Enter`
on the focused button runs its action. Testable by driving keys and asserting focus index / action.

**Acceptance Scenarios**:

1. **Given** a dialog with N buttons, **When** the user presses `Tab` N times, **Then** focus visits
   each button once and returns to the start (wrap-around).
2. **Given** focus on a button, **When** the user presses `Enter` (or `Space`), **Then** that button's
   action runs.
3. **Given** a dialog also has letter shortcuts (e.g. S/D/C, Y/N), **When** the user presses such a
   letter, **Then** the corresponding choice still runs (shortcuts and tab/click coexist).
4. **Given** a dialog opens, **When** it first appears, **Then** a sensible default button is focused
   (e.g. the confirming or the safe/cancel button, consistent per dialog).

### Edge Cases

- **Dialog narrower than its buttons**: buttons must wrap or truncate without corrupting the layout or
  panicking on a small terminal.
- **Single-button dialogs** (e.g. Help/About → "OK/Close"): Tab is a no-op cycle of one; Enter/click/Esc
  all dismiss.
- **Esc** still cancels/closes every dialog as before (unchanged), regardless of focus.
- **Mouse click on the focused vs an unfocused button**: clicking any button activates it directly (it
  does not merely move focus).
- **UTF-8 / wide button labels**: button width and hit-testing must be grapheme/width-correct.
- **List-style dialogs** (encoding select, plugin manager) keep their list navigation; buttons added to
  them (e.g. OK / Cancel) coexist with the list and are reachable by Tab and mouse.
- **Resize while a dialog is open**: buttons re-center/re-flow with the dialog; hit-testing matches the
  redrawn positions.
- **Clicking outside a dialog that has no Cancel** (e.g. a forced choice): no destructive default — the
  dialog stays open.

## Clarifications

### Session 2026-06-20

- Q: Which dialogs are in scope? → A: **All** modal dialogs — the confirm/choice prompts (unsaved-changes
  Save/Discard/Cancel, Revert Yes/No, external-change Reload/Keep, plugin consent Allow/Deny, session
  restore Restore/Decline), Help/About (Close), Find/Replace (its actions), and the list dialogs
  (encoding select, plugin manager) and file browser gain OK/Cancel-style buttons that coexist with their
  existing list/field navigation.
- Q: Button visual style? → A: **Boxed buttons** — each button drawn in its own box border (3 rows),
  the most DOS-window-like. Dialog heights grow to fit a button row. The focused button is rendered
  distinctly (e.g. a heavier/▶-marked border or inverted colors).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every modal dialog that presents discrete choices MUST render each choice as a button with
  a visible boundary (a bordered/bracketed button), replacing or augmenting the current text hints.
- **FR-002**: Exactly one button in an open dialog MUST be shown as focused, visually distinct from the
  unfocused buttons.
- **FR-003**: `Tab` MUST move button focus forward and `Shift+Tab` backward, in a defined order, wrapping
  at the ends.
- **FR-004**: `Enter` and `Space` MUST activate the focused button (run its action); `Esc` MUST cancel/
  close the dialog as before.
- **FR-005**: A left-click on a button MUST activate that button directly (same effect as focusing it and
  pressing Enter), via hit-testing that matches the drawn button position.
- **FR-006**: A click inside the dialog but not on a button MUST be inert (dialog stays open); a click
  outside the dialog box MUST cancel the dialog where a cancel/no/keep choice exists, and be inert for
  dialogs with no safe cancel.
- **FR-007**: Existing per-dialog letter shortcuts (e.g. S/D/C, Y/N) and existing list navigation
  (Up/Down in encoding-select and plugin-manager) MUST continue to work alongside buttons.
- **FR-008**: Each dialog MUST focus a sensible default button when it opens (the confirming action, or
  the safe/cancel action where that is safer), consistent per dialog.
- **FR-009**: Button rendering, focus, tab order, and hit-testing MUST be provided by a single shared,
  reusable mechanism so all dialogs behave consistently and the behavior is defined in one place.
- **FR-010**: Button layout and hit-testing MUST be UTF-8/width-correct and MUST not panic on small
  terminals (buttons wrap/clamp gracefully).
- **FR-011**: The change MUST NOT alter what each choice does, MUST keep every dialog modal (input does
  not leak to the buffer), and MUST NOT regress non-dialog editing or the file-browser/menu mouse
  behavior.

### Key Entities *(include if feature involves data)*

- **Dialog button**: a labeled action within a dialog — its label, the action/choice it triggers, and
  its position in the tab order. Rendered with a boundary; may be focused.
- **Button row / focus model**: the ordered set of buttons for a dialog plus which one is focused;
  supports next/previous (wrap), activate-focused, and hit-test-at-(col,row).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of dialogs that offer discrete choices render those choices as bordered buttons with
  exactly one focused.
- **SC-002**: Every dialog button is reachable by `Tab`/`Shift+Tab` (focus visits all N buttons within N
  steps) and activatable by `Enter`/`Space`.
- **SC-003**: Every dialog button is activatable by a single mouse click landing on its drawn boundary
  (100% correspondence between drawn position and clickable region).
- **SC-004**: Choosing an option by button (click or Tab+Enter) produces the exact same result as the
  pre-existing letter/key shortcut in 100% of cases.
- **SC-005**: No regression: Esc cancels as before; letter shortcuts and list navigation still work; no
  panic at any terminal size; non-dialog editing and file-browser/menu mouse unchanged.

## Assumptions

- The buttons reuse the editor's existing theme colors (a focused button uses the established
  selection/highlight styling); no new configurable colors are required, though a dedicated focus style
  may be introduced internally.
- The set of dialogs in scope and the exact button labels/order per dialog are confirmed during
  clarification; the shared button mechanism applies to all of them.
- "Visible boundary" is realized with terminal box-drawing/bracket characters; it degrades gracefully on
  terminals lacking box-drawing (the label stays readable).
- Mouse support for dialogs is added by hit-testing in the existing mouse handler (currently dialogs are
  excluded); the file-browser and menu mouse paths are unaffected.
- The default-focused button per dialog and whether an outside-click cancels are settled in
  clarification per dialog type.
