# Feature Specification: Boxed buttons + focus ring for the interactive/list dialogs

**Feature Branch**: `020-interactive-dialog-buttons`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "Boxed dialog buttons + focus ring for the interactive/list dialogs
(follow-up to feature 016, GitHub issue #38). Feature 016 added boxed, focusable, mouse-clickable
buttons with tab order to the confirm/dismiss dialogs. This feature extends that to the four remaining
interactive/list dialogs (encoding select, plugin manager, Find/Replace, file browser), each of which
needs a combined focus-ring where the list/field is one focus stop and each button is another. Reuse
`src/ui/buttons.rs`. All existing keyboard semantics must be preserved; scope is affordance/navigation
only."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Drive every interactive dialog with the mouse (Priority: P1)

When one of the four interactive/list dialogs is open — the encoding selector, the plugin manager, the
Find/Replace dialog, or the file browser — the user can click on-screen boxed buttons to confirm or
dismiss the dialog, instead of having to remember a key. Each dialog gains the same boxed buttons as the
confirm/dismiss dialogs already have (from feature 016): encoding select gets **OK / Cancel**, the plugin
manager gets **Close**, Find/Replace gets **Find / Replace / Replace All / Close**, and the file browser
gets **Open / Cancel**.

**Why this priority**: Three of these four dialogs are keyboard-only today; this is the core of the
request and immediately unblocks mouse-driven use of the last dialogs that still ignore on-screen
buttons.

**Independent Test**: Open the encoding selector, click **OK** → the highlighted encoding is applied and
the dialog closes; click **Cancel** → it closes with no change. Repeatable per dialog by hit-testing a
click at each button's drawn position.

**Acceptance Scenarios**:

1. **Given** an interactive dialog with buttons is open, **When** the user clicks a button, **Then** that
   button's action runs (identical to the keyboard equivalent) and the dialog closes if that choice
   closes it.
2. **Given** the encoding selector is open with an item highlighted, **When** the user clicks **OK**,
   **Then** the highlighted encoding is applied (same as pressing Enter on it).
3. **Given** Find/Replace is open with text typed in its field(s), **When** the user clicks **Find**,
   **Replace**, or **Replace All**, **Then** the corresponding existing action runs against the typed
   text.
4. **Given** any of these dialogs is open, **When** the user clicks a **Cancel/Close** button, **Then**
   the dialog dismisses with no destructive side effect (same as Esc).

### User Story 2 - One focus ring spanning the list/field and the buttons (Priority: P1)

Each interactive dialog has a single focus ring. The dialog's primary control (the encoding list, the
plugin list, the Find/Replace field group, or the file browser's list+path field) is one focus stop, and
each button is a further focus stop. `Tab` advances focus forward through the whole ring and `Shift+Tab`
backward, wrapping at the ends. Exactly one control is focused at a time and is drawn visually distinct,
so the user can always see what `Tab` moved to and what `Enter` will activate.

**Why this priority**: These dialogs were deferred from feature 016 precisely because they need a
combined list/field+button focus ring rather than a plain button bar; getting that ring right is what
makes them fully navigable by keyboard.

**Independent Test**: Open a dialog, press `Tab` repeatedly, and assert focus visits the primary control
and each button exactly once before returning to the start; assert exactly one control renders as
focused at each step.

**Acceptance Scenarios**:

1. **Given** an interactive dialog with a primary control and N buttons, **When** the user presses `Tab`
   N+1 times, **Then** focus visits the primary control and each button once and returns to the start
   (wrap-around).
2. **Given** any focus position, **When** the user presses `Shift+Tab`, **Then** focus moves to the
   previous stop in the same ring (reverse of `Tab`), wrapping at the start.
3. **Given** the dialog is shown, **When** it first appears, **Then** focus starts on the primary control
   (the list/field), so existing keyboard users land where they did before.
4. **Given** focus is on a button, **When** the user presses `Enter` (or `Space`), **Then** that button's
   action runs.
5. **Given** the user clicks a button with the mouse, **When** the click lands, **Then** that button's
   action runs directly (a click activates; it does not merely move focus).

### User Story 3 - Existing keyboard semantics are preserved (Priority: P1)

Every keystroke that worked in these dialogs before this feature keeps working exactly as it did. While
focus is on the primary control, the list dialogs still respond to `Up`/`Down` (and the file browser to
its navigation keys); the plugin manager still toggles with `Space`; Find/Replace still edits its fields,
toggles its options (`Alt+C/A/R/W`), and navigates matches (`F3`/`F2`/etc.); `Enter` on the primary
control still does what it always did; and `Esc` still cancels/closes every dialog regardless of focus.

**Why this priority**: This is an affordance/navigation-only change. A regression in any existing key
would make the feature a net negative, so preserving current behavior is as important as adding the
buttons.

**Independent Test**: For each dialog, with focus on the primary control, drive each previously-working
key and assert identical behavior to before; confirm the new buttons are only reached via `Tab`/click and
never intercept those keys.

**Acceptance Scenarios**:

1. **Given** a list dialog with focus on the list, **When** the user presses `Up`/`Down`, **Then** the
   list selection moves as before (the buttons do not consume these keys).
2. **Given** the plugin manager with focus on the list, **When** the user presses `Space`, **Then** the
   highlighted plugin's enabled state toggles as before.
3. **Given** Find/Replace with focus on a field, **When** the user types, toggles an option, or presses a
   match-navigation key, **Then** each behaves exactly as before this feature.
4. **Given** any of these dialogs, **When** the user presses `Esc`, **Then** the dialog closes/cancels as
   before, from any focus position.
5. **Given** a list dialog with focus on a **button** (not the list), **When** the user presses
   `Up`/`Down`, **Then** behavior is well-defined and non-destructive (see Assumptions).

### Edge Cases

- **Dialog narrower than its buttons**: buttons that do not fit are dropped from the drawn row (reusing
  the existing button layout behavior) but remain reachable by keyboard; layout must not corrupt or panic
  on a small terminal.
- **Single-button dialog** (plugin manager → **Close**): the focus ring is {list, Close}; `Tab` cycles
  between them; `Enter`/click/`Esc` on Close all dismiss.
- **Find/Replace mode differences**: in find-only mode the **Replace** / **Replace All** buttons are not
  applicable; the ring contains only the buttons relevant to the current mode.
- **Resize while a dialog is open**: buttons re-center/re-flow with the dialog and hit-testing matches the
  redrawn positions.
- **UTF-8 / wide button labels and list/field content**: button width, hit-testing, and field rendering
  remain grapheme/width-correct.
- **Clicking inside the dialog but not on a button or the list/field**: nothing happens; the dialog stays
  open.
- **Empty list** (e.g. plugin manager with no plugins, file browser of an empty directory): the buttons
  are still reachable and the dialog is still dismissable.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The encoding-select dialog MUST render boxed **OK** and **Cancel** buttons; the plugin
  manager MUST render a boxed **Close** button; the Find/Replace dialog MUST render boxed **Find**,
  **Replace**, **Replace All**, and **Close** buttons (only those applicable to the current find/replace
  mode); and the file browser MUST render boxed **Open** and **Cancel** buttons.
- **FR-002**: Each rendered button MUST use the same boxed, bordered visual style as the feature-016
  confirm/dismiss dialog buttons, and the dialog's height/layout MUST grow as needed to fit the button
  row without overlapping the primary control.
- **FR-003**: Each of these dialogs MUST expose a single focus ring whose stops are the primary control
  (list/field group) followed by each drawn button, in a predictable order.
- **FR-004**: Exactly one focus stop MUST be shown as focused at any time, visually distinct from the
  others.
- **FR-005**: `Tab` MUST advance focus forward through the ring and `Shift+Tab` MUST move it backward,
  both wrapping at the ends.
- **FR-006**: When the dialog first opens, focus MUST start on the primary control (the list/field), so
  existing keyboard flows are unchanged.
- **FR-007**: Pressing `Enter` or `Space` while a button is focused MUST run that button's action.
- **FR-008**: Clicking a button with the mouse MUST run that button's action directly, regardless of
  current focus.
- **FR-009**: Each button's action MUST be identical to the dialog's existing equivalent (OK/Open = the
  current Enter-confirm; Cancel/Close = the current Esc/close; Find/Replace/Replace All = the dialog's
  current find/replace/replace-all operations).
- **FR-010**: While focus is on the primary control, all keys that worked before this feature MUST behave
  identically (list `Up`/`Down`, plugin `Space` toggle, Find/Replace text editing, option toggles, and
  match navigation, and primary-control `Enter`).
- **FR-011**: `Esc` MUST continue to cancel/close every one of these dialogs from any focus position.
- **FR-012**: All button geometry, rendering, and hit-testing MUST remain correct under terminal resize
  and with wide/UTF-8 labels, and MUST NOT panic on a terminal too small to fit the buttons.

### Key Entities

- **Focus ring**: the ordered set of focus stops for one dialog — the primary control plus its buttons —
  with a current index, forward/backward movement, and wrap-around.
- **Dialog button**: a labeled, boxed, focusable, clickable control bound to one of the dialog's existing
  actions.
- **Primary control**: the dialog's pre-existing interactive element (encoding list, plugin list,
  Find/Replace field group, or file-browser list+path field) that retains its own internal navigation
  while it holds focus.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All four dialogs (encoding select, plugin manager, Find/Replace, file browser) can be fully
  operated — confirm and dismiss — using only the mouse.
- **SC-002**: All four dialogs can be fully operated using only the keyboard, including reaching every
  button via `Tab`/`Shift+Tab` and activating it with `Enter`/`Space`.
- **SC-003**: 100% of the keystrokes that worked in these dialogs before this feature still produce the
  same outcome (zero behavioral regressions), verified by tests.
- **SC-004**: At every moment a dialog is open, exactly one focus stop is shown as focused.
- **SC-005**: The dialogs render without corruption or panic across terminal sizes from the smallest that
  fits the dialog up to a full-screen terminal.

## Assumptions

- **Default focus on open** is the primary control (list/field), not a button, so existing keyboard
  muscle-memory (open dialog → arrow/type immediately) is preserved.
- **`Up`/`Down` while a button is focused** in a list dialog is treated as a no-op (or, optionally, moves
  focus back to the list); it never performs a destructive action. The implementation picks one
  consistent behavior; the spec only requires it be non-destructive and predictable.
- **Find/Replace button set follows the current mode**: in find-only mode only **Find** and **Close**
  are shown/ringed; in replace mode **Replace** and **Replace All** are added. This mirrors the dialog's
  existing mode behavior rather than introducing new modes.
- **Visual style and geometry** reuse the existing `src/ui/buttons.rs` helpers (`button_rects`,
  `render_buttons`, `hit_test_buttons`) so the new buttons match the feature-016 buttons exactly.
- **No new actions** are introduced; every button maps onto an action the dialog already performs.
- **File-browser confirm-button label follows the mode**: it reads **Open** in open mode and **Save**
  in save-as mode (the widget is shared between File›Open and File›Save As); **Cancel** is always
  present. (The issue described this generically as "Open/Cancel".)
- **Clicking outside the dialog** keeps each dialog's current outside-click behavior (this feature does
  not change it); only on-button clicks are newly meaningful.
- This builds on the decision recorded in `specs/016-dialog-buttons/` (all dialogs, boxed style; this
  feature covers the four interactive/list dialogs deferred there).
