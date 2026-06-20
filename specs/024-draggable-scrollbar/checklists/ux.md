# UX / Behavior-Quality Checklist: Interactive scrollbars

**Purpose**: Validate that the *requirements* for feature 024 are complete, clear, consistent, and
measurable before implementation.
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/scrollbar-interaction.md](../contracts/scrollbar-interaction.md)

## Track-click behavior

- [x] CHK001 Is the track-click effect (page toward the click, up/back vs down/forward) specified? [Clarity, Spec §FR-001]
- [x] CHK002 Is the page amount quantified (one viewport)? [Measurability, Spec §FR-001 / Assumptions]
- [x] CHK003 Is horizontal-bar track-click (left/right paging, non-wrap) specified? [Coverage, Spec §FR-001 scenario 3]

## Thumb-drag behavior

- [x] CHK004 Is proportional drag mapping (cursor fraction → scroll fraction) defined and clamped? [Clarity, Spec §FR-002 / contracts]
- [x] CHK005 Is "press on thumb starts a drag, no immediate jump" specified? [Clarity, Spec Edge Cases]
- [x] CHK006 Is drag-end on release specified (later moves don't scroll)? [Completeness, Spec §FR-003]
- [x] CHK007 Is "editor drag is viewport-only, cursor unchanged, no selection" specified? [Clarity, Spec §FR-005]

## Scope of surfaces

- [x] CHK008 Are all interactive surfaces enumerated (editor V + H, file browser, Help/About, encoding, plugin)? [Completeness, Spec §FR-004]
- [x] CHK009 Is "interactive only when the bar is drawn (content overflows)" stated? [Clarity, data-model]
- [x] CHK010 Is modal-wins (only the modal's bar interactive) specified? [Consistency, Spec §FR-008]

## No-regression (feature 017 overlap — highest risk)

- [x] CHK011 Is it specified that a press on a scrollbar does NOT start text selection or place the cursor? [Clarity, Spec §FR-006]
- [x] CHK012 Is it specified that a press/drag starting off any scrollbar behaves exactly as before (selection/click)? [Consistency, Spec §FR-006]
- [x] CHK013 Is ordering (scrollbar check before feature-017 drag/selection and modal entry/button handlers) specified? [Clarity, contracts ordering]
- [x] CHK014 Are wheel (023), keyboard scrolling, and dialog actions specified as unchanged? [Consistency, Spec §FR-007]

## Bounds & resilience

- [x] CHK015 Is bounded scrolling (no over-scroll/underflow) required? [Edge Case, Spec §FR-009]
- [x] CHK016 Is tiny-thumb (thumb fills track → no-op) behavior specified? [Edge Case, Spec Edge Cases]
- [x] CHK017 Is release-outside-track / resize-mid-drag resilience specified (no panic)? [Edge Case, Spec §FR-009 / L1]
- [x] CHK018 Is "no scrollbar shown → clicks fall through to existing behavior" specified? [Edge Case, Spec Edge Cases]

## Geometry & traceability

- [x] CHK019 Is "interactive region equals drawn region (shared feature-021 geometry + thumb formula)" stated? [Consistency, Spec Key Entities / FR-010]
- [x] CHK020 Is each success criterion (SC-001..SC-004) traceable to a requirement and a task? [Traceability, analyze coverage]

## Notes

- Validates requirement quality, not implementation. The feature-017 overlap (CHK011-013) is the key risk
  and must be airtight in the TDD tests.
