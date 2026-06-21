# Phase 1 Data Model: Harden Error Handling

No persisted data and no new domain types. The "model" is two test-only constructs plus the conversion
unit.

## Entity: Guarded-unwrap site (conversion unit)

A source location of the form `EXPR.unwrap()` / `EXPR.expect(..)` where `EXPR: Option<T>`/`Result<T,_>`
and an earlier guard in the same function proved the value present.

| Aspect | Before | After |
|---|---|---|
| Form | `let x = EXPR.unwrap(); use(x);` | `if let Some(x) = EXPR { use(x); }` (or `let Some(x)=EXPR else { return … };`) |
| Absent-arm | panic | the prior no-op / existing fall-through (behavior-identical) |
| Invariant held by | programmer's reasoning | the compiler (pattern match) |

Counts (post-041): `dispatch.rs` 14, `mouse.rs` 7, `dialogs.rs` 3, `app.rs` 3 = **27**.

## Entity: Fuzz input stream (test-only)

```text
seed: u64 (fixed constant, from a small array of seeds)
rng:  xorshift64 state (deterministic; no rand crate, no Date::now)
events: alternating
  - keyboard Action drawn from a curated representative set (overlay openers + editing + nav + Esc)
  - mouse event: { Down(Left|Right) | ScrollUp | ScrollDown | Drag | Up } at (col,row)
                 col,row pseudo-random in [0, w+2) × [0, h+2)  (deliberately includes a few OOB cells)
terminal sizes: { (80,24) min, (120,40), (200,60), (40,12) sub-minimum/too_small }
iterations: fixed N per (seed × size) — at least several thousand events total
```

**Invariant under test**: applying the entire stream (with periodic `render`) to a freshly built `App`
never panics. (A panic aborts the test thread → test fails; no `catch_unwind` required.)

## Entity: Lint guardrail (config)

- `#![deny(clippy::unwrap_used, clippy::expect_used)]` in `src/app.rs` (propagates to all `app::*`).
- `#![allow(clippy::unwrap_used, clippy::expect_used)]` in `src/app/tests.rs`; `#[allow(...)]` on the
  inline `#[cfg(test)] mod` debug modules in `app.rs`.
- Effect: a new `unwrap()`/`expect()` in app *production* code fails `cargo clippy -- -D warnings`.

## No schema / migration / serialization changes.
