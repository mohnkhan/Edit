---
description: "Task list for feature 031 — caret-on-click in dialog text fields (#58)"
---

# Tasks: Caret-on-click in dialog text fields

**Input**: Design documents from `specs/031-dialog-field-caret-click/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/behavior.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V. Tests first.

**Organization**: Setup → Foundational (`field_caret_at`) → US1 Find/Replace → US2 file-browser Name →
US3 Go-to-Line → Polish. Closing all three stories closes #58.

## Format: `[ID] [P?] [Story?] Description`

---

## Phase 1: Setup

- [X] T001 Confirm a clean baseline build on branch `031-dialog-field-caret-click` (`make tmpfs-setup`, `make`).
- [X] T002 Register a new integration test target `field_caret` in `Cargo.toml`; create `tests/integration/field_caret.rs`.

---

## Phase 2: Foundational — `field_caret_at` (blocking)

- [X] T003 [P] Unit tests in `src/ui/width.rs`: `field_caret_at` — value fits (offset → grapheme; past-end → len), overflow/right-anchored (offset → absolute index in the tail), multibyte/wide, empty value → 0, clamp.
- [X] T004 In `src/ui/width.rs`, add `field_caret_at(value: &str, field_w: u16, click_offset: u16) -> usize` per the contract (visible-window logic + `display_width`, clamped). (FR-001)

**Checkpoint**: helper proven in isolation; `make check` green.

---

## Phase 3: US1 — Find/Replace click-to-position (Priority: P1) — closes part of #58

- [X] T005 [P] [US1] Unit test in `src/ui/mod.rs`: `find_replace_field_rects(d, area)` returns the query text rect at row `dy+3` (and replacement at `dy+7` in replace mode), x `dx+2`, width `dw-4` — matching `render_find_field`.
- [X] T006 [US1] In `src/ui/mod.rs`, add `find_replace_field_rects` and have `render_find_field`/the renderer use the same geometry (or assert equality in a test). (FR-006)
- [X] T007 [US1] In `src/app.rs` `handle_mouse_event` (after the interactive button + list-row hit-tests), map a click in a Find/Replace field rect → set `d.focus` to that field and `d.caret = field_caret_at(value, rect.width, ev.col - rect.x)`. (FR-002)
- [X] T008 [P] [US1] Integration test in `tests/integration/field_caret.rs`: open Find, type text, click an interior column → `d.caret` is the expected grapheme; click past end → caret = len.

**Checkpoint**: Find/Replace click-to-position works; `make check` green.

---

## Phase 4: US2 — file-browser Name caret model (Priority: P2) — closes part of #58

- [X] T009 [P] [US2] Unit tests in `src/ui/file_browser.rs`: `push_char` inserts at the caret, `backspace` deletes before the caret, `move_left/right/home/end` clamp; a field-text-rect accessor returns the box interior.
- [X] T010 [US2] In `src/ui/file_browser.rs`, add `caret: usize` to the filename input; make `push_char`/`backspace` caret-aware; add `move_left/right/home/end`; render the caret mid-string (insert `▏` at the caret in the existing right-anchor logic); expose the field text rect from `compute_layout`. Reset caret when the field clears. (FR-003, FR-005)
- [X] T011 [US2] In `src/app.rs`, route the file-browser field intercept: `MoveLeft/MoveRight/MoveLineStart/MoveLineEnd` move the caret; a click in the field rect → `caret = field_caret_at(...)`. Preserve existing list/nav/activation behavior. (FR-003)
- [X] T012 [P] [US2] Integration test in `tests/integration/field_caret.rs`: type a name, Left twice + type → mid-string insert; click earlier in the field → caret moves.

**Checkpoint**: Name field is a first-class input; `make check` green.

---

## Phase 5: US3 — Go-to-Line caret model (Priority: P2) — closes part of #58

- [X] T013 [P] [US3] Unit tests in `src/app.rs`: Go-to-Line digit insert at caret, Backspace before caret, Left/Right/Home/End clamp; non-digits rejected; Enter still clamps to range.
- [X] T014 [US3] In `src/app.rs`, add `pending_goto_line_caret: usize` (reset on open); update the Go-to-Line key intercept for insert/delete/move at the caret (digits-only preserved); embed the caret mid-string in the render (`src/ui/mod.rs`). (FR-004, FR-005)
- [X] T015 [US3] In `src/app.rs`, route a Go-to-Line field click → `pending_goto_line_caret = field_caret_at(value, width, ev.col - value_x)` using the value origin `dx + 1 + "Go to line: ".len()`. (FR-004, FR-006)
- [X] T016 [P] [US3] Integration test in `tests/integration/field_caret.rs`: type digits, click between them → caret moves; insert a digit mid-string; a letter is ignored.

**Checkpoint**: Go-to-Line is a first-class digit input; `make check` green.

---

## Phase 6: Polish & cross-cutting

- [X] T017 [P] Update `CHANGELOG.md` (feature 031 under `[Unreleased]`), `docs/STATUS.md`, `docs/CAPABILITIES.md` (click-to-position + arrow caret editing in the Find/Replace, file-browser Name, and Go-to-Line fields).
- [X] T018 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check); fix findings; note the pre-existing F12/Ctrl+O PTY smoke failure is not a regression.
- [X] T019 Run the `specs/031-dialog-field-caret-click/quickstart.md` manual walkthrough; on merge the PR closes #58 and the ROADMAP #58 row is marked Complete.

---

## Dependencies & Execution Order

- Setup → none. Foundational (`field_caret_at`) blocks the click parts of US1/US2/US3. US1/US2/US3 are
  otherwise independent (different fields). Polish after the stories. MVP = Foundational + US1.

### Parallel opportunities

- All `[P]` unit/integration tests; T017 docs `[P]`.

## Implementation Strategy

TDD per story (Constitution V). No new crates (IV). Reuse `ui::width`, the existing field renderers'
geometry, and `FindReplaceDialog`'s existing caret. Branch `031-dialog-field-caret-click`, PR to `master`
(closes #58), merge via GitHub. No AI attribution.
