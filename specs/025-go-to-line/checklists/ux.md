# UX / Behavior-Quality Checklist: Go to Line

**Purpose**: Validate that the *requirements* for feature 025 are complete, clear, consistent, and
measurable before implementation.
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/go-to-line.md](../contracts/go-to-line.md)

## Invocation & prompt

- [x] CHK001 Are both entry points (Ctrl+G and Search-menu item) specified? [Completeness, Spec §FR-001]
- [x] CHK002 Is the prompt's input behavior defined (digits, Backspace, shows current entry)? [Clarity, Spec §FR-002]
- [x] CHK003 Is "1-based line number" stated explicitly? [Clarity, Spec Assumptions]

## Jump behavior

- [x] CHK004 Is the confirm result specified (cursor to column 1 of the line + scrolled into view + close)? [Clarity, Spec §FR-003]
- [x] CHK005 Is "target line visible after the jump" stated and measurable? [Measurability, Spec §SC-003]
- [x] CHK006 Is the cursor column on arrival defined (column 1, horizontal reset)? [Clarity, Spec Edge Cases]

## Clamp / cancel / invalid

- [x] CHK007 Is over-range clamp (to last line) specified? [Edge Case, Spec §FR-004]
- [x] CHK008 Is below-1 / `0` clamp (to first line) specified? [Edge Case, Spec §FR-004]
- [x] CHK009 Is Esc-cancel (no movement) specified? [Clarity, Spec §FR-005]
- [x] CHK010 Is empty/non-numeric behavior specified (no move; non-digits rejected)? [Clarity, Spec §FR-006]
- [x] CHK011 Is oversized/overflowing input handled (clamp to last, no overflow)? [Edge Case, Spec Edge Cases]

## Modal / no-regression

- [x] CHK012 Is "prompt captures input; buffer not edited while open" specified? [Clarity, Spec §FR-007]
- [x] CHK013 Is "only one modal at a time" specified? [Consistency, Spec §FR-007]
- [x] CHK014 Is non-interference with editing / find-replace / other dialogs / wheel / scrollbar specified? [Consistency, Spec §FR-008]

## Resilience & traceability

- [x] CHK015 Is no-panic across terminal sizes / empty buffer / oversized input required? [Non-Functional, Spec §FR-009]
- [x] CHK016 Is each success criterion (SC-001..SC-004) traceable to a requirement and a task? [Traceability, analyze coverage]

## Notes

- Validates requirement quality, not implementation. Behavioral verification lives in the TDD tasks and
  `quickstart.md`.
