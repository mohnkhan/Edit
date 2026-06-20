# Requirements Quality Checklist: File Browser Dialogs

**Purpose**: Validate that the requirements for the file browser are complete, clear, consistent,
and measurable before implementation — a "unit test suite" for the spec.
**Created**: 2026-06-20
**Feature**: [spec.md](../spec.md) · **Focus**: navigation, keyboard/mouse parity, UTF-8 names,
path validation/security, filesystem error handling, modal behavior
**Depth**: Standard · **Audience**: Reviewer (PR)

## Navigation Correctness

- [ ] CHK001 - Is the directory sort order (parent → folders → files, case-insensitive alpha)
  explicitly specified? [Clarity, Spec §Assumptions]
- [ ] CHK002 - Are requirements defined for the parent (`..`) entry's presence and its no-op
  behavior at the filesystem root? [Completeness, Spec §FR-002, §Edge Cases]
- [ ] CHK003 - Is the starting directory unambiguously specified (active file's dir vs CWD, with
  fallback)? [Clarity, Spec §Assumptions]
- [ ] CHK004 - Is it specified that the displayed current-directory path always reflects the
  location being viewed? [Completeness, Spec §FR-004]
- [ ] CHK005 - Are scrolling requirements for listings longer than the visible area defined
  (keep selection visible)? [Coverage, Spec §FR-009]
- [ ] CHK006 - Is the behavior for an empty directory specified? [Edge Case, Spec §Edge Cases]

## Keyboard / Mouse Parity

- [ ] CHK007 - Are the keyboard controls (move / activate / parent / cancel) enumerated for the
  browser? [Completeness, Spec §FR-003, §FR-004, §FR-008]
- [ ] CHK008 - Is the single-click activation model (enter folder / pick file) specified
  unambiguously? [Clarity, Spec §FR-003a, §Clarifications]
- [ ] CHK009 - Is parity between keyboard and mouse for the same logical action stated as a
  requirement, not just implied? [Consistency, Spec §FR-010, §SC-004]
- [ ] CHK010 - Is the outside-the-box click (cancel) behavior specified? [Completeness, Spec §FR-008]

## UTF-8 / Name Handling

- [ ] CHK011 - Is UTF-8-correct rendering of directory/file names required? [Completeness, Spec §FR-011]
- [ ] CHK012 - Is truncation behavior for over-long names specified to never corrupt/split a
  multi-byte character? [Clarity, Spec §FR-011, §Edge Cases]
- [ ] CHK013 - Can "renders correctly" for non-ASCII names be objectively verified via a stated
  success criterion? [Measurability, Spec §SC-006]

## Path Validation / Security

- [ ] CHK014 - Is it required that every open/save path is validated before any file read/write?
  [Completeness, Spec §FR-012]
- [ ] CHK015 - Is the handling of `..` / symlink path components specified (resolved/validated)?
  [Clarity, Spec §Edge Cases]
- [ ] CHK016 - Are the rules for a Save filename (non-empty, single segment, no separators/`..`)
  documented? [Completeness, Spec §FR-006; data-model validation rules]
- [ ] CHK017 - Is the empty-filename Save case defined as a no-op (no zero-named file)? [Edge Case,
  Spec §Edge Cases]

## Filesystem Error Handling

- [ ] CHK018 - Are requirements defined for an unreadable directory (no crash, notice, keep prior
  state)? [Coverage, Spec §FR-013, §SC-005]
- [ ] CHK019 - Are requirements defined for a non-writable Save destination (clear error, dialog
  stays open)? [Coverage, Spec §FR-013, §Edge Cases]
- [ ] CHK020 - Is "surfaced to the user" for errors specified concretely enough to be testable?
  [Measurability, Spec §FR-013]

## Modal Behavior

- [ ] CHK021 - Is the browser's modal precedence relative to other dialogs specified? [Consistency,
  Spec §FR-014; contract §Modal precedence]
- [ ] CHK022 - Is it required that only one file dialog is open at a time? [Completeness, Spec §FR-014]
- [ ] CHK023 - Is cancel-leaves-state-unchanged stated as a requirement for all dialog modes?
  [Completeness, Spec §FR-008, §Edge Cases]
- [ ] CHK024 - Is the Save-on-unnamed-buffer trigger (`Ctrl+S`) specified to route into the Save
  browser? [Clarity, Spec §FR-007, §Clarifications]

## Acceptance Criteria & Traceability

- [ ] CHK025 - Does each P1 user story have acceptance scenarios that map to specific FRs?
  [Traceability, Spec §User Story 1–2]
- [ ] CHK026 - Are success criteria (SC-001..006) measurable without referencing implementation
  details? [Measurability, Spec §Success Criteria]
- [ ] CHK027 - Are the clarified decisions (mouse model, text entry, dotfiles, save-on-unnamed)
  reflected consistently in FRs and Assumptions (no leftover contradictions)? [Consistency,
  Spec §Clarifications]

## Notes

- This checklist validates the **requirements**, not the implementation. Items reference spec
  sections; `[Gap]`/`[Edge Case]` mark coverage of conditions that must be specified.
