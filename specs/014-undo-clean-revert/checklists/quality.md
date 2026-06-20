# Quality Checklist: Undo-to-clean state and Revert

**Purpose**: Requirements-quality gate for feature 014 (focus: correctness/safety of clean-state
tracking and data-loss avoidance on Revert). Tests the requirements, not the implementation.
**Created**: 2026-06-20
**Feature**: [spec.md](../spec.md)

## Correctness of clean-state

- [x] CHK001 Is "clean" defined as content equality with a precise baseline (last saved, or open-time
  content for unsaved-opened files)? [Clarity, Spec §FR-001/§FR-003]
- [x] CHK002 Is the redo-away-from-saved transition specified to restore Modified? [Completeness, §FR-002]
- [x] CHK003 Is the post-save baseline update specified so the indicator is correct immediately after
  saving? [Completeness, §FR-005]

## Data-loss safety (no false-clean / no silent discard)

- [x] CHK004 Is it explicitly required that the buffer is never shown clean unless content equals the
  baseline, including after divergent edits? [Safety, §FR-004 / §SC-003]
- [x] CHK005 Is destructive Revert required to confirm before discarding unsaved changes? [Safety, §FR-007]
- [x] CHK006 Is Revert-cancel required to leave everything unchanged? [Completeness, §FR-007 / US3 AC2]

## Revert behavior coverage

- [x] CHK007 Is the no-file (never-saved) Revert case defined as a safe no-op with a notice? [Coverage, §FR-009]
- [x] CHK008 Is the reload-failure case (missing/unreadable) defined to leave the buffer unchanged? [Edge, §FR-010]
- [x] CHK009 Is the post-Revert state fully specified (content == disk, clean, valid cursor)? [Completeness, §FR-008]

## Non-regression & measurability

- [x] CHK010 Is no-regression for ordinary edit/undo/redo/save/autosave stated as a requirement? [§FR-011]
- [x] CHK011 Are success criteria objectively measurable (100% clean after N undos; 0 false-clean)? [§SC-001..003]
- [x] CHK012 Is multi-buffer independence of the clean baseline addressed? [Coverage, Assumptions]
