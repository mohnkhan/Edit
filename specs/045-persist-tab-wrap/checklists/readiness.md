# Checklist: Persist Per-Tab Soft-Wrap — Readiness

**Purpose**: Validate requirements quality before implementation (tests the requirements, not the code).
**Created**: 2026-06-21
**Feature**: [spec.md](../spec.md)

## Requirement Completeness
- [ ] CHK001 - Is the persisted unit specified (per-tab record gains a wrap value)? [Completeness, FR-001, data-model]
- [ ] CHK002 - Is restore application specified (each tab set from its recorded value, independently)? [Completeness, FR-002]
- [ ] CHK003 - Is legacy-file behavior specified (missing field → default, no error)? [Completeness, FR-003]
- [ ] CHK004 - Is schema-version handling specified (accept old + new)? [Completeness, FR-004, research §R2]
- [ ] CHK005 - Is the save-captures-live requirement present (round-trip fidelity)? [Completeness, FR-005]

## Requirement Clarity
- [ ] CHK006 - Is "default" unambiguous (the configured `config.soft_wrap`, per 044)? [Clarity, Assumptions]
- [ ] CHK007 - Is the boundary clear (only on/off persisted; cache/geometry recomputed)? [Clarity, Assumptions]
- [ ] CHK008 - Is forward/backward compatibility behavior spelled out (old binary vs new file)? [Clarity, data-model table]

## Requirement Consistency
- [ ] CHK009 - Do spec/research/data-model/tasks agree on the version bump (1→2, accept both)? [Consistency]
- [ ] CHK010 - Is "new tabs after restore still seed from config" consistent with 044? [Consistency, FR-006]

## Acceptance Criteria Quality
- [ ] CHK011 - Are SCs measurable (round-trip reproduces; legacy loads; suite green)? [Measurability, SC-001..003]
- [ ] CHK012 - Is there a baseline (1272/0/11) to compare the unchanged-suite claim? [Measurability, T001]

## Scenario & Edge-Case Coverage
- [ ] CHK013 - Missing wrap value (old file) covered? [Edge, FR-003, T007]
- [ ] CHK014 - Buffers excluded from the session (no path) covered? [Edge, spec Edge Cases]
- [ ] CHK015 - Bogus/unknown version still rejected (not silently accepted)? [Edge, T007]
- [ ] CHK016 - Struct-literal/compile impact of the new field acknowledged? [Coverage, tasks §T005a]

## Non-Functional & Scope
- [ ] CHK017 - Is session persistence the only mechanism (no new storage)? [Scope, Assumptions]
- [ ] CHK018 - Does the change keep the 042 unwrap guardrail satisfied? [Non-Functional, T009]
- [ ] CHK019 - Is the docs-gate impact identified (CHANGELOG/STATUS; no CAPABILITIES change)? [Dependency, T010]

## Notes
- Load-bearing: CHK002 (apply on restore), CHK003/CHK004 (legacy compat), CHK015 (don't over-accept
  versions). All answerable "yes" from the artifacts.
