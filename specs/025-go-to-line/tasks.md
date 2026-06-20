---
description: "Task list for feature 025 — Go to Line"
---

# Tasks: Go to Line

**Input**: Design documents from `specs/025-go-to-line/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/go-to-line.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V (Test-Gated Merges, NON-NEGOTIABLE). Tests first.

**Organization**: Setup → Foundational (action + state) → US1 (jump) → US2 (clamp/cancel/invalid) → US3
(modal/no-regression) → Polish.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files / independent)
- **[Story]**: US1 (jump) / US2 (clamp+cancel) / US3 (modal/no-regression)

## Path Conventions

Single-project Rust: `src/input/keymap.rs`, `src/ui/menubar.rs`, `src/app.rs`, `src/ui/mod.rs`;
integration tests under `tests/integration/`, units inline.

---

## Phase 1: Setup

- [x] T001 Confirm a clean baseline build on branch `025-go-to-line` (`make tmpfs-setup` then `make`).
- [x] T002 Re-read the encoding-dialog modal pattern (`pending_encoding_select` open + `handle_action` intercept), `set_cursor_lc`/`clamp_scroll`, the modal guards in `handle_mouse_event`/wheel/scrollbar, and `SEARCH_MENU` (src/ui/menubar.rs). No code change.

---

## Phase 2: Foundational — action + state

- [x] T003 In `src/input/keymap.rs`, add `Action::GoToLine`; bind `Ctrl+G` in `default_map`; map it in `action_from_string` (`"GoToLine"`); update the binding test that asserts key→action coverage if present.
- [x] T004 In `src/ui/menubar.rs`, add a `SEARCH_MENU` item `{ label: "Go to Line", action: Action::GoToLine, mnemonic: Some('g') }`.
- [x] T005 In `src/app.rs`, add `pending_goto_line: Option<String>` (init `None` in `App::new`); add `Action::GoToLine` handling in `handle_action` that opens the prompt (`Some(String::new())`) only when no other modal is open.

**Checkpoint**: builds; the action opens an (empty) prompt state.

---

## Phase 3: User Story 1 — jump to a line (Priority: P1) 🎯 MVP

**Goal**: Type a number + Enter → cursor at that line's start, scrolled into view.

**Independent Test**: `Ctrl+G`, type `42`, Enter → `cursor.line == 41`, column 0, line visible.

### Tests for US1 (write first, must fail)

- [x] T006 [P] [US1] Integration test in `tests/integration/go_to_line.rs`: with a 100-line buffer, `Ctrl+G` then digits `5`,`0` then Enter sets `cursor.line == 49`, `grapheme_col == 0`, and `scroll_offset.0` brings line 49 into view; `pending_goto_line` is `None` afterward.

### Implementation for US1

- [x] T007 [US1] In `src/app.rs`, add the `pending_goto_line` intercept in `handle_action`: `InsertChar(d)` (push when `d.is_ascii_digit()`), `Backspace` (pop), `MenuClose` (cancel), `InsertNewline` (confirm). On confirm, parse → clamp to `[1, line_count]` → `set_cursor_lc(line-1, 0)`; close. Consume all other actions (buffer untouched).
- [x] T008 [US1] In `src/ui/mod.rs`, render a small centered modal overlay showing `Go to line: <entry>▏` while `pending_goto_line` is `Some` (consistent with the other dialog overlays; width-correct; no panic on tiny terminals). Add an inline render test (TestBackend) that the overlay renders at 80×24 and on a tiny terminal (e.g. 10×3) without panicking (FR-009/L1).

**Checkpoint**: jump works by keyboard and via the Search menu; `make check` green.

---

## Phase 4: User Story 2 — clamp / cancel / invalid (Priority: P2)

**Goal**: Out-of-range clamps; Esc and invalid input never move the cursor wrongly.

### Tests for US2 (write first, must fail)

- [x] T009 [P] [US2] Inline unit test in `src/app.rs` for the clamp: `n` clamped to `[1, line_count]` (`0`→1, `> count`→count, overflow→count).
- [x] T010 [P] [US2] Integration test in `tests/integration/go_to_line.rs`: an over-range number clamps to the last line; `0` clamps to line 1; Esc closes with the cursor unchanged; Enter on an empty field does not move the cursor.

### Implementation for US2

- [x] T011 [US2] In `src/app.rs`, ensure the confirm path clamps with `n.clamp(1, line_count)` and that an empty / non-parsing entry closes with no movement; non-digit keystrokes are already rejected by the field (T007).

**Checkpoint**: bounds + cancel + invalid handled; `make check` green.

---

## Phase 5: User Story 3 — modal / no-regression (Priority: P2)

**Goal**: The prompt captures input and coexists with the other modals/features.

### Tests for US3 (write first, must fail)

- [x] T012 [P] [US3] Integration test in `tests/integration/go_to_line.rs`: while the prompt is open, a letter `InsertChar('a')` does NOT modify the buffer; the wheel/click over the editor does not scroll/move the cursor (modal wins); only one modal opens at a time.

### Implementation for US3

- [x] T013 [US3] In `src/app.rs`, add `pending_goto_line.is_some()` to the modal guards in `handle_mouse_event` (ignore editor clicks), the wheel block (023), and `scrollbar_regions` (024) so the editor doesn't act under the prompt; ensure `Action::GoToLine` does not open when another modal is already open.

**Checkpoint**: prompt is a well-behaved modal; `make check` green.

---

## Phase 6: Polish & Cross-Cutting

- [x] T014 [P] Update `CHANGELOG.md` (feature 025 under `[Unreleased]`), `docs/STATUS.md`, and `docs/CAPABILITIES.md` (Ctrl+G / Search ▸ Go to Line is a user-visible keybinding + menu item).
- [x] T015 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check) and fix findings.
- [x] T016 Run the `specs/025-go-to-line/quickstart.md` manual walkthrough (jump, clamp, Esc, invalid, menu item, buffer-not-edited).

---

## Dependencies & Execution Order

- **Setup (P1)** → none. **Foundational (P2)** → action+state, blocks US1.
- **US1 (P3)** intercept+render+jump. **US2 (P4)** clamp/cancel/invalid (extends the confirm path).
  **US3 (P5)** modal guards + no-regression.
- **Polish (P6)** → after the stories.

### Parallel opportunities

- T006/T009/T010/T012 are `[P]` test additions; T014 docs `[P]`.

---

## Implementation Strategy

### MVP

Setup → Foundational → US1 (T006–T008): `Ctrl+G` + type + Enter jumps. Then US2 bounds/cancel, then US3
modal hygiene.

### Notes

- TDD mandatory (Constitution V). No new deps/config (Constitution IV/VI). Reuse `set_cursor_lc`
  (scrolls into view via `clamp_scroll`); navigation only.
- Keep AI attribution out of commits/PR/issues. Branch `025-go-to-line`, PR to `master`, merge via GitHub.
