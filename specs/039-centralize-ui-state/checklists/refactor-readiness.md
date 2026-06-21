# Checklist: Refactor Readiness & Behavior Preservation

**Purpose**: Validate that the requirements for feature 039 are complete, unambiguous, and consistent
enough to implement a *behavior-preserving* refactor with confidence. These items test the
**requirements** (spec/plan/data-model/contracts/tasks), not the eventual code.
**Created**: 2026-06-21
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Are all overlays currently tracked by independent flags enumerated, each mapped to exactly one `Modal` case? [Completeness, data-model §field→variant table]
- [ ] CHK002 - Is it specified which state stays OUT of `Modal` (menu_bar, drag_anchor, scrollbar_drag, dialog_focus, pending_save_as_encoding) and why? [Completeness, research §R1]
- [ ] CHK003 - Are per-overlay sub-state items (Go-to-Line caret, Help scroll, encoding row, plugin cursor) explicitly assigned to their owning variant? [Completeness, Spec §FR-002]
- [ ] CHK004 - Is the cursor-bounds invariant requirement (clamp survives every overlay-close/buffer-switch path) stated as a requirement, not just an edge case? [Completeness, Spec §FR-011]
- [ ] CHK005 - Are the bug-regression guards (dropdown-over-tabs first-item click; field-caret click on non-default terminal size; no panic on rapid top-row clicks) captured as required outcomes? [Coverage, Spec §US2/US3 + quickstart]

## Requirement Clarity

- [ ] CHK006 - Is "behavior-preserving" defined precisely enough to be testable — i.e., "no existing test assertion changes except mechanical field→accessor renames"? [Clarity, Spec §FR-009]
- [ ] CHK007 - Is "two overlays open at once is unrepresentable" expressed as a structural property (single typed value) rather than a behavioral hope? [Clarity, Spec §FR-001, SC-003]
- [ ] CHK008 - Is "single-sourced layer precedence" defined concretely (one ordered list consumed by both paint and hit-test) rather than vaguely? [Clarity, Spec §FR-004, data-model §Layer]
- [ ] CHK009 - Is the removal of the `!dropdown_open` special-case stated as a requirement with its justification (precedence subsumes it)? [Clarity, Spec §FR-005]
- [ ] CHK010 - Is "shared rect helper" specified for the exact two divergent cases (Go-to-Line, Find/Replace fields), with the render and hit-test call-sites both named? [Clarity, Spec §FR-006, contracts §G1/G2]

## Requirement Consistency

- [ ] CHK011 - Do spec, plan, data-model, and tasks use the same names (`Modal`, `Layer`, `active_layers`, accessor names) without drift? [Consistency, all artifacts]
- [ ] CHK012 - Is the field count consistent across documents (the "~14" vs "13 folded + menu_active" reconciliation is stated)? [Consistency, tasks §field-count note]
- [ ] CHK013 - Are the three orderings (key dispatch, mouse dispatch, paint) consistently required to derive from the single `Modal`/precedence source — no document permitting a separate ordering? [Consistency, Spec §FR-003/FR-004]
- [ ] CHK014 - Do the kept-as-field decisions (e.g. `pending_save_as_encoding` is flow state, not an overlay) stay consistent between research and data-model? [Consistency, research §R1, data-model]

## Acceptance Criteria Quality

- [ ] CHK015 - Are success criteria objectively verifiable (suite passes unchanged; ci-local clean; precedence in one location; per-layer dispatch holds)? [Measurability, Spec §SC-001..005]
- [ ] CHK016 - Is there a measurable baseline for "suite passes unchanged" (a recorded pre-refactor test count to compare against)? [Measurability, tasks §T001]
- [ ] CHK017 - Is the new layer-dispatch invariant stated generally (all layers) rather than only for the previously-patched pair? [Measurability, Spec §SC-005, tasks §T012]

## Scenario & Edge-Case Coverage

- [ ] CHK018 - Are requirements defined for the open→open transition (opening overlay B while A is open) such that A cannot remain open? [Coverage, data-model §state transitions]
- [ ] CHK019 - Are requirements defined for overlay behavior under terminal resize (geometry re-derived from actual frame)? [Edge Case, Spec §Edge Cases]
- [ ] CHK020 - Is the context-menu open-blocking precedence specified to match today's behavior exactly? [Coverage, Spec §Edge Cases]
- [ ] CHK021 - Is the empty/degenerate buffer-set case (active-buffer access stays valid) addressed? [Edge Case, Spec §Edge Cases]
- [ ] CHK022 - Is plugin-consent (Vec, front-item prompted) coverage specified — empty Vec ⇔ no overlay? [Coverage, data-model §field→variant]

## Constitution & Non-Functional Gates

- [ ] CHK023 - Is the UTF-8 principle (II) explicitly assessed as N/A with rationale (no new byte-decoding path)? [Traceability, plan §Constitution Check]
- [ ] CHK024 - Is the test-gated principle (V) satisfied by a stated requirement that the full suite + smoke must pass, with the new invariant test added before its enabling change? [Traceability, plan §Constitution Check, tasks §T012/T013]
- [ ] CHK025 - Is the YAGNI principle (VI) addressed by showing net simplification (fields + dead flag removed; no speculative framework; retained-widget tree deferred)? [Traceability, plan §Constitution Check]
- [ ] CHK026 - Are performance baselines stated as "no regression" with the perf-check gate referenced? [Non-Functional, plan §Performance Goals, tasks §T017]

## Dependencies, Assumptions & Deferrals

- [ ] CHK027 - Is the assumption that the existing test suite is a sufficient behavior-preservation net stated explicitly? [Assumption, Spec §Assumptions]
- [ ] CHK028 - Are reused existing components (MenuState, per-widget hit-test helpers, clamp_all_cursors, active_buffer) listed so they are not reinvented? [Dependency, plan §Source Code, research]
- [ ] CHK029 - Is the deferral (splitting app.rs into multiple files; dialog_focus per-variant) recorded as out-of-scope, with the deferral-rule obligation (issue + ROADMAP row) noted? [Assumption/Scope, Spec §Assumptions, plan]
- [ ] CHK030 - Is the docs-gate obligation (CHANGELOG + STATUS; CAPABILITIES only if user-visible change — which there is none) correctly scoped? [Dependency, tasks §T019]

## Ambiguities & Conflicts

- [ ] CHK031 - Is there any remaining ambiguity about whether menu state belongs in `Modal` or as its own layer? (Should be resolved: own layer.) [Ambiguity, research §R1, data-model §Layer]
- [ ] CHK032 - Is there any conflict between "no assertion changes" (FR-009) and tasks that edit test files? (Resolved: only mechanical accessor renames permitted.) [Conflict, Spec §FR-009, tasks overriding constraint]
- [ ] CHK033 - Is the constitution's ncurses reference vs the live ratatui/crossterm stack reconciled so it cannot mislead implementation? [Ambiguity, plan §Constitution Check note]

## Notes

- This checklist validates requirement quality before `/speckit-implement`. All items should be
  answerable "yes, specified" from the existing artifacts; any "no" is a spec/plan gap to close first.
- Behavior-preservation is the dominant risk for this feature: CHK006, CHK016, CHK024, and CHK032 are
  the load-bearing items — if those requirements are crisp, the existing test suite carries the rest.
