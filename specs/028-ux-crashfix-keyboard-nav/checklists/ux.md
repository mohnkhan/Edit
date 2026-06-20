# UX / Behavior-Quality Checklist: UX crash-safety and keyboard navigation hardening

**Purpose**: Validate that the *requirements* for feature 028 are complete, clear, consistent, and
measurable before implementation.
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/behavior.md](../contracts/behavior.md)

## Crash-safety completeness

- [x] CHK001 Is "the renderer MUST NOT panic on any buffer content + wrap-cache state" stated as a requirement, not just an example? [Completeness, Spec §FR-001]
- [x] CHK002 Is the stale-cache condition (cached segment offsets exceeding the current line, empty lines) explicitly named as an input the renderer must tolerate? [Edge Case, Spec §FR-001 / Edge Cases]
- [x] CHK003 Are ALL active-buffer-change triggers that must invalidate the wrap cache enumerated (restore, next, prev, open, close)? [Completeness, Spec §FR-002]
- [x] CHK004 Is the panic-hook behavior specified as restore-terminal-BEFORE-print AND still-write-crash-log (both, ordered)? [Clarity, Spec §FR-003]
- [x] CHK005 Are the additional hardenings (copy/cut reversed/empty range; file-browser scroll underflow) specified with the expected safe outcome? [Completeness, Spec §FR-004]

## Crash-safety clarity & measurability

- [x] CHK006 Is "never panic" measurable (e.g. zero panics across the regression suite / across terminal sizes, buffer counts, malformed session)? [Measurability, Spec §SC-001 / Edge Cases]
- [x] CHK007 Is "restored/usable terminal" defined concretely (cooked mode, primary screen, visible cursor) rather than vaguely "usable"? [Clarity, Spec §FR-003 / US2]
- [x] CHK008 Is the post-restore correctness criterion stated (layout matches the now-active buffer), not only "no panic"? [Completeness, Spec §FR-002 / US1]
- [x] CHK009 Is the clamp target unambiguous (slice clamped to the current line length / valid char boundaries)? [Clarity, Spec §FR-001]

## Keyboard-navigation completeness

- [x] CHK010 Is "interactive dialogs open focused on the primary control" specified for ALL such dialogs (encoding, plugin mgr, find/replace, file browser)? [Completeness, Spec §FR-005]
- [x] CHK011 Is the Save-As symptom→requirement link explicit (focus-on-field ⇒ typing reaches field + caret shown)? [Clarity, Spec §FR-005 / US3]
- [x] CHK012 Are arrow-key button movements specified for BOTH dialog families (016 confirm + 020 interactive) with wrap-around? [Completeness, Spec §FR-006]
- [x] CHK013 Is the set of Help/About scroll keys complete (Up/Down/PageUp/PageDown/Home/End) AND keyboard dismissal specified? [Completeness, Spec §FR-007]
- [x] CHK014 Are Home/End editor semantics and list PageUp/PageDown both specified? [Completeness, Spec §FR-008, §FR-009]

## Keyboard-navigation clarity & consistency

- [x] CHK015 Is arrow-key button movement defined as consistent with existing Tab/Shift+Tab (same wrap, same ring)? [Consistency, Spec §FR-006]
- [x] CHK016 Is the Left/Right vs Up/Down equivalence for single-row button rows stated (so behavior is unambiguous)? [Clarity, Spec §FR-006 / Assumptions]
- [x] CHK017 Is "scroll clamped to content" specified for Help so offsets can't run past the ends? [Clarity, Spec §FR-007]
- [x] CHK018 Is "approximately one page" for list paging pinned to a concrete meaning (≈ visible rows, matching the editor)? [Ambiguity, Spec §FR-009 / Assumptions]

## Edge cases & no-regression

- [x] CHK019 Are degenerate inputs covered (single-button dialog + arrows; empty/1-item list + PageUp/Down; all-files-missing restore; terminal below minimum size)? [Edge Case, Spec §Edge Cases]
- [x] CHK020 Is no-regression scoped explicitly (existing Tab/Enter/Space/Esc, list nav, toggles, match nav, mouse, editing, file formats, crash-log file all unchanged)? [Consistency, Spec §FR-010]
- [x] CHK021 Is the no-new-dependency constraint stated (Constitution IV)? [Non-Functional, Spec §FR-011]
- [x] CHK022 Is keyboard reachability framed as the requirement (every dialog/overlay fully operable without a mouse), not just per-key additions? [Coverage, Spec §US3-US5]

## Traceability

- [x] CHK023 Does each success criterion SC-001..SC-006 trace to at least one FR and at least one task? [Traceability, analyze coverage]
- [x] CHK024 Is each P1 story (US1 restore, US2 terminal, US3 Save-As) independently testable per its Independent Test line? [Measurability, Spec §US1-US3]

## Notes

- Validates requirement quality, not implementation. The two highest risks are crash-safety
  defense-in-depth (CHK001-009: clamp AND invalidate AND restore-on-panic) and complete keyboard
  reachability (CHK010-022). Both must be covered by TDD tests per Constitution V.
- All items pass on the current spec; this checklist doubles as the PR reviewer's gate.
