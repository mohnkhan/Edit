# UX / Behavior-Quality Checklist: UX completeness hardening (round 2)

**Purpose**: Validate that the *requirements* for feature 029 are complete, clear, and measurable.
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/behavior.md](../contracts/behavior.md)

## Crash safety

- [x] CHK001 Is "no panic on multibyte delete/cut" specified with the char-safe extraction rule? [Clarity, FR-001]
- [x] CHK002 Is the recovery-path truncation specified as char/width-safe (no byte slice)? [Completeness, FR-002]
- [x] CHK003 Is byte→char specified to tolerate non-boundary offsets? [Edge Case, FR-003]
- [x] CHK004 Is the file-size limit defined and the over-limit behavior (clear error, no read) specified? [Clarity, FR-004 / Assumptions]

## No silent data loss

- [x] CHK005 Is save success vs failure feedback specified, with the modified flag retained on failure? [Completeness, FR-005]
- [x] CHK006 Is autosave/recovery-failure surfacing specified (not silent)? [Completeness, FR-006]

## Dialog & correctness

- [x] CHK007 Is SavePrompt Esc-cancel specified and tied to consistency with other dialogs? [Consistency, FR-007]
- [x] CHK008 Is "Save-As via browser keeps the pending encoding" specified? [Completeness, FR-008]
- [x] CHK009 Is the click→column mapping specified to account for gutter AND horizontal scroll, with gutter clicks not placing the cursor? [Clarity, FR-009]
- [x] CHK010 Is Go-to-Line modal precedence (not over a menu) specified? [Edge Case, FR-016]

## Display width

- [x] CHK011 Is a SINGLE width function mandated, with combining=0 / wide=2 / emoji, used by all surfaces? [Consistency, FR-010]
- [x] CHK012 Are the surfaces required to use it enumerated (editor, file browser, tab bar, dialog fields)? [Coverage, FR-010 / plan]

## Feedback

- [x] CHK013 Are copy/cut/paste (incl. empty clipboard + clipboard failure) feedback messages specified? [Completeness, FR-011]
- [x] CHK014 Is the read-only-edit message specified? [Completeness, FR-012]
- [x] CHK015 Is the file-open failure message (path + reason, not a silent blank) specified, preserving the new-file case? [Clarity, FR-013]

## Reachability & legibility

- [x] CHK016 Is Ctrl+W → Close (and a File menu item) specified, matching the docs? [Consistency, FR-014]
- [x] CHK017 Is "selected menu item legible in every theme" specified and measurable? [Measurability, FR-015]

## Scope & no-regression

- [x] CHK018 Are the deferred enhancements explicitly out of scope (and to be tracked)? [Scope, spec §Assumptions / tasks T034]
- [x] CHK019 Is no-regression of existing keys/mouse/editing/formats/crash-log stated? [Consistency, FR-017]
- [x] CHK020 Is the no-new-dependency constraint stated? [Non-Functional, FR-018]
- [x] CHK021 Does each SC-001..SC-008 trace to a requirement and a task? [Traceability, analyze]

## Notes

- Highest risk: the unified width function (CHK011-012, touches many surfaces) and the crash fixes
  (CHK001-004). Both must be TDD-covered. All items pass on the current spec.
