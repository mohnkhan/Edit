# Feature Specification: Sandboxed fuzz covering file-I/O actions

**Feature Branch**: `050-sandboxed-fuzz-io`

**Created**: 2026-06-21

**Status**: Draft

**Input**: User description: "Add a sandboxed no-panic fuzz sweep that covers the file-I/O actions (Save, SaveAs, SaveAsEncoding, Open, Revert) which the feature-042 sweep excludes because they reach the real filesystem (follow-up to #72/#79)."

## Overview

The feature-042 no-panic fuzz sweep (`no_panic_under_random_input_sweep`) deliberately excludes the
file-I/O actions — Save, SaveAs, SaveAsEncoding, Open, Revert — because they reach the real
filesystem: the file browser builds paths from the current working directory plus a typed name, and
buffer save/open read and write disk. As a result, a large and historically crash-prone slice of the
input space (modal browser navigation + path entry + encoding selection driving real reads/writes)
is never exercised by the panic guard.

This feature adds a **second, sandboxed** deterministic fuzz sweep that includes those I/O actions
while confining every read and write to a disposable temporary directory, so the working tree is
never touched. It is a test-only, behavior-preserving addition: no production code changes are
expected.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - File-I/O actions are fuzzed without crashing (Priority: P1)

As the maintainer (frustrated by avoidable crashes — see #72), I want the editor's file-open/save
paths exercised by the same kind of long pseudo-random input sweep that already guards the non-I/O
actions, so that a control-flow change which would panic on a browser/save/open sequence is caught by
the test suite rather than by a user.

**Why this priority**: This is the entire purpose of the feature and the only externally meaningful
outcome — closing the coverage gap left open by feature 042.

**Independent Test**: Run the new sweep; it drives randomized sequences that include Save / SaveAs /
SaveAsEncoding / Open / Revert across modal and non-modal states and several terminal sizes, and
completes with zero panics.

**Acceptance Scenarios**:

1. **Given** a deterministic seed, **When** the sandboxed sweep runs its full iteration budget
   including the file-I/O actions, **Then** the editor never panics and the test passes.
2. **Given** the same seed on any host or repeated run, **When** the sweep runs, **Then** it produces
   identical behavior (no wall-clock or RNG nondeterminism).
3. **Given** the file browser is open during the sweep, **When** path characters are typed and
   Open/Save are confirmed, **Then** reads/writes resolve inside the sandbox and do not panic.

---

### User Story 2 - The repository working tree stays clean (Priority: P1)

As a developer running `cargo test`, I want the new I/O sweep to leave my checkout exactly as it was,
so that running the suite never creates, modifies, or deletes tracked or untracked files in the repo.

**Why this priority**: A fuzz test that writes into the working tree (the exact reason feature 042
excluded these actions) would be unacceptable — it could corrupt source files. Hermeticity is a
non-negotiable precondition for including I/O actions at all.

**Independent Test**: Capture `git status` before and after the sweep run; they are identical, and no
new files appear anywhere under the repository root.

**Acceptance Scenarios**:

1. **Given** a clean working tree, **When** the sandboxed sweep runs to completion, **Then** the
   working tree is still clean (no added/modified/deleted files under the repo).
2. **Given** the sweep redirects the working directory and per-user state locations into the sandbox,
   **When** it finishes, **Then** the original working directory and environment are restored and the
   sandbox is removed.
3. **Given** other tests also depend on the process working directory or per-user state, **When** the
   sweep mutates those global resources, **Then** the mutation is serialized so concurrent tests are
   not disturbed.

---

### Edge Cases

- **Sub-minimum terminal size**: the sweep must include a size below the 80×24 minimum (the
  "too small" render path) as well as the 80×24 minimum itself and at least one larger size.
- **Open/Revert targets**: at least one real readable file must exist in the sandbox so Open and
  Revert have a valid target; the sweep must also tolerate Open/Revert when no path or a bad path is
  selected (no-op / surfaced message, never a panic).
- **Stacked overlays**: I/O actions may fire while a modal (file browser, encoding select, confirm
  dialog) is already open; the sweep must reach those combinations.
- **Large-file guard**: sandbox seed files are small, so the 256 MiB refusal path is not exercised
  here (out of scope; already covered elsewhere).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The test suite MUST include a sandboxed no-panic fuzz sweep whose action set contains
  the file-I/O actions (Save, SaveAs, SaveAsEncoding, Open, Revert) in addition to the actions already
  covered by the feature-042 sweep.
- **FR-002**: The sweep MUST confine all filesystem reads and writes performed during the run to a
  disposable sandbox directory; it MUST NOT create, modify, or delete any file under the repository
  working tree.
- **FR-003**: The sweep MUST redirect the process working directory and the per-user state/config
  locations into the sandbox for the duration of the run, and MUST restore them afterward.
- **FR-004**: The sweep MUST serialize its global working-directory/environment mutation so it does
  not interfere with other tests that read those globals.
- **FR-005**: The sweep MUST seed at least one small readable file in the sandbox so Open and Revert
  have a valid target.
- **FR-006**: The sweep MUST be deterministic — driven by a fixed-seed pseudo-random generator with no
  dependency on wall-clock time or an external RNG — so failures reproduce identically.
- **FR-007**: The sweep MUST exercise all overlay/modal states and at least three terminal sizes,
  including the 80×24 minimum and one sub-minimum ("too small") size.
- **FR-008**: The sweep MUST assert that no panic occurs across its full iteration budget.
- **FR-009**: The existing feature-042 sweep and all other existing tests MUST remain unchanged in
  behavior (this feature only adds a test; it does not alter assertions or production code).
- **FR-010**: On completion the sweep MUST remove the sandbox directory it created.

### Key Entities

- **Sandbox**: a uniquely-named temporary directory that serves as the working directory and the
  per-user state/config root for the duration of the run; seeded with small files; removed at the end.
- **Sweep driver**: the deterministic generator + dispatch loop that issues randomized actions and
  mouse events and asserts no panic (a sibling of the existing non-I/O sweep).

## Success Criteria *(mandatory)*

- **SC-001**: The sandboxed sweep runs its full iteration budget across the required terminal sizes
  and overlay states with **zero panics**.
- **SC-002**: Running the full test suite leaves the repository working tree **byte-identical** to
  before the run (no added/modified/deleted files).
- **SC-003**: The new sweep is **deterministic**: two runs from the same seed exhibit identical
  behavior.
- **SC-004**: `cargo fmt --check` and `cargo clippy -D warnings` are clean with the new test present.
- **SC-005**: The pre-existing test count is unchanged except for the newly added sweep test(s); no
  existing assertion is modified.

## Assumptions

- The existing `no_panic_under_random_input_sweep` and its `make_app`/xorshift helpers are the model
  and may be reused/refactored, provided existing behavior is preserved.
- Redirecting the per-user state/config locations is sufficient to keep session, recent-files,
  recovery, and log writes inside the sandbox (these already resolve via `$XDG_STATE_HOME` /
  `$XDG_CONFIG_HOME`).
- The file browser resolves typed relative names against the process working directory, so setting the
  working directory into the sandbox is sufficient to contain browser-driven I/O.
- No production behavior needs to change; if the sweep reveals a real panic, fixing it is in scope as a
  behavior-preserving guard (consistent with feature 042), but none is anticipated.

## Scope

**In scope**: a test-only sandboxed fuzz sweep covering the five file-I/O actions; the sandbox
setup/teardown and global-state serialization required to keep it hermetic.

**Out of scope**: changing the existing feature-042 sweep; the 256 MiB large-file refusal path; any
user-visible behavior or capability change (no `docs/CAPABILITIES.md` change); driving the real binary
via a PTY (this is an in-process API sweep).

## Dependencies

- Builds on feature 042 (the original no-panic sweep and its helpers).
- Closes #79 (follow-up to #72).
