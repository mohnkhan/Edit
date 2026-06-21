# Phase 0 Research: Harden Raw Slice/Index Access

Audit of the raw-index surface (post-045), grouped by panic risk.

## R1 — Categories & decision

**Decision**: convert the *input-influenced* categories; leave invariant-safe ones raw.

| Category | Examples (verified) | Risk | Conversion |
|---|---|---|---|
| String byte-slice `&s[a..b]` | ~30 sites in `ui/` field/editor/width render + Go-to-Line body | char-boundary panic when offset splits a multibyte grapheme | clamp offsets to char boundaries / reuse grapheme+width helpers; never raw-slice on an input offset |
| List index from selection/cursor | `ENCODING_OPTIONS[sel]` (dialogs), `ITEMS[idx]`/`ITEMS[menu.focus]` (mouse/dispatch), `instances[..]` | OOB if the selection/focus is stale | `.get(i)` → existing no-op/empty branch |
| Computed buffer index | `buffers[buf_idx]`, `buffers[i]`, `buffers[idx]`, `buffers[ec.buf_idx]`, `buffers[bidx]` | OOB if the index lags a buffer close | `.get()/.get_mut()` → safe no-op |
| Rope line index | `rope.line_to_char(file_line)` (editor render), `rope.line(idx)` (already inside the total `line_slice`) | panic past end on a stale line | clamp the line index (mirror `line_slice`, feature 042/034) |
| **Invariant-safe (OUT)** | `buffers[0]` (≥1-buffer invariant), compile-const indices, test code | none | leave raw (FR-008) |

**Rationale**: FR-001..004 target exactly the runtime-derived indices; FR-008 avoids churning safe ones.

## R2 — Discovery via fuzz

**Decision**: extend the feature-042 `no_panic_under_random_input_sweep` to seed each buffer with
multibyte content (e.g. lines mixing ASCII, combining marks, CJK, emoji) before driving events, so
line/grapheme/byte indexing is actually exercised (the original used empty buffers + typed chars).
Keep it deterministic (fixed xorshift seed, no RNG/clock) and free of real FS I/O (file-I/O actions stay
excluded — that's #79).

**Rationale**: FR-006 — the existing harness is the proof; content is what makes the index paths fire.
Every panic it raises is a genuine bug (FR-002/SC-002). **Alternatives**: crate-wide
`#![deny(clippy::indexing_slicing)]` — rejected here (hundreds of safe sites; massive churn; a separate
future call), though a *scoped* deny could be a follow-up.

## R3 — Behavior preservation

**Decision**: each converted site's out-of-range branch reproduces the surrounding code's existing
"nothing there" behavior (no-op/empty/clamp). In-range inputs are byte-identical. The full suite + the
extended fuzz are the proof (FR-005/SC-003).

## Open questions
None. No `NEEDS CLARIFICATION` remain.
