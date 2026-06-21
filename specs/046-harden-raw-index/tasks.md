# Tasks: Harden Raw Slice/Index Access

**Feature**: `046-harden-raw-index` | **Branch**: `046-harden-raw-index` | Closes #78
**Input**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/internal-api.md](./contracts/internal-api.md)

**Overriding constraint**: Behavior-preserving for in-range input (FR-005) — converted sites' out-of-range
branch reproduces the surrounding no-op/empty/clamp; no existing assertion changes. Deterministic fuzz,
no real FS I/O (FR-006). 042 `clippy::unwrap_used` guardrail holds.

## Phase 1: Setup
- [ ] T001 Baseline: `make tmpfs-setup`; `make check`; note count (1277/0/11).

## Phase 2: US1 — discovery via content-bearing fuzz (P1)
- [ ] T002 [US1] Extend the deterministic no-panic sweep in `src/app/tests.rs`: seed each buffer with
  multibyte content (ASCII + combining mark + CJK + emoji over several lines) before driving events, so
  line/grapheme/byte indexing fires. Keep fixed-seed xorshift, no RNG/clock, file-I/O actions excluded.
  (May be a new `no_panic_under_random_input_sweep_with_content` or a content param on the existing one.)
- [ ] T003 [US1] Run the fuzz; triage every panic (location + index source). Each is a real raw-index
  bug for Phase 3.

## Phase 3: US1/US2 — fix surfaced panics + convert audited categories (P1)
- [ ] T004 [US1] String byte-slices (`src/ui/*`: editor/field/width render, Go-to-Line body): make
  char-boundary-safe (clamp offsets to char boundaries / reuse grapheme+width helpers); never raw-slice
  on an input-derived offset.
- [ ] T005 [US1] List lookups from selection/cursor/focus: `ENCODING_OPTIONS[sel]` (`app/dialogs.rs`),
  `ITEMS[idx]`/`ITEMS[menu.focus]` (`app/mouse.rs`,`app/dispatch.rs`), plugin `instances[..]` → `.get()`
  with the existing no-op/empty branch.
- [ ] T006 [US1] Computed buffer indices not invariant-proven (`buffers[buf_idx|i|idx|ec.buf_idx|bidx]`)
  → `.get()/.get_mut()` safe no-op. Leave `buffers[0]` / constants raw (FR-008).
- [ ] T007 [US1] Rope line-index helpers taking input-derived lines (`line_to_char`, any `line(idx)`
  outside `line_slice`) → clamp the line index (mirror the total `line_slice`).
- [ ] T008 [US2] Re-run the extended fuzz until zero panics; for each fix, confirm the out-of-range
  branch matches prior intent (no behavior change for in-range input).

## Phase 4: Polish & Ship
- [ ] T009 `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check); count == baseline +
  new fuzz test(s); pre-existing env `encoding_select` smoke F12 failure unrelated.
- [ ] T010 Docs gate: `CHANGELOG.md` + `docs/STATUS.md` (no `CAPABILITIES.md` change — behavior-preserving).
  Also mark the stale ROADMAP #68 row Complete (done in feature 040) while here.
- [ ] T011 PR → `master` (`feat(046): harden raw slice/index access`), strip AI-attribution, green,
  `Closes #78`, merge.

## Dependencies & Order
Setup (T001) → extend fuzz (T002–T003) → fixes/conversions (T004–T008, fuzz-guided) → polish/ship
(T009–T011). T002/T003 drive which T004–T007 sites are mandatory; the audited categories are converted
regardless (proactive hardening).

## MVP
The content-bearing fuzz (T002) + fixing what it surfaces (T003–T008) is the feature; the category
conversions make it robust beyond what one seed set reaches.
