# UX / Behavior-Quality Checklist: Mouse-wheel scrolling

**Purpose**: Validate that the *requirements* for feature 023 are complete, clear, consistent, and
measurable before implementation.
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/wheel.md](../contracts/wheel.md)

## Routing & scope

- [x] CHK001 Is the routing target for every surface (editor, file browser, Help/About, encoding, plugin, Find/Replace) specified? [Completeness, contracts routing table]
- [x] CHK002 Is modal-wins precedence (wheel scrolls the open modal, not the editor) stated? [Clarity, Spec §FR-003]
- [x] CHK003 Is the behavior for a wheel over a non-scrollable row (menu/status bar) specified? [Edge Case, Spec §FR-009 / I1]
- [x] CHK004 Is the split-view pane-under-cursor selection rule specified? [Coverage, data-model]

## Editor behavior

- [x] CHK005 Is "viewport-only, cursor not moved" stated unambiguously? [Clarity, Spec §FR-002]
- [x] CHK006 Is the scroll step quantified (3 lines/notch)? [Measurability, Spec §FR-006]
- [x] CHK007 Is the soft-wrap bound (visual rows) vs non-wrap (lines) specified? [Completeness, data-model]

## Bounds & resilience

- [x] CHK008 Is top/bottom (and first/last item) clamp behavior specified (no over-scroll)? [Edge Case, Spec §FR-005]
- [x] CHK009 Is the content-fits case (no overflow) specified as a no-op? [Edge Case, Spec Edge Cases]
- [x] CHK010 Is no-panic required across sizes, empty content, split view, soft-wrap, and at limits? [Non-Functional, Spec §FR-010]

## No-regression

- [x] CHK011 Is it stated that wheel handling doesn't change click/button/drag behavior? [Consistency, Spec §FR-008]
- [x] CHK012 Is it specified that a wheel event never places the cursor or starts a selection? [Clarity, Spec §US3]
- [x] CHK013 Is keyboard navigation/scrolling specified as unchanged? [Consistency, Spec §FR-008]

## Integration

- [x] CHK014 Is the feature-021 scrollbar specified to reflect the post-wheel offset? [Consistency, Spec §FR-007]
- [x] CHK015 Is reuse of existing per-surface scroll state (no new scroll model) stated? [Assumption, plan]
- [x] CHK016 Is each success criterion (SC-001..SC-004) traceable to a requirement and a task? [Traceability, analyze coverage]

## Notes

- Validates requirement quality, not implementation. Behavioral verification lives in the TDD tasks and
  `quickstart.md`. Editor defaults (viewport-only, 3-line step) are recorded as assumptions.
