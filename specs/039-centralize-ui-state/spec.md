# Feature Specification: Centralize Editor UI State

**Feature Branch**: `039-centralize-ui-state`

**Created**: 2026-06-21

**Status**: Draft

**Input**: User description: "Centralize editor UI state to eliminate the two dominant bug classes (mode-flag soup and render-vs-hit-test ordering drift). Behavior-preserving refactor: collapse the modal flags into one `Modal` enum, define overlay z-order once for both render and mouse dispatch, and route remaining geometry through shared rect helpers. The entire existing test suite must pass unchanged."

## Overview

The editor's central state object tracks "which overlay/dialog is currently open" using roughly
fourteen independent optional/boolean flags. Nothing prevents two of them from being set at once, so
correctness depends entirely on three hand-maintained orderings agreeing with each other: the order
keyboard input checks the flags, the order mouse input checks them, and the order the screen paints
them. When those orderings drift apart — as they have repeatedly — clicks land on the wrong layer,
dialogs swallow keystrokes meant for another, or overlays paint in the wrong stacking order. Several
shipped fixes (the tab-bar/menu-dropdown overlap, the off-by-one click mapping on non-default
terminal sizes) were point patches to this structural fragility rather than removals of its cause.

This feature removes the cause. It replaces the bag of flags with a single value that can only ever
represent one open overlay at a time, derives all three orderings from one declared layer precedence,
and ensures screen geometry is computed in exactly one place per overlay (shared between drawing and
click-handling). It is a pure internal refactor: there is **no change to what the user sees or how the
editor behaves**. Its success is defined by the existing automated test suite continuing to pass
without any change to test expectations.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Only one overlay can ever be open (Priority: P1)

As a user, when I open any dialog or overlay (Find/Replace, Go-to-Line, file browser, Help, encoding
picker, plugin manager, confirmation prompts, the right-click context menu), exactly one is active at
a time, and the keyboard and mouse both interact with that same one — never a stale or hidden second
overlay.

**Why this priority**: This is the core defect class. Today the open-overlay state is spread across
many independent flags; the type of the state should make "two overlays open" impossible rather than
relying on careful ordering to avoid it.

**Independent Test**: Drive the editor through opening and closing every overlay via both keyboard and
mouse; assert that at all times at most one overlay is reported open and that input routes to it. The
existing inline and integration tests for each dialog exercise exactly these paths and must pass
unchanged.

**Acceptance Scenarios**:

1. **Given** the editor is in normal editing, **When** the user opens any single overlay, **Then**
   that overlay is the only one open and both keyboard and mouse act on it.
2. **Given** an overlay is open, **When** the user takes the action that opens a different overlay (or
   closes the current one), **Then** the first overlay is no longer open — there is never a moment
   where two are simultaneously open.
3. **Given** any overlay is open, **When** the user presses a key the overlay does not handle, **Then**
   the keystroke is consumed/blocked exactly as it is today (no leakage to the editor or another
   overlay).

---

### User Story 2 - Clicks and paint agree on stacking order (Priority: P1)

As a user, when overlays visually stack (for example a menu dropdown drawn over the tab bar), clicking
a visible element activates that element — the topmost visible thing at the clicked cell is what
receives the click, with no fall-through to a layer beneath it.

**Why this priority**: This is the second defect class — paint order and click order are maintained
separately and drift. The tab-bar/dropdown overlap bug is the canonical example: the dropdown painted
on top but the click was captured by the tab bar beneath it.

**Independent Test**: With two or more buffers open (tab bar visible) and a menu dropdown open over it,
click the first dropdown item and assert the dropdown action fires (not a tab switch). Exhaustively
press every cell in the top rows and assert no panic and correct top-layer routing. The existing
regression tests covering the dropdown-over-tabs case must pass unchanged.

**Acceptance Scenarios**:

1. **Given** the tab bar is visible and a menu dropdown is open over it, **When** the user clicks the
   first dropdown item, **Then** the dropdown item activates and the tab bar does not.
2. **Given** any stack of visible layers, **When** the user clicks a cell, **Then** the click is
   handled by the topmost layer occupying that cell.
3. **Given** any combination of visible layers, **When** the user clicks any cell on screen, **Then**
   the editor never panics.

---

### User Story 3 - Clicks land where things are drawn, on any terminal size (Priority: P2)

As a user on a terminal of any size, clicking inside a dialog's input field or button positions the
caret / activates the control exactly where it appears — the clickable region matches the drawn
region.

**Why this priority**: A prior bug had hit-testing geometry computed separately from drawing geometry,
so on non-default terminal sizes a click inside a visible dialog mapped to "outside" and dismissed it.
Routing both through one geometry source removes that whole class.

**Independent Test**: At several terminal sizes, open the Go-to-Line and Find/Replace dialogs, click
within the visible input fields, and assert the caret lands at the clicked position and the dialog is
not dismissed. Existing field-caret-click tests must pass unchanged.

**Acceptance Scenarios**:

1. **Given** a dialog with input fields is open at any supported terminal size, **When** the user
   clicks inside a visible field, **Then** the caret moves to the clicked position and the dialog stays
   open.
2. **Given** any dialog is open, **When** the user clicks a visible button, **Then** that button
   activates.

---

### Edge Cases

- **Stale cursor after overlay close / session restore / tab switch**: closing an overlay or switching
  buffers must still leave every buffer cursor within valid bounds before the next paint (the existing
  pre-render clamp must continue to run on all paths). No overlay-close path may leave a cursor that
  causes an out-of-range line access.
- **Overlay open while terminal is resized**: the open overlay must re-derive its geometry from the
  actual frame size on the next paint, so click regions track the redraw.
- **Right-click context menu over another would-be overlay**: opening the context menu must be blocked
  exactly when it is blocked today (i.e., when another overlay owns the foreground), with identical
  precedence.
- **Empty / degenerate buffer set**: active-buffer access must remain valid; the editor always has at
  least one buffer.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor MUST represent "which foreground overlay is open" as a single value that can
  hold at most one open overlay at a time, making the simultaneous-open-overlays state unrepresentable.
- **FR-002**: All overlays currently tracked by independent flags MUST be represented as cases of that
  single value: context menu, session-restore prompt, save prompt, external-change prompt, revert
  confirmation, tab-close confirmation, Find/Replace, Go-to-Line, encoding selection, file browser,
  Help, plugin consent, and plugin manager — together with any per-overlay sub-state they require
  (e.g. Go-to-Line caret position, Help scroll offset, plugin-manager cursor).
- **FR-003**: Keyboard input dispatch, mouse input dispatch, and screen painting MUST all derive which
  overlay is active from the single overlay value — not from separate per-overlay flags.
- **FR-004**: The relative stacking precedence of all layers (foreground overlay, menu dropdown, menu
  bar, tab bar, editor) MUST be declared in exactly one place and consumed by both painting (drawn
  bottom-to-top) and mouse hit-testing (resolved top-to-bottom), so the two cannot disagree.
- **FR-005**: A mouse click MUST be handled by the topmost layer whose drawn region contains the
  clicked cell; there MUST be no special-case guard reconciling click order with paint order for
  individual layer pairs (e.g. the tab-bar/dropdown special case is removed because precedence is now
  shared).
- **FR-006**: Each overlay's on-screen rectangle MUST be computed by a single shared helper used by
  both painting and hit-testing, so a click region cannot diverge from the drawn region. This MUST
  include the Go-to-Line and Find/Replace input fields, which currently recompute geometry separately.
- **FR-007**: The dead/unused active-menu flag MUST be removed; menu-active state MUST be read from the
  existing menu state machine only.
- **FR-008**: Access to the active buffer MUST go through the existing active-buffer accessors except
  where a specific non-active buffer index is deliberately required (e.g. confirmations referring to a
  particular buffer).
- **FR-009**: The refactor MUST be behavior-preserving: every existing automated test (unit/inline,
  integration, and headless smoke) MUST pass without any change to its assertions or expected
  behavior. Mechanical updates to tests are permitted ONLY where they read a renamed internal accessor
  in place of a removed field; the asserted behavior MUST be identical.
- **FR-010**: No user-visible behavior MAY change: identical keybindings, identical menu structure,
  identical dialog appearance and flow, identical mouse behavior, identical status-bar output.
- **FR-011**: The existing cursor-bounds invariant MUST be preserved: every code path that closes an
  overlay or changes the active buffer MUST leave all cursors valid before the next paint.

### Key Entities

- **Foreground overlay state**: The single value naming which overlay (if any) is currently open, plus
  that overlay's intrinsic sub-state. Replaces the previous collection of independent flags.
- **Layer precedence**: The single declared ordering of all stacked UI layers, from topmost to
  bottommost, used to drive both paint order and click resolution.
- **Overlay rectangle**: The on-screen region of an overlay (and its sub-controls), computed once and
  shared by paint and hit-test.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The complete pre-existing automated test suite passes with no changes to test
  expectations (only mechanical renames of removed fields to their replacement accessors are allowed).
- **SC-002**: The full local CI gate (formatting, linter with warnings-as-errors, all tests, headless
  smoke suite, and performance checks) passes clean.
- **SC-003**: It is structurally impossible to represent two foreground overlays open at once (the
  state's type permits only one), verifiable by inspection of the single overlay value.
- **SC-004**: Layer stacking order is defined in exactly one location; both paint and click-handling
  reference it, with no per-layer-pair click/paint reconciliation guards remaining.
- **SC-005**: For every stacked layer, a click inside that layer's drawn region routes to it and never
  to a lower layer — demonstrated by a test that holds for all layers, not just the previously-patched
  pair.
- **SC-006**: No user-visible behavior changes: a manual pass opening every overlay (including a menu
  dropdown over the tab bar) on a non-default terminal size shows identical behavior to the prior
  release, with no fall-through clicks and no panics.

## Assumptions

- The existing automated test suite (inline tests in the central module, the integration tests that
  drive keyboard and mouse handling end-to-end, and the headless terminal smoke tests) is a sufficient
  safety net to prove behavior preservation; this feature adds tests only for the new shared-precedence
  invariant and does not weaken any existing assertion.
- The menu state machine, per-widget rectangle/hit-test helpers, the pre-render cursor clamp, and the
  active-buffer accessors already exist and are reused rather than reimplemented.
- "Foreground overlay" and the always-visible menu/tab/editor layers are distinct: the menu bar can be
  active while editing, so menu state remains its own layer in the precedence rather than folding into
  the single overlay value.
- Flow state that is carried across steps but is not itself a concurrently-open overlay (e.g. an
  encoding choice remembered across a subsequent filename prompt) MAY remain a separate field.
- This is delivered as a single feature branch and PR; splitting the central module into multiple files
  is explicitly out of scope and, if pursued, will be a separate feature with its own deferral record.
