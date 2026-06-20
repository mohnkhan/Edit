# UX / Behavior-Quality Checklist: Interaction completeness

**Purpose**: Validate that the *requirements* for feature 030 are complete, clear, and measurable.
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/behavior.md](../contracts/behavior.md)

## US1 — in-dialog mouse

- [x] CHK001 Is "click a list row → select it + focus the list" specified for both the encoding and plugin lists? [Completeness, FR-001]
- [x] CHK002 Is "click in a field → caret to clicked grapheme (clamped) + focus" specified for all four fields? [Completeness, FR-002]
- [x] CHK003 Is the drawn==clickable geometry requirement (shared with the renderer) stated? [Consistency, FR-001/002]
- [x] CHK004 Is button-click / outside-click / not-on-anything behavior specified as unchanged/no-op? [Clarity, FR-003]

## US2 — double/triple-click

- [x] CHK005 Is "word" defined precisely (alphanumeric/`_` run vs adjacent run) so it's testable? [Clarity, Assumptions / FR-004]
- [x] CHK006 Is triple-click = whole logical line specified? [Completeness, FR-004]
- [x] CHK007 Is the click-count window (time + same cell) and the single-click-clears rule specified? [Clarity, FR-005]
- [x] CHK008 Is the selection usable by Copy/Cut stated? [Completeness, FR-006]
- [x] CHK009 Are boundary cases (line end, empty line, multibyte) covered as no-panic? [Edge Case, spec §Edge Cases]

## US3 — context menu

- [x] CHK010 Are the menu items (Cut/Copy/Paste/Select All) and the open trigger (right-click, near click, on-screen) specified? [Completeness, FR-007]
- [x] CHK011 Are both mouse and keyboard operation + both dismiss paths (Esc, outside-click) specified? [Completeness, FR-008]
- [x] CHK012 Are non-applicable items specified as safe no-ops? [Edge Case, FR-009]
- [x] CHK013 Is modal precedence (no menu over another modal) specified? [Consistency, FR-010]

## US4 — F-keys

- [x] CHK014 Are the five new bindings (F6/Shift+F6/F8/F9/F11) each specified with their action? [Completeness, FR-011]
- [x] CHK015 Is "additive, no shadowing of existing F-keys/Ctrl/Alt" specified and testable? [Consistency, FR-012]

## Cross-cutting

- [x] CHK016 Is no-regression of existing mouse (single-click, drag, wheel, scrollbars, buttons), keys, editing, and dialogs stated? [Consistency, FR-013]
- [x] CHK017 Is the no-new-dependency constraint stated? [Non-Functional, FR-014]
- [x] CHK018 Is each story independently testable/shippable, and does each close exactly one issue (#53–#56)? [Traceability, spec]
- [x] CHK019 Does each SC-001..SC-005 trace to a requirement and a task? [Traceability, analyze]

## Notes

- Highest risk: US1 field caret mapping (right-anchored visible text → grapheme) and US3's new overlay
  widget; both must be TDD-covered. All items pass on the current spec.
