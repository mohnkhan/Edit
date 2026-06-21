# Internal Contract: Harden Raw Slice/Index Access

No external API. Behavioral + safety guarantees.

## Safety (FR-001..004)
- C-1: No input-influenced string byte-slice can panic on a non-char-boundary offset.
- C-2: List lookups by selection/cursor/focus use checked access; OOB → existing no-op/empty branch.
- C-3: Computed (non-invariant) buffer indices use `.get()/.get_mut()`; OOB → safe no-op.
- C-4: Rope line-index helpers used with input-derived lines are total (clamp; no panic past end).

## Behavioral (FR-005)
- B-1: For every in-range input, observable state (buffer text, cursor, selection, overlay, status,
  render output) is identical before/after.
- B-2: Only the out-of-range outcome changes: panic → graceful no-op/empty/clamp.

## Fuzz/proof (FR-006/SC-001)
- F-1: `cargo test` includes a content-bearing deterministic no-panic sweep (multibyte buffers) across
  overlay states and ≥3 terminal sizes; zero panics; reproducible (fixed seed, no RNG/clock); no real FS.

## Guardrail/quality (FR-007)
- G-1: `cargo clippy --all-targets -D warnings` clean; the 042 `unwrap_used` deny still holds.
- G-2: Full pre-existing suite passes with no assertion changes.
