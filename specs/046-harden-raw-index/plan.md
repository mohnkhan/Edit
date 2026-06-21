# Implementation Plan: Harden Raw Slice/Index Access

**Branch**: `046-harden-raw-index` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/046-harden-raw-index/spec.md` (issue #78)

## Summary

Close the input-influenced raw `[index]`/`[a..b]` panic class in the editor's hot paths, continuing the
042/043 no-crash work. Discovery + proof are fuzz-driven: extend the deterministic no-panic sweep to run
on **content-bearing (multibyte) buffers** so line/grapheme/byte indexing is actually exercised, then fix
each panic at its source. Proactively convert the four risky categories the audit found: char-boundary
string slices, list lookups from a selection/cursor, computed (non-invariant) buffer indices, and
rope line-index helpers. Behavior-preserving for all in-range input; provably-safe indices (constants,
`buffers[0]`) are left alone.

## Technical Context

**Language/Version**: Rust 2021, MSRV 1.74
**Primary Dependencies**: `ratatui`/`crossterm`, `ropey`, `unicode-segmentation`. No new deps.
**Testing**: `cargo test` (incl. the extended fuzz), `make smoke`, `make perf-check`.
**Target Platform**: Linux TUI. **Project Type**: single-project desktop TUI.
**Performance**: no regression (checked access is a branch; fuzz bounded).
**Constraints**: behavior-preserving for in-range input (FR-005); deterministic fuzz, no real FS I/O
(FR-006); 042 `clippy::unwrap_used` guardrail intact.
**Scale/Scope**: targeted conversions in `src/app*`, `src/buffer`, `src/ui`; one fuzz extension; not a
crate-wide indexing sweep.

## Constitution Check
- I DOS UI — PASS (no UI change). II UTF-8 — PASS/strengthened (char-boundary-safe slicing). III Portable
  — PASS. IV Footprint — PASS (no dep). V Test-Gated (NON-NEGOTIABLE) — PASS (extends the fuzz; suite
  green). VI Simplicity — PASS (scoped, not crate-wide). VII Security — supportive (panic-resistance on
  untrusted file/terminal input). **Result**: all gates pass; Complexity Tracking empty.

## Project Structure
```text
src/
├── app/tests.rs        # extend the 042 fuzz: seed buffers with multibyte content; (optionally) a
│                       #   content-bearing variant so line/grapheme/byte indexing is driven.
├── app/*.rs            # convert input-influenced list/buffer indices to .get()/checked access where
│                       #   the fuzz/audit shows risk (e.g. ENCODING_OPTIONS[sel], ITEMS[idx],
│                       #   buffers[buf_idx] not invariant-proven).
├── buffer/rope.rs      # make line-index helpers total where they take input-derived lines
│                       #   (line_to_char etc.), mirroring the total line_slice.
└── ui/*.rs             # char-boundary-safe string slicing in editor/field/width rendering;
                        #   checked list access in hit-tests.
```
**Structure Decision**: unchanged layout; localized hardening in the hot-path files.

## Key decisions (detail in research.md)
- **Fuzz-first**: extend the existing deterministic sweep to seed multibyte content and run it; every
  panic it raises is a real bug fixed at source. This bounds "815 sites" to the ones that actually fire,
  plus the audited high-risk categories.
- **Conversion idioms**: `slice.get(i)` / `get_mut(i)` with the existing no-op/empty branch; for strings,
  clamp byte offsets to `char_indices` boundaries (or reuse existing width/grapheme helpers) instead of
  raw `&s[a..b]`; for rope, clamp the line index like `line_slice` does.
- **Leave invariant-safe indices raw** (FR-008) to avoid noise.

## Phased Approach (one PR, ordered commits)
1. **Extend fuzz** to content-bearing buffers; run → triage panics.
2. **Fix** each surfaced panic + convert the audited high-risk categories (string slices, list indices,
   buffer indices, rope helpers), building/testing after each cluster.
3. **Verify**: extended fuzz green deterministically; suite + clippy + fmt clean.

## Complexity Tracking
*No violations. Table intentionally empty.*
