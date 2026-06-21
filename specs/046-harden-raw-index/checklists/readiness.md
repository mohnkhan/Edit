# Checklist: Raw-Index Hardening Readiness

**Purpose**: Validate requirements quality before implementation (tests requirements, not code).
**Created**: 2026-06-21 | **Feature**: [spec.md](../spec.md)

## Completeness
- [ ] CHK001 - Are the risky categories enumerated (string slices, list-from-selection, computed buffer idx, rope line)? [Completeness, FR-001..004, research §R1]
- [ ] CHK002 - Is the discovery mechanism specified (content-bearing deterministic fuzz)? [Completeness, FR-006]
- [ ] CHK003 - Is the out-of-scope set explicit (constants, buffers[0], tests, file-I/O fuzz #79)? [Completeness, FR-008, Assumptions]

## Clarity
- [ ] CHK004 - Is "behavior-preserving" defined (in-range identical; only OOB outcome changes)? [Clarity, FR-005]
- [ ] CHK005 - Is "char-boundary-safe" unambiguous (never split a multibyte grapheme)? [Clarity, FR-001/Edge]
- [ ] CHK006 - Is "input-influenced" defined so the target set is bounded (index derived from runtime input)? [Clarity, Key Entities]

## Consistency
- [ ] CHK007 - Do spec/research/data-model/tasks agree on the four categories + conversion idioms? [Consistency]
- [ ] CHK008 - Is determinism + no-real-FS consistent with the 042 fuzz convention? [Consistency, FR-006]

## Acceptance
- [ ] CHK009 - Are SCs measurable (zero panics; no remaining input-influenced raw site; suite green)? [Measurability, SC-001..004]
- [ ] CHK010 - Is there a baseline (1277/0/11)? [Measurability, T001]

## Coverage & Edge
- [ ] CHK011 - Char boundaries on byte-offset slices covered? [Edge, FR-001]
- [ ] CHK012 - Empty-container / first-last indexing covered? [Edge, spec Edge Cases]
- [ ] CHK013 - Stale selection/cursor/line index covered (the 042/043 class)? [Edge, FR-002/FR-004]
- [ ] CHK014 - Does the content-bearing fuzz keep file-I/O excluded (no disk writes; autosave off)? [Coverage, FR-006]

## Non-Functional
- [ ] CHK015 - 042 unwrap guardrail still satisfied (no new unwraps in conversions)? [Non-Functional, FR-007]
- [ ] CHK016 - Scope is NOT crate-wide indexing churn (hot paths only)? [Scope, Assumptions]

## Notes
- Load-bearing: CHK001 (categories), CHK004 (behavior-preserving), CHK002/CHK014 (fuzz is discovery+proof
  and stays FS-safe). All answerable "yes" from artifacts.
