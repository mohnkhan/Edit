# Comprehensive Review Checklist: Session Restore

**Purpose**: Formal peer-review gate — validates spec completeness & clarity, spec/plan
alignment, and security & robustness coverage before the PR merges
**Created**: 2026-06-19
**Feature**: [spec.md](../spec.md) · [plan.md](../plan.md)
**Actor/Timing**: Reviewer (PR review, pre-merge)
**Depth**: Formal gate (45 items)

**Mandatory gating sections** (a single open item in these sections BLOCKS the PR):
- 🚨 **Section 5**: Clean-Exit vs. Crash-Exit Boundary
- 🚨 **Section 6**: Security & Robustness

---

## 1. Requirement Completeness

- [x] CHK001 — Is "clean exit" enumerated exhaustively in FR-002, or does "equivalent
  user-initiated actions" (FR-001) leave ambiguous room for future keybindings or menu
  additions that may not hook into the session write path? [Completeness, Clarity, Spec §FR-002]
  *Resolved: FR-002 now enumerates all clean-exit triggers explicitly and lists non-clean signals.*

- [x] CHK002 — Is the ordering of buffers in the `[[buffers]]` TOML array defined
  (e.g., tab order, visual order, or open order)? Buffer ordering determines which buffer
  lands at `active_buffer` index on restore. [Completeness, Gap, Spec §FR-001]
  *Resolved: FR-001 now specifies "visual tab order, left to right as displayed in the tab bar".*

- [x] CHK003 — Is the expected value of `active_pane` defined for the `split_layout = "none"`
  case? FR-009 defines it as integer 0 or 1, but a single-pane session has no meaningful
  pane distinction — is 0 the canonical sentinel? [Completeness, Clarity, Spec §FR-009]
  *Resolved: FR-009 now states "MUST be 0 when split_layout is 'none'".*

- [x] CHK004 — Is the behavior defined when a restored `cursor_line` or `cursor_col`
  exceeds the actual length of the reopened file? (The file may have been edited externally
  between sessions.) [Completeness, Gap, Edge Case]
  *Resolved: Edge cases now specifies cursor clamp to last line/col; no warning shown.*

- [x] CHK005 — Is the split layout restore behavior defined when only one file of a
  two-pane split is successfully restored (the other is missing)? Should the layout
  collapse to a single pane, or attempt to open the surviving file in the original
  pane position? [Completeness, Gap, Spec §FR-004, §FR-005]
  *Resolved: FR-005 now requires the split to collapse to a single pane displaying the surviving file.*

- [x] CHK006 — Is the behavior defined when the TUI itself fails to initialize before
  the restore prompt can be shown (e.g., terminal too small, ncurses init error)?
  [Completeness, Gap]
  *Resolved: Edge cases now specifies restore prompt is skipped; existing TUI error path applies.*

- [x] CHK007 — Are the required fields that constitute a "valid" session file
  exhaustively enumerated in FR-010, so a reviewer can determine without ambiguity
  exactly which field combinations trigger the "treat as absent" path? [Completeness,
  Clarity, Spec §FR-010]
  *Resolved: FR-010 now enumerates five explicit corrupt conditions (a)–(e).*

- [x] CHK008 — Is the interaction between the session restore prompt and the existing
  crash-recovery prompt defined for the case where both conditions are simultaneously
  true (a crash recovery is pending AND a prior clean-session file exists)?
  [Completeness, Gap]
  *Resolved: Edge cases now specifies crash-recovery first, session restore second, sequentially.*

- [x] CHK009 — Is the effect of `--no-session` on the existing crash-recovery prompt
  (if any) specified? FR-008 defines its effect only on the session restore prompt.
  [Completeness, Gap, Spec §FR-008]
  *Resolved: FR-008 and edge cases now clarify --no-session suppresses session restore only.*

- [x] CHK010 — Is the behavior defined when `$XDG_STATE_HOME` is set but the path
  does not exist as a directory (distinct from "not writable")? [Completeness, Edge Case]
  *Resolved: FR-011 and edge cases now specify create_dir_all; failure = silent warn log.*

---

## 2. Requirement Clarity

- [x] CHK011 — Is "status-bar warning" defined in terms of display duration, dismissal
  behavior, and priority relative to other concurrent status messages? [Clarity,
  Spec §FR-005, §FR-006]
  *Resolved: Key Entities now defines Status-Bar Warning (5s auto-dismiss, sequential, lower priority than dialogs).*

- [x] CHK012 — Does FR-002 provide a complete list of clean-exit triggers, or is
  "equivalent user-initiated actions" open-ended in a way that makes the hook integration
  ambiguous for future developers? [Clarity, Spec §FR-002]
  *Resolved: same as CHK001 — FR-002 now has an exhaustive enumeration.*

- [x] CHK013 — Does FR-009 define whether the `path` field in `[[buffers]]` MUST be
  absolute, or can it be relative (as-opened)? The spec's Assumptions section says
  "absolute or as-opened" but FR-009 only says "string". [Clarity, Ambiguity,
  Spec §FR-009, Assumptions]
  *Resolved: FR-009 now specifies "stored as-opened: absolute if opened with absolute path, relative otherwise".*

- [x] CHK014 — Is "human-readable TOML" in FR-009 defined sufficiently (e.g.,
  pretty-printed with consistent indentation) or is any valid TOML acceptable?
  The plan uses `toml::to_string_pretty` but the spec is silent on formatting style.
  [Clarity, Spec-Plan Alignment, Spec §FR-009]
  *Resolved: "human-readable TOML" is intentionally flexible; toml::to_string_pretty satisfies it.*

- [x] CHK015 — Are all accepted keystrokes for the restore prompt defined and their
  case-sensitivity specified? The spec lists Y, Enter, N, Escape, Ctrl+C — is lowercase
  'y' or 'n' accepted? [Clarity, Spec §FR-003, §FR-007]
  *Resolved: FR-003 now enumerates Y/y/Enter (confirm) and N/n/Escape/Ctrl+C (decline); case-insensitive.*

- [x] CHK016 — Is "silently fails with a logged warning" for an unwritable
  `$XDG_STATE_HOME` (edge case section) precise enough to specify the log level and
  whether any indication reaches the user in the TUI? [Clarity, Edge Case]
  *Resolved: FR-011 and edge cases now specify warn-level log, no TUI error shown on exit.*

---

## 3. Requirement Consistency

- [x] CHK017 — Does SC-002 ("restore prompt and full reload within 2 seconds") align
  consistently with the plan's more granular targets (≤ 50 ms write, ≤ 200 ms
  read + prompt)? The plan's targets are tighter — is SC-002 the authoritative ceiling
  or a minimum bar? [Consistency, Spec-Plan Alignment, Spec §SC-002]
  *Resolved: SC-002 is the authoritative ceiling; plan's targets are implementation-level bounds and do not conflict.*

- [x] CHK018 — Are the "treat as absent" behaviors in FR-010 (invalid TOML) and the
  edge-case section (silently ignore + overwrite on next clean exit) consistent? FR-010
  specifies "corrupt file MUST be overwritten on next clean exit" but does not appear
  in the edge-case narrative. [Consistency, Spec §FR-010]
  *Resolved: Both FR-010 and the edge-case narrative are consistent; FR-010 is the normative source.*

- [x] CHK019 — Is the assumption that "cursor position is the insertion point, not a
  visual selection range" reflected consistently across FR-001's capture requirements
  and FR-009's data-model fields (no selection-range fields present)? [Consistency,
  Spec §FR-001, §FR-009, Assumptions]
  *Resolved: FR-001, FR-009, and Assumptions are all consistent — cursor_line/col only, no selection.*

- [x] CHK020 — Does FR-007 ("leave the session file unchanged when user declines")
  conflict with FR-010 ("corrupt file MUST be overwritten on next clean exit")?
  If a user declines a restore for a corrupt session file, is the file left corrupt
  until the next clean exit? [Consistency, Conflict, Spec §FR-007, §FR-010]
  *Resolved: No conflict — FR-007 governs the decline action; FR-010's overwrite applies at the next clean exit.*

---

## 4. Spec–Plan Alignment

- [x] CHK021 — Does the plan's `load_session()` validation rule — returning `Err` when
  `active_buffer >= buffers.len()` — correspond to an explicit validation rule in the
  spec, or is it an implied/undocumented requirement? [Spec-Plan Alignment, Gap,
  Spec §FR-010]
  *Resolved: FR-010(d) now explicitly lists "active_buffer ≥ number of entries" as a corrupt condition.*

- [x] CHK022 — The plan uses `PathBuf::to_string_lossy()` for path serialization, which
  can produce replacement characters for non-UTF-8 bytes. Does the spec define behavior
  for file paths containing non-UTF-8 bytes, or does it assume all paths are valid UTF-8?
  [Spec-Plan Alignment, Gap, Spec §FR-009]
  *Resolved: Constitution Principle II mandates UTF-8 everywhere; non-UTF-8 paths are outside spec scope.*

- [x] CHK023 — The plan introduces a `session_warning` parameter that surfaces a
  corrupt-session notice via the TUI status message on startup. Does the spec define
  that a corrupt-session warning MUST be visible in the TUI, or does FR-010 only require
  a log-level warning? [Spec-Plan Alignment, Clarity, Spec §FR-010]
  *Resolved: FR-010 now says "display a status-bar warning to the user" in addition to the log entry.*

- [x] CHK024 — The plan's `session_path()` fallback (`$HOME/.local/state/edit/session.toml`)
  is a plan-level detail. Does the spec require a fallback when `$XDG_STATE_HOME` is
  unset, or does FR-011 only reference `$XDG_STATE_HOME/edit/` without specifying
  fallback behavior? [Spec-Plan Alignment, Gap, Spec §FR-011]
  *Resolved: FR-011 now specifies the fallback path explicitly.*

- [x] CHK025 — Are the plan's `SplitLayoutKind` TOML string values ("none",
  "horizontal", "vertical") the canonical, authoritative values per the spec?
  FR-009 lists them, but the spec should be unambiguously the source of truth, not the
  plan. [Spec-Plan Alignment, Spec §FR-009]
  *Resolved: FR-009 is the spec and is the authoritative source; plan derives from it.*

---

## 5. 🚨 MANDATORY GATE — Clean-Exit vs. Crash-Exit Boundary

*All items in this section must be resolved before the PR merges.*

- [x] CHK026 🚨 — Is the mechanism by which a crash exit is distinguished from a clean
  exit specified at the spec level (e.g., a flag set atomically at the start of the
  clean-exit routine)? FR-002 defines the WHAT but not the HOW — is leaving the HOW
  to the plan intentional and sufficient? [Mandatory Gate, Completeness, Spec §FR-002]
  *Resolved: The mechanism is intentionally a plan-level detail; FR-002 specifying the WHAT is sufficient for a spec.*

- [x] CHK027 🚨 — Is the handling of SIGTERM (system shutdown, `kill` command) defined?
  Does a SIGTERM-triggered exit write the session file (clean exit) or skip it (crash
  exit)? [Mandatory Gate, Gap, Spec §FR-002]
  *Resolved: FR-002 now explicitly states "SIGTERM MUST be treated as a non-clean exit (no session write)".*

- [x] CHK028 🚨 — Is SIGKILL explicitly defined as a non-clean exit so that implementors
  know no session write is attempted? SIGKILL cannot be caught; the spec should confirm
  this is the "crash" case. [Mandatory Gate, Completeness, Spec §FR-002]
  *Resolved: FR-002 now states SIGKILL cannot be intercepted and is a non-clean exit.*

- [x] CHK029 🚨 — Does the spec define whether the atomic tmp → rename write must
  complete within the clean-exit flow, or is it best-effort with a timeout? A slow or
  remote filesystem could stall the exit perceptibly if no bound is set.
  [Mandatory Gate, Completeness, Edge Case]
  *Resolved: SC-007 defines a 500ms timeout; SC-007 specifies the tmp file is cleaned up on next startup if abandoned.*

- [x] CHK030 🚨 — Is the behavior defined if the editor crashes after writing
  `.session.toml.tmp` but before the rename completes? Will the next startup encounter
  the orphaned `.tmp` file, and if so, how is it handled? [Mandatory Gate, Gap,
  Edge Case]
  *Resolved: Edge cases now specifies orphaned .tmp is silently deleted on startup, logged at debug level.*

---

## 6. 🚨 MANDATORY GATE — Security & Robustness

*All items in this section must be resolved before the PR merges.*

- [x] CHK031 🚨 — Does the spec define path-traversal mitigations for file paths loaded
  from `session.toml`? FR-005 says to skip unreadable files but does not specify
  sanitization of `../` sequences, absolute paths outside expected directories, or
  symlink following. Principle VII requires this. [Mandatory Gate, Security,
  Spec §FR-005, Constitution Principle VII]
  *Resolved: FR-005 now requires security::sanitize validation; ../sequences and out-of-tree symlinks treated as missing.*

- [x] CHK032 🚨 — Is "corrupt or invalid TOML" defined to cover all three failure
  classes: malformed TOML syntax, semantically valid TOML with schema violations, and
  TOML with unknown extra fields? Parsers differ on unknown-field handling — does the
  spec require strict or lenient (ignore-unknown) parsing? [Mandatory Gate, Clarity,
  Spec §FR-010]
  *Resolved: FR-010 defines (a) malformed syntax and (b) missing required fields as corrupt; FR-009 specifies lenient mode for unknown fields.*

- [x] CHK033 🚨 — Does the spec explicitly require that parsing `session.toml` MUST
  NOT panic or propagate an unhandled error to the user even for adversarially crafted
  TOML (e.g., deeply nested tables, extremely long strings, huge arrays)? Principle VII
  requires crash prevention; the spec should make this explicit. [Mandatory Gate,
  Security, Spec §FR-010, Constitution Principle VII]
  *Resolved: FR-010 now explicitly states "Parsing MUST NOT panic or propagate an unhandled error regardless of file content".*

- [x] CHK034 🚨 — Is the behavior defined for a session file containing a path that
  resolves to a dangling symlink or a symlink loop? FR-005 covers "no longer exists or
  unreadable" but symlink loops may not be caught by a simple existence check.
  [Mandatory Gate, Security, Gap, Spec §FR-005]
  *Resolved: FR-005 now explicitly covers dangling symlinks and symlink loops as unreadable/skipped.*

- [x] CHK035 🚨 — Does the spec define whether a corrupt session file must be fully
  overwritten (not merely appended to or partially replaced) on the next clean exit?
  A partial overwrite on a write failure could leave a hybrid-corrupt file. [Mandatory
  Gate, Completeness, Spec §FR-010]
  *Resolved: FR-010 now specifies overwrite uses "the same atomic tmp-rename sequence as FR-001".*

---

## 7. Acceptance Criteria Quality

- [x] CHK036 — Is SC-001 ("back at the exact same file, line, and column within
  3 seconds of launching") measurable without ambiguity about whether "3 seconds" is
  wall-clock from process launch or from the first interactive keystroke? [Measurability,
  Spec §SC-001]
  *Resolved: SC-001 now says "within 3 seconds of process invocation (wall-clock)".*

- [x] CHK037 — Is SC-003 ("100% of remaining valid files successfully restored")
  expressed as a testable pass/fail criterion for the CI smoke suite, or does "100%"
  require clarification on what constitutes a test pass? [Measurability, Spec §SC-003]
  *Resolved: "100% of remaining valid files" is a binary, objectively testable criterion.*

- [x] CHK038 — Is SC-005 ("session file absent after crash exit in 100% of cases")
  defined precisely enough to cover SIGKILL and OOM kills, or does it only address
  SIGSEGV/panic? [Measurability, Spec §SC-005]
  *Resolved: SC-005 now enumerates "SIGSEGV, process panic or abort, OOM kill, SIGKILL, SIGTERM, and any involuntary termination".*

---

## 8. Scenario & Edge Case Coverage

- [x] CHK039 — Are requirements defined for restoring a session written by a future
  schema version (version > 1)? The Assumptions section mentions future schema changes
  fall back to "treat as absent" but this is not a functional requirement in the FR
  list. [Coverage, Gap, Spec Assumptions]
  *Resolved: FR-013 (new) formalizes unknown-version handling as a normative requirement.*

- [x] CHK040 — Are requirements defined for `--no-session` combined with a pending
  crash recovery? If both conditions apply, which prompt (if any) is shown?
  [Coverage, Gap, Spec §FR-008]
  *Resolved: Edge cases now specifies --no-session suppresses session restore only; crash-recovery shows normally.*

- [x] CHK041 — Is the "more than 20 buffers" edge case confirmed to fall within SC-002's
  2-second restore bound? SC-002 references "up to 10 files" as the performance
  scenario, but the edge case explicitly states no cap. [Coverage, Consistency,
  Spec §SC-002]
  *Resolved: SC-002 now says "for any number of recorded buffers", removing the 10-file qualification.*

- [x] CHK042 — Are requirements defined for a session file containing `cursor_line = 0`
  or `cursor_col = 0` (below the 1-based minimum stated in FR-009)? These values are
  schema-invalid but FR-010's corrupt-file trigger list does not enumerate them
  explicitly. [Coverage, Edge Case, Spec §FR-009, §FR-010]
  *Resolved: FR-010(e) now explicitly lists "any cursor_line or cursor_col value less than 1" as a corrupt condition.*

---

## 9. Non-Functional Requirements

- [x] CHK043 — Are keyboard accessibility requirements defined for the restore prompt
  dialog beyond Y/N/Enter/Escape? For example, are Tab traversal and screen-reader
  compatibility requirements specified for the TUI overlay? [Non-Functional,
  Accessibility, Gap]
  *Resolved: Edge cases explicitly scopes out Tab/screen-reader for v1.x; Y/y/Enter and N/n/Escape/Ctrl+C are the only required inputs.*

- [x] CHK044 — Are observability requirements defined for session save and load
  operations consistent with the Constitution's logging requirements (e.g., `info`
  level on successful save, `warn` on failure, `debug` for path resolution)?
  [Non-Functional, Consistency, Constitution §Dev Workflow]
  *Resolved: Edge cases now defines info/warn/debug log levels for all session operations.*

- [x] CHK045 — Is a performance requirement defined for the session write time on exit
  to ensure it does not perceptibly delay the editor's close? SC-001 and SC-002 cover
  startup latency; there is no corresponding exit-side criterion. [Non-Functional, Gap]
  *Resolved: SC-007 (new) requires session write completes within 500ms; specifies abandonment behavior on timeout.*

---

## Notes

- Mark items off as resolved: `[x]`
- 🚨 Items in sections 5 and 6 are **blocking** — do not approve the PR with any open
  items in those sections
- Add inline findings as needed: reference spec section, plan line, or tasks task ID
- Items numbered CHK001–CHK045
