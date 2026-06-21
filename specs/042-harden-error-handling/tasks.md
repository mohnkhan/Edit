# Tasks: Harden Error Handling — Eliminate Residual Panic Surfaces

**Feature**: `042-harden-error-handling` | **Branch**: `042-harden-error-handling`
**Input**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/internal-api.md](./contracts/internal-api.md)

**Overriding constraint**: Behavior-preserving (FR-002). Each converted site's absent-arm reproduces the
prior no-op / fall-through; for any input that didn't panic before, observable state is identical. No
existing test assertion changes (FR-007); the only test additions are the fuzz sweep. The panic
hook/signal handler in `diagnostics/crash.rs` stay untouched (FR-008).

**Story → phase map**: US1 = no-crash (P1), US2 = compiler-enforced invariants (P1). The conversions
serve both; they're sequenced under US2 (the enabling change) and US1's fuzz test proves the outcome.

---

## Phase 1: Setup

- [ ] T001 Record the baseline: `make tmpfs-setup` then `make check`; note the passing test count
  (current baseline 1262/0/11) for the "unchanged suite" comparison. No code changes.

---

## Phase 2: User Story 2 — Compiler-enforced invariants (Priority: P1)

**Goal**: Replace every guarded `unwrap()`/`expect()` in the App input/dialog code with a pattern match,
and add the lint guardrail so they can't return.

**Independent test**: `clippy -D warnings` clean with the new `deny`; a stray `unwrap()` in app
production code fails the build; inspection shows no guarded unwrap remains.

- [ ] T002 [US2] Convert the 14 guarded `unwrap()`/`expect()` in `src/app/dispatch.rs` to `if let` /
  `let … else` with behavior-preserving absent-arms (e.g. `if let Some(d) = self.find_replace_mut() { … }`;
  `if let Some(ec) = self.pending_external_change.take() { … }`). Build + `cargo test` green after.
- [ ] T003 [US2] Convert the 7 in `src/app/mouse.rs` (incl. `self.scrollbar_drag` and
  `file_browser*()` unwraps). Build + test green.
- [ ] T004 [US2] Convert the 3 in `src/app/dialogs.rs`. Build + test green.
- [ ] T005 [US2] Convert the 3 in `src/app.rs`. Build + test green.
- [ ] T006 [US2] Add `#![deny(clippy::unwrap_used, clippy::expect_used)]` as an inner attribute at the
  top of `src/app.rs` (propagates to all `src/app/*` submodules).
- [ ] T007 [US2] Re-allow in test code: `#![allow(clippy::unwrap_used, clippy::expect_used)]` at the top
  of `src/app/tests.rs`, and `#[allow(clippy::unwrap_used, clippy::expect_used)]` on the two inline
  `#[cfg(test)] mod …` debug modules in `src/app.rs`. Run `cargo clippy --all-targets -- -D warnings` →
  must be clean. **Also confirm (FR-006) the deny did NOT force changes outside the `app` tree** — the
  `highlight/languages/*` `Regex::new(literal).unwrap()` and best-effort `let _ =` cleanup are in other
  modules and remain untouched (the clean clippy run with no diffs there is the evidence).
- [ ] T008 [US2] Demonstrate the guardrail: temporarily add a stray `unwrap()` in `src/app/dispatch.rs`,
  confirm `cargo clippy --all-targets -- -D warnings` FAILS, then revert. (Verification only; no
  committed change.)

**Checkpoint (US2)**: zero guarded unwraps in app production code; clippy guardrail active; suite green.

---

## Phase 3: User Story 1 — No crash on ordinary input (Priority: P1)

**Goal**: Prove the editor never panics on arbitrary keyboard+mouse input.

**Independent test**: the new deterministic fuzz sweep runs with zero panics.

- [ ] T009 [US1] **(write the test first — it should already pass once US2 lands)** Add a deterministic
  no-panic fuzz sweep in `src/app/tests.rs`: a fixed-seed `xorshift64` PRNG (no `rand`, no `Date::now`),
  a curated `Action` list (overlay openers Find/Replace/GoToLine/Help/About/SaveAsEncoding/
  OpenPluginManager/OpenFile + editing + nav + Esc) and random `MouseEvent`s (Down Left/Right, Scroll,
  Drag, Up) at in- and slightly-out-of-bounds coords; iterate over a fixed seed array × terminal sizes
  `{(80,24) min, (120,40), (200,60), (40,12) sub-min}`, interleaving `render`. Assert no panic (a panic
  fails the test). Reuse the existing `make_app` helper. Several-thousand combined events total (SC-002).
- [ ] T010 [US1] Run the sweep (`cargo test no_panic` or the chosen name); confirm it passes
  deterministically (run twice → identical). If it surfaces a genuine panic, fix that site by the same
  pattern-match conversion (it's a real bug the audit missed) and note it.

**Checkpoint (US1)**: fuzz sweep green; the no-crash guarantee holds across overlays + sizes.

---

## Phase 4: Polish & Ship

- [ ] T011 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check). Confirm test
  count == baseline + the fuzz test(s); fix any fmt fallout WITHOUT behavior change. Note the
  pre-existing environmental `encoding_select` smoke failure (F12 delivery) is unrelated.
- [ ] T011a Confirm FR-008: `git diff` touches no file under `src/diagnostics/` — the panic hook,
  terminal-restore, and SIGSEGV handler are unchanged (this feature reduces the need for the recovery
  net, it does not modify it).
- [ ] T012 Docs gate: `CHANGELOG.md` (feature 042) + `docs/STATUS.md`. No `CAPABILITIES.md` change
  (behavior-preserving, FR-005/SC-005).
- [ ] T013 Open PR targeting `master` (`feat(042): harden error handling — remove residual panic
  surfaces`), strip any AI-attribution footer, ensure green, `Closes #72`, merge.

---

## Dependencies & Execution Order

- Setup (T001) → US2 conversions+guardrail (T002–T008) → US1 fuzz (T009–T010) → Polish (T011–T013).
- US2 before US1: the fuzz test (US1) is the *proof*; the conversions (US2) are what make the guarantee
  robust. The fuzz test can be written first and should pass even pre-conversion (the sites are guarded
  today) — but landing US2 first means any panic the fuzz finds is a genuinely new discovery, not a
  known guarded site.
- T002–T005 are sequential per-file (each builds+tests); T006/T007 after all conversions (deny must find
  zero residual unwraps). T009 depends only on `make_app` existing (independent of conversions to write,
  but run after for a clean signal).

## Parallel Opportunities

- T002/T003/T004/T005 touch different files and could be done in parallel, but each must build+test, so
  in practice they're sequenced. Low parallelism (small, localized feature).

## MVP Scope

US2 (the conversions + guardrail) is the structural fix; US1 (fuzz) is the proof. Both are small and
ship together in one PR.

## Implementation Strategy

Convert file-by-file behind the existing 1262-test suite (green after each file), then add the lint deny
(now clean), then add the fuzz sweep as the standing no-panic guarantee. Ship as ordered commits in one
PR closing #72.
