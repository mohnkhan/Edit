# Feature Review Checklist: Soft-Wrap Mode (Feature 005)

**Purpose**: Validate quality, clarity, completeness, and consistency of requirements across
spec.md, plan.md, and tasks.md — before implementation begins. Tests the *requirements* as
written, not the runtime behavior.
**Created**: 2026-06-19
**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md) | [tasks.md](../tasks.md)

---

## Requirement Completeness

- [ ] CHK001 Are requirements defined for ALL eight cursor movement commands listed in FR-006 (↑ ↓ ← → Home End PgUp PgDn), or do user stories and test tasks only cover a subset (arrows, Home, End)? Is PgUp/PgDn in wrap mode fully specified? [Completeness, Spec §FR-006, Spec US2]
- [ ] CHK002 Is the terminal fallback for U+00BB (`»`) fully specified — is the fallback character (`>`) defined, and is the trigger condition (terminal incapability detection method) documented? [Completeness, Spec Assumptions]
- [ ] CHK003 Are requirements defined for wrap-cache management when multiple buffers are open — if the user switches the active buffer while soft-wrap is enabled, does the cache apply globally or is it buffer-specific? [Completeness, Gap, Spec Assumptions (global toggle)]
- [ ] CHK004 Is the behavior of ← and → arrow keys in soft-wrap mode specified — do they move within visual segments or between grapheme positions in the logical line regardless of visual breaks? [Completeness, Spec §FR-006]
- [ ] CHK005 Are requirements defined for what happens when soft-wrap is enabled and the line-number gutter is simultaneously visible — does the `»` marker appear in the gutter column or the content column? [Completeness, Gap, Spec Assumptions]
- [ ] CHK006 Is the "paste of a very long line" edge case (Spec Edge Cases) linked to a specific functional requirement (FR-???), or does it exist only in the edge case section without a corresponding testable FR? [Completeness, Traceability, Spec Edge Cases §6]

## Requirement Clarity

- [ ] CHK007 Is "immediately" in the US1 edge case ("visual reflow updates immediately after insertion") quantified with a timing threshold, or is it defined only by the general SC-001 "one screen refresh" criterion? [Clarity, Spec US1 Edge Cases, Spec §SC-001]
- [ ] CHK008 Is "within one screen refresh" in SC-001 defined with a concrete duration (e.g., ≤ 16 ms for 60 Hz, or ≤ 50 ms consistent with the constitution's keystroke-latency baseline)? [Clarity, Spec §SC-001]
- [ ] CHK009 Is the punctuation set in FR-003 ("word-boundary character: space, tab, punctuation") exhaustively enumerated and consistent with the Assumptions ("comma, period, semicolon, colon, hyphen, slash") and the wrap algorithm in contracts/wrap-cache.md? Do the three sources agree? [Clarity, Consistency, Spec §FR-003 vs Assumptions vs contracts/wrap-cache.md]
- [ ] CHK010 Is "leftmost gutter position" for the `»` marker in FR-005 defined for both states: (a) line-number gutter disabled (leftmost content column), (b) line-number gutter enabled (does `»` replace the `|` separator, or appear in the content area)? [Clarity, Spec §FR-005]
- [ ] CHK011 Is "correctly matching patterns that span visual wrap boundaries" in FR-008 clarified with a concrete example — e.g., "the pattern `foo bar` where `foo` is at the end of one visual row and `bar` at the start of the next continuation row"? [Clarity, Spec §FR-008]
- [ ] CHK012 Is "restored on toggle-off" in FR-009 precisely defined — is the restored horizontal scroll offset the value it held before wrap was enabled, or always zero? [Clarity, Spec §FR-009]
- [ ] CHK013 Is "disappear immediately" in FR-010 defined as synchronous-on-toggle (same frame) or asynchronous (next render tick)? [Clarity, Spec §FR-010]
- [ ] CHK014 Is the scope of "all user-facing named labels" in FR-014 (after M2 resolution) exhaustively enumerated — are there other text surfaces beyond the menu item where "Soft Wrap" could appear (e.g., help text, Options dialog, config comments)? [Clarity, Spec §FR-014]

## Requirement Consistency

- [ ] CHK015 Is FR-014's revised definition ("(ext) suffix in named labels; status bar uses abbreviated [WRAP]") consistent with SC-006's "no occurrence of unmarked 'Soft Wrap' alone" — does `[WRAP]` satisfy SC-006, or does SC-006 require `[WRAP (ext)]`? [Consistency, Spec §FR-014 vs §SC-006]
- [ ] CHK016 Does the global-toggle scope (Assumptions: "global editor-wide toggle, not per-buffer") align with FR-011 which only mentions writing `soft_wrap` to config — is there a requirement that the runtime state and config state always agree? [Consistency, Spec Assumptions vs §FR-011]
- [ ] CHK017 Is the minimum viewport width (10 columns) consistent across all three locations: spec Assumptions, the revised edge case section, and contracts/wrap-cache.md? Do all three agree on the exact threshold and the response when violated? [Consistency, Spec Assumptions vs Edge Cases vs contracts/wrap-cache.md]
- [ ] CHK018 Does SC-003 ("automated unit tests covering all movement keys") align with the current task set — does tasks.md T024 now cover all six movement-key scenarios listed in SC-003, and do they include PgUp/PgDn? [Consistency, Spec §SC-003 vs tasks.md T024]

## Acceptance Criteria Quality

- [ ] CHK019 Is SC-001's "visual reflow completing within one screen refresh" measurable in an automated test, or does it require manual observation? If the former, is a performance assertion (timing check) documented in tasks? [Measurability, Spec §SC-001]
- [ ] CHK020 Is SC-005's "no display corruption" for CJK/emoji measurable without visual inspection — is the automated assertion defined (e.g., checking that all break offsets in `visual_starts` land at grapheme-cluster boundaries)? [Measurability, Spec §SC-005, tasks.md T032]
- [ ] CHK021 Can SC-006's "no occurrence of unmarked 'Soft Wrap' alone" be verified automatically (e.g., grep for the string in source) without human inspection of every UI surface? [Measurability, Spec §SC-006]
- [ ] CHK022 Is SC-003's "500-column logical line" scenario tied to a concrete test fixture file, or does the requirement leave fixture content and column count unspecified? [Measurability, Spec §SC-003, tasks.md T001]

## Scenario & Edge Case Coverage

- [ ] CHK023 Are requirements defined for soft-wrap behavior in split-view mode — when two editor panes have different widths, does each pane use its own viewport width for wrap-point computation? [Coverage, Gap]
- [ ] CHK024 Are requirements defined for undo/redo of content changes while soft-wrap is active — does the requirement specify that the wrap display reflects the undone state immediately? [Coverage, Gap]
- [ ] CHK025 Are requirements defined for WrapCache behavior when the buffer has zero lines (empty file) or exactly one empty line? [Coverage, Edge Case, Spec Edge Cases §4]
- [ ] CHK026 Are requirements defined for clipboard Cut/Copy when the selection spans a visual wrap boundary — is the copied text the logical text (no extra newlines), consistent with FR-007? [Coverage, Gap, Spec §FR-007]
- [ ] CHK027 Are requirements defined for auto-save behavior (the 30-second autosave timer) when soft-wrap is active — does FR-007 explicitly cover auto-save, or does it only address user-initiated Ctrl+S? [Coverage, Gap, Spec §FR-007]
- [ ] CHK028 Are requirements defined for what happens if config-write fails on toggle (e.g., disk full, read-only config directory) — is there a specified error recovery path per FR-011? [Coverage, Exception Flow, Spec §FR-011]

## Non-Functional Requirements

- [ ] CHK029 Is a memory budget defined for WrapCache when used with large files — e.g., is there a specified upper bound on cache size for a 100 MB file with many long lines, consistent with the constitution's ≤ 50 MB memory baseline? [Performance, Gap]
- [ ] CHK030 Is a worst-case computation time specified for `WrapCache::compute()` on pathological inputs (e.g., a single 4096-character line with no whitespace)? [Performance, Gap]
- [ ] CHK031 Are accessibility requirements defined for the soft-wrap toggle — e.g., is the `[WRAP]` status indicator readable by screen readers, or is there a requirement to expose wrap state via terminal accessibility APIs? [Accessibility, Gap]

## Dependencies & Assumptions

- [ ] CHK032 Is the assumption that "`unicode_segmentation` and `unicode_width` are already in Cargo.toml" validated against the current Cargo.toml lock file — and is there a requirement that a crate version upgrade cannot silently change grapheme-cluster boundary behavior? [Dependency, Assumption]
- [ ] CHK033 Is the assumption that "buffer, rope, and encoding subsystems require no changes" a formal boundary constraint — what happens if a future change to the rope API breaks the grapheme-walk invariant that `WrapCache::compute()` depends on? [Assumption, Risk]
- [ ] CHK034 Is the assumption that "the existing config-save path uses atomic tmp-rename" documented with a reference to the implementation, so that T027 (config write on toggle) does not accidentally bypass the atomic write pattern? [Assumption, Spec §FR-011, tasks.md T027]

## Integration Requirements

- [ ] CHK035 Are requirements defined for Find/Replace match highlighting in soft-wrap mode — specifically, when a match spans a visual wrap boundary, is the highlight required to appear on both visual rows (continuation rows)? [Integration, Spec §FR-008, Gap]
- [ ] CHK036 Are requirements defined for mouse click-to-position in wrap mode at the spec level (not just in Assumptions) — is there a formal FR for mouse click mapping, or is it only referenced in Assumptions and contracts? [Integration, Traceability, Spec Assumptions]
- [ ] CHK037 Is the interaction between soft-wrap and syntax highlighting specified — do highlighted spans need to be preserved correctly across visual wrap boundaries in the rendering path? [Integration, Gap]

---

## Notes

- Mark items `[x]` when the underlying requirement is confirmed clear and complete
- Add inline comments with findings or spec references where gaps are found
- Items marked `[Gap]` indicate requirements that may be missing and should be addressed in spec before implementation
- Items marked `[Consistency]` indicate cross-document alignment issues requiring resolution
- `[Traceability]` items are missing a formal FR anchor and may be discoverable only in prose sections
