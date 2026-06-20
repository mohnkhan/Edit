---
description: "Task list for feature 030 — interaction completeness (#53–#56)"
---

# Tasks: Interaction completeness

**Input**: Design documents from `specs/030-interaction-completeness/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/behavior.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V. Tests first.

**Organization**: Setup → US4 (F-keys, smallest) → US2 (double/triple-click) → US1 (in-dialog mouse) →
US3 (context menu) → Polish. Each user story closes one GitHub issue.

## Format: `[ID] [P?] [Story?] Description`

---

## Phase 1: Setup

- [X] T001 Confirm a clean baseline build on branch `030-interaction-completeness` (`make tmpfs-setup`, `make`).
- [X] T002 Register a new integration test target `interaction` in `Cargo.toml` and create `tests/integration/interaction.rs`.

---

## Phase 2: US4 — DOS F-key bindings (Priority: P2) — closes #56

- [X] T003 [P] [US4] Unit tests in `src/input/mod.rs`: `default_map` maps F6→NextBuffer, Shift+F6→PrevBuffer, F8→Cut, F9→Copy, F11→Paste; F1/F2/F3/F5/F10/F12 unchanged.
- [X] T004 [US4] In `src/input/keymap.rs` `default_map()`, add the five F-key bindings (no conflict with existing). (FR-011, FR-012)

**Checkpoint**: F-keys work; existing keys unchanged; `make check` green.

---

## Phase 3: US2 — double/triple-click selection (Priority: P2) — closes #54

- [X] T005 [P] [US2] Unit tests in `src/app.rs`: a word-selection helper selects the alphanumeric/`_` run under a grapheme col (and the adjacent run for non-word/space); a line-selection helper selects the whole line; multibyte + boundary (line end, empty line) cases don't panic.
- [X] T006 [US2] In `src/app.rs`, add a word-range + line-range helper (grapheme columns) and `last_editor_click: Option<(u16,u16,u8,Instant)>`; in `handle_mouse_event`'s editor left-press path classify click count (DOUBLE_CLICK_MS + same cell) → 1 position (existing), 2 select word, 3 select line; build `Selection`. (FR-004, FR-005, FR-006)
- [X] T007 [P] [US2] Integration test in `tests/integration/interaction.rs`: synth two/three left presses on a word → `selection_text()` returns the word / line; a following single press clears the selection.

**Checkpoint**: double/triple-click select; single-click clears; `make check` green.

---

## Phase 4: US1 — in-dialog mouse content hit-testing (Priority: P1) — closes #53

- [X] T008 [P] [US1] Unit tests in `src/ui/dialog.rs` + `src/ui/plugin_manager.rs`: a row-hit helper maps a click row inside the dialog to the correct list index (and `None` past the entries); in `src/ui/file_browser.rs`/shared, `field_caret_at` maps a click column to a caret grapheme (multibyte; clamps to end).
- [X] T009 [US1] Add the list-row geometry helpers (`encoding_row_hit`, plugin-manager row hit) next to their renderers, using the renderer's inner-list origin; add the shared `field_caret_at` helper built on `ui::width::display_width`. (FR-001, FR-002)
- [X] T010 [US1] In `src/app.rs` `handle_mouse_event`, after the button hit-test and before the modal drop: for the encoding/plugin dialogs map an in-list click → set selection + `dialog_focus = 0`; for the Find/Replace, Go-to-Line, and file-browser fields map an in-field click → set that field's caret + focus. No-op when on neither row/field/button; preserve button + outside-click behavior. (FR-001, FR-002, FR-003)
- [X] T011 [P] [US1] Integration test in `tests/integration/interaction.rs`: click an encoding row → `pending_encoding_select` updates; click into a Find query field → caret moves; a button click still activates; an inside-but-empty click is a no-op.

**Checkpoint**: dialog rows + fields are mouse-operable; buttons unaffected; `make check` green.

---

## Phase 5: US3 — right-click context menu (Priority: P3) — closes #55

- [X] T012 [P] [US3] Unit tests in `src/ui/contextmenu.rs`: rect/anchor clamps on-screen at edges/tiny terminal; `hit_test` maps a click to an item index; focus next/prev wraps; render shows the four items + focus marker without panic.
- [X] T013 [US3] Create `src/ui/contextmenu.rs` (`ContextMenu` items=[Cut,Copy,Paste,Select All], focus, anchor; `menu_rect`/`render`/`hit_test`) modelled on the menubar dropdown + `buttons`; export from `src/ui/mod.rs` and render it in `Ui::render` when open. (FR-007)
- [X] T014 [US3] In `src/app.rs`, add `pending_context_menu: Option<ContextMenu>`; on a Right press in the editor open it (clamped, only when no other modal/menu active — FR-010); handle keyboard (Up/Down focus, Enter/Space activate, Esc dismiss) and mouse (click item activates, outside dismisses); route items to Cut/Copy/Paste/SelectAll then close. (FR-008, FR-009, FR-010)
- [X] T015 [P] [US3] Integration test in `tests/integration/interaction.rs`: right-click opens the menu; activating Copy runs it and closes; Esc and outside-click dismiss; it does not open while another modal is active.

**Checkpoint**: context menu works by mouse + keyboard; `make check` green.

---

## Phase 6: Polish & cross-cutting

- [X] T016 [P] Update `CHANGELOG.md` (feature 030 under `[Unreleased]`), `docs/STATUS.md`, `docs/CAPABILITIES.md` (in-dialog clicks, double/triple-click, right-click menu, new F-keys).
- [X] T017 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check); fix findings; note the pre-existing F12/Ctrl+O PTY smoke failure is not a regression.
- [X] T018 Run the `specs/030-interaction-completeness/quickstart.md` manual walkthrough.
- [X] T019 On merge, the PR closes #53/#54/#55/#56 (reference them in the PR body); remove their ROADMAP "Deferred" rows (or mark Complete) since they ship here.

---

## Dependencies & Execution Order

- Setup → none. US4 → independent (do first). US2 → editor click path (independent). US1 → per-dialog
  (independent; shares `field_caret_at`/`ui::width`). US3 → adds the one new widget (independent).
- Polish → after the stories. Any subset is shippable (MVP: US4 + US2).

### Parallel opportunities

- All `[P]` unit/integration tests; T016 docs `[P]`.

## Implementation Strategy

TDD per story (Constitution V). No new crates (IV). Reuse `ui::width`, the dialog geometry helpers, the
`last_browser_click` timing pattern, the menu/button render+hit-test, and existing edit actions. Branch
`030-interaction-completeness`, PR to `master` (closes #53–#56), merge via GitHub. No AI attribution.
