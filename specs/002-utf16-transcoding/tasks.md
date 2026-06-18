# Tasks: UTF-16 Transcoding Support

**Feature**: 002-utf16-transcoding | **Branch**: `002-utf16-transcoding`

**Input**: Design documents from `specs/002-utf16-transcoding/`

**Prerequisites**: plan.md ✅ spec.md ✅ research.md ✅ data-model.md ✅ contracts/ ✅

**TDD Note**: Tests are REQUIRED by Constitution §V — write each test before its implementing task.
Verify tests FAIL before implementing.

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no shared state dependencies)
- **[Story]**: Maps to user stories in spec.md (US1 P1, US2 P1, US3 P2, US4 P3)
- US4 (save-as encoding conversion) is **DEFERRED** — see Phase 6

---

## Phase 1: Setup (Branch + Test Fixtures)

**Purpose**: Create the working branch and binary test fixtures required by every later phase.
All fixture tasks are independent of each other and of the source changes — do them first so
integration tests can reference real binary files.

- [ ] T001 Create git branch `002-utf16-transcoding` from `origin/master` and push tracking branch
- [ ] T002 [P] Generate `tests/fixtures/utf16le_bom.bin` — UTF-16 LE with BOM containing ASCII + non-ASCII ("Hello, UTF-16 LE! こんにちは") via `python3 -c "open('tests/fixtures/utf16le_bom.bin','wb').write('Hello, UTF-16 LE! こんにちは'.encode('utf-16'))"`
- [ ] T003 [P] Generate `tests/fixtures/utf16be_bom.bin` — UTF-16 BE with BOM via `python3 -c "import codecs; open('tests/fixtures/utf16be_bom.bin','wb').write(codecs.BOM_UTF16_BE + 'Hello, UTF-16 BE! こんにちは'.encode('utf-16-be'))"`
- [ ] T004 [P] Generate `tests/fixtures/utf16le_nobom.bin` — UTF-16 LE without BOM via `python3 -c "open('tests/fixtures/utf16le_nobom.bin','wb').write('BOM-less LE file'.encode('utf-16-le'))"`
- [ ] T005 [P] Generate `tests/fixtures/utf16le_surrogate.bin` — UTF-16 LE with BOM + supplementary chars (emoji 🌍 U+1F30D, which encodes as surrogate pair) via `python3 -c "open('tests/fixtures/utf16le_surrogate.bin','wb').write('Emoji: 🌍 U+1F30D'.encode('utf-16'))"`
- [ ] T006 [P] Verify all four fixtures: `hexdump -C tests/fixtures/utf16le_bom.bin | head -2` must show `ff fe` prefix; `tests/fixtures/utf16be_bom.bin` must show `fe ff` prefix; `tests/fixtures/utf16le_nobom.bin` must NOT start with `ff fe`; `tests/fixtures/utf16le_surrogate.bin` must contain `d8`/`dc` surrogate bytes

---

## Phase 2: Foundational (EncodingId Extension — Blocks All Story Phases)

**Purpose**: Extend the `EncodingId` enum with `Utf16Le` and `Utf16Be` variants, update the
registry, and add stub match arms so the project compiles before any story implementation begins.

**⚠️ CRITICAL**: Every later phase depends on this phase completing successfully.
`cargo build` must succeed (with stub arms) before Phase 3 work starts.

- [ ] T007 Add `Utf16Le` and `Utf16Be` variants to `EncodingId` enum in `src/encoding/detect.rs` (after `Windows1252` variant; no logic change yet — enum extension only)
- [ ] T008 [P] Add two new `EncodingProfile` entries to `ENCODING_REGISTRY` in `src/encoding/detect.rs`: `{ id: EncodingId::Utf16Le, name: "UTF-16 LE", bom: Some(&[0xFF, 0xFE]) }` and `{ id: EncodingId::Utf16Be, name: "UTF-16 BE", bom: Some(&[0xFE, 0xFF]) }` (depends on T007)
- [ ] T009 [P] Add stub `EncodingId::Utf16Le | EncodingId::Utf16Be => unimplemented!()` arms to all exhaustive `match encoding` blocks in `src/encoding/transcode.rs` to make the file compile (depends on T007)
- [ ] T010 [P] Add stub `"utf-16-le" | "utf16-le" | "utf16le" | "utf-16-be" | "utf16-be" | "utf16be" | "utf-16" => todo!()` branch to `encoding_from_str()` in `src/encoding/mod.rs` (depends on T007)
- [ ] T011 Run `cargo build` and confirm zero errors (stubs may warn about `unimplemented!`/`todo!` — that is expected); fix any compile errors before proceeding (depends on T007–T010)

**Checkpoint**: `cargo build` passes → user story implementation can begin

---

## Phase 3: User Story 1 — Open a UTF-16 File Without Garbled Display (Priority: P1) 🎯 MVP

**Goal**: BOM detection returns the correct `EncodingId` variant; `decode()` correctly
transcodes UTF-16 LE/BE bytes to UTF-8 so the buffer contains readable text; status bar
shows `UTF-16 LE` / `UTF-16 BE`.

**Independent Test**: `cargo test -- encoding::detect encoding::transcode` — all new
UTF-16 tests pass; existing tests unchanged.

### Tests for User Story 1 (TDD — write FIRST, verify FAIL before implementing)

- [ ] T012 [P] [US1] Write unit tests for `detect_encoding()` BOM handling in `src/encoding/detect.rs` `#[cfg(test)]` block — cover: UTF-16 LE BOM (`\xFF\xFE`) → `EncodingId::Utf16Le`; UTF-16 BE BOM (`\xFE\xFF`) → `EncodingId::Utf16Be`; UTF-8 still → `EncodingId::Utf8`; empty bytes → `EncodingId::Utf8` (4 new test cases)
- [ ] T013 [P] [US1] Write unit tests for `decode(bytes, Utf16Le)` and `decode(bytes, Utf16Be)` in `src/encoding/transcode.rs` `#[cfg(test)]` block — cover: UTF-16 LE bytes with BOM → correct UTF-8 string with BOM stripped; UTF-16 BE bytes with BOM → correct UTF-8 string with BOM stripped; empty `&[]` → `Ok("")`; odd-byte-length input → `Err(TranscodeError::InvalidUtf8)`; surrogate pair input (from `tests/fixtures/utf16le_surrogate.bin`) → correct UTF-8 (6 new test cases)
- [ ] T014 [US1] Run `cargo test -- encoding` and confirm T012/T013 tests FAIL with `unimplemented!` or similar (depends on T012, T013; must FAIL to validate TDD setup)

### Implementation for User Story 1

- [ ] T015 [US1] Fix `detect_encoding()` in `src/encoding/detect.rs` — replace lines that currently return `EncodingId::Utf8` for UTF-16 BOMs with: `if bytes.starts_with(UTF16_LE_BOM) { return EncodingId::Utf16Le; }` and `if bytes.starts_with(UTF16_BE_BOM) { return EncodingId::Utf16Be; }` (depends on T014)
- [ ] T016 [US1] Replace stub arm for `Utf16Le` in `decode()` in `src/encoding/transcode.rs` with: `EncodingId::Utf16Le => { let (cow, _, _) = encoding_rs::UTF_16_LE.decode(bytes); Ok(cow.into_owned()) }` — `encoding_rs::UTF_16_LE.decode()` strips the BOM automatically (depends on T014)
- [ ] T017 [US1] Replace stub arm for `Utf16Be` in `decode()` in `src/encoding/transcode.rs` with: `EncodingId::Utf16Be => { let (cow, _, _) = encoding_rs::UTF_16_BE.decode(bytes); Ok(cow.into_owned()) }` (depends on T014)
- [ ] T018 [US1] Add odd-byte-length guard before the `encoding_rs` call in both UTF-16 decode arms in `src/encoding/transcode.rs`: `if bytes.len() % 2 != 0 { return Err(TranscodeError::InvalidUtf8); }` (depends on T016, T017)
- [ ] T019 [US1] Verify status bar display by running `cargo test` and checking that opening a UTF-16 LE buffer shows `"UTF-16 LE"` — this should work automatically via `ENCODING_REGISTRY` names set in T008; if a test is missing, add one that checks `EncodingProfile.name` for `Utf16Le` and `Utf16Be` in `src/encoding/detect.rs` tests
- [ ] T020 [US1] Run `cargo test -- encoding` and confirm all T012/T013 tests now PASS (depends on T015–T018)

**Checkpoint**: US1 fully functional — open UTF-16 LE/BE files, correct display, status bar correct

---

## Phase 4: User Story 2 — Save a File Preserving Its Original Encoding (Priority: P1)

**Goal**: `encode()` produces UTF-16 LE/BE bytes with BOM prepended, enabling byte-identical
round-trips when no edits are made.

**Independent Test**: `cargo test -- encoding_roundtrip` — UTF-16 LE round-trip produces
identical bytes verified by byte-level comparison.

### Tests for User Story 2 (TDD — write FIRST, verify FAIL before implementing)

- [ ] T021 [P] [US2] Write unit tests for `encode(s, Utf16Le)` in `src/encoding/transcode.rs` `#[cfg(test)]` block — cover: basic ASCII string produces `[0xFF, 0xFE]` + UTF-16 LE bytes; empty string `""` → `Ok(vec![0xFF, 0xFE])` (BOM only); string with non-BMP char produces correct surrogate-pair bytes (3 new test cases)
- [ ] T022 [P] [US2] Write unit tests for `encode(s, Utf16Be)` in `src/encoding/transcode.rs` `#[cfg(test)]` block — cover: basic ASCII string produces `[0xFE, 0xFF]` + UTF-16 BE bytes; empty string → `Ok(vec![0xFE, 0xFF])` (2 new test cases)
- [ ] T023 [US2] Write failing integration round-trip tests in `tests/integration/encoding_roundtrip.rs` — cover: open `tests/fixtures/utf16le_bom.bin`, encode result, compare byte-by-byte to original (pure round-trip); open `tests/fixtures/utf16be_bom.bin`, round-trip; open `tests/fixtures/utf16le_surrogate.bin`, round-trip (3 new integration test cases)
- [ ] T024 [US2] Run `cargo test -- encoding::transcode` and confirm T021/T022 tests FAIL; run `cargo test --test encoding_roundtrip` and confirm T023 tests FAIL (depends on T021–T023)

### Implementation for User Story 2

- [ ] T025 [US2] Replace stub arm for `Utf16Le` in `encode()` in `src/encoding/transcode.rs` with: `EncodingId::Utf16Le => { let (cow, _, _) = encoding_rs::UTF_16_LE.encode(s); let mut out = vec![0xFF, 0xFE]; out.extend_from_slice(&cow); Ok(out) }` (depends on T024)
- [ ] T026 [US2] Replace stub arm for `Utf16Be` in `encode()` in `src/encoding/transcode.rs` with: `EncodingId::Utf16Be => { let (cow, _, _) = encoding_rs::UTF_16_BE.encode(s); let mut out = vec![0xFE, 0xFF]; out.extend_from_slice(&cow); Ok(out) }` (depends on T024)
- [ ] T027 [US2] Run `cargo test -- encoding::transcode encoding_roundtrip` and confirm all T021–T023 tests now PASS and no pre-existing tests regress (depends on T025, T026)

**Checkpoint**: US1 + US2 both functional — open UTF-16 files, edit, save, byte-identical round-trip verified

---

## Phase 5: User Story 3 — Force UTF-16 Encoding via CLI Flag (Priority: P2)

**Goal**: `encoding_from_str()` accepts all specified UTF-16 aliases; `--encoding utf-16-le`
and `--encoding utf-16-be` CLI flags are correctly parsed and applied on file open.

**Independent Test**: `cargo test -- encoding::mod` — all alias tests pass.

### Tests for User Story 3 (TDD — write FIRST, verify FAIL before implementing)

- [ ] T028 [US3] Write unit tests for `encoding_from_str()` UTF-16 aliases in `src/encoding/mod.rs` `#[cfg(test)]` block — cover: `"utf-16-le"` → `Utf16Le`; `"utf16-le"` → `Utf16Le`; `"utf16le"` → `Utf16Le`; `"UTF-16-LE"` (uppercase) → `Utf16Le`; `"utf-16-be"` → `Utf16Be`; `"utf16be"` → `Utf16Be`; `"utf-16"` (no endian) → `Utf16Le` (LE default); invalid alias `"utf-16-xx"` → error/None (8 new test cases)
- [ ] T029 [US3] Run `cargo test -- encoding::mod` and confirm T028 tests FAIL (depends on T028)

### Implementation for User Story 3

- [ ] T030 [US3] Replace stub branch in `encoding_from_str()` in `src/encoding/mod.rs` with full alias table (case-insensitive match): `"utf-16-le" | "utf16-le" | "utf16le" | "utf-16 le" | "utf16_le" => Some(EncodingId::Utf16Le)`, `"utf-16-be" | "utf16-be" | "utf16be" | "utf-16 be" | "utf16_be" => Some(EncodingId::Utf16Be)`, `"utf-16" => Some(EncodingId::Utf16Le)` (depends on T029)
- [ ] T031 [US3] Run `cargo test -- encoding::mod` and confirm all T028 tests now PASS (depends on T030)

**Checkpoint**: US1 + US2 + US3 all functional — all three P1/P2 user stories complete

---

## Phase 6: User Story 4 — Convert a File to UTF-16 via Save-As (Priority: P3) — DEFERRED

**Status**: DEFERRED — US4 requires save-as dialog encoding-selection UI that is not in the
current plan scope. The encoding engine (encode/decode) from earlier phases already provides
the underlying mechanism; only the UI layer is missing.

**Required before PR merges**:

- [ ] T032 [US4] Create GitHub issue for US4 save-as encoding-selection UI: title "US4: Save-As encoding selection (UTF-16)", body includes: problem (save-as dialog has no encoding picker), why deferred (UI changes out of scope for 002), suggested approach (add encoding dropdown to save-as dialog in src/ui/dialogs/saveas.rs), effort (M), label `follow-up`; note parent issue #5
- [ ] T033 [US4] Add ROADMAP.md row for US4: `| US4 | Save-As encoding selection | GH#<new issue> | P3 | not started |` referencing the issue created in T032

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, linting, full test suite validation, and PR creation.

- [ ] T034 [P] Update `CHANGELOG.md` — add feature 002 entry under `[Unreleased]`: "Add UTF-16 LE/BE transcoding support: BOM detection, decode, encode, round-trip, CLI --encoding flag aliases"
- [ ] T035 [P] Update `docs/STATUS.md` — mark UTF-16 transcoding as Implemented with link to specs/002-utf16-transcoding/
- [ ] T036 [P] Update `docs/CAPABILITIES.md` — add UTF-16 LE and UTF-16 BE to supported encodings table and document `--encoding utf-16-le` / `--encoding utf-16-be` CLI flags
- [ ] T037 Run `cargo fmt` on all changed files: `src/encoding/detect.rs`, `src/encoding/transcode.rs`, `src/encoding/mod.rs`; confirm `cargo fmt --check` exits 0
- [ ] T038 Run `cargo clippy -- -D warnings` and fix any warnings in changed files (depends on T037)
- [ ] T039 Run full `cargo test` suite — verify SC-003 (all 196+ pre-existing tests pass) and SC-004 (≥ 15 new UTF-16 tests visible); capture total test count for PR description (depends on T037, T038)
- [ ] T040 Run `make smoke` — confirm all smoke tests still pass; no regressions (depends on T039)
- [ ] T041 Run quickstart.md validation scenarios: generate fixtures with `python3`, open with `./target/debug/edit`, verify status bar shows `UTF-16 LE`, verify hexdump round-trip (depends on T039)
- [ ] T042 Open PR for `002-utf16-transcoding` targeting `master` — title: "002: Add UTF-16 LE/BE transcoding support"; include self-certification for Constitution §VII (security: no new path traversal surface, no terminal escape injection from decoded content)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately; T002–T006 are parallel
- **Phase 2 (Foundational)**: Depends on Phase 1 completion; T008–T010 are parallel after T007; T011 is the compile checkpoint
- **Phase 3 (US1)**: Depends on Phase 2 (T011 compile checkpoint); T012–T013 are parallel
- **Phase 4 (US2)**: Depends on Phase 3 completion (decode must work for round-trip to make sense); T021–T022 are parallel
- **Phase 5 (US3)**: Depends on Phase 2 only (alias parsing is independent of decode/encode); can run in parallel with Phases 3–4 after Phase 2
- **Phase 6 (US4 deferral)**: T032–T033 are independent; can run anytime after Phase 2
- **Phase 7 (Polish)**: Depends on Phases 3–5 completion; T034–T036 are parallel

### User Story Dependencies

- **US1 (P1)**: Requires Foundational (Phase 2) — no dependency on other stories
- **US2 (P1)**: Requires US1 decode logic (round-trip test decodes then re-encodes)
- **US3 (P2)**: Requires Foundational (Phase 2) only — alias parsing is independent of decode/encode
- **US4 (P3)**: DEFERRED — encoding engine from US1/US2 is sufficient once save-as UI is added

### Parallel Opportunities

- **Phase 1**: T002–T006 run in parallel (independent fixture files)
- **Phase 2**: T008–T010 run in parallel after T007 (different files: transcode.rs vs mod.rs vs detect.rs)
- **Phase 3**: T012–T013 (tests) run in parallel; T016–T017 (decode arms) run in parallel
- **Phase 4**: T021–T022 (tests) run in parallel; T025–T026 (encode arms) run in parallel
- **Phase 5**: T028–T029 are single-story; can run concurrently with Phase 4 after Phase 2
- **Phase 7**: T034–T036 (doc updates) run in parallel

---

## Parallel Execution Example: Phase 2

```
# After T007 (enum extension), launch simultaneously:
Task A: Add ENCODING_REGISTRY entries (T008) in src/encoding/detect.rs
Task B: Add stub arms in transcode.rs (T009) in src/encoding/transcode.rs
Task C: Add stub aliases in mod.rs (T010) in src/encoding/mod.rs
# Then: T011 cargo build checkpoint (sequential — waits for A, B, C)
```

## Parallel Execution Example: Phase 3 (TDD)

```
# Write tests in parallel:
Task A: detect_encoding() tests (T012) in src/encoding/detect.rs
Task B: decode() tests (T013) in src/encoding/transcode.rs
# Then: T014 run tests (must FAIL — sequential)
# Then implement in parallel:
Task C: Fix detect_encoding() (T015) in src/encoding/detect.rs
Task D: decode(Utf16Le) arm (T016) in src/encoding/transcode.rs
Task E: decode(Utf16Be) arm (T017) in src/encoding/transcode.rs
# Then: T018 odd-length guard (depends on T016, T017)
# Then: T020 run tests (must PASS — sequential)
```

---

## Implementation Strategy

### MVP First (US1 + US2 only — covers core gap)

1. Complete Phase 1: Setup (fixtures)
2. Complete Phase 2: Foundational (enum + compile)
3. Complete Phase 3: US1 (detect + decode)
4. Complete Phase 4: US2 (encode + round-trip)
5. **STOP and VALIDATE**: open `tests/fixtures/utf16le_bom.bin`, verify display; hexdump round-trip
6. Skip US3 if time-constrained (not blocking for most users)

### Full Delivery (US1 + US2 + US3)

1. MVP scope above
2. Phase 5: US3 (alias parsing for `--encoding` flag)
3. Phase 6: US4 deferral housekeeping (GitHub issue + ROADMAP.md row)
4. Phase 7: Polish, docs gate, PR

---

## Test Coverage Ledger

Tracks new test cases against SC-004 requirement (≥ 15 new tests):

| Phase | Task | New Tests | Description |
|-------|------|-----------|-------------|
| 3 | T012 | 4 | `detect_encoding()` BOM detection (LE, BE, UTF-8 unchanged, empty) |
| 3 | T013 | 6 | `decode()` UTF-16 (LE+BOM, BE+BOM, empty, odd-length, surrogate, mixed) |
| 3 | T019 | 2 | Status bar name (`EncodingProfile.name` for Utf16Le, Utf16Be) |
| 4 | T021 | 3 | `encode(s, Utf16Le)` (ASCII, empty, surrogate) |
| 4 | T022 | 2 | `encode(s, Utf16Be)` (ASCII, empty) |
| 4 | T023 | 3 | Round-trip integration (LE file, BE file, surrogate file) |
| 5 | T028 | 8 | `encoding_from_str()` aliases (LE variants, BE variants, default, invalid) |
| **Total** | | **28** | SC-004 requires ≥ 15 ✅ |

---

## Notes

- `[P]` tasks modify different files with no shared state — safe to run concurrently
- `[Story]` label maps each task to its user story for traceability and independent delivery
- The `encoding_rs` crate's `UTF_16_LE.decode()` strips the BOM automatically — do not strip it manually before calling decode
- `encoding_rs::UTF_16_LE.encode()` does NOT prepend a BOM — prepend `[0xFF, 0xFE]` manually
- Odd-byte-length guard must come before the `encoding_rs` call — `encoding_rs` does not error on odd lengths; it silently drops the trailing byte
- All fixture files are committed to `tests/fixtures/` — they are binary files, not generated at test time, so CI is hermetic
- Constitution §VII self-cert for PR: UTF-16 decode goes through `encoding_rs` (no raw bytes into buffer); file path handling unchanged in `src/security/sanitize.rs`; decoded content goes through existing terminal escape sanitizer before rendering
