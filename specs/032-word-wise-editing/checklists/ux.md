# UX / Behavior-Quality Checklist: Word-wise navigation, selection, and deletion

**Purpose**: Validate the *requirements* for feature 032 before implementation.
**Created**: 2026-06-21
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/behavior.md](../contracts/behavior.md)

## Boundary definition

- [x] CHK001 Is "word" defined precisely and tied to feature 030's `grapheme_class` (consistency)? [Clarity, FR-001 / Assumptions]
- [x] CHK002 Is the step rule for each direction specified (skip current run + following whitespace / preceding whitespace + token run)? [Completeness, research D2]

## Movement (US1)

- [x] CHK003 Is line-crossing specified for both directions (EOL → next line; col 0 → prev line end)? [Edge Case, FR-002]
- [x] CHK004 Is buffer start/end specified as a no-op (no panic)? [Edge Case, FR-002]
- [x] CHK005 Is selection-clearing + scroll-into-view on a plain move specified? [Consistency, FR-003]

## Selection (US2)

- [x] CHK006 Is word-wise selection specified as consistent with Shift+Arrow and usable by Copy/Cut? [Consistency, FR-004]
- [x] CHK007 Is anchor preservation across multiple steps implied/testable? [Measurability, US2 §Independent Test]

## Deletion (US3)

- [x] CHK008 Is single-undo-step deletion specified, with cursor landing at the deletion point? [Clarity, FR-005]
- [x] CHK009 Is "delete the selection instead when one exists" specified? [Completeness, FR-005]
- [x] CHK010 Is the read-only guard (no change + message) specified? [Consistency, FR-006]
- [x] CHK011 Is buffer-end no-op + undo-restores-text-and-cursor specified? [Edge Case, FR-006 / Edge Cases]

## Keys, scope & no-regression

- [x] CHK012 Are all six bindings enumerated and "no existing binding changes" stated and testable? [Completeness, FR-007]
- [x] CHK013 Is graceful degradation for terminals without Ctrl+Arrow specified? [Edge Case, spec §Edge Cases]
- [x] CHK014 Is no-regression of per-character editing/selection/clipboard/undo stated? [Consistency, FR-008]
- [x] CHK015 Is the no-new-dependency constraint stated? [Non-Functional, FR-009]
- [x] CHK016 Does each SC-001..SC-005 trace to a requirement and a task? [Traceability, analyze]

## Notes

- Multibyte/wide/combining correctness (CHK001) and single-undo-step deletion (CHK008) are the key risks;
  both must be TDD-covered. All items pass on the current spec.
