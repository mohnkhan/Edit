# UX / Interaction-Quality Checklist: Interactive-dialog buttons + focus ring

**Purpose**: Validate that the *requirements* for feature 020 are complete, clear, consistent, and
measurable before implementation — "unit tests for the English."
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/focus-ring.md](../contracts/focus-ring.md)

## Focus-Ring Correctness (keyboard)

- [x] CHK001 Is the focus-ring stop order specified for every dialog and mode (encoding, plugin, file browser, Find, Replace)? [Completeness, data-model.md]
- [x] CHK002 Is the wrap behavior of `Tab`/`Shift+Tab` at both ends of the ring explicitly defined? [Clarity, Spec §FR-005]
- [x] CHK003 Is the default focus stop on open unambiguously specified (primary control, not a button) for all four dialogs? [Clarity, Spec §FR-006]
- [x] CHK004 Are the conditions under which a button is "focused" vs the primary control defined in measurable terms (`dialog_focus` vs `field_stops`)? [Measurability, data-model.md]
- [x] CHK005 Is the requirement that exactly one stop is focused at any time stated and verifiable? [Clarity, Spec §FR-004 / SC-004]
- [x] CHK006 For Find/Replace, is the relationship between the ring's field stops and the existing `FindReplaceDialog.focus` field specified? [Consistency, contracts focus-ring §Find/Replace]

## Mouse / Click Affordance

- [x] CHK007 Is it specified that a click activates the clicked button directly (not merely moves focus)? [Clarity, Spec §FR-008 / US2 scenario 5]
- [x] CHK008 Is the drawn-equals-clickable geometry invariant stated as a requirement, not left implicit? [Completeness, Spec §FR-012 / data-model Invariants]
- [x] CHK009 For the file browser, is mouse precedence between buttons and entry hit-testing defined? [Consistency, contracts focus-ring §File browser]
- [x] CHK010 Is the behavior of a click inside the dialog but not on a button or primary control specified? [Edge Case, Spec Edge Cases]
- [x] CHK011 Is each dialog's existing outside-click behavior stated as unchanged by this feature? [Consistency, Spec Assumptions]

## Zero-Regression of Existing Keys

- [x] CHK012 Are the exact pre-existing keys to preserve enumerated per dialog (list `Up/Down`, plugin `Space`, Find/Replace typing, `Alt+C/A/R/W`, `Ctrl+A`, `F3/F2`, `Enter`)? [Completeness, Spec §FR-010, tasks T034/T038]
- [x] CHK013 Is "preserved exactly" defined as a measurable outcome (same effect as before), e.g. via SC-003's zero-regression claim? [Measurability, Spec §SC-003]
- [x] CHK014 Is `Esc` specified to close every dialog from any focus position? [Coverage, Spec §FR-011]
- [x] CHK015 Is the behavior of list-nav keys (`Up/Down`) while a button is focused defined and non-destructive? [Clarity, Spec Assumptions / US3 scenario 5]
- [x] CHK016 Is the behavior of text-editing keys while a button is focused in Find/Replace specified (ignored vs routed)? [Clarity, contracts focus-ring §Find/Replace]

## Visual Focus Indication

- [x] CHK017 Are the requirements for how the focused button is rendered distinctly stated (reusing the 016 style)? [Completeness, Spec §FR-002 / FR-004]
- [x] CHK018 Is the requirement that the primary control retains its own focus highlight when it (not a button) is focused specified? [Consistency, tasks T016/T024/T031/T039]
- [x] CHK019 Is the dialog-height growth to fit the button row stated so the primary control is not overlapped? [Completeness, Spec §FR-002]

## Mode-Dependent Button Sets

- [x] CHK020 Is the Find-mode vs Replace-mode button set explicitly defined (Find/Close vs Find/Replace/Replace All/Close)? [Completeness, data-model §Find/Replace]
- [x] CHK021 Is the file-browser confirm-button label rule by mode (Open vs Save) specified and reconciled with the issue's "Open/Cancel" wording? [Consistency, Spec Assumptions / research D4]
- [x] CHK022 Are the per-button → existing-action mappings specified for every button so no new action is implied? [Traceability, contracts focus-ring]

## Edge-Case Coverage (resize, tiny terminal, empty list, wide/UTF-8)

- [x] CHK023 Is the behavior when the terminal is too narrow for all buttons specified (overflow dropped but keyboard-reachable, no panic)? [Edge Case, Spec Edge Cases / §FR-012]
- [x] CHK024 Is re-flow/consistency of buttons across a terminal resize specified as a requirement? [Coverage, Spec Edge Cases / SC-005, tasks T040b]
- [x] CHK025 Is the empty-list case (no plugins, empty directory) specified to still allow reaching/activating buttons? [Edge Case, Spec Edge Cases]
- [x] CHK026 Are width-correctness requirements for wide/UTF-8 button labels and list/field content stated? [Coverage, Spec §FR-012 / Constitution II]
- [x] CHK027 Is the single-button dialog (plugin manager → Close) ring behavior specified? [Edge Case, Spec Edge Cases]

## Consistency, Scope & Assumptions

- [x] CHK028 Is the scope bounded to affordance/navigation only, with "no behavioral change to what dialogs do" stated? [Clarity, Spec Assumptions / US3]
- [x] CHK029 Is the reuse of the feature-016 button component and `dialog_focus` stated as a constraint (no divergent style)? [Consistency, plan.md / research]
- [x] CHK030 Are all four target dialogs, and only those, named as in-scope (no confirm-dialog re-work)? [Scope, Spec §US1]
- [x] CHK031 Is each success criterion (SC-001..SC-005) traceable to at least one requirement and task? [Traceability, analyze coverage table]
- [x] CHK032 Are there any conflicting statements between the issue text ("Open/Cancel") and the resolved design ("Open/Save")? [Conflict — resolved in Spec Assumptions + research D4]

## Notes

- This checklist validates requirement *quality*, not implementation. Behavioral verification lives in
  the TDD tasks (tasks.md phases 3–6) and `quickstart.md`.
- Items reference the spec/plan/contracts so a reviewer can confirm each is actually written down before
  `/speckit-implement`.
