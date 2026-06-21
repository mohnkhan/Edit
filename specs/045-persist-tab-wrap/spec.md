# Feature Specification: Persist Each Tab's Soft-Wrap Across Restart

**Feature Branch**: `045-persist-tab-wrap`

**Created**: 2026-06-21

**Status**: Draft

**Input**: User description: "Now persist each tab's wrap state across restart."

## Overview

Feature 044 made soft-wrap a per-tab setting, but it is not remembered across restarts: on session
restore every reopened tab falls back to the configured default. This feature persists each tab's
soft-wrap setting in the saved session, so when the editor reopens a session each tab comes back in the
wrap state the user left it in — matching the per-tab model end to end.

It is a small additive change to the existing session-restore (feature 003): one more value recorded per
buffer, applied on restore. Older session files (written before this change) remain loadable; their tabs
restore to the configured default exactly as today.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Tabs reopen in their saved wrap state (Priority: P1)

As a user who has set wrap differently on different tabs, when I quit and reopen the editor and restore
the session, each tab comes back with the same wrap setting it had when I left.

**Why this priority**: This is the whole feature — per-tab wrap is only fully useful if it survives a
restart like the rest of the session (open files, cursor positions).

**Independent Test**: Build a session where some tabs are wrapped and others are not; save it; restore it
into a fresh editor; confirm each restored tab's wrap setting matches what was saved.

**Acceptance Scenarios**:

1. **Given** a session saved with tab A wrapped and tab B unwrapped, **When** the session is restored,
   **Then** tab A is wrapped and tab B is unwrapped.
2. **Given** a saved session, **When** it is restored, **Then** each tab's wrap setting is restored
   independently of the others and independently of the global default.

---

### User Story 2 - Old sessions still load (Priority: P1)

As an existing user, when the editor restores a session file written before this feature (which has no
recorded wrap state), it loads without error and each tab uses the configured default — exactly the prior
behavior.

**Why this priority**: A schema change must not break or discard existing saved sessions.

**Independent Test**: Restore a session file with no recorded wrap field; confirm it loads successfully
and tabs come up at the configured default with no error.

**Acceptance Scenarios**:

1. **Given** a pre-existing session file with no wrap field, **When** it is loaded, **Then** it restores
   successfully (no version/parse error) and tabs use the default wrap setting.
2. **Given** a session file written by this feature, **When** it is loaded by this version, **Then** each
   tab's saved wrap state is applied.

---

### Edge Cases

- **Missing wrap value** (old file): treated as the configured default (the prior behavior), not an
  error.
- **Buffers excluded from the session**: a buffer with no on-disk path (or whose file no longer exists)
  is already excluded from session save; its wrap state is simply not persisted (nothing to restore to).
- **Schema version**: bumping the session schema must keep older files loadable (accept the previous
  version too), so a downgrade-then-upgrade or an old file never errors.
- **New files opened after restore**: continue to seed from the configured default (unchanged); only
  *restored* tabs use a persisted value.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The saved session MUST record each persisted tab's soft-wrap setting (alongside its path
  and cursor position).
- **FR-002**: On session restore, each restored tab's soft-wrap setting MUST be set from its recorded
  value, independently per tab.
- **FR-003**: A session file written without a recorded wrap value (older schema) MUST still load
  successfully, with each such tab defaulting to the configured default wrap setting (no error, no data
  loss of the rest of the session).
- **FR-004**: The session schema version MUST be handled so that both the previous version and the new
  version load correctly; loading must not reject a file solely because it predates this field.
- **FR-005**: Saving the session MUST capture the *current* per-tab wrap state (the live value on each
  buffer), so a round-trip (save → restore) reproduces the wrap layout.
- **FR-006**: Behavior for users who never restore a session, and for newly opened tabs after a restore,
  MUST be unchanged (new tabs still seed from the configured default).

### Key Entities

- **Per-buffer session record**: the existing per-tab entry in the saved session (path + cursor), now
  extended with that tab's soft-wrap setting.
- **Session schema version**: the marker that lets the loader accept both pre- and post-feature files.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A save→restore round-trip of a session with mixed wrap states reproduces each tab's wrap
  setting exactly (verifiable by a test that saves, restores, and asserts per-tab wrap).
- **SC-002**: A session file lacking the wrap field loads successfully and restores tabs at the default
  (verifiable by a test deserializing a legacy payload).
- **SC-003**: The full existing test suite passes, including the feature-003 session tests (adjusted only
  for the additive field) and the 044 per-tab tests; `fmt` + `clippy -D warnings` clean.
- **SC-004**: No change to the editor's behavior for users who do not use session restore.

## Assumptions

- The session-restore mechanism (feature 003) is the persistence vehicle; this feature only adds one
  recorded value per buffer and applies it on restore. No new storage location or format family.
- The recorded wrap value is a simple on/off per buffer; nothing else about wrap (cache, geometry) is
  persisted — those are recomputed at runtime as today.
- The configured default (`config.soft_wrap`) remains the fallback for tabs without a recorded value and
  for newly opened tabs (feature 044 semantics).
- Persisting wrap does not change which buffers are eligible for the session (still only buffers with an
  existing on-disk path, per feature 003).
