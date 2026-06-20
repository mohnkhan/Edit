---
description: "Task list for feature 023 — mouse-wheel scrolling (app-wide)"
---

# Tasks: Mouse-wheel scrolling (app-wide)

**Input**: Design documents from `specs/023-mouse-wheel-scroll/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/wheel.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V (Test-Gated Merges, NON-NEGOTIABLE). Tests first.

**Organization**: Setup → Foundational (editor scroll helper) → US1 (editor wheel) → US2 (lists/overlays
wheel) → US3 (no-regression) → Polish. Nearly all change is in `src/app.rs`.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files / independent)
- **[Story]**: US1 (editor) / US2 (lists+overlays) / US3 (no-regression)

## Path Conventions

Single-project Rust: `src/app.rs`, integration tests under `tests/integration/`, units inline.

---

## Phase 1: Setup

- [x] T001 Confirm a clean baseline build on branch `023-mouse-wheel-scroll` (`make tmpfs-setup` then `make`).
- [x] T002 Re-read `handle_mouse_event` (the drag block + the `Press/Left` guard), `viewport_height`, `buffers[].scroll_offset`, `wrap_cache.total_visual_rows`, `FileBrowser::{move_up,move_down,visible_rows}`, `help_scroll`, `pending_encoding_select`, `plugin_manager_cursor` in `src/app.rs`. No code change.

---

## Phase 2: Foundational — editor scroll helper + step constant

- [x] T003 [P] Unit tests in `src/app.rs`: a `wheel_scroll_editor(buf_idx, down, step)` helper increments/decrements `scroll_offset.0` by the step, clamps at 0 (top) and `content-1` (bottom), and never changes `cursor`; soft-wrap uses the visual-row bound.
- [x] T004 In `src/app.rs`, add `const WHEEL_STEP: usize = 3;` and `fn wheel_scroll_editor(&mut self, buf_idx: usize, down: bool, step: usize)` clamping `scroll_offset.0` to `[0, content_rows-1]` (content = `total_visual_rows()` in soft-wrap else `line_count()`), leaving the cursor untouched.

**Checkpoint**: helper compiles; unit tests green; no behavior wired yet.

---

## Phase 3: User Story 1 — editor wheel (Priority: P1) 🎯 MVP

**Goal**: Wheel over the editor scrolls the viewport (3 lines/notch), bounded, cursor unchanged.

**Independent Test**: synth ScrollDown over the editor → `scroll_offset.0` += 3; at bottom it clamps; at
top ScrollUp is a no-op; cursor unchanged.

### Tests for US1 (write first, must fail)

- [x] T005 [P] [US1] Integration test in `tests/integration/mouse_wheel.rs`: with a tall buffer and no modal, a `MouseEventKind::ScrollDown` advances `buffers[0].scroll_offset.0` by 3 and leaves `cursor` unchanged; `ScrollUp` at the top is a no-op; repeated ScrollDown clamps at the bottom.

### Implementation for US1

- [x] T006 [US1] In `src/app.rs` `handle_mouse_event`, add a `match ev.kind { ScrollUp | ScrollDown => … }` block **after the drag block and before the `Press/Left` guard** that, when no modal is open, computes the editor pane buffer (by `ev.col` vs `width/2` in split view; else active/0) and calls `wheel_scroll_editor`, then `return Ok(())`. Per FR-009 (I1): a non-modal wheel on the menu-bar row (`row == 0`) or the status-bar row (`row == term_rows-1`) is ignored (return without scrolling).

**Checkpoint**: editor wheel scrolls bounded, cursor unchanged; `make check` green.

---

## Phase 4: User Story 2 — lists & overlays wheel (Priority: P1)

**Goal**: Wheel scrolls the open modal (Help/About, file browser, encoding, plugin) instead of the editor.

**Independent Test**: open Help (overflowing) → ScrollDown increases `help_scroll`, ScrollUp at 0 is a
no-op; open the file browser on a long dir → wheel moves the selection/scroll; with a modal open the
editor offset is unchanged.

### Tests for US2 (write first, must fail)

- [x] T007 [P] [US2] Integration test in `tests/integration/mouse_wheel.rs`: with Help open, ScrollDown raises `help_scroll` and ScrollUp clamps at 0; with the file browser open on a long directory, the wheel changes the selected/scroll position; with a modal open, `buffers[0].scroll_offset.0` is unchanged (modal wins).

### Implementation for US2

- [x] T008 [US2] In the wheel block (`src/app.rs`), route by modal precedence per `contracts/wheel.md`: Help/About → `help_scroll ± step` (saturating); encoding select → cursor `± step` clamped `[0,n-1]`; file browser → `move_up`/`move_down(visible_rows)` ×step; plugin manager → cursor `± step` clamped; Find/Replace → ignore. The editor branch (T006) is the `else`.

**Checkpoint**: every scrollable surface responds, bounded; `make check` green.

---

## Phase 5: User Story 3 — no regression (Priority: P1)

**Goal**: Existing click/drag/keyboard behavior is unchanged.

- [x] T009 [P] [US3] Integration test in `tests/integration/mouse_wheel.rs`: a left-click still places the cursor; a press→drag still extends the selection (feature 017); a wheel event does NOT place the cursor or start a selection; a wheel on the menu-bar row (0) and status-bar row (last) with no modal makes no state change (I1/FR-009).

**Checkpoint**: no-regression test green.

---

## Phase 6: Polish & Cross-Cutting

- [x] T010 [P] Update `CHANGELOG.md` (feature 023 under `[Unreleased]`), `docs/STATUS.md`, and `docs/CAPABILITIES.md` (mouse-wheel scrolling is a user-visible capability).
- [x] T011 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check) and fix findings.
- [x] T012 Run the `specs/023-mouse-wheel-scroll/quickstart.md` manual walkthrough (editor, file browser, Help, dialogs; bounds; click/drag unaffected).

---

## Dependencies & Execution Order

- **Setup (P1)** → none. **Foundational (P2)** → blocks US1 (uses the helper).
- **US1 (P3)** → the wheel-block skeleton + editor branch. **US2 (P4)** extends the same block with modal
  routing (depends on the block from US1). **US3 (P5)** is assertions over the finished block.
- **Polish (P6)** → after the stories.

### Parallel opportunities

- T003 (helper test) `[P]`; T005/T007/T009 are `[P]` test additions; T010 docs `[P]`.

---

## Implementation Strategy

### MVP

Setup → Foundational helper → US1 editor wheel (T005–T006): the headline fix. Then US2 routing to
lists/overlays, then US3 no-regression assertions.

### Notes

- TDD mandatory (Constitution V). No new deps/config — fixed `WHEEL_STEP = 3` (Constitution IV/VI).
- Wheel block must sit before the `Press/Left` guard so events aren't dropped, and must `return Ok(())`
  so existing click/drag paths are never entered for a wheel event.
- Keep AI attribution out of commits/PR/issues. Branch `023-mouse-wheel-scroll`, PR to `master`, merge
  via GitHub.
