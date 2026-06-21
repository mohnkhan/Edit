# Checklist: Per-Tab Soft-Wrap Readiness

**Purpose**: Validate the requirements are complete/unambiguous/consistent before implementation.
These test the requirements, not the eventual code.
**Created**: 2026-06-21
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Is the storage location of the per-tab setting specified (per-buffer, replacing the global flag)? [Completeness, Spec §FR-001, data-model]
- [ ] CHK002 - Is the default for new/opened/restored tabs specified (seed from `config.soft_wrap`)? [Completeness, Spec §FR-005, research §R2]
- [ ] CHK003 - Are all reader categories enumerated (geometry, render single + split panes, status bar, menu check, cache gate)? [Completeness, Spec §FR-006, research §R3]
- [ ] CHK004 - Is the toggle's exact effect specified (active buffer only; invalidate cache; no config write)? [Completeness, Spec §FR-002, research §R5]
- [ ] CHK005 - Is the wrap-cache↔active-buffer relationship specified (single active cache, invalidated on switch)? [Completeness, Spec §FR-003, research §R4]

## Requirement Clarity

- [ ] CHK006 - Is "behavior-preserving" defined for this feature (single-tab + untouched-default identical to before)? [Clarity, Spec §FR-007]
- [ ] CHK007 - Is the split-view behavior unambiguous (each pane uses its own buffer's flag for layout; active pane gets the cache; non-active wrapped pane best-effort, no corruption)? [Clarity, Spec §Edge Cases, tasks §T006]
- [ ] CHK008 - Is "indicators reflect the active tab" concrete (which indicators: View menu check + status bar)? [Clarity, Spec §FR-004]

## Requirement Consistency

- [ ] CHK009 - Do spec/research/data-model/tasks agree that `App::soft_wrap` is removed and readers move to a buffer's flag? [Consistency, all artifacts]
- [ ] CHK010 - Is the "config is a default seed, not live state" decision consistent across spec Assumptions, research R2/R5, and the toggle task (no config write)? [Consistency]

## Acceptance Criteria Quality

- [ ] CHK011 - Are success criteria measurable (toggle-one-leaves-other; round-trip preserves; indicators match; suite green)? [Measurability, Spec §SC-001..005]
- [ ] CHK012 - Is there a baseline to compare the unchanged-suite claim (1268/0/11)? [Measurability, tasks §T001]

## Scenario & Edge-Case Coverage

- [ ] CHK013 - Are new/opened/restored tab defaults covered? [Coverage, Spec §FR-005, tasks §T004]
- [ ] CHK014 - Is the sole-tab case covered (behaves as today)? [Edge Case, Spec §Edge Cases]
- [ ] CHK015 - Is tab-close covered (remaining tabs keep their own settings; nothing migrates)? [Edge Case, Spec §Edge Cases]
- [ ] CHK016 - Is the no-panic/no-corruption guarantee tied to a check (042 fuzz toggles wrap + switches)? [Coverage, Spec §FR-008, tasks §T013]

## Non-Functional & Scope

- [ ] CHK017 - Is session persistence of wrap state explicitly out of scope? [Scope, Spec §Assumptions]
- [ ] CHK018 - Is a second split-view cache explicitly out of scope? [Scope, Spec §Assumptions]
- [ ] CHK019 - Is the docs-gate impact identified (CAPABILITIES soft-wrap line updated for per-tab scope)? [Dependency, tasks §T014]
- [ ] CHK020 - Does the change keep the 042 `clippy::unwrap_used` guardrail satisfied (no new unwraps in app code)? [Non-Functional, tasks §T012]

## Notes

- Load-bearing: CHK001 (storage), CHK004 (toggle scope), CHK007 (split-view clarity), CHK006
  (behavior-preserving). All answerable "yes, specified" from the artifacts.
