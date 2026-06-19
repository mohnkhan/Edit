# Implementation Checklist: Menu Check-State Indicator

**Purpose**: Requirements-quality validation ("unit tests for the English") — confirms that spec,
plan, and tasks are complete, clear, consistent, and measurable before implementation begins.
**Created**: 2026-06-19
**Evaluated**: 2026-06-19
**Feature**: [spec.md](../spec.md) · [plan.md](../plan.md) · [tasks.md](../tasks.md)
**Depth**: Standard | **Audience**: PR reviewer

**Legend**: `[x]` = pass · `[R]` = pass after remediation applied · `[~]` = accepted as by-design gap

---

## Requirement Completeness

- [x] CHK001 — Are rendering requirements defined for ALL three item-state cases: checked (`true`), explicitly unchecked (`false`), and absent (action not in `toggle_states`)? [Completeness, Spec §FR-002, FR-003]
- [x] CHK002 — Are requirements defined for both directions of the `has_checkable` flag — menus that ARE checkable-aware AND menus that are NOT? [Completeness, Spec §FR-006]
- [x] CHK003 — Does the spec define the expected behavior for every menu in the bar (File, Edit, Search, View, Options, Help) when only View-scoped toggle states are provided? [Completeness, Spec §FR-006, FR-007]
- [x] CHK004 — Are requirements documented for the `toggle_states` empty-slice case (`&[]`)? [Completeness, Spec §FR-007, Assumptions]
- [R] CHK005 — Does the spec define what happens when `toggle_states` contains an action that appears in MULTIPLE menus simultaneously? [Completeness, Gap — only View is currently toggled but FR-007 allows any menu] → *Remediated: FR-007 updated with explicit multi-menu statement*
- [x] CHK006 — Are all docs-gate requirements listed (CHANGELOG, STATUS.md, CAPABILITIES.md, ROADMAP.md closure of issue #13)? [Completeness, Plan §Phase E]
- [x] CHK007 — Is there a requirement specifying that `toggle_states` entries must be unique per action (no duplicate actions in the slice)? [Completeness, contracts/menu-widget.md Preconditions] → *Caller precondition documented in contract; no spec MUST needed since this is an internal invariant*

---

## Requirement Clarity

- [~] CHK008 — Is the exact column layout of the dropdown specified? (prefix at `start_col+1`, label at `start_col+3` when checkable, `start_col+1` otherwise — is this derivable from the spec without reading the plan?) [Clarity, Spec §FR-002, FR-003, FR-008] → *By design: spec states behavior, plan states mechanism. Column numbers are plan-level detail; derivable from FR-002/FR-003/FR-008 together.*
- [x] CHK009 — Is the visual form of the checkmark (`✓ ` = U+2713 + U+0020) specified, or is it left to implementor discretion? [Clarity, Spec §FR-002]
- [x] CHK010 — Is "label column alignment" (FR-008) defined precisely enough to write a deterministic test without reading the plan? [Clarity, Spec §FR-008] → *FR-008 states "consistent" column; precise offsets are in plan + contract. Sufficient for test writing.*
- [~] CHK011 — Is the `content_width` expansion (+2 chars for prefix column) stated in the spec, or only derivable from the plan? A reviewer reading only the spec cannot know the column budget. [Clarity, Gap — spec describes behavior, plan describes mechanism] → *By design: content_width is mechanism-level. Spec correctly describes the visual outcome (alignment); plan describes how. This split is intentional per speckit conventions.*
- [R] CHK012 — Is the term "checkable-aware menu" (or equivalent) defined in the spec, or only implied? [Clarity, Spec §FR-001, data-model.md] → *Remediated: "Checkable-aware dropdown" added to spec §Key Entities with full definition*
- [x] CHK013 — Is the spec clear that "non-toggleable items" in a checkable-aware menu still receive a 2-space prefix for alignment (as opposed to no prefix at all)? [Clarity, Spec §FR-003]

---

## Rendering & Visual Requirements Quality

- [R] CHK014 — Is the prefix character's display width (1 terminal cell for U+2713) explicitly stated, or assumed? East-Asian-width varies by codepoint; readers must not assume from visual appearance. [Clarity, plan.md §Research Decision 3] → *Remediated: display-width = 1 terminal cell added to spec §Assumptions*
- [x] CHK015 — Are rendering requirements consistent with the project's existing rendering idioms (e.g., same `.set_char()` API used for `»` in feature 005)? [Consistency, plan.md §Phase B]
- [x] CHK016 — Does the spec define whether the `✓` prefix uses the same foreground/background color as the rest of the dropdown item, or a distinct highlight color? [Completeness, Gap — no color requirement stated] → *Addressed in contracts/menu-widget.md §Behavioral Contract Postcondition 3: same item_style*
- [x] CHK017 — Is "no prefix column" (for non-checkable menus) specified to mean zero change to the existing layout — i.e., no indentation, no space, no change to `content_width`? [Clarity, Spec §FR-006]
- [x] CHK018 — Is the rendering requirement for selected (highlighted) dropdown items with a `✓` prefix defined? (e.g., should `✓` appear in the inverted-video selected-item style?) [Coverage, Gap — no requirement for checked-item rendering in selected state] → *Addressed in contract: "same item_style (normal or selected/inverted-video)" — Principle I compliance*

---

## Acceptance Criteria Quality

- [x] CHK019 — Is SC-001 ("100% of menu opens reflect correct state") objectively measurable via the specified tests (T010/T011), or does it require live-session observation? [Measurability, Spec §SC-001]
- [x] CHK020 — Is SC-002 ("zero stale-state occurrences") verifiable through the design? The spec should confirm that no caching layer exists between `app.soft_wrap` and `toggle_states`. [Measurability, Spec §SC-002, Spec §FR-005]
- [x] CHK021 — Is SC-003 ("no regression in menu navigation") backed by a specific test gate (e.g., `make ci-local` running existing integration/unit tests)? [Measurability, Spec §SC-003, tasks.md T020]
- [x] CHK022 — Is SC-005 ("✓ renders correctly on all CI matrix terminals") tied to a specific CI run or terminal enumeration, rather than left as aspirational? [Measurability, Spec §SC-005, tasks.md T020]
- [x] CHK023 — Can SC-004 ("no perceptible latency — single scan of ≤ 8 items") be objectively verified without instrumentation? Is the O(n) scan bound documented in the spec? [Measurability, Spec §SC-004]

---

## Scenario Coverage

- [x] CHK024 — Are requirements defined for the PRIMARY flow (toggle ON → open menu → `✓` shown)? [Coverage, Spec §US1, SC-001]
- [x] CHK025 — Are requirements defined for the ALTERNATE flow (toggle OFF → open menu → no `✓`)? [Coverage, Spec §US1 acceptance scenario 1]
- [x] CHK026 — Are requirements defined for the IN-SESSION TOGGLE flow (ON → select item to toggle OFF → reopen → `✓` gone)? [Coverage, Spec §US1 acceptance scenario 3]
- [x] CHK027 — Are requirements defined for the CONFIG-PERSISTED RESTART flow (write `soft_wrap = true`, relaunch, open menu immediately)? [Coverage, Spec §US3]
- [x] CHK028 — Are requirements defined for the NON-VIEW-MENU flow (File, Edit, Search — verify no prefix column appears even when soft-wrap is ON)? [Coverage, Spec §FR-006, tasks.md T012]
- [x] CHK029 — Are requirements defined for the SECOND TOGGLE action flow (a hypothetical future toggle in Options or another menu using the same mechanism)? [Coverage, Spec §FR-007, Spec §US2, tasks.md T013b]
- [x] CHK030 — Is there a requirement covering what happens when the user opens a menu while BOTH a checkable item (checked) AND a non-checkable item in the SAME dropdown coexist? (This is the View menu baseline — but is it made explicit?) [Coverage, Spec §FR-003, FR-008]

---

## Edge Case Coverage

- [x] CHK031 — Is the narrow-terminal edge case specified? The spec states "prefix takes priority and label is truncated" — but is the minimum terminal width for showing even the prefix defined? [Completeness, Spec §Edge Cases, tasks.md T007]
- [R] CHK032 — Is "terminal too narrow to show even the first prefix character" addressed? (e.g., width = 1 or 2 at the dropdown position) [Edge Case, Gap — spec says prefix takes priority but doesn't define minimum width] → *Remediated: spec §Edge Cases updated with explicit 0-width guard description*
- [R] CHK033 — Is the zero-items-in-dropdown edge case defined? (A menu with 0 items — `has_checkable` would be false; dropdown renders as empty) [Edge Case, Gap — not addressed in spec or plan] → *Remediated: spec §Edge Cases updated — empty menu → has_checkable=false → existing behavior*
- [x] CHK034 — Is the "live state changes while menu is open" edge case explicitly scoped OUT with a clear rationale? [Coverage, Spec §Edge Cases — yes, present; verifying it's explicit]
- [x] CHK035 — Are requirements defined for duplicate entries in `toggle_states` (same action appears twice with conflicting booleans)? The contract says "undefined behavior" — should the spec say this is a caller error? [Edge Case, contracts/menu-widget.md Preconditions] → *"Caller must not pass duplicates" in contract is sufficient; spec correctly defers to contract for API preconditions*

---

## Non-Functional Requirements

- [x] CHK036 — Is the UTF-8 encoding of `✓` (U+2713, 3 bytes: E2 9C 93) specified as a hard requirement — not negotiable per constitution Principle II? [Non-Functional, Spec §Assumptions, plan.md §Constitution Check]
- [~] CHK037 — Is the zero-heap-allocation rendering requirement stated explicitly in the spec, or only in the plan? [Non-Functional, plan.md §Performance Goals] → *By design: heap-allocation is a performance/mechanism concern, correctly placed in plan §Performance Goals. Spec captures the O(n) scan bound in SC-004. No change needed.*
- [x] CHK038 — Are cross-platform rendering requirements specified? (Linux x86_64, aarch64, FreeBSD, macOS — all declared in-scope by constitution Principle III) [Non-Functional, plan.md §Technical Context]
- [x] CHK039 — Is there a requirement that the `Action` type used in `toggle_states` must implement `PartialEq`? This is a correctness precondition for lookup. [Non-Functional, contracts/menu-widget.md, data-model.md]

---

## Constitution Alignment

- [R] CHK040 — Does the spec explicitly label the `✓` prefix as a non-DOS extension (like "Soft Wrap (ext)") so Principle I (DOS-Faithful UI) is documented as preserved? [Constitution §I, Spec §FR-002 — note: not labeled "(ext)" in current spec] → *Remediated: non-DOS extension note added inline to FR-002*
- [x] CHK041 — Is the UTF-8 requirement for U+2713 tied to constitution Principle II's non-negotiable mandate (not just a "preference")? [Constitution §II, Spec §Assumptions]
- [x] CHK042 — Does the plan's constitution check accurately reflect Principle V (test-gated merges): are all 7 unit tests and the `make ci-local` gate listed as required before merge? [Constitution §V, plan.md §Constitution Check] → *Remediated (I1): plan updated with full 7-test smoke-substitute rationale*
- [x] CHK043 — Is Principle VI (YAGNI) compliance verifiable? Specifically: does the spec avoid requiring any speculative toggle-state features beyond what the current single toggleable item needs? [Constitution §VI, Spec §FR-007 — general mechanism is justified by avoiding future tech-debt, not speculation]
- [x] CHK044 — Is there a security self-certification statement confirming no user-controlled data reaches the prefix rendering path? [Constitution §VII, plan.md §Security self-certification]

---

## Dependencies & Assumptions Quality

- [x] CHK045 — Is the dependency on `Action` deriving `PartialEq` documented as a precondition, and is it verifiable without reading the Rust source? [Dependency, contracts/menu-widget.md]
- [x] CHK046 — Is the assumption that "terminals in the CI matrix support U+2713" traceable to the CI matrix declaration in the constitution or STATUS.md? [Assumption, Spec §Assumptions, plan.md §Technical Context]
- [x] CHK047 — Is the assumption that `App::soft_wrap` is the ONLY toggleable state in the current codebase documented and agreed? [Assumption, Spec §Assumptions, tasks.md T008] → *Remediated (A1): merge-time verify note added to spec §Assumptions*
- [x] CHK048 — Is the dependency on `ratatui`'s `Buffer::get_mut().set_char()` rendering the correct cell (1 terminal column for U+2713) documented, or assumed? [Dependency, plan.md §Research Decision 3]
- [x] CHK049 — Are the docs-gate dependencies (CHANGELOG, STATUS, CAPABILITIES, ROADMAP) listed as blocking requirements for merge, not optional post-merge cleanup? [Dependency, plan.md §Phase E, CLAUDE.md §Docs gate]

---

## Consistency & Traceability

- [x] CHK050 — Are FR-001 through FR-008 all traceable to at least one task in tasks.md? [Traceability]
- [x] CHK051 — Are all three user stories (US1/US2/US3) traceable to specific test tasks? [Traceability]
- [x] CHK052 — Is there consistency in terminology: "checked state," "toggle state," and "check-state indicator" are used interchangeably — is one canonical term established? [Consistency, Spec §FR-001, plan.md §Summary] → *Remediated (D1): glossary added to contracts/menu-widget.md establishing canonical term mapping*
- [x] CHK053 — Are the task test counts consistent across plan.md ("7 unit tests"), tasks.md (T010–T015 + T013b = 7 functions), and contracts/menu-widget.md? [Consistency, contracts/menu-widget.md §Unit Test Expectations] → *All three agree: 7 unit tests. Contract table already listed all 7 rows.*
- [x] CHK054 — Is the `content_width` expansion rule (+2 for checkable menus) consistent between plan.md §Phase B, data-model.md §DropdownRenderContext, and tasks.md T006? [Consistency]

---

## Summary

| Result | Count |
|--------|-------|
| `[x]` Pass | 42 |
| `[R]` Pass after remediation | 9 |
| `[~]` Accepted by-design gap | 3 |
| `[ ]` Unresolved | **0** |

**All 54 items resolved. Ready for `/speckit-implement`.**
