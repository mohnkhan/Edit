# Feature Specification: Per-Tab Soft-Wrap

**Feature Branch**: `044-per-tab-soft-wrap`

**Created**: 2026-06-21

**Status**: Draft

**Input**: User description: "Spec per-tab soft-wrap — it cannot be global. Each file tab should have its
own word-wrap setting; toggling wrap on one tab must not change another."

## Overview

Soft-wrap is currently a single global setting: one toggle (`View ▸ Soft Wrap`, Ctrl+W) flips wrapping
for every open tab at once. That is the wrong model for a multi-file editor — wrapping is a property of
how you want to view *a particular file*. You might want a wide code file unwrapped (with the
horizontal scrollbar) while a prose file beside it is wrapped. The recent fix (feature 043) stopped the
*rendering corruption* from a shared cache, but the underlying setting is still global.

This feature makes soft-wrap **per tab**. Each open buffer remembers its own wrap on/off. Toggling wrap
affects only the active tab; switching tabs shows that tab's own setting; a new/opened tab starts from
the configured default. Nothing else about wrapping behavior changes.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Each tab keeps its own wrap setting (Priority: P1)

As a user with several tabs open, when I toggle soft-wrap on the active tab, only that tab changes; the
others keep whatever wrap setting they had. When I switch back to a tab later, it is still in the wrap
state I left it in.

**Why this priority**: This is the whole feature — wrap is a per-file view choice, not a global mode.

**Independent Test**: Open two tabs, toggle wrap on tab A only, switch to tab B and confirm it is
unchanged, switch back to tab A and confirm it is still wrapped.

**Acceptance Scenarios**:

1. **Given** two tabs both unwrapped, **When** I toggle soft-wrap while tab A is active, **Then** tab A
   becomes wrapped and tab B stays unwrapped.
2. **Given** tab A wrapped and tab B unwrapped, **When** I switch from A to B and back to A, **Then**
   each tab still shows its own setting (A wrapped, B unwrapped) — no leakage either way.
3. **Given** any per-tab wrap states, **When** I switch tabs, **Then** the now-active tab renders
   correctly for its own setting (correct line-number gutter, no ghost wrap from the other tab).

---

### User Story 2 - Toggle and indicators reflect the active tab (Priority: P1)

As a user, the wrap toggle and its on-screen indicators (the `View ▸ Soft Wrap` menu check mark and the
status bar's wrap indicator) always reflect the **active** tab's setting, and acting on them changes
only the active tab.

**Why this priority**: If the indicator showed a global/other-tab state, the per-tab model would be
confusing and untrustworthy.

**Independent Test**: With tabs in different wrap states, switch between them and confirm the menu check
mark and status-bar indicator track the active tab; invoke the toggle and confirm only the active tab
flips.

**Acceptance Scenarios**:

1. **Given** the active tab is wrapped, **When** I open the View menu, **Then** "Soft Wrap" shows its
   checked indicator; **When** the active tab is unwrapped, **Then** it shows unchecked.
2. **Given** I switch to a tab with a different wrap state, **When** the status bar / menu re-render,
   **Then** the indicators update to the now-active tab's state.

---

### Edge Cases

- **New / opened tabs**: a newly created or opened buffer starts at the configured default wrap setting
  (the existing `config.soft_wrap`), so behavior for someone who never toggles is unchanged.
- **Sole tab**: with one tab open, toggling behaves exactly as today (just that the setting now lives on
  the buffer).
- **Split view**: each visible pane reflects *its own* buffer's wrap setting for layout (the pane that
  is wrapped reserves no horizontal-scrollbar row; the unwrapped pane keeps it). The active pane uses
  the live wrap cache; a wrapped non-active pane renders best-effort (it has no precomputed cache, as
  today only the active buffer is cached) — it must not corrupt or crash.
- **Closing a tab**: the remaining tabs keep their own settings; no setting "moves" to another tab.
- **Persistence across the session**: switching away and back preserves a tab's setting for the life of
  that buffer. (Persisting wrap state to disk/session-restore is out of scope — see Assumptions.)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Soft-wrap state MUST be stored per buffer (per tab), not as a single global setting.
- **FR-002**: Toggling soft-wrap MUST change only the active tab's setting; all other tabs MUST be
  unaffected.
- **FR-003**: Switching tabs MUST display the now-active tab's own wrap setting, with correct rendering
  (line-number gutter aligned, no wrap artifacts carried over from another tab). The wrap cache MUST
  correspond to the buffer being rendered.
- **FR-004**: The `View ▸ Soft Wrap` menu check indicator and the status-bar wrap indicator MUST
  reflect the **active** tab's setting and update when the active tab changes.
- **FR-005**: A newly created or opened tab MUST initialize its wrap setting from the configured default
  (`config.soft_wrap`), preserving today's behavior for users who never toggle per-tab.
- **FR-006**: All editor geometry that depends on wrap (horizontal-scrollbar row reservation, content
  height/width, scroll math, mouse hit-testing) MUST use the wrap setting of the buffer that geometry
  belongs to — the active buffer for the single view, and each pane's buffer in split view.
- **FR-007**: The change MUST be behavior-preserving for the single-tab case and for users who never
  toggle wrap: with one tab, or with the default setting untouched, the editor looks and behaves as it
  did before this feature.
- **FR-008**: No crash or rendering corruption in any tab/pane/wrap combination (extends the feature-043
  guarantee to per-tab settings).

### Key Entities

- **Per-buffer wrap setting**: a boolean on each open buffer indicating whether that tab is soft-wrapped.
  Replaces the single global flag as the source of truth.
- **Active wrap cache**: the existing single computed wrap layout for the active buffer; now meaningful
  only when the active buffer's own wrap setting is on, and invalidated on tab switch (feature 043).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With two tabs, toggling wrap on one leaves the other's setting unchanged, and each tab
  retains its setting across any number of switches (verifiable by a test that toggles, switches, and
  asserts each tab's state).
- **SC-002**: After switching tabs, the rendered output of the now-active tab matches its own wrap
  setting — a wrapped tab wraps, an unwrapped tab does not, with a correct line-number gutter and no
  ghost wrap (verifiable by rendering and inspecting output for each tab).
- **SC-003**: The menu check indicator and status-bar wrap indicator match the active tab's setting in
  every tab.
- **SC-004**: Single-tab behavior and default-setting behavior are unchanged (the existing soft-wrap
  tests continue to pass, adjusted only to read the setting from its new per-buffer location).
- **SC-005**: No panic or corruption across tab/pane/wrap combinations; the full suite (incl. the
  feature-043 wrap-cache tests and the 042 fuzz sweep) passes; `fmt` + `clippy -D warnings` clean.

## Assumptions

- The configured default (`config.soft_wrap`) remains the *initial* value for new/opened buffers; the
  global config key is no longer the live runtime state but a default seed. (Toggling no longer rewrites
  the config; if "remember my last toggle as the default" is desired, that's a separate enhancement.)
- Persisting each tab's wrap state into the session-restore file is **out of scope**; restored buffers
  initialize from the configured default like any opened file. (Can be a follow-up.)
- Split view already renders two buffers with a single active wrap cache; this feature makes each pane
  honor its own buffer's wrap *flag* for layout, but does not add a second precomputed cache — a wrapped
  non-active pane renders best-effort without corruption. A full two-cache split is out of scope.
- The wrap cache continues to be invalidated on every buffer switch (feature 043), so it always matches
  the active buffer.
