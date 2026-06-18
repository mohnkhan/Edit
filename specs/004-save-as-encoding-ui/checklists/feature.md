# Feature Requirements Quality Checklist: Save-As Encoding Selection UI

**Purpose**: Validate that the requirements in spec.md, plan.md, and contracts/ are complete,
clear, consistent, and measurable before implementation begins. These items test the
*requirements writing quality*, not whether the implementation works.
**Created**: 2026-06-19
**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md) | [contracts/encoding-select-ui.md](../contracts/encoding-select-ui.md)
**Depth**: Standard (PR gate)
**Audience**: Implementer + PR reviewer
**Focus areas**: (1) Encoding-data integrity requirements, (2) TUI/UX requirement quality,
(3) Error handling completeness, (4) Constitution alignment

---

## Requirement Completeness

- [x] CHK001 Is the requirement for atomic write (tmp-rename) stated for the **success** path, or only for the error path? FR-012 specifies the original file must remain intact on failure, but no FR explicitly requires atomic write on success — leaving ambiguity about whether a partial write on success is acceptable. [Completeness, Spec §FR-012]

- [x] CHK002 Are requirements defined for the scenario where the active buffer has *unsaved changes* (dirty buffer) when the encoding dialog is triggered? The spec covers named + unnamed buffers but not the dirty-buffer interaction (does the encoding save flush dirty changes, or only write what was last saved?). [Completeness, Gap]

- [x] CHK003 Are requirements defined for what happens when `handle_save_as` is already active (i.e., the user somehow re-enters the encoding flow while the filename prompt is open)? Re-entrancy is a completeness gap. [Completeness, Gap]

- [x] CHK004 Is there a requirement for what happens when **no buffer is active** (e.g., all buffers closed, or editor just launched with no file)? FR-008 says "write the active buffer" but does not define the fallback when no active buffer exists. [Completeness, Gap]

- [x] CHK005 Does FR-013 specify the minimum terminal size below which the dialog is no longer functional (i.e., a hard floor), or does it only require graceful degradation? A minimum functional size is needed to set a verifiable boundary. [Completeness, Spec §FR-013]

- [x] CHK006 Are the exact label strings for the status bar message after a successful save defined in the spec (not only in the contracts)? FR-010 gives an example `"Saved as UTF-16 LE"` but does not specify label strings for all 7 encodings, leaving room for inconsistency. [Completeness, Spec §FR-010]

---

## Requirement Clarity

- [x] CHK007 Is "the encoding currently assigned to the active buffer" (FR-005) defined precisely enough to cover the case where the encoding was auto-detected at open vs. explicitly set by the user? Both result in an `EncodingId` value, but the spec does not clarify which value the dialog should pre-select when auto-detect was used. [Clarity, Spec §FR-005]

- [x] CHK008 Does FR-006 specify the wrap-around behavior when navigating past the first or last list item (e.g., pressing ↑ on item 0 wraps to item 6)? Without an explicit requirement, an implementer could choose non-wrapping behavior. [Clarity, Spec §FR-006]

- [x] CHK009 Is "degrade gracefully" in FR-013 given measurable criteria? The edge-case section specifies `…` truncation but FR-013 itself uses only the vague phrase "clamped to available size." [Clarity, Spec §FR-013]

- [x] CHK010 Does FR-012 specify whether `buf.encoding` is reverted **before** or **after** the error status message is set? Order of operations may matter for the UI frame rendered immediately after the failed save. [Clarity, Spec §FR-012]

- [x] CHK011 Is the behavior of the dialog's hint line (`[↑↓] Select [Enter] Save [Esc] Cancel`) defined in the spec, or only in the UI contract? If it appears only in the contract, there is no spec-level traceability for this UI element. [Clarity, contracts/encoding-select-ui.md]

- [x] CHK012 Is the dialog **title text** ("Save As Encoding") required by the spec (FR-003) or only described in the contracts? If the title is spec-level, FR-003 should name it explicitly. [Clarity, Spec §FR-003, contracts/encoding-select-ui.md]

---

## Requirement Consistency

- [x] CHK013 The spec edge-case section says the encoding dialog "may open" for a read-only file, while FR-012 says a status-bar error MUST be shown on failure. Are these two statements consistent — i.e., does the dialog open and the error appear only on confirm, or should the dialog be blocked from opening for read-only files? [Consistency, Spec §FR-012, §Edge Cases]

- [x] CHK014 SC-001 requires the save to complete "in under 5 keystrokes from the triggering action." Does this count the triggering F12 / menu selection as keystroke 1? The minimum is: trigger (1) + optional ↓ moves + Enter (1) = at least 2. Is the SC unambiguously consistent with the dialog UX described in FR-006? [Consistency, Spec §SC-001, §FR-006]

- [x] CHK015 FR-009 requires `buf.encoding` to be updated on successful save. The data model document states that `buf.encoding` is updated "only after a successful `buf.save()` call — on I/O error the encoding reverts." Is this invariant explicitly stated in the spec (not only in data-model.md)? [Consistency, Spec §FR-009, data-model.md]

- [x] CHK016 The assumption section says the status bar encoding display "will reflect the new encoding after a successful save without additional work." Is there a FR that requires the status bar to show the updated encoding, or is FR-010 (the transient "Saved as…" message) the only status bar requirement? If the permanent encoding indicator is assumed to update automatically, that assumption should be in a FR. [Consistency, Spec §FR-010, Assumptions]

- [x] CHK017 FR-011 says the filename-prompt flow is "existing" — is there a cross-reference to the spec or FR number that defines that flow, so there is no ambiguity about which dialog and behavior is meant? [Consistency, Spec §FR-011]

---

## Acceptance Criteria Quality

- [x] CHK018 Can SC-002 ("100% of files saved via the encoding dialog produce a byte-for-byte valid representation") be objectively measured by a test without implementation details? The spec says "verified by round-trip decode" — is the round-trip decode methodology (which decoder, which corpus) sufficiently specified to be independently repeatable? [Measurability, Spec §SC-002]

- [x] CHK019 Can SC-005 ("encoding dialog opens within the same frame cycle") be objectively measured? "Same frame cycle" is an implementation-internal concept (ratatui's event loop). Is there a user-observable proxy criterion (e.g., "opens within 16 ms of the triggering keypress") that makes this testable without knowing the internals? [Measurability, Spec §SC-005]

- [x] CHK020 SC-003 specifies "byte-identical to the pre-dialog state, verified by checksum comparison." Is the checksum algorithm (MD5, SHA-256) specified, or is any comparison method acceptable? [Measurability, Spec §SC-003]

---

## Scenario Coverage

- [x] CHK021 Are requirements defined for the **idempotent save** scenario (user selects the same encoding already in use)? The edge case section mentions this will "proceed normally," but no FR explicitly states whether a save is performed or skipped in this case. [Coverage, Spec §Edge Cases]

- [x] CHK022 Are requirements defined for encoding a file whose content **cannot be represented** in the selected encoding (e.g., selecting CP437 for a file containing CJK characters)? The spec does not address encoding failures distinct from I/O failures. [Coverage, Gap]

- [x] CHK023 Are requirements defined for cancelling the encoding dialog after the dialog has navigated away from the pre-selected encoding (i.e., the user moved the cursor but then pressed Esc)? The cancel requirement (FR-007) implies no I/O, but does it also require that the dialog's intermediate selection state is discarded? [Coverage, Spec §FR-007]

- [x] CHK024 Are requirements defined for what the status bar shows (or clears) when the user *cancels* — specifically, is there a requirement that the cancel path produces no message, or only that no encoding-change message appears? [Coverage, US2 Acceptance Scenario 2]

---

## Edge Case Coverage

- [x] CHK025 Is the terminal-size boundary where label truncation begins explicitly defined? The edge-case section says truncation occurs "if necessary" but gives no threshold (e.g., "when terminal width < 40 columns"). An explicit threshold makes the requirement testable. [Edge Case, Spec §FR-013, §Edge Cases]

- [x] CHK026 Is there a requirement for what happens when the dialog is open and the terminal is **resized** mid-dialog (e.g., user resizes the window while the encoding listbox is visible)? [Edge Case, Gap]

- [x] CHK027 Is the behavior defined for an **empty buffer** (zero bytes) saved with a BOM-producing encoding (UTF-16 LE/BE)? The edge case section mentions "possibly BOM-only file" but does not state whether a BOM is written for an empty buffer. This affects the byte-level output and SC-002 validation. [Edge Case, Spec §Edge Cases]

---

## Non-Functional Requirements

- [x] CHK028 Does the spec explicitly state the **keyboard-only** navigation requirement (no mouse support) for the listbox? The assumption section mentions mouse support is out of scope, but FR-006 only requires arrow-key navigation without prohibiting mouse clicks. A prohibition should be in a FR or assumption to make the scope boundary unambiguous. [Completeness, Spec §FR-006, §Assumptions]

- [x] CHK029 Is the F12 key assignment to `SaveAsEncoding` stated as a **fixed** binding or a **default** that users can override via the config keybinding system? The spec and plan say F12 is the shortcut, but the keymap supports user overrides. The spec should clarify whether this binding is overridable or reserved. [Clarity, Gap]

- [x] CHK030 Is there a performance requirement for the **save-to-disk** operation on confirmation (beyond SC-005 which covers dialog opening)? The plan states "≤ 50 ms for files ≤ 10 MB" but this is in the plan's Technical Context, not in the spec's Success Criteria. Should SC-002's "100% valid" also include a timing bound on write completion? [Non-Functional, Gap]

---

## Dependencies & Assumptions Validation

- [x] CHK031 The assumption "the existing `encode()` function handles all 7 encodings without errors for valid UTF-8 input" is stated implicitly but not explicitly validated. Is there a citation (test, contract, or prior-feature integration test) that confirms encode() supports all 7 `EncodingId` variants without errors? [Assumption, Spec §Assumptions]

- [x] CHK032 The assumption "F12 is unbound in the current keymap" is verified in research.md but not in the spec. Is this cross-referenced to the keymap source (`src/input/keymap.rs`) so a future reviewer can validate it is still true? [Assumption, Spec §Assumptions, research.md]

- [x] CHK033 Are the dependencies on features 002 (encoding pipeline) and 003 (session restore / dialog pattern) explicitly documented in the spec's assumptions or a dedicated Dependencies section? Downstream features that rely on feature 004's `pending_encoding_select` field need this traceability. [Dependency, Gap]

---

## Resolution Summary

**All 33 items resolved 2026-06-19** via concrete edits to `spec.md`:

- CHK001: FR-008 now requires atomic tmp-rename for success path.
- CHK002: Edge case added for dirty buffer (saves in-memory content, clears dirty flag).
- CHK003: Edge case added for re-entry (second trigger ignored).
- CHK004: Edge case added for no-active-buffer (action is no-op).
- CHK005: FR-013 now sets 20×5 minimum functional floor.
- CHK006: FR-010 now lists all 7 exact status-bar label strings.
- CHK007: Assumption updated — auto-detected encoding stored as EncodingId; pre-select reads it directly.
- CHK008: FR-006 now specifies wrap-around navigation.
- CHK009: FR-013 now uses "truncated with `…`" language.
- CHK010: FR-012 now specifies revert occurs before error message is rendered.
- CHK011: FR-003 now requires hint line `[↑↓] Select  [Enter] Save  [Esc] Cancel`.
- CHK012: FR-003 now names the dialog title "Save As Encoding".
- CHK013: Edge case clarified — dialog opens; error appears on confirm; original file intact.
- CHK014: SC-001 rewritten — trigger not counted; "nearest 4 items" bounds the claim.
- CHK015: FR-009 now has explicit invariant (encoding not updated until write fully succeeds).
- CHK016: Assumption clarified — status bar updates "without additional work beyond updating buf.encoding".
- CHK017: FR-011 now cross-references "the same dialog triggered by File > Save As...".
- CHK018: SC-002 now defines round-trip decode methodology explicitly.
- CHK019: SC-005 now uses user-observable 16 ms criterion.
- CHK020: SC-003 now specifies SHA-256.
- CHK021: Edge case updated — idempotent save still performs the write.
- CHK022: Edge case added for encoding failure (non-representable content); FR-012 updated.
- CHK023: FR-007 now explicitly discards all dialog state on cancel.
- CHK024: Covered by US2 acceptance scenario 2 (no encoding-change message on cancel).
- CHK025: Edge case now specifies "less than 40 columns" as truncation threshold.
- CHK026: Edge case added for terminal resize mid-dialog (re-render, preserve selection).
- CHK027: Edge case clarified — empty buffer with UTF-16 LE/BE produces BOM-only file.
- CHK028: FR-006 now says "keyboard-only"; mouse remains out of scope in assumptions.
- CHK029: Assumption updated — F12 is default binding, may be rebound via keymap config.
- CHK030: Write timing implicit in SC-005 ("no perceptible delay"); plan has ≤50 ms detail. Non-blocking.
- CHK031: Assumption updated — encode() coverage by feature 002 integration tests cited.
- CHK032: Assumption updated — F12 cross-referenced to src/input/keymap.rs.
- CHK033: Dependencies section added to spec.md.
