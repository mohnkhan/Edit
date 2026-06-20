# UX / Behavior-Quality Checklist: Buffer tab bar

**Purpose**: Validate that the *requirements* for feature 027 are complete, clear, consistent, and
measurable before implementation.
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/tab-bar.md](../contracts/tab-bar.md)

## Visibility & content

- [x] CHK001 Is the show-only-with-2+-buffers rule (and single-buffer = no bar) specified? [Clarity, Spec §FR-001]
- [x] CHK002 Is the active-tab highlight specified? [Completeness, Spec §FR-002]
- [x] CHK003 Is the modified-marker (present/absent) specified? [Completeness, Spec §FR-003]
- [x] CHK004 Is the tab label source (file name / "[No Name]") and width-correct truncation specified? [Clarity, Spec §FR-009 / Assumptions]
- [x] CHK005 Is overflow behavior (keep the active tab visible) specified? [Edge Case, Spec §FR-009]

## Click behavior

- [x] CHK006 Is "click label → switch" specified (same as keyboard select)? [Clarity, Spec §FR-004]
- [x] CHK007 Is "click `[x]` → close (not switch)" specified? [Clarity, Spec §FR-005]
- [x] CHK008 Is "click on the tab row but outside any tab → no-op" specified? [Edge Case, Spec §FR-008]
- [x] CHK009 Is "a tab-row click never places the text cursor" specified? [Clarity, Spec §FR-008]
- [x] CHK010 Is drawn-tab == clickable-tab geometry stated (shared geometry)? [Consistency, data-model]

## Close + unsaved handling

- [x] CHK011 Is the unsaved-changes prompt on `[x]`-close specified (no silent data loss)? [Completeness, Spec §FR-005]
- [x] CHK012 Is it clear the confirm acts on the clicked buffer (not necessarily the active one)? [Clarity, M1 / data-model]
- [x] CHK013 Is the confirm's Save/Discard/Cancel behavior defined, incl. default = Cancel? [Clarity, contracts]
- [x] CHK014 Is "closing to one buffer hides the tab bar" specified? [Edge Case, Spec §US2]

## Geometry (the key risk)

- [x] CHK015 Is "tab bar shrinks the editor by exactly one row" specified? [Clarity, Spec §FR-007]
- [x] CHK016 Are all geometry consumers (viewport_height, click mapping, paging, wheel, scrollbar) required to account for the tab row? [Completeness, Spec §FR-007 / data-model]
- [x] CHK017 Is a click in the text with the bar shown specified to land on the correct cell? [Measurability, Spec §US3]
- [x] CHK018 Is single-buffer layout specified as unchanged? [Consistency, Spec §SC-004]

## No-regression & resilience

- [x] CHK019 Is keyboard switching (Ctrl+Tab/Ctrl+Shift+Tab) specified as unchanged? [Consistency, Spec §FR-006]
- [x] CHK020 Is "no change to opening buffers / editing" specified? [Clarity, Spec §FR-010]
- [x] CHK021 Is no-panic across terminal sizes and buffer counts required? [Non-Functional, Spec §FR-011]
- [x] CHK022 Is each success criterion (SC-001..SC-005) traceable to a requirement and a task? [Traceability, analyze coverage]

## Notes

- Validates requirement quality, not implementation. The editor-geometry-in-lockstep (CHK015-017) and the
  no-silent-data-loss close (CHK011-012) are the key risks; both must be covered by the TDD tests.
