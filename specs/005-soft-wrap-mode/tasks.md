# Tasks: Soft-Wrap Mode (Feature 005)

**Input**: Design documents from `specs/005-soft-wrap-mode/`

**Prerequisites**: plan.md ✅ | spec.md ✅ | research.md ✅ | data-model.md ✅ | contracts/ ✅

**Organization**: Tasks grouped by user story for independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no shared state dependencies)
- **[Story]**: Which user story this task belongs to (US1–US4 from spec.md)
- All file paths are repository-relative

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Test fixtures, build registration, and scaffolding that every subsequent phase depends on.

- [ ] T001 Create test fixture `tests/fixtures/long_line.txt` (line 1: "Short line.", line 2: 200 `A` chars + space + long word string, line 3: "Another short line.")
- [ ] T002 [P] Create test fixture `tests/fixtures/cjk_wide.txt` (100 CJK `字` chars each 2 columns wide, followed by space separator, followed by 50 more CJK chars)
- [ ] T003 [P] Create test fixture `tests/fixtures/mixed_width.txt` (mix of emoji 🎉, CJK `字`, and ASCII on one 200-column logical line)
- [ ] T004 Create empty scaffold `tests/integration/soft_wrap.rs` with `mod tests { use super::*; }` block
- [ ] T005 Register soft_wrap integration test in `Cargo.toml` as `[[test]] name = "soft_wrap" path = "tests/integration/soft_wrap.rs"`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core WrapCache module, Action variant, Config field, and App fields that ALL user story
phases require. No US work can begin until this phase is complete.

**⚠️ CRITICAL**: These tasks MUST be complete before any Phase 3+ work begins.

- [ ] T006 Create `src/ui/wrap.rs` with `WrapCache` struct definition — fields: `viewport_width: u16`, `text_version: u64`, `visual_starts: Vec<Vec<u32>>`, `visual_line_map: Vec<(u32, u32)>` (see `contracts/wrap-cache.md`)
- [ ] T007 Implement `WrapCache::compute(rope, viewport_width, text_version)` in `src/ui/wrap.rs` — grapheme-cluster walk using `unicode_segmentation::graphemes(true)` + `unicode_width::UnicodeWidthStr::width()`; break at last whitespace ≤ viewport; hard-break at grapheme boundary when no whitespace fits; build `visual_line_map` from `visual_starts`
- [ ] T008 Implement `WrapCache::is_stale(width, version)`, `visual_to_logical(row)`, `total_visual_rows()`, `visual_row_count(line)` in `src/ui/wrap.rs` *(must follow T007 — same file; [P] only relative to T010, T013)*
- [ ] T009 Write unit tests for `WrapCache` in `src/ui/wrap.rs` `#[cfg(test)]` block: (a) ASCII word-wrap at whitespace; (b) no-whitespace hard-break; (c) CJK double-width char straddles boundary → moved to next line; (d) empty line → single visual row, no marker; (e) line exactly at viewport width → no break; (f) line one grapheme over width → two visual rows; (g) `visual_to_logical` round-trips all rows; (h) `is_stale` returns true on width/version change, false otherwise
- [ ] T010 [P] Add `pub mod wrap;` to `src/ui/mod.rs`
- [ ] T011 Add `ToggleSoftWrap` variant to `Action` enum in `src/input/keymap.rs`
- [ ] T012 Add `map.insert("Alt+Z".to_string(), Action::ToggleSoftWrap)` to `KeybindingMap::default()` and `"ToggleSoftWrap" => Some(Action::ToggleSoftWrap)` to `action_from_str()` in `src/input/keymap.rs`; update keymap unit tests to assert Alt+Z → ToggleSoftWrap
- [ ] T013 [P] Add `pub soft_wrap: bool` field to `Config` struct in `src/config/schema.rs` with `#[serde(default)]`; add `soft_wrap: false` to `Default::default()`; update `default_values_match_contract` and `serde_round_trip_default` tests to assert `soft_wrap == false`

**Checkpoint**: WrapCache, Action::ToggleSoftWrap, and Config.soft_wrap are all ready. US phases may now begin.

---

## Phase 3: User Story 1 — Enable Soft-Wrap Visual Reflow (Priority: P1) 🎯 MVP

**Goal**: User presses Alt+Z and long lines immediately reflow into visual sub-lines with `»` markers.
Subsequent Ctrl+S saves without extra newlines.

**Independent Test**: Open `tests/fixtures/long_line.txt`, press Alt+Z, verify status shows `[WRAP]`
and long line is visually broken; press Ctrl+S, verify file bytes unchanged.

### Implementation for User Story 1

- [X] T014 [US1] Add `soft_wrap: bool` and `wrap_cache: Option<WrapCache>` fields to `App` in `src/app.rs`; initialize `soft_wrap` from `config.soft_wrap` and `wrap_cache` from `None` in `App::new()`
- [X] T015 [US1] Implement `Action::ToggleSoftWrap` dispatch in `App` event loop in `src/app.rs`: check viewport content_width ≥ 10 (else set status "Terminal too narrow for soft wrap (min 10 columns)" and return); toggle `app.soft_wrap`; sync to `app.config.soft_wrap`; on enable: compute and store `wrap_cache`; on disable: drop `wrap_cache` (set to `None`), reset all `buffer.scroll_offset.1 = 0`, restore horizontal-scroll rendering path; verify toggle-off path with unit test asserting `wrap_cache.is_none()` and `scroll_offset.1 == 0` after disable
- [X] T016 [US1] Add `soft_wrap: bool` and `wrap_starts: &'a [Vec<u32>]` fields to `EditorWidget` in `src/ui/editor.rs`; update `EditorWidget::new()` to accept these; update `EditorWidget::render()` to branch on `self.soft_wrap` — when false use existing path unchanged
- [X] T017 [US1] Implement soft-wrap rendering branch in `EditorWidget::render()` in `src/ui/editor.rs`: enumerate visual rows from `wrap_starts`; skip rows above `scroll_offset.0` (visual row units); render `»` glyph at leftmost gutter column for continuation rows (`seg_idx > 0`); render grapheme clusters for the visible segment of each logical line; suppress horizontal scroll offset (`scroll_vcol = 0`) in this branch
- [X] T018 [US1] Implement cursor rendering in soft-wrap mode in `EditorWidget::render()` in `src/ui/editor.rs`: find which visual segment the cursor falls in by scanning `wrap_starts[cursor.line]` for the largest `start_byte ≤ cursor_byte_offset`; compute screen column as `cursor.visual_col - segment_start_visual_col`; render cursor if within content_width
- [X] T019 [P] [US1] Add `soft_wrap: bool` field to `StatusBar` in `src/ui/statusbar.rs`; update `StatusBar::new()` constructor; update `flags()` to append `" [WRAP]"` when `soft_wrap == true` (abbreviated indicator — "(ext)" suffix is conveyed via the menu label per M2 resolution: the status bar uses `[WRAP]` as a terse mode flag consistent with `[Modified]`/`[Read Only]`); update all `StatusBar::new()` call sites in `src/app.rs` render path; add unit test asserting `[WRAP]` appears in flags when `soft_wrap == true` and absent when `false`
- [X] T020 [P] [US1] Add `MenuItem { label: "Soft Wrap (ext)", action: Action::ToggleSoftWrap }` to `VIEW_MENU` static array in `src/ui/menubar.rs`
- [X] T020b [US1] Update mouse-click handler in `src/input/mouse.rs`: when `soft_wrap == true` and a click event fires, map the clicked `(visual_row, visual_col)` to `(logical_line, grapheme_col)` via `app.wrap_cache.as_ref()?.visual_to_logical(visual_row)` then walk grapheme clusters from `start_grapheme` to accumulate visual width up to `visual_col`; clamp past-end-of-segment clicks to the last grapheme of that segment; when `soft_wrap == false` use existing click-position logic unchanged

**Checkpoint**: Alt+Z toggles visual reflow; `»` markers appear; status bar shows `[WRAP]`; Ctrl+S produces byte-identical file.

---

## Phase 4: User Story 2 — Edit Text Normally While Soft-Wrap Active (Priority: P1)

**Goal**: Cursor keys, End, Ctrl+S, Find, and undo/redo all operate on logical text — no wrap-point
artifacts in saved files or cursor positions.

**Independent Test**: Enable soft-wrap, type/delete text on long line, press Ctrl+S, disable wrap,
compare file bytes to pre-edit expected content — no extra newlines inserted.

### Implementation for User Story 2

- [X] T020c [US2] Update `EditorWidget` soft-wrap rendering branch in `src/ui/editor.rs` to handle Find/Replace match highlights that span visual wrap boundaries: when rendering a segment, check if any search highlight span overlaps the segment's byte range; if so, apply the highlight style to the overlapping grapheme clusters on that segment row, so the highlight is visible on all visual rows the match occupies (per updated FR-008)
- [X] T021 [US2] Wire `WrapCache` invalidation on buffer text edit in `src/app.rs`: after any action that modifies buffer content (insert, delete, paste, undo, redo), if `soft_wrap == true` call `wrap_cache.as_mut().map(|c| c.text_version = buffer.version)` or recompute cache; use buffer version counter to detect staleness
- [X] T022 [US2] Wire `WrapCache` invalidation on terminal resize in `src/app.rs`: in the `Event::Resize` handler, if `soft_wrap == true` recompute `wrap_cache` with the new viewport width; update `buffer.scroll_offset.0` if it now exceeds `wrap_cache.total_visual_rows()`
- [X] T023 [US2] Update scroll-to-cursor logic in `src/app.rs` for wrap mode: when `soft_wrap == true`, convert cursor's logical position to visual row using `wrap_cache.visual_to_logical()` inverse; ensure visual row is in `[scroll_offset.0, scroll_offset.0 + visible_rows)`; adjust `scroll_offset.0` in visual-row units
- [X] T024 [US2] Add unit tests to `src/app.rs` `#[cfg(test)]` verifying (SC-003 coverage): (a) `Action::Save` (Ctrl+S) dispatched while `soft_wrap == true` writes `buffer.rope` content unchanged (no extra newlines); (b) auto-save path (30-second timer) also writes byte-identical content while `soft_wrap == true` (per updated FR-007); (c) `Action::Find` dispatched while `soft_wrap == true` searches logical text; (d) on a 500-grapheme logical line with soft-wrap active, `Home` moves cursor to grapheme_col 0 of the logical line; (e) `End` moves cursor to grapheme_col 499 of the logical line; (f) `↑`/`↓` move between logical lines skipping all visual continuation rows; (g) `←`/`→` advance through grapheme positions within the logical line correctly

**Checkpoint**: All editing operations (type, delete, paste, undo, save, find) produce correct logical output regardless of soft-wrap state.

---

## Phase 5: User Story 3 — Toggle Off Returns to Horizontal Scroll (Priority: P2)

**Goal**: Pressing Alt+Z when wrap is active restores horizontal-scroll mode with no side effects on
buffer content or cursor position.

**Independent Test**: Enable wrap, move cursor to a known logical position, disable wrap — cursor is
at same logical position; horizontal scrolling works; no `[WRAP]` in status bar.

### Implementation for User Story 3

- [X] T025 [US3] Write unit test in `src/app.rs` `#[cfg(test)]` for the complete toggle cycle: enable soft-wrap (verify `wrap_cache.is_some()`, status contains `[WRAP]`), then disable (verify `wrap_cache.is_none()`, `buffer.scroll_offset.1 == 0`, status does not contain `[WRAP]`, horizontal-scroll branch active); also verify cursor's logical position is unchanged across the cycle
- [X] T026 [P] [US3] Write integration test `test_toggle_on_off` in `tests/integration/soft_wrap.rs`: open long_line fixture, simulate Alt+Z (toggle on), assert status contains `[WRAP]`; simulate Alt+Z again (toggle off), assert status no longer contains `[WRAP]` and buffer bytes are unchanged

**Checkpoint**: Alt+Z cycles cleanly between wrap and horizontal-scroll modes; buffer content is never altered.

---

## Phase 6: User Story 4 — Persist Soft-Wrap Preference (Priority: P3)

**Goal**: User's soft-wrap preference survives quit-and-relaunch without re-toggling.

**Independent Test**: Enable wrap, quit editor, relaunch without arguments — wrap is active on first
frame; `~/.config/edit/config.toml` contains `soft_wrap = true`.

### Implementation for User Story 4

- [X] T027 [US4] Add config-to-disk write in `Action::ToggleSoftWrap` handler in `src/app.rs`: after toggling `app.config.soft_wrap`, serialize `app.config` to TOML and write atomically to `$XDG_CONFIG_HOME/edit/config.toml` via the existing tmp-rename pattern (`.config.toml.tmp` → rename); on I/O failure: log at warn level, set `app.status_message = Some("Config save failed: [reason]")`, do NOT revert the toggle (per FR-011); add unit test asserting toggle persists in-memory even when disk write fails
- [X] T028 [US4] Verify `App::new()` reads `soft_wrap` from the loaded `Config` and sets `app.soft_wrap` accordingly (covered by T014); add startup integration assertion: write config with `soft_wrap = true`, launch App, assert `app.soft_wrap == true` and `app.wrap_cache.is_some()`
- [X] T029 [P] [US4] Write integration test `test_persistence` in `tests/integration/soft_wrap.rs`: write `soft_wrap = true` to a temp config file; construct App from that config; assert `app.soft_wrap == true`; assert status bar includes `[WRAP]` on first render

**Checkpoint**: Preference survives restart; config file reflects the last-set state.

---

## Phase 7: Integration Test Suite Completion

**Purpose**: Full suite covering all acceptance scenarios from spec.md and quickstart.md.

- [ ] T030 Write integration test `test_toggle_on_reflows` in `tests/integration/soft_wrap.rs`: load long_line.txt into a Buffer, toggle soft_wrap, verify `WrapCache.total_visual_rows() > 3` (long line produces multiple visual rows)
- [ ] T031 [P] Write integration test `test_save_no_extra_newlines` in `tests/integration/soft_wrap.rs`: toggle wrap on, call save action, read bytes from disk, assert byte-identical to original fixture
- [ ] T032 [P] Write integration test `test_cjk_no_corrupt_render` in `tests/integration/soft_wrap.rs`: load cjk_wide.txt, compute `WrapCache`, assert all break points in `visual_starts` land on even byte boundaries (no split double-width char) by checking that each break is at a grapheme-cluster boundary
- [ ] T033 [P] Write integration test `test_too_narrow_guard` in `tests/integration/soft_wrap.rs`: set `viewport_width = 5`, call ToggleSoftWrap dispatch, assert `app.soft_wrap == false` and `app.status_message` contains "too narrow"
- [ ] T034 Run `cargo test` and confirm all new unit + integration tests pass; run `cargo clippy -- -D warnings` and fix any warnings introduced by this feature

---

## Phase 8: Polish & Docs Gate

**Purpose**: Documentation updates required by CLAUDE.md Docs Gate rule before PR merge.

- [ ] T035 Update `CHANGELOG.md` — add `## [Unreleased] — feature 005: Soft-Wrap Mode` section listing all added capabilities, FRs implemented, and test counts
- [ ] T036 [P] Update `docs/STATUS.md` — add rows for F005-US1 through F005-US4 all marked "Complete"; update project version note
- [ ] T037 [P] Update `docs/CAPABILITIES.md` — add "Soft Wrap (ext)" under View menu capabilities; note Alt+Z shortcut and `soft_wrap` config key
- [ ] T038 [P] Update `ROADMAP.md` — mark "Soft-Wrap Mode" deferred item as shipped; add closed Issue #4 reference; confirm "Menu Item Checked-State Indicator" deferral entry is present (added during analysis phase)
- [ ] T038b File GitHub issue for "Menu Item Checked-State Indicator" (deferred from FR-001): problem statement — `MenuItem` struct lacks `checked: bool` field; why deferred — menu-bar-wide refactor exceeds feature 005 scope; suggested approach — `Option<bool>` checked field + `✓` prefix rendering; effort Small; label `follow-up`
- [ ] T039 Run `make ci-local` (fmt → clippy → tests → smoke → perf-check → docs-gate) and resolve all failures

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — BLOCKS Phases 3–6
- **Phase 3 (US1)**: Depends on Phase 2 — primary MVP
- **Phase 4 (US2)**: Depends on Phase 3 (EditorWidget + App soft_wrap must exist)
- **Phase 5 (US3)**: Depends on Phase 3 (toggle-on path must exist to test toggle-off)
- **Phase 6 (US4)**: Depends on Phase 2 (Config.soft_wrap) and Phase 3 (App.soft_wrap toggle)
- **Phase 7 (Integration)**: Depends on Phases 3–6 all complete
- **Phase 8 (Docs)**: Depends on Phase 7

### User Story Dependencies

- **US1 (P1)**: After Phase 2 — no dependency on other stories
- **US2 (P1)**: After US1 (needs EditorWidget + App fields)
- **US3 (P2)**: After US1 (needs toggle-on path to test toggle-off)
- **US4 (P3)**: After Phase 2 (Config.soft_wrap) + partially US1 (App field)

### Within Each Phase

- T007 (WrapCache::compute) must complete before T008 (helper methods) for correctness verification
- T014 (App fields) must complete before T015 (dispatch handler)
- T016 (EditorWidget fields) must complete before T017 (render branch)
- T017 must complete before T018 (cursor rendering in wrap mode)
- T021/T022/T023 (US2 cache + scroll) must follow T015 (dispatch)

### Parallel Opportunities

```
Phase 1: T001 ‖ T002 ‖ T003 ‖ T004 ‖ T005
Phase 2: T006 → T007 → T009; T008 ‖ T010 ‖ T013 (all after T006)
Phase 3: T019 ‖ T020 (both independent of T016-T018)
Phase 5-6: T026 ‖ T029 (different test functions)
Phase 7: T030 → T031 ‖ T032 ‖ T033 (after T030 scaffolding)
Phase 8: T035 → T036 ‖ T037 ‖ T038 (after T035 for consistency)
```

---

## Implementation Strategy

### MVP (US1 Only — Phases 1–3)

1. Complete Phase 1: Setup (fixtures + Cargo.toml)
2. Complete Phase 2: Foundational (WrapCache + Action + Config)
3. Complete Phase 3: US1 (App + EditorWidget + StatusBar + MenuBar)
4. **STOP and VALIDATE**: Press Alt+Z in editor, verify visual reflow and `[WRAP]` indicator
5. Run `cargo test` — all unit tests must pass

### Incremental Delivery

1. Phases 1–3 → Soft-wrap visual reflow works (**MVP**)
2. Phase 4 → Editing semantics verified correct under wrap
3. Phase 5 → Toggle-off works cleanly
4. Phase 6 → Preference persists across sessions
5. Phases 7–8 → Full test suite + docs gate → PR ready

---

## Notes

- `[P]` tasks operate on different files or independent code paths — safe to run in parallel
- `[Story]` label maps each task to its user story for traceability to spec.md acceptance criteria
- The `»` continuation marker is a compile-time constant — not derived from buffer content (security-safe per Constitution Principle VII)
- Cursor movement code in `src/app.rs` does NOT need changes for Up/Down — it already operates on logical lines; only `scroll_offset.0` interpretation changes in wrap mode
- Zero new crate dependencies — `unicode-segmentation` and `unicode-width` already in `Cargo.toml`
- `Config.soft_wrap` uses `#[serde(default)]` — existing user configs parse without error
