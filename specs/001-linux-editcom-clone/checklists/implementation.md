---
name: implementation-readiness
description: Pre-implementation gate — validates that all requirements are complete, clear, and measurable before coding begins
metadata:
  type: checklist
  domain: implementation
  phase: pre-implementation
  created: 2026-06-18
  resolved: 2026-06-18
  audience: implementer + PR reviewer
  depth: standard
  status: ALL RESOLVED (42/42)
---

# Implementation Readiness Checklist: Linux EDIT.COM Clone

**Purpose**: Unit-test the requirements for completeness, clarity, consistency, and measurability
before any code is written. This is NOT a verification of implementation behavior — it validates
whether the _written requirements_ are of sufficient quality to hand off to an implementer.

**Created**: 2026-06-18 | **Resolved**: 2026-06-18
**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md) | [tasks.md](../tasks.md)
**Constitution**: [.specify/memory/constitution.md](../../../../.specify/memory/constitution.md)

Items marked `[NON-NEGOTIABLE]` correspond to constitution Principles II, V, and VII — blocking gates.
Every item below was evaluated and resolved; `→` records the disposition.

---

## Requirement Completeness

- [x] CHK001 Is the `LineEnding` enum (T104) formally defined? [Completeness, Gap]
  → **Resolved**: added `Enum: LineEnding` (Lf/Crlf, default, detection, persistence) to data-model.md.
- [x] CHK002 Is the `Buffer.scroll_col` field (T028) defined? [Completeness, Gap]
  → **Resolved**: reconciled — horizontal scroll uses the existing `scroll_offset.1`; data-model.md clarified and tasks T028/T105/T111 updated. No redundant field.
- [x] CHK003 Is `EDIT_AUTOSAVE_INTERVAL` documented? [Completeness, Gap]
  → **Resolved**: added an Environment Variables table to contracts/cli.md.
- [x] CHK004 Is `arboard` headless failure documented as an assumption? [Completeness, Assumption]
  → **Resolved**: added "System clipboard" assumption to spec.md.
- [x] CHK005 Are FR-021–FR-025 each mapped to a test task? [Completeness, Principle VII]
  → **Pass**: FR-021/022/023 → T101/T094; FR-024 → T006/T084; FR-025 → T007. T094 adds the UID/GID assertion.
- [x] CHK006 Are T098–T116 traceable to an FR/SC? [Traceability, Gap]
  → **Pass**: each maps to an FR/SC or is flagged infra; new FR-026–FR-029 added to anchor T103/T105/T106/T107.

## Requirement Clarity

- [x] CHK007 Is horizontal-scroll-for-long-lines documented? [Clarity, Gap]
  → **Resolved**: data-model.md "Long-line handling" + spec edge case; choice over soft-wrap stated with rationale.
- [x] CHK008 Is binary detection quantified? [Clarity, Ambiguity]
  → **Resolved**: FR-029 specifies null-byte scan of first 512 bytes; BufferError::BinaryContent added.
- [x] CHK009 Is CRLF preservation an explicit requirement? [Clarity, Spec §Edge Cases]
  → **Resolved**: FR-007a added.
- [x] CHK010 Is Save As target-exists behavior defined? [Clarity, Gap]
  → **Resolved**: FR-028 (overwrite prompt + path update + clear-modified); T103 updated.
- [x] CHK011 Is terminal resize an explicit requirement? [Clarity, Spec §Edge Cases]
  → **Resolved**: FR-026.
- [x] CHK012 Is clipboard degradation specified? [Clarity, Assumption]
  → **Resolved**: spec assumption (no-op + status-bar warning, no panic).
- [x] CHK013 Is `EDIT_STRESS_DURATION_SECS` documented? [Clarity, Gap]
  → **Resolved**: contracts/cli.md Environment Variables table.

## Security Requirements (Constitution Principle VII) [NON-NEGOTIABLE]

- [x] CHK014 Does FR-022 specify which control-sequence categories are stripped? [Clarity, §FR-022]
  → **Resolved**: FR-022 enumerates CSI, OSC, DCS, APC, PM + lone control chars.
- [x] CHK015 Does FR-023 define the traversal boundary? [Clarity, §FR-023]
  → **Resolved**: FR-023 — boundary is launch CWD; absolute CLI args trusted, dialog inputs not.
- [x] CHK016 Is FR-021 testable as written? [Measurability, §FR-021]
  → **Resolved**: FR-021 asserts EUID/EGID identical across `Buffer::save`; T094 adds the assertion.
- [x] CHK017 Is permission-denied-without-elevation defined? [Completeness, §FR-021]
  → **Resolved**: FR-021 + FR-027 (Retry/Cancel only, never elevation).
- [x] CHK018 Does FR-025 exclude sensitive content from crash logs? [Completeness, §FR-025]
  → **Resolved**: FR-025 — metadata only; no buffer text, clipboard, or search strings.

## UTF-8 and Encoding (Constitution Principle II) [NON-NEGOTIABLE]

- [x] CHK019 Are legacy encodings enumerated and consistent with crate choices? [Consistency, §FR-007]
  → **Pass**: FR-007 lists CP437/CP850/ISO-8859-1/Windows-1252; consistent with research.md §5.
- [x] CHK020 Is the heuristic fallback specified? [Clarity, §FR-006]
  → **Resolved**: FR-007 — `chardetng`, fall back to UTF-8 below 0.6 confidence; assumption documents the constant.
- [x] CHK021 Is `--encoding` vs BOM conflict resolved? [Coverage, §FR-007]
  → **Resolved**: FR-007 priority order — flag > BOM > heuristic > UTF-8.
- [x] CHK022 Are wide/combining widths quantified? [Measurability, §FR-005]
  → **Resolved**: FR-005 cites UAX #11 (2 cols wide, 0 combining) and UAX #29 (grapheme nav).
- [x] CHK023 Is the CP437 round-trip fixture defined? [Measurability, §SC-007]
  → **Resolved**: SC-007 names `tests/fixtures/cp437_box.bin` byte sequence.
- [x] CHK024 Is invalid-byte handling defined? [Coverage, §FR-007]
  → **Resolved**: FR-007 + BufferError::DecodeError (dialog + U+FFFD on "Open anyway").

## Performance Requirements

- [x] CHK025 Is startup measurement bounded? [Measurability, §SC-003]
  → **Resolved**: SC-003 — execve → first `terminal.draw()`, hyperfine median.
- [x] CHK026 Is the 100 MB fixture type specified? [Clarity, §SC-004]
  → **Resolved**: SC-004 — ASCII UTF-8 primary, mixed-Unicode informational.
- [x] CHK027 Is keystroke latency bounded? [Measurability, §SC-005]
  → **Resolved**: SC-005 — `event::read()` return → `terminal.draw()` completion.
- [x] CHK028 Does SC-008 define pass/fail beyond "no crash"? [Measurability, §SC-008]
  → **Resolved**: SC-008 — ≤ 5 MB RSS growth over baseline, zero panics.
- [x] CHK029 Is highlight+large-file latency budgeted? [Coverage, Gap]
  → **Resolved**: FR-016 — per-visible-line cached highlight; SC-005 budget holds at ≥ 10 MB.

## TDD and Test Gate (Constitution Principle V) [NON-NEGOTIABLE]

- [x] CHK030 Do TDD tasks have committed-before constraints? [Completeness, Principle V]
  → **Pass**: per-phase TDD gates (T099–T101, T108–T110, T112–T113) + Dependencies section ordering.
- [x] CHK031 Are smoke scripts required in source control before impl? [Clarity, Principle V]
  → **Resolved**: tasks.md Notes state TDD test files "must be committed before any implementation in that phase".
- [x] CHK032 Is a minimum coverage standard stated? [Clarity, Gap, Principle V]
  → **Resolved**: T095 — all FR-001–FR-029 have ≥ 1 automated test, enforced by phase TDD gates.
- [x] CHK033 Are `expect`/`tmux` listed as CI deps? [Completeness, Gap]
  → **Resolved**: T095 documents `expect` + `tmux` as required CI-image dependencies in docs/STATUS.md.

## Recovery and Reliability

- [x] CHK034 Is recovery-format version compatibility defined? [Completeness, §recovery.md]
  → **Resolved**: recovery.md "Version compatibility" — V1 always readable; unknown higher version declines, file untouched.
- [x] CHK035 Is autosave-only-when-modified explicit? [Clarity, §FR-014]
  → **Resolved**: recovery.md "Autosave precondition" (modified==true) + FR-014.
- [x] CHK036 Is the stale-lock mechanism specified? [Clarity, §recovery.md]
  → **Resolved**: recovery.md mandates `kill(pid, 0)` (ESRCH/EPERM semantics); `/proc` prohibited for portability.

## Terminal and Compatibility

- [x] CHK037 Does color degradation cover all chrome? [Completeness, §FR-008]
  → **Resolved**: FR-008 + T050 — menu bar, status bar, dialogs, selected-item highlights all reverse-video.
- [x] CHK038 Is a minimum terminal size defined? [Coverage, Gap]
  → **Resolved**: FR-026 — 80×24 minimum with "Terminal too small" notice.
- [x] CHK039 Is WSL clipboard behavior addressed? [Coverage, Gap, §FR-011]
  → **Resolved**: spec assumption — best-effort, graceful no-op fallback; no `clip.exe` bridge required for v1.x.

## Scenario Coverage and Edge Cases

- [x] CHK040 Is `--readonly` + `--no-autosave` defined? [Coverage, §FR-018]
  → **Resolved**: contracts/cli.md Behavior Notes — valid, redundant, no lock/recovery in read-only.
- [x] CHK041 Is `--encoding` multi-file behavior specified? [Clarity, §cli.md]
  → **Pass**: contracts/cli.md already states it applies to ALL files; separate invocations for mixed encodings.
- [x] CHK042 Are `chardetng` / `ropey` / segmentation assumptions documented? [Dependency, Assumption]
  → **Resolved**: spec assumptions for heuristic confidence and grapheme-segmentation crate compatibility (guarded by T108/T109).

---

## Resolution Summary

- **42 / 42 items resolved** (0 open).
- **New requirements added**: FR-007a (line endings), FR-026 (min terminal size), FR-027 (save-failure dialog),
  FR-028 (Save As + overwrite), FR-029 (binary refusal). FR-005, FR-007, FR-008, FR-016, FR-021, FR-022, FR-023,
  FR-025 strengthened.
- **New data-model entities**: `Enum: LineEnding`, `Enum: BufferError`; `scroll_offset` long-line semantics.
- **Contracts updated**: cli.md (Environment Variables, flag-combination notes); recovery.md (version policy,
  autosave precondition, `kill(pid,0)` stale-lock mechanism).
- **Success criteria** SC-003–SC-008 given explicit measurement boundaries and fixtures.
- No remaining blockers for constitution Principles II, V, or VII. **Ready for `/speckit-implement`.**
