# Feature Specification: Restore Scroll, Selection & Encoding in Session

**Feature Branch**: `047-session-more-fields` | **Created**: 2026-06-21 | **Status**: Draft
**Input**: Issue #83 — "Persist scroll offset / selection / encoding in session restore."

## Overview

Session restore (003/045) brings back each tab's path, cursor, split layout, and soft-wrap. This feature
extends it so a restored tab also returns to its **scroll position**, its **active selection**, and the
**character encoding** it was viewed in — a fuller "pick up exactly where I left off." It is a small
additive extension of the existing session schema; older session files remain loadable (the new values
default to "none/as-opened", i.e. today's behavior).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Reopen exactly where I was (Priority: P1)
As a user restoring a session, each tab returns to the same scroll position and (if any) selection, and
opens in the same encoding it was using.

**Independent Test**: Save a session whose tabs have non-zero scroll, an active selection, and a
non-UTF-8 encoding; restore; assert each is reproduced (clamped to the file's current bounds).

**Acceptance Scenarios**:
1. **Given** a tab scrolled down N rows, **When** restored, **Then** it reopens scrolled to N (clamped).
2. **Given** a tab with an active selection, **When** restored, **Then** the same selection is active
   (clamped to current content).
3. **Given** a tab opened as e.g. UTF-16/CP437, **When** restored, **Then** it reopens decoded in that
   encoding.

### User Story 2 - Old sessions still load (Priority: P1)
A session file written before this feature loads without error; tabs restore with no selection, scroll 0,
and the as-opened encoding (today's behavior).

**Acceptance Scenarios**:
1. **Given** a pre-feature session (no scroll/selection/encoding fields), **When** loaded, **Then** it
   restores successfully with defaults (no error).

### Edge Cases
- Saved positions out of range after external edits → clamped to current content (never panic).
- Selection endpoints clamped independently; an empty/degenerate selection restores as no selection.
- Unknown/absent encoding string → fall back to the default decode (as today).
- Buffers excluded from the session (no on-disk path) persist nothing — unchanged.

## Requirements *(mandatory)*

- **FR-001**: A persisted tab MUST record its scroll offset (row + column).
- **FR-002**: A persisted tab MUST record its active selection (anchor + active positions), if any.
- **FR-003**: A persisted tab MUST record its character encoding.
- **FR-004**: On restore, scroll/selection/encoding MUST be applied per tab, clamped to current content
  (no panic on out-of-range), and the buffer decoded in the recorded encoding.
- **FR-005**: A session file lacking these fields MUST still load (defaults: scroll 0, no selection,
  as-opened encoding) — backward compatible, no error.
- **FR-006**: Behavior for users who never restore, and for newly opened tabs, MUST be unchanged.
- **FR-007**: Full suite passes (003/045 session tests adjusted only for additive fields); `fmt` +
  `clippy -D warnings` clean; the 042 unwrap & 046 index guardrails hold.

## Success Criteria *(mandatory)*

- **SC-001**: A save→restore round-trip reproduces scroll, selection, and encoding per tab (test).
- **SC-002**: A legacy file (without the fields) loads with defaults (test).
- **SC-003**: Out-of-range saved positions restore clamped with no panic (test).
- **SC-004**: Full suite + lints green; no behavior change for non-session users.

## Assumptions
- Additive `#[serde(default)]` fields on the per-tab session record; schema stays v2 (045) — old files
  load. Encoding is stored as a canonical string that round-trips through the existing parser.
- Selection is restored as-is (clamped); restoring a selection on reopen is acceptable UX per #83.
- The buffer is (re)opened/decoded in the recorded encoding via the existing open path.
