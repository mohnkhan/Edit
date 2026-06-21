# Quickstart / Validation Guide: Harden Error Handling

Behavior-preserving robustness work. Validation = existing behavior unchanged + the new no-panic
guarantee + the guardrail is live.

## Prerequisites

```bash
cd /home/main/MyOS-2026/edit
make tmpfs-setup        # keep build off the SSD (idempotent)
make                    # debug build
```

## Automated validation (the gate)

```bash
make check              # cargo test: full suite unchanged + the new fuzz sweep, all green
make ci-local           # fmt --check → clippy -D warnings → test → smoke → perf-check
```

**Expected**: all green. The suite count rises only by the added fuzz test(s); no existing assertion
changes. `clippy -D warnings` must be clean with the new `#![deny(clippy::unwrap_used, expect_used)]` in
the `app` module tree (proves zero residual guarded unwraps in app production code).

## Run just the fuzz sweep

```bash
cargo test --lib fuzz   # or the exact test name, e.g. no_panic_random_input_sweep
```

**Expected**: passes deterministically (same result every run — fixed seeds, no RNG/clock). It applies
thousands of random keyboard+mouse events across every overlay state and several terminal sizes (incl.
80×24 and a sub-minimum) with zero panics.

## Demonstrate the guardrail is active (SC-004)

Temporarily add a stray `let _x: Option<u8> = None; _x.unwrap();` into e.g. `src/app/dispatch.rs`, then:

```bash
cargo clippy --all-targets -- -D warnings    # MUST fail: clippy::unwrap_used denied in the app tree
```

Revert the stray line; clippy is clean again. (The same line added in `src/highlight/languages/rust.rs`
would NOT fail — the guardrail is scoped to the `app` module tree, per FR-006.)

## Manual sanity

```bash
./target/debug/edit specs/042-harden-error-handling/spec.md
```

Open/close each overlay, navigate menus, click around, resize — identical behavior to before, no crash.

## References

- Conversion shape + lint scoping + PRNG choice: [research.md](./research.md)
- Contracts (behavior, fuzz, guardrail, recovery): [contracts/internal-api.md](./contracts/internal-api.md)
