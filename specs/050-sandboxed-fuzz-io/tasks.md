# Tasks: Sandboxed fuzz covering file-I/O actions (050)

**Feature**: Sandboxed no-panic fuzz sweep over Save/SaveAs/SaveAsEncoding/Open/Revert.
**Branch**: `050-sandboxed-fuzz-io` | **Closes**: #79 (follow-up to #72)
**Nature**: test-only, behavior-preserving (no production code change expected).

## Phase 1: Setup

- [x] T001 Confirm baseline green + clean tree: `make tmpfs-setup` then `cargo test` passes and
  `git status --porcelain` is empty in repo root (records the hermeticity baseline).

## Phase 2: Foundational (sandbox harness)

- [x] T002 Add a process-wide `static SWEEP_ENV_LOCK: std::sync::Mutex<()>` in the `#[cfg(test)]`
  module of `src/app/tests.rs` (mirrors `src/session/mod.rs` `ENV_LOCK`), to serialize global
  cwd/env mutation (FR-004, C-2).
- [x] T003 Add an RAII `EnvGuard` struct in `src/app/tests.rs` that, on construction, captures the
  original `std::env::current_dir()` and prior `XDG_STATE_HOME`/`XDG_CONFIG_HOME` (`Option<OsString>`),
  creates the sandbox (`std::env::temp_dir().join(format!("edit_io_fuzz_{}", std::process::id()))`)
  with `state/` and `config/` subdirs, sets cwd + the two XDG vars into the sandbox; and on `Drop`
  restores cwd + both vars (set-back-or-remove) and `remove_dir_all`s the sandbox (FR-002, FR-003,
  FR-010, C-3, R3).

## Phase 3: User Story 1 — sandboxed no-panic I/O sweep, clean tree (P1)

**Goal**: fuzz the five file-I/O actions across overlays + sizes with zero panics, leaving the repo
working tree untouched.

**Independent test**: run `no_panic_under_sandboxed_io_sweep`; it passes (no panic) and a full-suite
run leaves `git status --porcelain` unchanged.

- [x] T004 [US1] In `src/app/tests.rs`, seed the sandbox: write small UTF-8 files `seed_a.txt` and
  `seed_b.txt` (relative to the now-sandboxed cwd) and construct the App so its initial buffer has an
  in-sandbox path (so bare Save/Revert have a target) (FR-005, C-4).
- [x] T005 [US1] Add `no_panic_under_sandboxed_io_sweep` in `src/app/tests.rs`, modelled on
  `no_panic_under_random_input_sweep`: take `SWEEP_ENV_LOCK`, build `EnvGuard`, then run the
  deterministic xorshift64 loop with the action set = the 042 set **plus** `Save`, `SaveAs`,
  `SaveAsEncoding`, `Open`, `Revert`, and `insert_chars` biased to path-ish characters
  (letters/digits/`.`/`/`/`_` + the multibyte stress chars); iterate 3 fixed seeds × 4 sizes
  `[(80,24),(120,40),(200,60),(40,12)]` (last = sub-min), ~1500 events each, rendering every Nth
  iteration; assert no panic (FR-001, FR-006, FR-007, FR-008, C-1, C-5).
- [x] T006 [US1] Run `cargo test --lib no_panic_under_sandboxed_io_sweep`. If a **real** panic
  surfaces, fix it as a behavior-preserving guard in the relevant `src/app/*` path (if-let/let-else or
  checked access, consistent with feature 042/046) — no behavior change; otherwise leave production
  code untouched (FR-009, plan Risk section).
- [x] T007 [US1] Verify hermeticity: with a clean tree, run the full `cargo test`; assert
  `git status --porcelain` is byte-identical before/after and no files appear under the repo root
  (SC-002, C-3/T).

## Phase 4: Polish & gate

- [x] T008 `cargo fmt` then `make ci-local` (fmt --check → clippy -D warnings → test → smoke →
  perf-check); confirm fmt/clippy/tests clean. Note any pre-existing `.exp` smoke failures
  (`encoding_select`, `file_browser`) also fail on `master` (environment, not this change) (SC-004).
- [x] T009 Docs gate: add a feature-050 entry to `CHANGELOG.md` and a row to `docs/STATUS.md`
  (version line `038–050`). **No** `docs/CAPABILITIES.md` change (test-only, behavior-preserving).
- [x] T010 Open PR `test(050): sandboxed fuzz covering file-I/O actions` targeting `master`, closing
  #79; merge immediately after creation.

## Dependencies

- T001 → T002, T003 (harness) → T004, T005 (sweep) → T006 (run/fix) → T007 (hermeticity) → T008 → T009 → T010.
- All work is in one file (`src/app/tests.rs`), so tasks are sequential (no `[P]` parallelism).

## MVP scope

US1 (T001–T007) is the whole feature; T008–T010 are the standard gate + ship steps.
