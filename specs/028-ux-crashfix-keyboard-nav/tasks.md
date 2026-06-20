---
description: "Task list for feature 028 — UX crash-safety and keyboard navigation hardening"
---

# Tasks: UX crash-safety and keyboard navigation hardening

**Input**: Design documents from `specs/028-ux-crashfix-keyboard-nav/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/behavior.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V (Test-Gated Merges, NON-NEGOTIABLE). Tests first.

**Organization**: Setup → Foundational (shared crash-safety primitives) → US1 (restore no-panic) →
US2 (panic restores terminal) → US3 (Save-As typing) → US4 (arrow-key buttons) → US5 (Help keyboard) →
US6 (Home/End + list paging) → Polish.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files / independent)
- **[Story]**: US1..US6 per spec.md

## Path Conventions

Single-project Rust: edits in `src/ui/editor.rs`, `src/app.rs`, `src/diagnostics/crash.rs`,
`src/ui/buttons.rs`, `src/ui/file_browser.rs`, `src/input/keymap.rs`; new integration tests in
`tests/integration/ux_hardening.rs` (registered in `Cargo.toml`); unit tests inline.

---

## Phase 1: Setup

- [X] T001 Confirm a clean baseline build on branch `028-ux-crashfix-keyboard-nav` (`make tmpfs-setup` then `make`).
- [X] T002 Register a new integration test target `ux_hardening` in `Cargo.toml` (`[[test]] name="ux_hardening" path="tests/integration/ux_hardening.rs"`) and create an empty `tests/integration/ux_hardening.rs` so later tasks can append.

---

## Phase 2: Foundational — crash-safety primitives (Blocking)

**⚠️ CRITICAL**: the renderer must be panic-proof and the wrap cache must invalidate before/at the same
time the restore/switch paths are touched.

- [X] T003 [P] Unit tests in `src/ui/editor.rs`: rendering with a deliberately oversized/stale wrap cache (cached seg offsets larger than the line, plus an empty line) does NOT panic (build an `EditorWidget` over a small `TuiBuffer` and render).
- [X] T004 In `src/ui/editor.rs`, clamp every runtime line slice to the current line length: change `&line_str[seg_start..seg_end]` to use `seg_start.min(len)..seg_end.min(len)` (and `seg_start<=seg_end`), and audit the non-wrap render path for any other unchecked `&line_str[..]`/index; render truncated/blank instead of panicking.
- [X] T005 [P] Unit test in `src/app.rs`: a helper that invalidates the wrap cache bumps `wrap_text_gen` (value strictly changes).
- [X] T006 In `src/app.rs`, add `fn invalidate_wrap_cache(&mut self)` that bumps `wrap_text_gen` (wrapping_add(1)); call it from `do_restore_session`, `next_buffer`, `prev_buffer`, `handle_open_file`, and `close_buffer_at` so the cache rebuilds for the new active buffer.

**Checkpoint**: renderer is panic-proof and the wrap cache always matches the active buffer; `make check` green.

---

## Phase 3: User Story 1 — restoring a session never crashes (Priority: P1) 🎯

**Goal**: Session restore (and any active-buffer change) with soft-wrap on renders without panic.

**Independent Test**: with soft-wrap on, restore a session whose buffers differ in length from the
prior content → no panic, correct layout.

- [X] T007 [US1] Integration test in `tests/integration/ux_hardening.rs`: enable soft-wrap, populate a wrap cache against one buffer, then `do_restore_session` (or switch buffers) to content with shorter/empty lines and render via the app — asserts no panic and that `wrap_text_gen` changed across the switch.
- [X] T008 [US1] Verify (and adjust if needed) that the render loop's staleness rebuild (`WrapCache::is_stale`) picks up the bumped generation from T006 so the first post-restore frame uses a fresh cache. No new code if T004+T006 suffice; add a focused assertion test if a gap is found.

**Checkpoint**: US1 acceptance scenarios pass; `make check` green.

---

## Phase 4: User Story 2 — a crash leaves the terminal usable (Priority: P1)

**Goal**: On panic, restore the terminal before printing; still write the crash log.

**Independent Test**: the panic-hook restore path runs best-effort without panicking; `write_report`
still produces a full report.

- [X] T009 [P] [US2] Unit test in `src/diagnostics/crash.rs`: the terminal-restore helper (extracted) runs without panicking even when no terminal is initialized (headless), and `write_report` still emits the full report (existing test preserved).
- [X] T010 [US2] In `src/diagnostics/crash.rs`, extract a `restore_terminal_best_effort()` (execute `LeaveAlternateScreen`, `DisableMouseCapture`, `Show`; then `disable_raw_mode()`, ignoring all errors) and call it at the START of the panic hook closure, before writing to stderr. Keep the crash-file write unchanged.

**Checkpoint**: US2 passes; `make check` green.

---

## Phase 5: User Story 3 — Save-As typing works and is visible (Priority: P1)

**Goal**: Interactive dialogs open with focus on the primary control so typing reaches the field.

**Independent Test**: open the Save browser (and again after another dialog), type → chars accumulate;
caret shown.

- [X] T011 [P] [US3] Unit test in `src/app.rs`: after opening an interactive dialog (file browser) with `dialog_focus` pre-set to a button value, `ensure_dialog_focus()` resets `dialog_focus` to 0 (primary control) and `interactive_focus_is_button()` returns `None`.
- [X] T012 [US3] In `src/app.rs` `ensure_dialog_focus()`, extend initialization to interactive dialogs: when `interactive_dialog().is_some()` and `!dialog_focus_init`, set `dialog_focus = 0` (primary control) and set the init guard; preserve the existing button-dialog branch and the no-dialog reset.
- [X] T013 [US3] Integration test in `tests/integration/ux_hardening.rs`: open the Save browser, dispatch printable `InsertChar` actions → `file_browser.filename` accumulates them; render shows the filename + caret (reuse the feature-012 render assertion pattern).

**Checkpoint**: US3 passes; `make check` green.

---

## Phase 6: User Story 4 — arrow keys move between dialog buttons (Priority: P2)

**Goal**: Left/Right (and Up/Down) move focus across buttons in both ring families.

**Independent Test**: in each multi-button dialog, arrows advance/retreat focus with wrap.

- [X] T014 [P] [US4] Unit tests in `src/app.rs`: for a 016 confirm dialog, `MoveRight`/`MoveDown` advance `dialog_focus` (wrap) and `MoveLeft`/`MoveUp` retreat (wrap); for a 020 interactive dialog with a button focused, arrows cycle the ring.
- [X] T015 [US4] In `src/app.rs`, in the feature-016 button-dialog key intercept, map `Action::MoveRight|MoveDown` → `buttons::next` and `Action::MoveLeft|MoveUp` → `buttons::prev` (same as Tab/Shift+Tab).
- [X] T016 [US4] In `src/app.rs`, in the feature-020 interactive-dialog intercept, when `interactive_focus_is_button()` is `Some`, route the arrow keys through the ring `next/prev`; when the primary control is focused, leave existing arrow behavior (list/field) unchanged.

**Checkpoint**: US4 passes for every dialog family; `make check` green.

---

## Phase 7: User Story 5 — Help/About keyboard scroll & dismiss (Priority: P2)

**Goal**: Up/Down/PageUp/PageDown/Home/End scroll the overlay (clamped); Esc/Enter dismiss.

**Independent Test**: open Help on a short terminal → keys scroll within bounds; Esc closes.

- [X] T017 [P] [US5] Unit/integration test in `tests/integration/ux_hardening.rs`: with Help open and overflowing, `MoveLineEnd`(End) clamps `help_scroll` to the last page and `MoveLineStart`(Home) returns it to 0; PageDown/PageUp move by a page within bounds.
- [X] T018 [US5] In `src/app.rs` Help intercept, handle `Action::MoveLineStart` → `help_scroll = 0` and `Action::MoveLineEnd` → clamp to `max(0, total_lines - body_rows)`; confirm Up/Down/PageUp/PageDown clamp to content; keep Esc/Enter dismissal.

**Checkpoint**: US5 passes; `make check` green.

---

## Phase 8: User Story 6 — Home/End + list paging (Priority: P3)

**Goal**: Home/End move the editor cursor to line start/end; PageUp/Down page through lists.

**Independent Test**: editor Home/End move the cursor; long list PageUp/Down jump a page.

- [X] T019 [P] [US6] Unit tests: in `src/input/keymap.rs`, `default_map()` maps `Home`→`MoveLineStart` and `End`→`MoveLineEnd`; in `src/ui/file_browser.rs`, PageUp/Down move by a page and clamp; scroll uses `saturating_sub`.
- [X] T020 [US6] In `src/input/keymap.rs` `default_map()`, bind `Home`→`Action::MoveLineStart` and `End`→`Action::MoveLineEnd`.
- [X] T021 [US6] In `src/app.rs`, add `Action::PageUp`/`PageDown` handling to the file-browser, encoding-select, and plugin-manager key intercepts (page by visible rows, clamped); in `src/ui/file_browser.rs`, change the scroll arithmetic to `(self.selected + 1).saturating_sub(visible_rows)` (FR-004 file-browser hardening — paired with the copy guard in T022).
- [X] T021b [P] [US6] Unit tests in `src/app.rs` (resolves analyze C1): PageDown/PageUp on the **encoding-select** and **plugin-manager** lists move the cursor by a page and clamp to `[0, n-1]` (and are no-ops on an empty/single-item list).

**Checkpoint**: US6 passes; `make check` green.

---

## Phase 9: Polish & Cross-Cutting

- [X] T022 [P] In `src/app.rs` `copy_selection`, clamp the slice to `lo.min(hi)..hi.min(len)` so a degenerate/reversed selection yields empty text; add a unit test for an empty/reversed range. (FR-004 copy hardening — paired with the file-browser scroll fix in T021.)
- [X] T023 [P] Update `CHANGELOG.md` (feature 028 under `[Unreleased]`), `docs/STATUS.md`, and `docs/CAPABILITIES.md` (Home/End, arrow-key button movement, Help keyboard scroll, list PageUp/Down; crash-safety notes).
- [X] T024 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check) and fix findings; note the known pre-existing sandbox smoke failure (F12/Ctrl+O PTY delivery) is not a regression.
- [X] T025 Run the `specs/028-ux-crashfix-keyboard-nav/quickstart.md` manual walkthrough (restore no-crash, crash→usable terminal, Save-As typing, arrow buttons, Help keyboard, Home/End + paging).

---

## Dependencies & Execution Order

- **Setup (P1)** → none. **Foundational (P2)** → blocks US1 (shared crash-safety primitives).
- **US1** depends on Foundational. **US2/US3/US4/US5/US6** are largely independent of each other (different
  surfaces) and can proceed in priority order after Setup; US1 should land first (highest-severity crash).
- **Polish (P9)** → after the stories.

### Parallel opportunities

- T003/T005 (unit tests), T009/T011/T014/T017/T019 ([P] unit tests), and T022/T023 polish are `[P]`.

---

## Implementation Strategy

### MVP

Setup → Foundational → US1 (restore no-panic) + US2 (terminal restore): the two P1 crashes. Then US3
(Save-As typing, P1), then US4/US5 (P2), US6 (P3), Polish.

### Notes

- TDD mandatory (Constitution V). No new crates (Constitution IV). Reuse `buttons::next/prev`, existing
  scroll/cursor-move helpers, the `wrap_text_gen` staleness channel, and crossterm teardown calls.
- Keep AI attribution out of commits/PR/issues. Branch `028-ux-crashfix-keyboard-nav`, PR to `master`,
  merge via GitHub.
