# Feature Specification: Buffer tab bar

**Feature Branch**: `027-buffer-tab-bar`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "Add a tab bar listing open buffers below the menu bar (filename + modified
marker, active tab highlighted). Shown only with 2+ buffers. Each tab is clickable to switch and has a
clickable [x] close box (reusing the existing close-buffer flow incl. the unsaved-changes prompt).
Keyboard switching unchanged. The tab bar shrinks the editor by a row, so the editor geometry
(viewport_height, click mapping, wheel/scrollbar areas) must account for it. Long tab lists truncate
gracefully, keeping the active tab visible. Scope: the tab-bar UI + click/close only."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See and switch open files via tabs (Priority: P1)

When two or more files are open, a tab bar appears directly below the menu bar showing each open buffer's
filename. The active buffer's tab is visually highlighted, and a buffer with unsaved changes shows a
modified marker. Clicking a tab switches to that buffer. With a single buffer open, no tab bar is shown
(the editor keeps its full height).

**Why this priority**: Today the only multi-file cue is a terse `[n/m]` status number; a visible,
clickable list of open files is the core value and how users orient themselves across buffers.

**Independent Test**: Open two files → a tab bar lists both; the active one is highlighted. Click the
other tab → the editor switches to that buffer and its tab becomes highlighted.

**Acceptance Scenarios**:

1. **Given** 2+ buffers are open, **When** the editor renders, **Then** a one-row tab bar below the menu
   bar shows each buffer's filename, with the active buffer's tab highlighted.
2. **Given** a buffer has unsaved changes, **When** its tab renders, **Then** the tab shows a modified
   marker distinguishing it from saved buffers.
3. **Given** the tab bar is shown, **When** the user clicks a tab, **Then** the editor switches to that
   buffer (identical to selecting it by keyboard).
4. **Given** only one buffer is open, **When** the editor renders, **Then** no tab bar is shown and the
   editor occupies the full area as before.

### User Story 2 - Close a buffer from its tab (Priority: P1)

Each tab has a small `[x]` close box. Clicking it closes that buffer, using the existing close-buffer
behavior — including prompting to save when the buffer has unsaved changes. After closing, the tab bar
updates (and disappears when only one buffer remains).

**Why this priority**: Closing files is a basic multi-file need; doing it from the tab is the expected
companion to switching, and it must not bypass the unsaved-changes safeguard.

**Independent Test**: With 3 buffers (one modified), click the `[x]` on a clean buffer's tab → it closes
and the bar shows 2 tabs. Click `[x]` on the modified buffer → the existing save prompt appears.

**Acceptance Scenarios**:

1. **Given** 2+ buffers, **When** the user clicks a tab's `[x]`, **Then** that buffer is closed via the
   existing close-buffer flow (a click on `[x]` closes the buffer, not merely switches to it).
2. **Given** the buffer being closed has unsaved changes, **When** `[x]` is clicked, **Then** the existing
   unsaved-changes save prompt is shown (no silent data loss).
3. **Given** closing leaves a single buffer, **When** the close completes, **Then** the tab bar is no
   longer shown.

### User Story 3 - Editor geometry stays correct with the tab bar (Priority: P1)

With the tab bar present, everything that depends on the editor's position and size stays correct: the
cursor lands on the right line/column when clicking in the text, page up/down and cursor-visibility use
the reduced height, and the mouse wheel (feature 023) and scrollbars (features 021/024) act on the
correct region. A click on the tab-bar row interacts with tabs, not the text.

**Why this priority**: The tab bar shifts the editor down by a row; if the geometry isn't updated in
lockstep, clicks misplace the cursor and scrolling/scrollbars desync — a serious regression.

**Independent Test**: With 2 buffers open, click a position in the text → the cursor lands on the
clicked cell (accounting for the tab-bar row); roll the wheel / drag the scrollbar → the editor scrolls
correctly; a click on the tab row switches/closes a tab and never moves the text cursor.

**Acceptance Scenarios**:

1. **Given** the tab bar is shown, **When** the user clicks in the editor text, **Then** the cursor lands
   on the correct line/column (the tab-bar row is accounted for).
2. **Given** the tab bar is shown, **When** the user pages up/down or moves the cursor to an edge, **Then**
   scrolling uses the reduced editor height (cursor stays visible).
3. **Given** the tab bar is shown, **When** the user uses the wheel or a scrollbar, **Then** the editor
   scrolls correctly against the reduced area.
4. **Given** the tab bar is shown, **When** the user clicks the tab-bar row, **Then** it switches/closes a
   tab and does not place the text cursor.

### Edge Cases

- **Single buffer**: no tab bar; closing down to one buffer hides it; opening a second shows it.
- **Many buffers / long names**: tabs that overflow the width truncate gracefully and keep the **active**
  tab visible (e.g. condense/scroll), without corrupting the row or hiding the active tab.
- **Unnamed buffer** (never saved): the tab shows a stable placeholder label (e.g. "[No Name]").
- **Modified marker**: present for unsaved buffers, absent once saved.
- **Tiny terminal**: the tab bar (and its truncation) renders without panic; if the terminal is too short
  to be usable the existing too-small handling still applies.
- **Click between/outside tabs** on the tab row: no-op (no switch/close, no cursor move).
- **Split view**: the tab bar still reflects the open buffers; switching/closing behaves consistently with
  the existing multi-buffer behavior.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When 2+ buffers are open, the editor MUST show a one-row tab bar directly below the menu bar
  listing each open buffer by filename; when only one buffer is open the tab bar MUST NOT be shown.
- **FR-002**: The active buffer's tab MUST be visually distinct (highlighted) from the others.
- **FR-003**: A buffer with unsaved changes MUST show a modified marker on its tab; a saved buffer MUST
  not.
- **FR-004**: Clicking a tab (its label area) MUST switch the editor to that buffer, identical to
  selecting it by keyboard.
- **FR-005**: Each tab MUST include a `[x]` close affordance; clicking it MUST close that buffer via the
  existing close-buffer flow, including the unsaved-changes save prompt — a `[x]` click MUST close, not
  merely switch.
- **FR-006**: Keyboard buffer switching (`Ctrl+Tab` / `Ctrl+Shift+Tab`) MUST be unchanged.
- **FR-007**: When the tab bar is shown, the editor area MUST shrink by exactly its one row, and the
  editor geometry used for cursor placement (click→line/column), paging, cursor-visibility, the mouse
  wheel, and the scrollbars MUST account for the tab-bar row so all stay correct.
- **FR-008**: A click on the tab-bar row MUST be handled by the tab bar (switch/close or no-op) and MUST
  NOT place the text cursor; a click between/outside tabs MUST be a no-op.
- **FR-009**: Tab labels MUST be UTF-8/width-correct and MUST truncate gracefully when the row overflows,
  keeping the active tab visible.
- **FR-010**: The feature MUST NOT change how buffers are opened or any editing behavior; it adds only the
  tab-bar UI and its click/close interaction.
- **FR-011**: The tab bar MUST render without panic on any terminal size and for any number of buffers.

### Key Entities

- **Tab**: a visual representation of one open buffer — its display name (filename or placeholder), a
  modified marker, an active/inactive state, a clickable label region, and a clickable `[x]` close region.
- **Tab bar**: the ordered row of tabs for all open buffers, shown only when 2+ buffers exist, with
  overflow handling that keeps the active tab visible.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With 2+ files open, the user can see all open filenames at once and switch between them by
  clicking, and the active/modified state is visible at a glance.
- **SC-002**: A buffer can be closed by clicking its tab's `[x]`, and an unsaved buffer always triggers
  the save prompt (zero silent data loss).
- **SC-003**: With the tab bar shown, clicking in the text places the cursor on the correct cell and
  wheel/scrollbar/paging operate on the correct region — no off-by-the-tab-row errors (verified by tests).
- **SC-004**: Single-buffer editing is visually and behaviorally unchanged (no tab bar, full-height
  editor).
- **SC-005**: The tab bar never panics or corrupts the layout across terminal sizes and buffer counts,
  including very long names and many buffers.

## Assumptions

- **Placement**: a single row directly below the menu bar (above the editor), shown only when 2+ buffers
  are open (per decision); single-buffer layout is unchanged.
- **Tab interaction** (per decision): clicking a tab's label switches; clicking its `[x]` closes via the
  existing close-buffer flow (which already handles the unsaved-changes prompt). No drag-reorder, no
  scroll-wheel-over-tabs behavior in this feature.
- **Modified marker**: a small glyph (e.g. `●`/`*`) on unsaved tabs; exact glyph is a presentation detail.
- **Display name**: the file name (not full path); unnamed buffers use the existing placeholder
  ("[No Name]"); overflow truncates the name (width-correct), keeping the active tab visible.
- **Geometry**: the editor's top row and height become a single shared computation (tab-bar-aware) used by
  rendering, `viewport_height`, click mapping, and the wheel/scrollbar editor-area logic.
- **Scope**: tab-bar UI + click/close only; opening buffers, editing, and other dialogs are unchanged; no
  new config key.
- Builds on the existing multi-buffer model (`buffers`/`active_idx`, NextBuffer/PrevBuffer), the
  close-buffer flow, and the editor-geometry helpers from features 021/023/024.
