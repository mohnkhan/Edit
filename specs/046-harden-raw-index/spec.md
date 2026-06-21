# Feature Specification: Harden Raw Slice/Index Access

**Feature Branch**: `046-harden-raw-index`

**Created**: 2026-06-21

**Status**: Draft

**Input**: Issue #78 — "Harden raw slice/index access (panic surface follow-up to #72)."

## Overview

Feature 042 made the editor's guarded `unwrap()`s panic-safe and added a deterministic no-panic fuzz
sweep, but it deliberately deferred the *other* big panic class: raw `[index]` and `[a..b]` slice
access. When an index is derived from runtime input (a cursor/scroll/mouse position, a dialog selection,
a buffer number) and that value is momentarily out of range, a raw index **panics** instead of degrading
gracefully. For a text editor — where the maintainer's standing requirement is "must not crash on normal
use" — these are exactly the silly crashes to eliminate.

This feature removes the *input-influenced* slice/index panic surface in the editor's hot paths: string
byte-slices become char-boundary-safe, list lookups from a selection/cursor become bounds-checked,
computed buffer indices that aren't invariant-proven use checked access, and rope line-index helpers are
made total. It is **behavior-preserving** for every in-range input (which is all real input); the change
only converts "panic on a stale/out-of-range index" into the same graceful no-op/clamp the surrounding
code already intends. Discovery and proof come from extending the existing deterministic fuzz so it
exercises content-bearing buffers and asserts zero panics.

Indices that are provably in range (compile-time constants, `buffers[0]` under the always-≥1-buffer
invariant) are out of scope — converting them adds noise without removing risk.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - No crash from any index, on any input (Priority: P1)

As a user, no sequence of editing, navigation, selection, dialog interaction, or mouse action causes the
editor to crash because some internal index briefly pointed out of range.

**Why this priority**: It is the feature — closing the raw-index panic class so the editor degrades
gracefully instead of aborting (continuing the 042/043 no-crash work).

**Independent Test**: An extended deterministic fuzz drives random keyboard + mouse events against
buffers that contain real (multibyte) content across several terminal sizes and overlay states, and
asserts zero panics. Any panic it surfaces is a genuine raw-index bug, fixed at the source.

**Acceptance Scenarios**:

1. **Given** the editor in any state with non-empty (incl. multibyte) buffer content, **When** an
   arbitrary long sequence of input is applied, **Then** it never panics.
2. **Given** a dialog/list overlay (encoding select, plugin manager, context menu) whose selection or
   cursor is at a boundary, **When** an action that indexes the list runs, **Then** it resolves safely
   (no out-of-bounds panic).
3. **Given** a string position derived from a caret/scroll/mouse value, **When** the editor slices the
   string, **Then** it never slices on a non-char-boundary (no byte-slice panic).

---

### User Story 2 - Graceful degradation equals prior intent (Priority: P1)

As a maintainer, where a raw index is replaced by a checked access, the out-of-range branch does exactly
what the surrounding code already intended for "nothing there" (a no-op, an empty result, or a clamp) —
so behavior is identical for every in-range input.

**Why this priority**: Hardening must not change observable behavior for normal use; only the
crash-vs-degrade outcome at the (rare, stale) boundary changes.

**Independent Test**: The full existing suite passes unchanged; the new content-bearing fuzz passes; for
representative converted sites, an in-range input yields the same result as before.

**Acceptance Scenarios**:

1. **Given** any in-range index, **When** the converted access runs, **Then** the result is identical to
   the prior raw-index result.
2. **Given** an out-of-range index (only reachable in stale/transient states), **When** the converted
   access runs, **Then** it yields the surrounding code's intended empty/no-op/clamped behavior — not a
   panic.

---

### Edge Cases

- **Char boundaries**: byte-offset string slices (`&s[a..b]`) must not split a multibyte grapheme;
  use char-boundary-safe slicing or clamp to the nearest boundary.
- **Empty containers**: indexing the first/last element of a possibly-empty list resolves safely.
- **Stale selection/cursor**: a dialog list index or rope line index derived from a value not yet
  re-clamped (between events) resolves safely (the same class 042/043 hardened for unwrap/line_slice).
- **Out of scope (must not regress, not converted here)**: compile-time-constant indices; `buffers[0]`
  under the always-≥1-buffer invariant; pure test-code indexing. The fuzz's file-I/O coverage is a
  separate effort (#79).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Input-influenced string byte-slices in the editor's hot paths MUST be char-boundary-safe
  (never panic on a non-boundary offset); an out-of-range/non-boundary offset degrades to the intended
  clamped/empty result.
- **FR-002**: List lookups indexed by a selection/cursor/focus value (e.g. encoding options, context-menu
  items, plugin instances) MUST use checked access; an out-of-range index resolves to the surrounding
  code's no-op/empty branch, not a panic.
- **FR-003**: Computed buffer indices that are not invariant-proven MUST use checked access
  (`.get()`/`.get_mut()`); an out-of-range buffer index is a safe no-op.
- **FR-004**: Rope line-index helpers used with input-derived line numbers MUST be total (no panic past
  the end), consistent with the already-total `line_slice` (feature 042/034).
- **FR-005**: All conversions MUST be behavior-preserving for in-range inputs: identical observable
  result; only the out-of-range outcome changes from panic to graceful.
- **FR-006**: The deterministic no-panic fuzz MUST be extended to exercise **content-bearing**
  (including multibyte) buffers — so line/grapheme/byte indexing is actually driven — across overlay
  states and terminal sizes, asserting zero panics. It MUST remain deterministic (fixed seed, no
  RNG/wall-clock) and MUST NOT perform real filesystem I/O.
- **FR-007**: The full pre-existing test suite MUST pass unchanged; the `clippy::unwrap_used` guardrail
  (042) MUST keep holding; `fmt`/`clippy -D warnings` clean.
- **FR-008**: Provably-in-range indices (compile-time constants, `buffers[0]`) MAY remain raw; this
  feature does not churn them.

### Key Entities

- **Input-influenced index site**: a raw `[i]`/`[a..b]` whose index derives from runtime input and can be
  momentarily out of range; the unit of conversion (→ checked/boundary-safe access).
- **Content-bearing fuzz buffer**: a buffer seeded with real multibyte text so the sweep exercises line,
  grapheme, and byte indexing (not just empty-buffer dispatch).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The extended content-bearing fuzz applies thousands of random keyboard+mouse events across
  overlay states and ≥3 terminal sizes with **zero panics**, deterministically.
- **SC-002**: No input-influenced raw slice/index panic remains in the covered hot paths (verifiable by
  inspection + the fuzz); any the fuzz found is fixed at its source.
- **SC-003**: The full pre-existing suite passes with no assertion changes; behavior identical for
  in-range inputs (SC for FR-005).
- **SC-004**: `cargo fmt --check` and `cargo clippy --all-targets -D warnings` are clean (042 guardrail
  intact).

## Assumptions

- The existing deterministic fuzz harness (042) is the discovery + proof vehicle; extending it to seed
  multibyte content is sufficient to surface the input-influenced index panics in practice.
- "Hot paths" = the editor's input/dispatch/render/edit/geometry code (`src/app*`, `src/buffer`,
  `src/ui` widgets) — not exhaustive crate-wide conversion. Provably-safe and test-only indices are out
  of scope.
- File-I/O fuzz coverage and a full crate-wide `clippy::indexing_slicing` deny are separate efforts
  (#79 and a potential future lint), not part of this feature.
