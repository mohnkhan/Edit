# Quickstart: Sandboxed file-I/O fuzz sweep (050)

## Prerequisites
- `make tmpfs-setup` (redirect `target/` to tmpfs), then a normal toolchain.

## Run the new sweep
```bash
cargo test --lib no_panic_under_sandboxed_io_sweep -- --nocapture
```
Expected: the test passes (zero panics) across all seeds/sizes.

## Verify hermeticity (the key guarantee)
```bash
git status --porcelain                      # capture clean baseline
cargo test                                  # run the WHOLE suite (includes the new sweep)
git status --porcelain                      # MUST be identical to the baseline — no new/modified files
ls -la                                      # no stray files in the repo root
```
Expected: `git status --porcelain` output is unchanged before vs. after; no files created under the
repository working tree (all I/O went to the per-pid temp sandbox, which is removed at the end).

## Determinism check
```bash
cargo test --lib no_panic_under_sandboxed_io_sweep
cargo test --lib no_panic_under_sandboxed_io_sweep   # same result, same behavior
```

## Gate
```bash
make ci-local      # fmt --check → clippy -D warnings → test → smoke → perf-check
```
Expected: fmt + clippy + tests clean. (The pre-existing `encoding_select` / `file_browser` `.exp`
smoke tests may fail in environments lacking F-key/escape-sequence delivery; verify they fail
identically on `master` and are unrelated to this change.)

## Success
- New sweep green, zero panics (SC-001).
- Working tree byte-identical before/after a full suite run (SC-002).
- Deterministic (SC-003); fmt + clippy clean (SC-004); only the new test added, no existing assertion
  changed (SC-005).
- No `docs/CAPABILITIES.md` change (test-only, behavior-preserving).
