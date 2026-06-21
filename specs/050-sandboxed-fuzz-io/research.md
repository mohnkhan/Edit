# Research: Sandboxed fuzz covering file-I/O actions (050)

## R1 — How to confine the file browser's I/O to a sandbox

- **Decision**: Set the process working directory into the sandbox (`std::env::set_current_dir`) and
  redirect `$XDG_STATE_HOME` / `$XDG_CONFIG_HOME` to sandbox subdirs.
- **Rationale**: `App::browser_start_dir()` falls back to `std::env::current_dir()`, and typed
  relative names in the browser resolve against cwd; session/recent/recovery/log writes resolve via
  `dirs::state_dir()`/`config_dir()` which honor the XDG env vars. Redirecting both fully contains the
  five I/O actions plus any incidental persistence (recent.toml, session, logs).
- **Alternatives considered**: (a) Monkeypatching paths via a test-only config — rejected: requires
  production seams that don't exist and would be a behavior change. (b) Mocking the filesystem —
  rejected: no FS-abstraction layer exists; would be a large refactor for a test.

## R2 — Serializing global cwd/env mutation under multithreaded `cargo test`

- **Decision**: Guard the whole sweep body with a process-wide `static SWEEP_ENV_LOCK: Mutex<()>`,
  mirroring `src/session/mod.rs`'s `ENV_LOCK`.
- **Rationale**: cwd and env vars are process-global; cargo runs tests on multiple threads. Without a
  lock, a concurrent test reading cwd/XDG could observe the sandbox. A single mutex held for the sweep
  body makes the mutation atomic w.r.t. any other test that takes the *same* lock; for tests that use
  their own locks, the unique per-pid sandbox + prompt restore bounds the exposure window.
- **Alternatives considered**: `serial_test` crate — rejected (new dependency; std Mutex suffices and
  matches the existing pattern).

## R3 — Restoring state safely even if the sweep panics

- **Decision**: Perform cwd/env restoration in an RAII guard's `Drop`, capturing the original cwd and
  prior env values at construction.
- **Rationale**: A panic mid-sweep unwinds; `Drop` still runs, so the original cwd/env are restored
  and the sandbox is removed before the panic propagates. The test's purpose is "no panic", so a panic
  is a legitimate failure we want surfaced — but we must not leave the process cwd pointing at a
  deleted temp dir, which would cascade failures into unrelated tests.
- **Alternatives considered**: plain sequential restore after the loop — rejected: skipped on panic.
  `std::panic::catch_unwind` then re-panic — rejected: more complex than Drop and would need
  `AssertUnwindSafe`; Drop is the idiomatic choice.

## R4 — Determinism without `rand` / wall-clock

- **Decision**: Reuse the existing inline `xorshift64` PRNG seeded by fixed constants (XORed with
  width/height), exactly as `no_panic_under_random_input_sweep` does. Written file contents are fixed
  strings.
- **Rationale**: Project rule (features 042/046) forbids `rand`/`Date::now`/`Instant::now` in fuzz
  control flow so failures reproduce identically. `Buffer::save` may stamp `self_write_times` with
  `Instant::now()`, but that is data, not control flow, and does not affect determinism of the sweep.
- **Alternatives considered**: `rand` with a fixed seed — rejected (new dependency + the project
  already standardized on inline xorshift).

## R5 — Reaching the I/O paths meaningfully (not just opening/closing the browser)

- **Decision**: Add the five I/O actions to the action list AND bias `InsertChar` toward path-ish
  characters (letters, digits, `.`, `/`, and the seed file names' characters) so that while the file
  browser/save dialog is open, typed input forms plausible relative names that the confirm path
  actually reads/writes inside the sandbox.
- **Rationale**: Save/Open often open a browser modal; without typed names + Enter, the sweep would
  rarely exercise the actual disk read/write. Path-ish characters maximize the chance of hitting the
  real I/O branch while staying in-sandbox.
- **Alternatives considered**: scripting exact "open browser → type name → Enter" sequences —
  rejected: that's a targeted integration test, not a fuzz sweep; the random approach with biased
  input covers more state combinations including stacked overlays.

## R6 — Iteration budget & sizes

- **Decision**: Reuse the 042 shape: 3 seeds × 4 terminal sizes `[(80,24),(120,40),(200,60),(40,12)]`
  (the last is sub-minimum / "too small"), ~1500 events each. Tune down if wall-clock in CI is a
  concern, but keep ≥ the 042 budget per size.
- **Rationale**: Matches an already-accepted runtime cost and satisfies FR-007 (≥3 sizes incl 80×24
  and a sub-min).
