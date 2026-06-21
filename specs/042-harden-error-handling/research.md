# Phase 0 Research: Harden Error Handling

All decisions are codebase-internal (this is a robustness refactor of existing code), verified against
the post-041 source.

## R1 — Conversion shape for guarded unwraps

**Decision**: Replace `let x = EXPR.unwrap(); <use x>` (where an earlier guard proved `EXPR` is `Some`)
with `if let Some(x) = EXPR { <use x> }`. For early-return arms, `let Some(x) = EXPR else { return … };`
using the function's existing early-return value. The absent-arm does exactly what the code did when the
value was absent — which, for these guarded sites, is "nothing" (the guard guaranteed presence).

**Rationale**: This is the minimal change that makes the compiler enforce the invariant (FR-001/FR-002)
without altering observable behavior. **Alternatives**: (a) `expect("invariant")` — still panics, doesn't
remove the class; rejected. (b) restructure each block to bind the value once at the guard — cleaner but a
larger diff with higher behavior-change risk; rejected for a behavior-preserving PR.

**Verified site counts (post-041)**: `dispatch.rs` 14, `mouse.rs` 7, `dialogs.rs` 3, `app.rs` 3 = 27.
Representative sites: `self.find_replace_mut().unwrap()` (after a `Modal::FindReplace` guard),
`self.file_browser_mut().unwrap()`, `self.pending_external_change.take().unwrap()`,
`self.scrollbar_drag.unwrap()`.

## R2 — Lint guardrail scoping

**Decision**: Add `#![deny(clippy::unwrap_used, clippy::expect_used)]` as an inner attribute at the top
of `src/app.rs`. Rust lint-level attributes propagate to child modules, so this one attribute covers all
`src/app/*.rs` submodules. Re-allow in test code: `#![allow(clippy::unwrap_used, clippy::expect_used)]`
at the top of `src/app/tests.rs`, and `#[allow(...)]` on the two inline `#[cfg(test)] mod …` debug
modules in `app.rs`.

**Rationale**: One attribute gives whole-app-tree coverage (FR-005) while the accepted cases (FR-006) —
`Regex::new("<literal>").unwrap()` in `highlight/languages/*` and best-effort `let _ =` cleanup — live in
*other* modules and are untouched. `clippy::unwrap_used`/`expect_used` are allow-by-default restriction
lints, so this is opt-in and won't affect the rest of the crate. **Alternatives**: (a) crate-wide deny —
would flag the accepted highlight/cleanup cases and the deferred raw-index sites; rejected. (b) per-file
deny in only the 3 converted files — misses prevention in the other app submodules; the propagating
`app.rs` attribute is strictly better.

**Verified**: post-conversion the app production code has zero `unwrap`/`expect`, so the deny is clean
under `cargo clippy -- -D warnings`. The lint applies to test code unless re-allowed → hence the
`allow` in `tests.rs` (which uses `.unwrap()` heavily) and on the inline dbg modules.

**Caveat to confirm at implement time**: clippy must actually run over the deny'd modules; `make
ci-local` runs `cargo clippy -- -D warnings` (all targets) which does. A stray new `unwrap()` in app
production code will then fail the build — the demonstrable guardrail (SC-004).

## R3 — Deterministic fuzz sweep

**Decision**: Add a `#[test]` in `tests.rs` that, for a fixed array of seeds and a set of terminal sizes
(incl. the 80×24 minimum and ≥1 larger and a sub-minimum to hit the `too_small` path), builds an `App`
(reusing the existing `make_app` helper), then applies a long sequence of pseudo-random events —
alternating keyboard `Action`s drawn from a curated representative list (incl. overlay openers: Find,
Replace, GoToLine, Help, About, SaveAsEncoding, OpenPluginManager, OpenFile; editing: InsertChar,
Backspace, Delete, arrows, Home/End; menu/Esc) and mouse events (`Down(Left/Right)`, `ScrollUp/Down`,
`Drag`, `Up`) at random in-bounds (and some out-of-bounds) coordinates — calling `render` periodically.
A panic fails the test (no `catch_unwind` needed). PRNG: a hand-rolled `xorshift64` seeded from a fixed
constant (no `rand`, no `Date::now`).

**Rationale**: Generalizes the existing `repro_menu_click_over_tabs` single-row sweep (FR-003) to the
whole input space with reproducibility (FR-004). A hand-rolled xorshift keeps the build dependency-free
(Principle IV) and deterministic (project test convention forbids wall-clock/RNG nondeterminism).
**Alternatives**: (a) the `rand` crate — adds a dep for a test; rejected. (b) `cargo-fuzz`/libfuzzer —
heavyweight, nondeterministic, CI-unfriendly for this purpose; rejected. (c) exhaustive sweep — infeasible
over the combined key+mouse+size space; seeded random with enough iterations is the pragmatic cover.

## R4 — Behavior-preservation guarantee

**Decision**: Rely on the existing suite (1262 tests) for equivalence; the conversions are local and the
absent-arm is provably the prior no-op. The fuzz test adds the no-panic guarantee. No assertion changes
(FR-007). The panic hook / signal handler in `diagnostics/crash.rs` are not touched (FR-008).

## Open questions

None. No `NEEDS CLARIFICATION` remain.
