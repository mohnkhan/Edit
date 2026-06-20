# UX / Rendering-Quality Checklist: Scroll affordances + dialog button polish

**Purpose**: Validate that the *requirements* for feature 021 are complete, clear, consistent, and
measurable before implementation.
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/scroll-affordances.md](../contracts/scroll-affordances.md)

## Scrollbar correctness (all surfaces)

- [x] CHK001 Is each scrollable surface that gets a scrollbar enumerated (editor, file browser, Help/About, encoding, plugin)? [Completeness, Spec §FR-001–004]
- [x] CHK002 Are the three inputs (content length, viewport length, position) defined per surface and per axis? [Clarity, data-model.md]
- [x] CHK003 Is the relationship between thumb size/position and (content, viewport, offset) specified? [Measurability, Spec §FR-005]
- [x] CHK004 Is the editor's vertical content length defined for both non-wrap and soft-wrap modes (lines vs total visual rows)? [Completeness, Spec §FR-001]
- [x] CHK005 Is the editor's horizontal content-length source explicitly bounded (visible lines, not full file) and is that simplification documented? [Clarity, Spec Assumptions]

## "No content hidden / reserve the edge" invariant

- [x] CHK006 Is it stated that a scrollbar must not overlap content (the view reserves the bar's edge)? [Completeness, Spec §FR-006]
- [x] CHK007 Is the reserved edge specified per surface (right column for vertical, bottom row for horizontal)? [Clarity, contracts]
- [x] CHK008 Is the file-browser entry-name budget requirement (shrink by the reserved column) stated? [Completeness, contracts §File browser]
- [x] CHK009 Is the "content exactly fits → no bar / nothing hidden" behavior unambiguous? [Ambiguity, Spec §FR-007]

## Editor geometry single-source-of-truth

- [x] CHK010 Is the editor text area defined as one geometry shared by render, scroll math, and mouse mapping? [Consistency, data-model.md]
- [x] CHK011 Is it specified that `viewport_height` accounts for the reserved horizontal-bar row (and only in non-wrap mode)? [Clarity, tasks T009]
- [x] CHK012 Is the horizontal content-width source specified to subtract the reserved vertical-bar column? [Completeness, tasks T009]
- [x] CHK013 Is the requirement that a click on a reserved bar cell does not move the cursor stated? [Coverage, contracts §Editor]
- [x] CHK014 Are paging (PgUp/PgDn) and cursor-visibility specified to remain consistent with the reserved area? [Consistency, plan.md]

## Soft-wrap vs non-wrap

- [x] CHK015 Is it specified that the editor shows only the vertical bar in soft-wrap (no horizontal)? [Clarity, Spec §FR-001]
- [x] CHK016 Is the bottom-row reservation specified as conditional on non-wrap mode? [Consistency, tasks T009]

## Help / About Close button

- [x] CHK017 Is a bordered Close button required on both Help and About? [Completeness, Spec §FR-008]
- [x] CHK018 Is the Close button's effect specified as identical to the existing dismiss key? [Clarity, Spec §FR-008]
- [x] CHK019 Is the existing keyboard dismissal (Esc/Enter/printable) specified to remain unchanged? [Consistency, contracts §Help]

## Key-hint labels

- [x] CHK020 Is the requirement that every dialog button label includes its activating key stated, with scope (all dialogs)? [Completeness, Spec §FR-009]
- [x] CHK021 Is it specified that labels carry hints without changing the action, click/focus mapping, or layout? [Clarity, Spec §FR-010]
- [x] CHK022 Is the per-button key-hint mapping documented so it's testable? [Measurability, contracts key-hint table]
- [x] CHK023 Is it specified that dispatch keys on button identity/index, not the displayed text? [Consistency, contracts]

## Edge cases

- [x] CHK024 Is the tiny-terminal behavior specified (bars/buttons degrade without panic or corruption)? [Edge Case, Spec §FR-012]
- [x] CHK025 Is resize re-flow consistency (bars match drawn content, clicks still land) specified? [Coverage, Spec Edge Cases]
- [x] CHK026 Are split-view and line-number (gutter) editor scrollbar requirements specified? [Coverage, Spec Edge Cases / FR-012]
- [x] CHK027 Is empty-content behavior (empty file/dir → no spurious bar) specified? [Edge Case, Spec Edge Cases]
- [x] CHK028 Are wide/UTF-8 button label width and hit-testing requirements stated for the longer key-hint labels? [Coverage, Spec Edge Cases / Constitution II]

## Non-functional & scope

- [x] CHK029 Is the performance constraint (editor render stays within budget; no full-file scan per frame) stated? [Non-Functional, Spec Assumptions / plan.md]
- [x] CHK030 Is the scope bounded to affordance/visibility only (no scroll-behavior/action/navigation change)? [Clarity, Spec §FR-011]
- [x] CHK031 Is each success criterion (SC-001..SC-005) traceable to a requirement and a task? [Traceability, analyze coverage]
- [x] CHK032 Are the three product decisions (ratatui Scrollbar widget; key hints on all buttons; editor V+H) recorded as assumptions, not buried? [Assumption, Spec Assumptions]

## Notes

- Validates requirement *quality*, not implementation. Behavioral verification lives in the TDD tasks
  (tasks.md) and `quickstart.md`.
- The file-dialog glob-filter / detail-columns request is explicitly out of scope (feature 022).
