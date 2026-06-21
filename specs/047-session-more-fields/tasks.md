# Tasks: Restore Scroll, Selection & Encoding in Session (#83)
**Branch**: 047-session-more-fields. Behavior-preserving for non-session users; old files load (FR-005).
## Setup
- [ ] T001 Baseline `make check` (note 1279/0/11).
## Foundational
- [ ] T002 Add `encoding_to_str(EncodingId) -> &'static str` in `src/encoding/mod.rs` (round-trips via `encoding_from_str`); unit test the round-trip for all variants.
- [ ] T003 `src/session/mod.rs`: add `SelectionEntry` (Serialize/Deserialize/PartialEq/Clone/Debug) and the 4 `#[serde(default)]` fields on `BufferEntry`.
## US1 (P1)
- [ ] T004 `build_session_data` (`fileops.rs`): write scroll_line/col, selection (from `buf.selection`, 1-based; None if absent), encoding (`encoding_to_str`).
- [ ] T005 `do_restore_session` (`fileops.rs`): open each buffer in the recorded encoding (else default); after cursor seek, apply clamped scroll and clamped selection (drop degenerate). Checked access (046) — no panic.
## US2 (P1)
- [ ] T006 Confirm legacy load: a v2 file without the new keys deserializes to defaults (serde(default)); add a legacy test.
## Tests
- [ ] T007 Round-trip test (session/mod.rs): BufferEntry with scroll/selection/encoding survives save+load.
- [ ] T008 App restore test (tests/integration/session.rs): real files, a SessionData with scroll/selection/encoding → restore → assert applied + clamped on a short file.
- [ ] T008a Update existing 003/045 session literals for the additive fields (compiler-enumerated).
## Ship
- [ ] T009 `make ci-local`; count == baseline + new tests.
- [ ] T010 Docs: CHANGELOG + STATUS (no CAPABILITIES change — session already documented).
- [ ] T011 PR -> master (`feat(047): restore scroll/selection/encoding`), Closes #83, merge.
