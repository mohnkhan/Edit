# Implementation Plan: Harden Error Handling — Eliminate Residual Panic Surfaces

**Branch**: `042-harden-error-handling` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/042-harden-error-handling/spec.md`

## Summary

Issue #72. Remove the residual *panic class* in the editor's input handling: ~27 guarded
`unwrap()`/`expect()` calls that are safe only by a hand-tracked local invariant. Three workstreams,
all behavior-preserving: (1) convert each to a pattern match (`if let`/`let … else`) whose absent-arm is
the existing no-op/fall-through, so the compiler enforces the invariant; (2) add a deterministic
no-panic fuzz sweep driving random keyboard `Action`s + mouse events across overlay states and several
terminal sizes; (3) add a `clippy::unwrap_used`/`expect_used` deny scoped to the App module tree so new
guarded unwraps can't creep in. Proven by the unchanged existing suite + the new fuzz test, with
`fmt`/`clippy -D warnings` clean.

## Technical Context

**Language/Version**: Rust, edition 2021, MSRV 1.74

**Primary Dependencies**: `ratatui` + `crossterm`, `ropey`. No new runtime deps. The fuzz PRNG is a
hand-rolled fixed-seed xorshift (no `rand` crate — keeps the build dep-free and deterministic).

**Storage**: N/A

**Testing**: `cargo test` (incl. the new fuzz sweep); `make smoke`; `make perf-check`

**Target Platform**: Linux TUI (VT100+)

**Project Type**: Single-project desktop TUI application

**Performance Goals**: No regression. The fuzz test is bounded (fixed iteration count) so it stays well
within the unit-test time budget.

**Constraints**: Behavior-preserving (FR-002); existing test assertions unchanged (FR-007); the panic
hook + signal handler stay intact (FR-008); determinism — no `Date::now`/RNG nondeterminism (FR-004,
matches the project's test convention).

**Scale/Scope**: ~27 call-site conversions across 4 files + one new test + a few lint attributes.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. DOS-Faithful UI** — PASS. No UI change (behavior-preserving).
- **II. UTF-8 First** — PASS / N/A. No new byte-decoding path.
- **III. Portable Build** — PASS. Pure Rust; no platform code. (Constitution text predates the
  `ratatui`/`crossterm` move and still says ncurses; the live stack per `CLAUDE.md` is
  `ratatui`/`crossterm` — followed here. Not a deviation needing Complexity Tracking.)
- **IV. Minimal Footprint** — PASS. No new dependency (PRNG is hand-rolled).
- **V. Test-Gated Merges (NON-NEGOTIABLE)** — PASS, strengthened. Adds a no-panic fuzz sweep and a lint
  guardrail; full suite stays green. This feature *increases* the safety net.
- **VI. Simplicity / YAGNI** — PASS. Smallest change that removes the class: pattern matches + one test
  + lint attributes. No framework. Raw-index hardening explicitly deferred (separate issue).
- **VII. Security Hardening** — PASS / supportive. Panic-resistance on untrusted input (e.g. a file's
  bytes reaching the buffer, terminal events) reduces denial-of-service-by-crash; recovery net retained.

**Result**: All gates pass. Complexity Tracking empty.

## Project Structure

### Documentation (this feature)

```text
specs/042-harden-error-handling/
├── plan.md, spec.md, research.md, data-model.md, quickstart.md
├── contracts/internal-api.md
├── checklists/requirements.md
└── tasks.md            # /speckit-tasks output
```

### Source Code (repository root)

```text
src/
├── app.rs              # add #![deny(clippy::unwrap_used, clippy::expect_used)] (propagates to submods);
│                       #   #[allow(...)] on the inline dbg test modules; convert its 3 guarded unwraps.
├── app/
│   ├── dispatch.rs     # convert 14 guarded unwraps → if let / let-else
│   ├── mouse.rs        # convert 7
│   ├── dialogs.rs      # convert 3
│   └── tests.rs        # add #![allow(clippy::unwrap_used, clippy::expect_used)]; add the fuzz sweep test
```

**Structure Decision**: Unchanged layout (post-041 module split). Work is localized to the four App
input/dialog files plus the test module.

## Phased Approach (one PR, ordered commits)

- **Phase 1 — convert guarded unwraps** (dispatch → mouse → dialogs → app.rs), building + testing after
  each file. Each conversion's absent-arm reproduces prior behavior.
- **Phase 2 — fuzz sweep**: add the deterministic random Action+mouse no-panic test in `tests.rs`.
- **Phase 3 — lint guardrail**: add the `deny` in `app.rs` + `allow` in test code; confirm `clippy -D
  warnings` clean; demonstrate (locally) that a temporary stray `unwrap()` makes it fail, then revert.

## Complexity Tracking

*No constitution violations. Table intentionally empty.*
