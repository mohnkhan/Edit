# Quality Checklist: Interactive Find and Replace dialogs

**Purpose**: Requirements-quality gate for feature 015. Tests the requirements, not the implementation.
**Created**: 2026-06-20
**Feature**: [spec.md](../spec.md)

## Find correctness

- [x] CHK001 Is the search-on-confirm behavior (highlight all, jump to current, show "X of Y") fully
  specified? [Completeness, §FR-002]
- [x] CHK002 Is the not-found behavior defined (message, no document/cursor change)? [§FR-003]
- [x] CHK003 Is next/prev with wrap and indicator update specified? [§FR-004 / §SC-003]
- [x] CHK004 Is the current-vs-other match highlight distinction required? [§FR-005]

## Replace correctness & safety

- [x] CHK005 Are Replace-current and Replace-All defined with reported counts? [§FR-007 / §SC-004]
- [x] CHK006 Is replace required to be undoable and to mark the buffer modified? [§FR-008 / §SC-005]
- [x] CHK007 Is stale-offset avoidance specified (recompute after any document change/replace)? [§FR-012]

## Modality & non-regression

- [x] CHK008 Is it explicit that while a dialog is open, input edits fields not the buffer, and Esc
  cancels? [§FR-009]
- [x] CHK009 Is Ctrl+A scoped to Replace-All only while the Replace dialog is open (else Select-All)?
  [§FR-010a / Clarifications]
- [x] CHK010 Is no-regression for ordinary editing/other dialogs stated? [§FR-013 / §SC-007]

## Options & UTF-8

- [x] CHK011 Are the four toggles (case/wrap/regex/whole-word) specified, with whole-word noted as new
  engine work? [§FR-010 / Clarifications]
- [x] CHK012 Is UTF-8/grapheme correctness required for both field input and match offsets/highlights?
  [§FR-011]
- [x] CHK013 Is invalid-regex behavior defined (reported, no crash, no matches)? [Edge Cases]

## Acceptance measurability

- [x] CHK014 Are success criteria objectively checkable (exact counts, visiting all matches, zero
  remaining after Replace All, single-undo restore)? [§SC-001..006]
