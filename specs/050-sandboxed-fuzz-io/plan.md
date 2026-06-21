# Implementation Plan: Sandboxed fuzz covering file-I/O actions

**Branch**: `050-sandboxed-fuzz-io` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/050-sandboxed-fuzz-io/spec.md`

## Summary

Add a second deterministic no-panic fuzz sweep (sibling of feature 042's
`no_panic_under_random_input_sweep`) whose action set **includes** the file-I/O actions — Save,
SaveAs, SaveAsEncoding, Open, Revert — while confining every read/write to a disposable sandbox so
the repository working tree is never touched. Test-only and behavior-preserving: no production code
change is expected. Closes #79 (follow-up to #72).

## Technical Context

**Language/Version**: Rust 2021, MSRV 1.74

**Primary Dependencies**: ratatui + crossterm (TUI), ropey (buffer); test uses
`ratatui::backend::TestBackend` and `std::sync::Mutex` only — no new crates.

**Storage**: filesystem, but redirected — the sweep sets the process working directory and
`$XDG_STATE_HOME` / `$XDG_CONFIG_HOME` into a temp sandbox for its duration.

**Testing**: `cargo test` (inline `#[cfg(test)]` in `src/app/tests.rs`).

**Target Platform**: Linux (x86_64, aarch64).

**Project Type**: desktop-app (terminal TUI), single binary.

**Performance Goals**: the sweep must complete quickly within the normal `cargo test` run (budget
sized like the existing sweep — thousands of dispatched events, not millions).

**Constraints**: deterministic (fixed-seed xorshift64, no `rand`, no `Date::now`/`Instant::now` for
control flow); hermetic (no writes outside the sandbox); must not deadlock or starve concurrent tests
(global cwd/env mutation serialized behind a process-wide mutex; held only for the sweep body).

**Scale/Scope**: one new test function (plus a small shared helper refactor if warranted). ~1 file
touched (`src/app/tests.rs`).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle V — Test-Gated Merges (NON-NEGOTIABLE)**: This feature *is* a test. It adds an
  automated unit/integration-level guard for the file-open/save/revert code paths. ✅ Aligned — it
  strengthens the gate rather than bypassing it. No production behavior changes, so no new user-facing
  behavior needs additional tests.
- **Principle IV — Minimal footprint**: no new runtime dependencies; no new dev-dependencies
  (uses std `Mutex` + existing `TestBackend`). ✅
- **Principle VI — Simplicity/YAGNI**: scope is a single sweep test + minimal helper sharing; no
  speculative infra. ✅
- **Principle VII — Security Hardening**: the sweep reduces a latent crash surface (panic on I/O
  input sequences), consistent with the hardening intent of #72. ✅
- **UTF-8 hygiene**: no new byte→text construction; the sweep drives existing decode paths. ✅

Note: the constitution text mentions ncurses (Principle IV); the real stack is ratatui/crossterm per
`CLAUDE.md` — no impact on this plan.

**Result**: PASS (no violations; no complexity-tracking entries required).

## Project Structure

### Documentation (this feature)

```text
specs/050-sandboxed-fuzz-io/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── internal-api.md  # Phase 1 output (internal test contract)
└── checklists/
    └── requirements.md  # spec quality checklist
```

### Source Code (repository root)

```text
src/app/tests.rs         # NEW test: no_panic_under_sandboxed_io_sweep (+ optional shared helper)
```

No other source files are expected to change. If a shared dispatch helper is extracted, it lives in
the same `#[cfg(test)]` module so production code is untouched.

## Approach

1. **Reuse the existing sweep skeleton.** Model the new test on
   `no_panic_under_random_input_sweep`: same xorshift64 seeding style, same per-iteration
   "pick an action, dispatch, occasionally feed a mouse event, occasionally resize" loop, same set of
   terminal sizes including the 80×24 minimum and a sub-minimum "too small" size. Where practical,
   factor the action-pick + dispatch + render step into a private helper the two tests share; if the
   action sets differ enough that sharing hurts readability, duplicate the small loop (keep the two
   tests independently legible).

2. **Build the sandbox and serialize global mutation.** A process-wide `static SWEEP_ENV_LOCK:
   Mutex<()>` (same pattern as the session tests' `ENV_LOCK`) guards the whole sweep body, because the
   test mutates two pieces of global state — the process current directory and `$XDG_*` env vars —
   that other tests read. Inside the lock:
   - create `std::env::temp_dir().join(format!("edit_io_fuzz_{}", std::process::id()))` (+ `state/`
     and `config/` subdirs);
   - capture `std::env::current_dir()` and the prior `XDG_STATE_HOME` / `XDG_CONFIG_HOME` values;
   - `set_current_dir(sandbox)`, `set_var("XDG_STATE_HOME", …)`, `set_var("XDG_CONFIG_HOME", …)`;
   - run the sweep;
   - restore cwd + env (set back if previously present, else remove) and `remove_dir_all(sandbox)`.

3. **Seed real targets.** Write 1–2 small UTF-8 files into the sandbox (e.g. `seed_a.txt`,
   `seed_b.txt`) so Open/Revert have valid targets, and prime the App's initial buffer with a path
   inside the sandbox so a bare Save/Revert has somewhere to go.

4. **Expand the action set.** Action list = the feature-042 set **plus** `Save`, `SaveAs`,
   `SaveAsEncoding`, `Open`, `Revert`, **plus** `InsertChar` weighted toward path-ish characters
   (letters, `.`, `/`, the sandbox seed names) so that when the file browser is open it receives
   plausible typed names and the Enter/confirm path actually reads/writes inside the sandbox.

5. **Assert no panic across the full budget.** The body runs under normal dispatch (a panic fails the
   test, exactly like the 042 sweep). Hermeticity is structural (all I/O is relative to the sandboxed
   cwd / XDG); the suite-level guarantee — no repo files change — is verified once in quickstart via
   `git status` before/after.

## Risk / mitigation

- **Teardown on panic**: if the sweep panics mid-run, normal-flow restore of cwd/env would be skipped.
  Mitigation: restore via a small RAII guard (`Drop`) holding the saved cwd/env so it runs even on
  unwind; the mutex is allowed to poison (a real panic should fail loudly and is the signal we want).
  This keeps a genuine regression visible without silently corrupting the rest of the test run.
- **Parallel test interference**: cargo runs tests multithreaded. The unique per-pid sandbox + RAII
  restore bound interference to "cwd is briefly the sandbox while the lock is held". Tests that assert
  on cwd/XDG use their own locks; this sweep does not assume isolation from them beyond restoring
  state promptly.
- **No new flakiness**: determinism preserved (no clock/RNG in control flow); written file contents
  are deterministic.

## Phase 0 / Phase 1 outputs

See `research.md` (decisions), `data-model.md` (entities), `contracts/internal-api.md` (the internal
test contract), and `quickstart.md` (how to run + verify clean tree).
