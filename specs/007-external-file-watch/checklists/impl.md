# Implementation Readiness Checklist: External File Modification Detection

**Purpose**: Validate that spec, plan, and tasks are clear, complete, and consistent enough to begin implementation without ambiguity
**Created**: 2026-06-19
**Feature**: [spec.md](../spec.md)

---

## Requirement Completeness

- [X] CHK001 Are detection requirements defined for all file-event kinds expected in scope (Modified, Deleted)? [Spec §FR-001, FR-006 — both present]
- [X] CHK002 Are both user-response paths (reload and keep) specified with their resulting buffer/dirty-state consequences? [Spec §FR-004, FR-005 — both present]
- [X] CHK003 Is an unsaved-changes variant of the reload prompt specified for when the buffer has pending edits? [Spec §FR-003, US2 — present]
- [X] CHK004 Is the self-write suppression requirement documented with a measurable grace window? [Spec §FR-007, Assumptions — 2-second grace window specified after remediation H1]
- [X] CHK005 Is the debounce requirement documented with a measurable time window? [Spec §FR-008 — "1-second window" present]
- [X] CHK006 Is the `--no-watch` escape hatch scoped clearly (suppresses watching only, not saving)? [Spec §US4 Acceptance Scenario 2 — explicitly states "the flag only suppresses watching, not saving"]
- [X] CHK007 Are requirements defined for unnamed buffers (no backing file) regarding watching? [Spec §Edge Cases — "No watch is registered; unnamed buffers are unaffected"]
- [X] CHK008 Are requirements defined for what happens when the watcher fails to initialize at startup? [Tasks §T015 — "on failure, log warning and set file_watcher = None"; covered by Observability requirement T045]
- [X] CHK009 Is the FR-011 uniqueness requirement (one watch per file path, not per buffer) complete enough to implement? [Spec §FR-011, Contracts §watch_path edge cases — parent-dir refcount mechanism documented]
- [X] CHK010 Are requirements specified for what happens when the config file sets `no_watch = true` (TOML path, not just CLI flag)? [Spec §Assumptions — "can add `no_watch = true` to `config.toml`"; Tasks §T012 covers TOML deserialization]

## Requirement Clarity

- [X] CHK011 Is "external process" defined clearly enough to distinguish from editor self-writes? [Spec §FR-007, Assumptions — self-write suppression via 2-second grace window is the mechanism; definition is behavioral, not definitional, which is testable]
- [X] CHK012 Is "non-blocking notification" (for deletions) distinguished clearly from "modal dialog" (for modifications)? [Spec §FR-006 explicitly says "non-blocking notification (distinct from the reload prompt)"; US3 description says "non-modal"]
- [X] CHK013 Is "coalesced" in FR-008 defined precisely enough (debounce window duration specified)? [Spec §FR-008 says "1-second window"; plan.md says `DEBOUNCE_SECS = Duration::from_secs(1)` — consistent]
- [X] CHK014 Is "byte-for-byte identical" in SC-002 qualified with the expected encoding caveat? [Spec §SC-002 says "modulo encoding transcoding" — qualified]
- [X] CHK015 Is it specified what "reload from disk" means for the undo history (cleared or preserved)? [FR-004 now includes "undo history MUST be cleared" as a sub-requirement — fixed by remediation CHK015]
- [X] CHK016 Is the status-bar indicator change required by US1 ("file indicator in the status bar changes") defined in the Functional Requirements? [Spec §US1 story text mentions it; FR-002 and FR-005 define the "modified" indicator state change — covered]
- [X] CHK017 Is the distinction between "5 seconds on inotify" and "10 seconds on poll" in SC-001 specific enough that different test environments can select the correct threshold? [Spec §SC-001 — "inotify-capable Linux systems" / "polling-fallback systems" — both qualified; Tests can detect via notify::WatcherKind or use the higher bound]

## Requirement Consistency

- [X] CHK018 Does the grace window value in spec.md match the plan.md constant `SELF_WRITE_GRACE`? [Spec §Assumptions now says "2-second grace window"; plan.md says `Duration::from_secs(2)` — consistent after remediation H1]
- [X] CHK019 Does the debounce window value in spec.md match the plan.md constant `DEBOUNCE_SECS`? [Spec §FR-008 says "1-second window"; plan.md says `Duration::from_secs(1)` — consistent]
- [X] CHK020 Are the user-story acceptance scenarios in spec.md consistent with the behavioral contracts in `contracts/file-watcher.md`? [Contracts document Y/Enter → reload, N/Esc → dirty-flag; Spec §US1 Acceptance Scenarios say the same]
- [X] CHK021 Is the reload-clears-undo-history design decision consistent between research.md and spec.md? [research.md Decision 7 says "Reload clears undo history"; spec.md §Assumptions says the same — consistent]
- [X] CHK022 Does the --no-watch scope (suppress watching only, not saving) appear consistently in spec.md, plan.md, and tasks? [Spec §US4 AS2 explicit; Tasks §T035/T036 test watcher=None not save disabled; plan.md §Phase B covers flag — consistent]

## Acceptance Criteria Quality

- [X] CHK023 Is SC-001 (detection within 5s/10s) measurable by an automated test? [Tasks §T024 polls for up to 3s — within 5s bound; T038 (debounce) uses timing assertions — yes, measurable]
- [X] CHK024 Is SC-003 (zero self-write false positives) verifiable by an automated test? [Tasks §T037 (`test_self_write_suppressed_no_prompt`) directly tests this criterion]
- [X] CHK025 Is SC-006 (10 writes → 1 prompt) verifiable by an automated test? [Tasks §T038 (`test_debounce_10_writes_1_event`) directly tests this criterion]
- [X] CHK026 Are the success criteria in SC-005 (startup ≤2s, keystroke ≤50ms) already enforced by the existing CI gate (`make perf-check`)? [Tasks §T043 runs `make ci-local` which includes `perf-check` — yes, inherited]
- [X] CHK027 Is SC-002 ("buffer byte-for-byte identical to on-disk content") verified by test T026 with a specific assertion? [T026 now specifies `assert_eq!(buffer.as_bytes(), fs::read(&path).unwrap())` byte-level comparison and `undo_history.is_empty()` — fixed by remediation CHK027]

## Scenario Coverage

- [X] CHK028 Does the task list cover all four user story acceptance scenarios for US1? [US1 AS1: T024 (detection); AS2: T026 (reload); AS3: T027 (dismiss)] — 3 scenarios covered; AS1 timing is covered by T024's 3s poll]
- [X] CHK029 Does the task list cover all three user story acceptance scenarios for US2? [US2 AS1: T029 (dialog shows warning); AS2: T030 (reload discards edits); AS3: T027 (decline preserves edits + dirty=true)] — covered]
- [X] CHK030 Does the task list cover all three user story acceptance scenarios for US3? [US3 AS1: T033 (notice not dialog); AS2: T034a (save recreates); AS3: T034b (close-without-save prompts)] — covered after remediation M3]
- [X] CHK031 Does the task list cover both user story acceptance scenarios for US4? [US4 AS1: T036 (no events with --no-watch); AS2: T035 (save still works — watcher=None only)] — covered]
- [X] CHK032 Are atomic-rename external writes (e.g., `mv tmp original`) covered as a test scenario? [Tasks §T025 (`test_atomic_rename_detected`) — yes]

## Edge Case Coverage

- [X] CHK033 Is the "multiple rapid external writes (10 in 1 second)" edge case from the spec covered by an automated test? [Tasks §T038 — yes]
- [X] CHK034 Is the "same file open in two split panes" edge case (FR-011) covered by a test that verifies only one OS-level inotify watch is registered? [T046 (`test_same_file_two_buffers_single_watch`) added to integration suite — real inotify, two buffer registrations, asserts exactly 1 WatchEvent — fixed by remediation CHK034]
- [X] CHK035 Is the "editor auto-save triggers watcher" edge case (FR-007) covered? [Tasks §T037 (`test_self_write_suppressed_no_prompt`) — yes, integration test]
- [X] CHK036 Is the "binary file overwrites text file on reload" edge case covered? [Tasks §T026b (`test_reload_binary_shows_encoding_error`) — added in remediation M1]
- [X] CHK037 Is the "/proc or /sys path silently skipped" edge case addressed in contracts? [Contracts §watch_path edge cases — "Paths under pseudo-filesystems silently accepted; no events will arrive" — documented]
- [X] CHK038 Is the "watcher initialization failure" edge case (e.g., ENOSPC inotify limit) documented with required user-visible behavior? [T015 now specifies: on failure, set `watcher_notice = Some("⚠ File watching unavailable…")` so user sees a status-bar notice — fixed by remediation CHK038]

## Non-Functional Requirements Quality

- [X] CHK039 Is FR-012 (no perceptible startup or latency impact) quantified with measurable thresholds? [Constitution §Performance baselines: startup ≤2s, keystroke ≤50ms — inherited thresholds; FR-012 references "constitution's performance baselines"]
- [X] CHK040 Is the "no perceptible impact" claim for FR-012 testable by the existing CI perf-check gate? [Tasks §T043 — yes, `make perf-check` includes startup and keystroke latency checks]
- [X] CHK041 Are observability requirements for the watcher module specified (log levels, what gets logged)? [Tasks §T045 — log watcher init at info/warn, suppressed events at debug, emitted events at debug, notify errors at warn — after remediation H3]
- [X] CHK042 Is the encoding safety requirement (FR-004: reload via `Buffer::open()`, no raw-byte bypass) testable independently of the reload dialog? [Tasks §T026b covers the encoding error case; T021 specifies `Buffer::open(path)` which contains the full validation pipeline]
- [X] CHK043 Is security hardening for Principle VII documented for this feature's three touch points (file I/O, CLI, rendering)? [Tasks §T044 now includes Principle VII self-certification checklist for PR — after remediation M4]

## Dependencies & Assumptions Quality

- [X] CHK044 Is the dependency on `notify = "6"` (crate version) pinned precisely enough for reproducible builds? [Tasks §T002 says `notify = "6"` which pins the major version; Cargo.lock will pin exact version — acceptable for Rust projects]
- [X] CHK045 Is the assumption that `notify::recommended_watcher` automatically selects PollWatcher as fallback documented and tested? [Spec §Assumptions: "polling fallback (configurable interval, default 5 seconds)"; Tasks §Notes: "may fall back to PollWatcher automatically via notify::recommended_watcher" — documented; not explicitly tested (acceptable, as this is crate behavior)]
- [X] CHK046 Is the assumption that `Buffer::open()` rejects binary/non-UTF-8 files documented as a pre-existing behavior that F007 depends on? [T047 added: verify existing `Buffer::open()` binary-rejection test exists before implementing T026b; if absent, add it as prerequisite — fixed by remediation CHK046]
- [X] CHK047 Is the TOML config path for `no_watch = true` (not just the CLI flag) validated as working by tests? [Tasks §T012 says "ensure TOML deserialization defaults to false when key is absent" — the TOML path is covered by the schema test]

## Notes

- Items marked `[X]` indicate the requirement is adequately specified — no spec update needed
- Items marked `[ ]` indicate gaps or clarifications needed before implementation is risk-free
- **All 47 items resolved** — all open items (CHK015, CHK027, CHK034, CHK038, CHK046) addressed via concrete remediations
- Implementation may begin after `/speckit-analyze` + `/speckit-checklist` cycles are complete

### Post-Remediation Status

Following `/speckit-analyze` remediations applied before this checklist was generated:
- H1 grace window: fixed in spec.md (now "2-second grace window")
- H2 FR-011 test: added `test_two_buffers_same_file_single_watch` to T011
- H3 observability: T045 added for structured logging
- M1 binary reload: T026b added `test_reload_binary_shows_encoding_error`
- M2 plan.md: `statusbar.rs` added to file list
- M3 US3 AS3: T034b added `test_deleted_file_close_without_save_prompts`
- M4 security gate: Principle VII self-certification added to T044
- L1 T031→T016 dependency: added to Sequential section
- L2 T032 ambiguity: notice-clear location clarified (clears in mod.rs Ui::render, not statusbar.rs)
