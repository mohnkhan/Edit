# UX / Visual-Rendering Requirements Checklist: Find/Replace field boxes

**Purpose**: Validate that the requirements for feature 019 are complete, clear, consistent, and
measurable before implementation. Tests the *requirements*, not the implementation.
**Created**: 2026-06-20
**Feature**: [spec.md](../spec.md) · [contracts/render-contract.md](../contracts/render-contract.md)

## Visual Consistency

- [ ] CHK001 - Is "matching the visual style of the file-browser input box" defined concretely enough
  to verify (label row + 3-row bordered box + caret)? [Clarity, Spec §FR-001/FR-002]
- [ ] CHK002 - Are the field label strings specified (`Find what:` / `Replace with:`) rather than left
  implicit? [Completeness, Contract §Find/Replace]
- [ ] CHK003 - Are the box-border characters that constitute "bordered" enumerated for objective
  assertion? [Measurability, Contract §C-1]

## Caret Visibility & Focus Indication

- [ ] CHK004 - Is "visible caret" defined (glyph + that it is not the hardware cursor)? [Clarity,
  Spec §Assumptions, Research D3]
- [ ] CHK005 - Is the focus-indication rule unambiguous — caret present only in the focused box,
  absent in the unfocused box? [Consistency, Spec §FR-005, Contract §C-2]
- [ ] CHK006 - Is the caret glyph choice (`▏` vs the border `│`) specified to avoid ambiguity inside
  a bordered box? [Ambiguity, Research D3]

## Long-Text Horizontal Scroll

- [ ] CHK007 - Is the behavior for text wider than the inner box width specified (right-anchored so
  caret + trailing text stay visible)? [Completeness, Spec §Edge Cases, Contract §C-6]
- [ ] CHK008 - Is the empty-field rendering case defined (caret at start, empty-term search
  unchanged)? [Edge Case, Spec §Edge Cases]

## Small-Terminal Graceful Degradation

- [ ] CHK009 - Is "render correctly on small terminals" quantified as no panic + no drawing outside
  the frame + clamping to available width/height? [Measurability, Spec §FR-009, Contract §C-5]
- [ ] CHK010 - Is the narrow-terminal case (width < dialog preferred width) addressed in addition to
  the short-terminal case? [Coverage, Spec §Edge Cases]
- [ ] CHK011 - Is the baseline graceful-degradation behavior tied to the existing feature-015
  small-terminal guard rather than newly invented? [Consistency, Spec §Assumptions]

## Scope Guard (issue #38)

- [ ] CHK012 - Is it explicitly stated that no focus-ring or boxed-button behavior is introduced?
  [Consistency, Spec §FR-010, Contract §C-7]
- [ ] CHK013 - Are the preserved interactions enumerated so "no behavior change" is verifiable
  (editing, Tab, Alt+C/A/R/W toggles, match count, find/replace/replace-all, Esc)? [Completeness,
  Spec §FR-004–FR-008]
- [ ] CHK014 - Is it specified that no keybinding, menu item, or option is added or removed?
  [Coverage, Spec §SC-004]

## Acceptance Criteria Quality

- [ ] CHK015 - Are the success criteria (SC-001..SC-004) objectively verifiable without naming an
  implementation? [Measurability, Spec §Success Criteria]
- [ ] CHK016 - Does every functional requirement map to at least one acceptance scenario or contract
  invariant? [Traceability, Spec §FR-*, Contract §C-*]

## Notes

- Derived non-interactively from spec/plan/contract; depth=standard, audience=PR reviewer.
- All 16 items carry a traceability reference (100% ≥ the 80% minimum).
