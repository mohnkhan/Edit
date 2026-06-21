# Feature Specification: Harden Error Handling — Eliminate Residual Panic Surfaces

**Feature Branch**: `042-harden-error-handling`

**Created**: 2026-06-21

**Status**: Draft

**Input**: User description: "Harden error handling to eliminate residual panic surfaces (#72). Convert the guarded `unwrap()`/`expect()` calls in the App submodules into pattern matches so the compiler enforces the invariant; add a panic-free random key+mouse fuzz sweep; add a clippy lint guardrail so new panic surfaces can't creep in. Behavior-preserving; no user-visible change."

## Overview

The editor already has a real crash-recovery net (a panic hook that restores the terminal and writes a
crash log) — but recovery exists precisely because too many code paths can still *panic*. An audit
(issue #72) found the dominant residual risk is ~27 "guarded" `unwrap()`/`expect()` calls in the input
and dialog handling: each is safe *today* only because a separate check earlier in the same function
established the value is present (e.g. "a dialog is open, therefore unwrapping the open dialog is fine").
That safety lives in a programmer's head, not in the type system — so a future change to the control flow
silently turns one into a crash on ordinary input.

This feature removes that fragility three ways, all **behavior-preserving**: (1) rewrite each guarded
unwrap as a pattern match whose absent-arm is the existing no-op/fall-through, so the compiler enforces
the invariant; (2) add an automated test that drives long pseudo-random sequences of keystrokes and
mouse events through the editor — across every modal state and several terminal sizes — and asserts it
never panics; (3) add a lint that fails the build if new guarded unwraps are introduced in the editor's
core input code. The maintainer's stated priority is blunt and fair: a basic text editor must not crash
on normal use, and that expectation drives the acceptance bar below.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The editor does not crash on ordinary input (Priority: P1)

As a user, no sequence of normal keystrokes and mouse actions — opening/closing dialogs, navigating
menus, clicking around, typing, resizing — causes the editor to crash, regardless of order or timing.

**Why this priority**: This is the entire point of the feature and the maintainer's explicit demand. A
crash on normal input is the worst failure mode for an editor (it can lose work and breaks trust).

**Independent Test**: A test harness drives long pseudo-random sequences of keyboard and mouse events
(deterministic seed) against the editor across several terminal sizes and asserts zero panics. Running it
is the acceptance check; it generalizes the existing single-row no-panic sweep to the whole input space.

**Acceptance Scenarios**:

1. **Given** the editor in any state, **When** an arbitrary long sequence of valid keyboard actions and
   mouse events is applied, **Then** the editor never panics (it may ignore inputs, but it stays alive).
2. **Given** any overlay open (Find/Replace, Go-to-Line, file browser, Help, encoding, plugin manager,
   confirmations, context menu), **When** further arbitrary input arrives, **Then** no panic occurs.
3. **Given** a range of terminal sizes (including the minimum and non-default sizes), **When** the same
   input sequences are applied, **Then** no panic occurs.

---

### User Story 2 - Invariants are enforced by the compiler, not by convention (Priority: P1)

As a maintainer, the places that previously "knew" a value was present now express that knowledge in a
form the compiler checks, so a later edit cannot reintroduce a crash without the build catching it.

**Why this priority**: Removing the *class* of bug matters more than patching instances. If the invariant
is compiler-enforced, the fuzz test (US1) can't regress silently and future contributors can't redo it.

**Independent Test**: Inspect the targeted input/dialog code — no guarded `unwrap()`/`expect()` remains;
each is a pattern match whose absent-arm preserves prior behavior. A lint rejects new occurrences in the
covered code.

**Acceptance Scenarios**:

1. **Given** the editor's core input/dialog code, **When** it is reviewed/linted, **Then** it contains no
   `unwrap()`/`expect()` on a runtime/input-derived value; each is a checked pattern match.
2. **Given** a contributor adds a new `unwrap()` in that code, **When** the build runs, **Then** the lint
   fails the build (the guardrail catches it).

---

### Edge Cases

- **Absent-arm behavior must equal prior behavior.** Converting `X.unwrap()` (after a guard) to
  `if let Some(x) = X { … }` must leave the "else" path doing exactly what the code did before — which,
  for these guarded sites, is "nothing / fall through," because the guard guaranteed presence. The
  conversion must not introduce a new branch that changes output.
- **Async/stacked overlays.** The external-change prompt and plugin-consent queue can be pending while
  another overlay is open; the fuzz sweep must be able to reach those states (not just the simple ones).
- **Determinism.** The fuzz sweep must be reproducible (fixed seed, no wall-clock/RNG nondeterminism) so
  a failure is debuggable and CI is stable.
- **Out of scope (must not regress, not fixed here):** raw slice/index access (`buf[i]`) is tracked
  separately; this feature does not convert those except a trivially-safe input-derived index if one is
  encountered. `Regex::new("<literal>").unwrap()` in syntax highlighting and best-effort `let _ =`
  cleanup are acceptable and untouched.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every guarded `unwrap()`/`expect()` on a runtime/input-derived value in the editor's core
  input and dialog handling (the App key-dispatch, mouse, and dialog code) MUST be replaced by a pattern
  match (`if let` / `let … else`) whose absent-arm reproduces the prior behavior (no-op or existing
  fall-through). No user-visible behavior may change.
- **FR-002**: The conversion MUST be behavior-preserving: for inputs that previously did not panic, the
  observable result (buffer contents, cursor, open overlay, status message) MUST be identical.
- **FR-003**: An automated test MUST drive long pseudo-random sequences of keyboard actions AND mouse
  events against the editor and assert it never panics. It MUST cover: the no-overlay editing state, each
  foreground overlay, the menu/dropdown layers, and a set of terminal sizes including the minimum and at
  least one non-default size.
- **FR-004**: The fuzz test MUST be deterministic — driven by a fixed seed with no reliance on wall-clock
  time or a nondeterministic RNG — so failures reproduce and CI is stable.
- **FR-005**: A lint guardrail MUST be configured so that introducing a new `unwrap()`/`expect()` on a
  fallible value in the covered core input code fails the build (or the project's warnings-as-errors
  gate), preventing silent reintroduction of the panic class.
- **FR-006**: The guardrail MUST NOT flag the explicitly-accepted cases: compile-time-constant
  `Regex::new("<literal>").unwrap()` in syntax highlighting, and best-effort `let _ =` cleanup (terminal
  restore, lock-file removal, crash-report writes). These remain as-is.
- **FR-007**: The full pre-existing automated test suite MUST pass unchanged (no assertion altered). The
  only test additions are the new fuzz sweep and any helper it needs.
- **FR-008**: The existing crash-recovery net (panic hook restoring the terminal + crash log, signal
  handler) MUST remain intact — this feature reduces the *need* for it, it does not remove it.

### Key Entities

- **Guarded unwrap site**: a place that unwraps a value already proven present by an earlier check; the
  unit of conversion. After conversion it is a pattern match with a behavior-preserving absent-arm.
- **Fuzz input sequence**: a deterministic, seeded stream of keyboard actions + mouse events applied to a
  freshly built editor; the test asserts no panic across the stream.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero guarded `unwrap()`/`expect()` on runtime/input-derived values remain in the covered
  core input/dialog code (verifiable by inspection + the lint).
- **SC-002**: The new fuzz sweep applies at least several thousand combined keyboard+mouse events across
  all overlay states and ≥3 terminal sizes (incl. the minimum) with **zero panics**.
- **SC-003**: The full pre-existing test suite passes with no assertion changes; total pass count is
  unchanged except for the added test(s).
- **SC-004**: The local CI gate (formatting, linter with warnings-as-errors, all tests) passes clean,
  and a newly added `unwrap()` in the covered code makes it fail (guardrail demonstrably active).
- **SC-005**: No user-visible behavior changes — the editor looks and acts identically for all inputs
  that did not previously crash.

## Assumptions

- The existing test suite plus the new fuzz sweep is a sufficient safety net to prove behavior
  preservation; the conversions are local and mechanical (guard already established presence).
- "Covered core input code" means the App's key-dispatch, mouse, and dialog modules
  (`src/app/dispatch.rs`, `src/app/mouse.rs`, `src/app/dialogs.rs`, and the residual handlers in
  `src/app.rs`). Widening the lint to the whole crate is out of scope (would flag the accepted
  highlight/cleanup cases and the separately-tracked raw-index work).
- Raw `[index]`/slice access hardening is a separate, already-tracked effort (follow-up to #72) and is
  not part of this feature.
- A deterministic pseudo-random generator seeded from a fixed constant is acceptable for the fuzz sweep
  (the project's test convention forbids wall-clock/RNG nondeterminism).
