# UX / Behavior-Quality Checklist: Caret-on-click in dialog text fields

**Purpose**: Validate the *requirements* for feature 031 before implementation.
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/behavior.md](../contracts/behavior.md)

## The shared helper

- [x] CHK001 Is the `field_caret_at` contract precise (visible-window logic, display-width, clamp, ignore caret glyph)? [Clarity, FR-001]
- [x] CHK002 Are both regimes specified — value fits vs. overflow/right-anchored? [Completeness, FR-001]
- [x] CHK003 Are multibyte/wide/combining and empty-value cases specified as no-panic? [Edge Case, spec §Edge Cases]

## Per-field behavior

- [x] CHK004 Is Find/Replace click→caret+focus specified, with label/border = no-op and buttons still active? [Clarity, FR-002]
- [x] CHK005 Is the file-browser Name caret model fully specified (Left/Right/Home/End, insert/delete at caret, click), preserving filter/append/activation/nav? [Completeness, FR-003]
- [x] CHK006 Is the Go-to-Line caret model specified with digits-only preserved and Enter/Esc unchanged? [Completeness, FR-004]
- [x] CHK007 Is the rendered caret required to appear at the caret position (mid-string)? [Consistency, FR-005]

## Geometry & consistency

- [x] CHK008 Is drawn==clickable required per field (text rect shared with the renderer)? [Consistency, FR-006]
- [x] CHK009 Is click routing precedence stated (after buttons + list rows, before fall-through)? [Clarity, research D5]

## Scope & no-regression

- [x] CHK010 Is no-regression of existing keys/mouse/editing/formats/dialog flows stated? [Consistency, FR-007]
- [x] CHK011 Is the no-new-dependency constraint stated? [Non-Functional, FR-008]
- [x] CHK012 Is each story independently testable and is the union = closing #58? [Traceability, spec]
- [x] CHK013 Does each SC-001..SC-004 trace to a requirement and a task? [Traceability, analyze]

## Notes

- The crux is `field_caret_at` (CHK001-003) and giving the two append-only inputs a caret model
  (CHK005-006); both must be TDD-covered. All items pass on the current spec.
