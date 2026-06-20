# UX / Behavior-Quality Checklist: File dialog — glob filtering + richer entry details

**Purpose**: Validate that the *requirements* for feature 022 are complete, clear, consistent, and
measurable before implementation.
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/file-dialog.md](../contracts/file-dialog.md)

## Filter interpretation

- [x] CHK001 Are all four field interpretations (empty, absolute path, glob, plain substring) enumerated with their effect? [Completeness, contracts filter table]
- [x] CHK002 Is the precedence unambiguous when text could be read multiple ways (e.g. absolute path vs substring)? [Clarity, Spec §FR-005]
- [x] CHK003 Is "glob" precisely defined (which wildcards: `*`, `?`; no character classes; whole-name anchored)? [Clarity, contracts glob_match]
- [x] CHK004 Is case-insensitivity stated for both glob and substring matching? [Consistency, Spec §FR-002]
- [x] CHK005 Is "live as you type" specified (filter on each edit, not only on Enter)? [Clarity, Spec Assumptions]
- [x] CHK006 Is the empty-field restore behavior specified? [Completeness, Spec §FR-004]

## Navigation invariants

- [x] CHK007 Is the rule that directories and `..` always remain (regardless of filter) stated? [Completeness, Spec §FR-003]
- [x] CHK008 Is selection re-clamp specified when the filter hides the selected entry? [Coverage, Spec §FR-006]
- [x] CHK009 Is scroll-offset behavior under filtering defined (no out-of-range)? [Coverage, data-model invariants]

## Mode semantics

- [x] CHK010 Is the Open-mode confirm (absolute path jump / open selected) specified as unchanged? [Consistency, Spec §FR-009]
- [x] CHK011 Is the Save-mode confirm (save the typed filename) specified as unchanged, even when filtering hides all files? [Clarity, Spec §FR-009 / tasks T011]
- [x] CHK012 Is whether filtering applies in Save mode (and its purpose as a preview) stated? [Completeness, Spec Edge Cases]

## Detail columns

- [x] CHK013 Are the per-entry detail fields enumerated (size for files, `<DIR>` for dirs/`..`, modified date)? [Completeness, Spec §FR-007]
- [x] CHK014 Is the size format defined (human-readable units, precision) so it's testable? [Measurability, contracts human_size]
- [x] CHK015 Is the date format defined (e.g. `YYYY-MM-DD HH:MM`, UTC)? [Clarity, contracts format_mtime]
- [x] CHK016 Is the truncation rule specified (name truncates, detail columns don't; ellipsis; width-correct)? [Clarity, Spec §FR-008]
- [x] CHK017 Is column alignment across rows required? [Completeness, Spec §FR-007]

## Edge cases

- [x] CHK018 Is no-match behavior specified (still list `..`/dirs)? [Edge Case, Spec §FR-003 / Edge Cases]
- [x] CHK019 Is unreadable metadata behavior specified (render blank, no failure)? [Edge Case, Spec §FR-011]
- [x] CHK020 Are multi-byte/wide names addressed for column alignment + truncation? [Coverage, Spec §FR-008 / Constitution II]
- [x] CHK021 Is tiny-terminal degradation specified (drop detail columns; no corruption/panic)? [Edge Case, Spec §FR-011 / Edge Cases]
- [x] CHK022 Is huge-file size formatting (bytes→GB) addressed without overflow/mis-rounding? [Edge Case, Spec Edge Cases]

## Interaction with prior features

- [x] CHK023 Is preservation of feature-012 navigation (arrows, parent/enter, mouse) over the filtered list stated? [Consistency, Spec §FR-010]
- [x] CHK024 Is interaction with the feature-020 buttons/focus ring specified under an active filter? [Coverage, Spec Edge Cases]
- [x] CHK025 Is the feature-021 scrollbar specified to reflect the filtered count and not overlap detail columns? [Consistency, Spec Edge Cases / FR-010]

## Non-functional & scope

- [x] CHK026 Is the per-keystroke filtering cost bounded (O(entries), metadata read once per reload)? [Non-Functional, plan.md]
- [x] CHK027 Is security preserved (open/save still validate the path; no new traversal surface)? [Non-Functional, plan Constitution VII]
- [x] CHK028 Is the no-new-dependency constraint stated (in-house glob/size/date)? [Assumption, plan Constitution IV]
- [x] CHK029 Is scope bounded to the file browser only (no editor/other dialogs)? [Clarity, Spec Assumptions]
- [x] CHK030 Is each success criterion (SC-001..SC-005) traceable to a requirement and a task? [Traceability, analyze coverage]

## Notes

- Validates requirement *quality*, not implementation. Behavioral verification lives in the TDD tasks
  and `quickstart.md`. Mouse-wheel scrolling is explicitly out of scope (queued as feature 023).
