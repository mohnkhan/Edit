# Checklist: Implementation-Readiness & Hermeticity (Feature 050)

**Purpose**: Validate that the requirements for the sandboxed file-I/O fuzz sweep are complete, clear,
and consistent before implementation. Unit-tests-for-English over `spec.md` / `plan.md` / `tasks.md`.
**Created**: 2026-06-21
**Feature**: [spec.md](../spec.md)

## Hermeticity (no working-tree impact)

- [ ] CHK001 - Is "must not touch the repository working tree" stated as a measurable, verifiable
  criterion (e.g. `git status` identical before/after)? [Measurability, Spec §SC-002]
- [ ] CHK002 - Are ALL write sinks the sweep can reach enumerated as in-sandbox — buffer save/save-as,
  session, recent-files, recovery, and log writes? [Completeness, Spec §Assumptions]
- [ ] CHK003 - Is the mechanism for confining browser-typed relative paths (cwd redirection)
  explicitly tied to the requirement, not assumed? [Clarity, Spec §FR-003, §Assumptions]
- [ ] CHK004 - Are restore-on-failure requirements specified for the case where the sweep panics
  mid-run (cwd/env restored, sandbox removed via RAII)? [Edge Case, Spec §US2-AC2, Plan §Risk]
- [ ] CHK005 - Is sandbox uniqueness specified well enough to avoid collisions across concurrent test
  binaries/processes? [Clarity, Data-model §Sandbox]

## Determinism

- [ ] CHK006 - Is "deterministic" defined concretely (fixed-seed xorshift64; no `rand`,
  `Date::now`, or `Instant::now` in control flow)? [Clarity, Spec §FR-006]
- [ ] CHK007 - Are sources of nondeterminism that are *data, not control flow* (e.g. `Instant::now`
  inside `Buffer::save`'s `self_write_times`) acknowledged as not affecting reproducibility?
  [Consistency, Research §R4]
- [ ] CHK008 - Is reproducibility expressed as a checkable outcome (same seed → same behavior)?
  [Measurability, Spec §SC-003]

## I/O action & input coverage

- [ ] CHK009 - Are all five I/O actions (Save, SaveAs, SaveAsEncoding, Open, Revert) named as required
  members of the action set? [Completeness, Spec §FR-001]
- [ ] CHK010 - Is the requirement to actually *exercise the read/write branch* (not just open/close the
  browser) captured via path-ish typed input? [Coverage, Spec §US1-AC3, Research §R5]
- [ ] CHK011 - Is the need for a real readable target (seed file + primed buffer path) specified so
  Open/Revert have something valid to act on? [Completeness, Spec §FR-005]
- [ ] CHK012 - Are no-valid-target cases (Open/Revert with no path or a bad path) required to be
  no-panic no-ops/messages? [Edge Case, Spec §Edge Cases]
- [ ] CHK013 - Is stacked-overlay coverage (I/O fired while a modal is already open) stated as in
  scope? [Coverage, Spec §Edge Cases]

## Terminal-size & overlay coverage

- [ ] CHK014 - Are the required terminal sizes specified, including the 80×24 minimum and a
  sub-minimum "too small" size? [Completeness, Spec §FR-007]
- [ ] CHK015 - Is "all overlay/modal states" defined by reference to the existing 042 sweep's coverage
  so the set is unambiguous? [Clarity, Spec §FR-007, Research §R6]

## Concurrency / global-state serialization

- [ ] CHK016 - Is the requirement to serialize global cwd/env mutation under multithreaded
  `cargo test` stated, with the mechanism (process-wide mutex) identified? [Completeness, Spec §FR-004]
- [ ] CHK017 - Are interactions with *other* cwd/XDG-sensitive tests (session tests) considered, and
  the bound on interference stated? [Consistency, Plan §Risk]

## Behavior preservation

- [ ] CHK018 - Is "no production code change expected" reconciled with the allowance to add a
  behavior-preserving guard if a real panic is found? [Conflict-check, Spec §FR-009, §Assumptions]
- [ ] CHK019 - Is "existing 042 sweep and all other assertions unchanged" stated as a hard
  requirement? [Completeness, Spec §FR-009, §SC-005]
- [ ] CHK020 - Is the no-`docs/CAPABILITIES.md`-change boundary explicit (test-only,
  behavior-preserving)? [Scope, Spec §Scope]

## Acceptance-criteria quality & traceability

- [ ] CHK021 - Does every functional requirement (FR-001…FR-010) map to at least one task?
  [Traceability, Tasks §Coverage]
- [ ] CHK022 - Are success criteria SC-001…SC-005 each objectively verifiable without naming
  implementation internals? [Measurability, Spec §Success Criteria]
- [ ] CHK023 - Is the iteration budget (seeds × sizes × events) specified concretely enough to be
  reproducible and bounded in runtime? [Clarity, Data-model §Sweep parameters]

## Constitution

- [ ] CHK024 - Does the feature uphold Principle V (test-gated): it adds an automated guard for the
  I/O code paths and does not weaken the gate? [Constitution, Plan §Constitution Check]

## Notes

- All items interrogate requirement *quality*, not implementation behavior. Resolve any unchecked item
  by editing spec/plan/tasks before `/speckit-implement`.
