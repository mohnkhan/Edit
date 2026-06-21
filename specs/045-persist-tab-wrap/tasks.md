# Tasks: Persist Each Tab's Soft-Wrap Across Restart

**Feature**: `045-persist-tab-wrap` | **Branch**: `045-persist-tab-wrap`
**Input**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/internal-api.md](./contracts/internal-api.md)

**Overriding constraint**: Backward compatible — older session files (no wrap field, version 1) MUST
still load and default to the configured wrap (FR-003/FR-004). Behavior unchanged for non-session users
and for new tabs after restore (FR-006). The 042 `clippy::unwrap_used` guardrail holds.

**Story → phase map**: US1 = round-trip restore (P1), US2 = legacy files still load (P1).

## Phase 1: Setup
- [ ] T001 Baseline: `make tmpfs-setup`; `make check`; note count (1272/0/11).

## Phase 2: Foundational (schema)
- [ ] T002 In `src/session/mod.rs`: add `#[serde(default)] pub soft_wrap: bool` to `BufferEntry`; add a
  `SESSION_SCHEMA_VERSION = 2` (or bump the literal) and change the loader's `version != 1` reject to
  accept `1` or `2` (still reject anything else).

## Phase 3: US1 — Tabs reopen in their saved wrap state (P1)
- [ ] T003 [US1] `build_session_data` (`src/app/fileops.rs`): write `soft_wrap: buf.soft_wrap` in each
  `BufferEntry`; set `SessionData.version` to 2.
- [ ] T004 [US1] `do_restore_session` (`src/app/fileops.rs`): in the restore loop, set
  `buf.soft_wrap = entry.soft_wrap` after opening + cursor seek, before pushing (supersedes the 044
  config-seed for restored tabs).

## Phase 4: US2 — Old sessions still load (P1)
- [ ] T005 [US2] Confirm a legacy payload (no `soft_wrap`, version 1) deserializes (`serde(default)` →
  false) and the loader accepts version 1; add/adjust a test proving it loads with default wrap.

## Phase 5: Tests
- [ ] T005a Note: adding `soft_wrap` to `BufferEntry` makes every struct-literal construction require
  the field — the compiler enumerates them (writer + any session test literals). `serde(default)` only
  covers *deserialization*, not literals. Fix each as the build flags it.
- [ ] T006 [US1] Round-trip test (in `src/session/mod.rs` or `tests/integration/`): construct/serialize a
  `SessionData` with two `BufferEntry`s, one `soft_wrap: true` one `false`; deserialize; assert the field
  round-trips. Plus an app-level test: build session from an App with mixed per-tab wrap and restore,
  asserting each restored buffer's `soft_wrap` (SC-001).
- [ ] T007 [US2] Legacy-load test: deserialize a TOML/JSON string with no `soft_wrap` key → `false`
  default; and a version-1 file still loads (SC-002). Keep `test_unknown_version_returns_err` rejecting a
  bogus version.
- [ ] T008 Update existing feature-003 session tests that assert `version: 1` to expect `2` where they
  check the *written* version (round-trip/save tests); legacy-read tests keep version 1.

## Phase 6: Polish & Ship
- [ ] T009 `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check); count == baseline +
  new tests; pre-existing env `encoding_select` smoke F12 failure unrelated.
- [ ] T010 Docs gate: `CHANGELOG.md` + `docs/STATUS.md`. No `CAPABILITIES.md` change (no new
  keybinding/menu/flag; session restore already documented — wrap is just additionally remembered).
- [ ] T011 PR → `master` (`feat(045): persist per-tab soft-wrap across restart`), strip AI-attribution,
  green, merge.

## Dependencies & Order
Setup (T001) → schema (T002) → writer/reader (T003–T004) → legacy verify (T005) → tests (T006–T008) →
polish/ship (T009–T011). T002 is the forcing function (field + version-accept); writer/reader follow.

## MVP
US1 (write + apply per-tab wrap) is the feature; US2 (legacy compat) is the non-negotiable safety net.
Both ship in one PR.
