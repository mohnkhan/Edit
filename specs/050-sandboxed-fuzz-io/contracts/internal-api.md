# Internal Contract: Sandboxed file-I/O fuzz sweep (050)

No public/production API changes. This documents the test-internal contract.

- C-1: `no_panic_under_sandboxed_io_sweep` exists in `src/app/tests.rs` and dispatches a deterministic
  (fixed-seed xorshift64) sequence of `Action`s + `MouseEvent`s whose action set **includes**
  `Save`, `SaveAs`, `SaveAsEncoding`, `Open`, `Revert` across all overlay states and ≥3 terminal
  sizes incl `(80,24)` and a sub-minimum `(40,12)`. It asserts no panic across the full budget (FR-001,
  FR-007, FR-008).
- C-2: The sweep body runs under a process-wide `SWEEP_ENV_LOCK` mutex; while held, cwd =
  sandbox and `$XDG_STATE_HOME`/`$XDG_CONFIG_HOME` point inside the sandbox (FR-003, FR-004).
- C-3: An RAII `EnvGuard` restores cwd + both XDG vars to their pre-sweep values and removes the
  sandbox directory on scope exit, including on panic unwinding (FR-002, FR-010, R3).
- C-4: At least one readable seed file exists in the sandbox before the sweep so Open/Revert have a
  valid target; the initial buffer is primed with an in-sandbox path (FR-005).
- C-5: No `rand`, `Date::now`, or `Instant::now` governs sweep control flow; runs are reproducible
  (FR-006).
- C-6: No production source file changes; the feature-042 sweep and all other tests are unchanged
  (FR-009).
- T (verification): full suite green; `git status` identical before/after a suite run (no repo files
  added/modified/deleted); `cargo fmt --check` + `cargo clippy -D warnings` clean.
