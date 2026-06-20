---
description: "Task list for File Browser Dialogs (feature 012)"
---

# Tasks: File Browser Dialogs

**Input**: Design documents from `specs/012-file-browser-dialogs/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/file-browser-interaction.md

**Tests**: INCLUDED — Constitution Principle V (Test-Gated Merges) is NON-NEGOTIABLE, so each story
ships with tests.

**Organization**: Grouped by user story. US1 (Open) and US2 (Save) are both P1; US3 (consistent
keyboard+mouse + scrolling) is P2 and builds on them.

## Path Conventions

Single project: `src/`, `tests/` at repo root. New module: `src/ui/file_browser.rs`.

---

## Phase 1: Setup

- [X] T001 Declare the new module `pub mod file_browser;` in `src/ui/mod.rs` and create an empty
  `src/ui/file_browser.rs` with the module doc comment and `use` of ratatui + `crate::ui::theme::Theme`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The `FileBrowser` model + pure logic that BOTH Open and Save depend on. No app wiring yet.

- [X] T002 [P] Define `BrowseMode`, `EntryKind`, `Entry`, and `Outcome` enums/structs in
  `src/ui/file_browser.rs` per `data-model.md`.
- [X] T003 Define the `FileBrowser` struct (mode, cwd, entries, selected, scroll, filename, error)
  and `FileBrowser::open(start_dir, mode)` (canonicalize with CWD then `/` fallback) in
  `src/ui/file_browser.rs`.
- [X] T004 Implement `reload()` — `std::fs::read_dir(cwd)`, build entries, sort `..` → dirs → files
  (case-insensitive alpha), include dot-files, prepend `..` unless at root; on error set `error`
  and keep prior entries — in `src/ui/file_browser.rs`.
- [X] T005 Implement navigation: `move_up`/`move_down` (clamped, scroll-aware via `visible_rows`),
  `enter_parent` (via `cwd.parent()`, no-op at root) in `src/ui/file_browser.rs`.
- [X] T006 Implement `activate`/`activate_index` returning `Outcome` (Navigated / OpenFile /
  SaveFile / None) and `selected_save_path` (validate dir via `validate_path`, reject empty /
  separator / `..` filenames, return `cwd.join(filename)`) in `src/ui/file_browser.rs`.
- [X] T007 [P] Implement grapheme/width-aware `truncate_name(name, max_cols)` (reuse the existing
  wide-char width approach; append `…`) in `src/ui/file_browser.rs`.
- [X] T008 [P] Unit tests for the model in `src/ui/file_browser.rs` (`#[cfg(test)]`): sort order +
  dot-files, `move_*` clamping + scroll, `enter_parent` (incl. root no-op), `selected_save_path`
  validation (empty/separator/`..` rejected), `truncate_name` never splits a multi-byte char, and
  **the `reload()` error path** — a non-existent/unreadable `cwd` sets `error`, keeps prior entries,
  and never panics (covers FR-013 / SC-005).

**Checkpoint**: model compiles and its unit tests pass independently of the app.

---

## Phase 3: User Story 1 — Open a file by browsing (Priority: P1) 🎯 MVP

**Goal**: File ▸ Open / `Ctrl+O` shows a navigable listing; choosing a file loads it.

**Independent test**: Launch, open the browser, navigate into a subdir, select a file → it loads;
Esc cancels with no change.

- [X] T009 [US1] In `src/app.rs`, add `file_browser: Option<FileBrowser>` to `App` (init `None`) and
  a `browser_start_dir()` helper (active buffer's parent dir, else CWD).
- [X] T010 [US1] Route `Action::Open` in `handle_action` to
  `self.file_browser = Some(FileBrowser::open(self.browser_start_dir(), BrowseMode::Open))`
  in `src/app.rs`.
- [X] T011 [US1] Add the modal intercept block for `file_browser` in `handle_action` (among the
  existing modal guards, before the menu-bar guard): ↑/↓ move, Enter/→ activate, ←/Backspace parent,
  printable → filename/path field, Esc cancel; on `Outcome::OpenFile(p)` call `handle_open_file(p)`
  and close — in `src/app.rs`. In **Open** mode, when the path field holds an absolute path on Enter,
  jump the browser there (cd if a directory, else open it as a file); if it is not a valid path, fall
  back to activating the highlighted entry (covers FR-006a).
- [X] T012 [US1] Implement `FileBrowserWidget` (bordered box, dir-path header, scrollable entry list
  with selection highlight + dir/file markers, footer hints) in `src/ui/file_browser.rs`, and render
  it from `src/ui/mod.rs` when `app.file_browser` is `Some`.
- [X] T013 [US1] Integration test `tests/integration/file_browser.rs`: `Action::Open` opens the
  browser; driving Down/Enter into a temp subdir then Enter on a file results in a new buffer with
  that file's contents; Esc leaves buffers unchanged.

**Checkpoint**: Open-by-browsing works end-to-end via keyboard.

---

## Phase 4: User Story 2 — Save to a chosen folder by browsing (Priority: P1)

**Goal**: File ▸ Save As (and `Ctrl+S` on an unnamed buffer) shows the browser; choosing a folder +
filename writes the buffer.

**Independent test**: Type text, Save As, navigate to a temp dir, enter a filename, Enter → file
exists on disk with the buffer contents.

- [X] T014 [US2] Route `Action::SaveAs` to open the browser in `BrowseMode::Save` in `src/app.rs`.
- [X] T015 [US2] In `handle_save_action` (`src/app.rs`), when the active buffer has no path, open the
  Save browser instead of failing.
- [X] T016 [US2] In the `file_browser` intercept, handle Save confirmation: Enter with a non-dir
  selection / filename field → `selected_save_path()` → on Ok call `do_save_as(path)` and close; on
  Err set the browser `error` and stay open; selecting an existing file populates the filename field
  — in `src/app.rs`.
- [X] T017 [US2] Extend `FileBrowserWidget` to render the filename input line in Save mode
  (`src/ui/file_browser.rs`).
- [X] T018 [US2] Integration tests in `tests/integration/file_browser.rs`: Save-by-browse writes the
  file with buffer contents; empty filename is a no-op; `Ctrl+S`/`handle_save_action` on an unnamed
  buffer opens the Save browser.

**Checkpoint**: Save-by-browsing works end-to-end via keyboard.

---

## Phase 5: User Story 3 — Consistent keyboard & mouse + scrolling (Priority: P2)

**Goal**: Mouse drives the browser identically to the keyboard; long listings scroll.

**Independent test**: Perform an open-nested-file task with mouse only and keyboard only → identical
results; a listing longer than the window scrolls to keep the selection visible.

- [X] T019 [US3] Add a hit-test method on `FileBrowser` mapping a terminal `(col,row)` + box
  geometry to `Option<entry_index>` / outside, sharing the widget's geometry, in
  `src/ui/file_browser.rs`.
- [X] T020 [US3] Extend `App::handle_mouse_event` (`src/app.rs`): when `file_browser` is open and the
  browser isn't behind a higher modal, left-press inside → `activate_index`; outside → cancel.
- [X] T021 [US3] Ensure scroll math keeps `selected` visible for a listing taller than the box (in
  `move_up`/`move_down` and at render); covered by a unit test in `src/ui/file_browser.rs`.
- [X] T022 [US3] Unit test for the hit-test (entry row → index; outside → None) in
  `src/ui/file_browser.rs`; integration test that a synthetic left-click opens a folder then a file.

**Checkpoint**: Mouse + keyboard parity and scrolling verified.

---

## Phase 6: Polish & Cross-Cutting

- [X] T023 Remove the superseded `OpenFileDialog` and `SaveAsFileDialog` widgets from
  `src/ui/dialog.rs`, the `pending_open` / `pending_save_as` fields and their old intercept/render
  blocks from `src/app.rs` and `src/ui/mod.rs`, and update/replace the old feature-010/011 tests that
  referenced them.
- [X] T024 [P] Add headless smoke test `tests/smoke/file_browser.exp` (keyboard navigate + open a
  file confirmed via `--debug` log; deterministic, no screen-scraping).
- [X] T025 [P] Docs gate: update `CHANGELOG.md` (feature 012), `docs/STATUS.md` (F012 rows), and
  `docs/CAPABILITIES.md` (file browser replaces path-text dialogs; keyboard + mouse).
- [X] T026 Verification: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` all green; live expect check per `quickstart.md`.

---

## Dependencies & Execution Order

- **Setup (T001)** → **Foundational (T002–T008)** block everything.
- **US1 (T009–T013)** depends on Foundational; is the MVP.
- **US2 (T014–T018)** depends on Foundational; mostly independent of US1 (shares the intercept block
  added in T011 — coordinate edits to that block).
- **US3 (T019–T022)** depends on the widget (T012) and app wiring (T009–T011).
- **Polish (T023–T026)** last; T023 removal must follow US1/US2 render+wiring being in place.

## Parallel Opportunities

- T002 and T007 are `[P]` (independent items in the new module before the struct logic lands).
- T008 (model tests) can be written alongside T002–T007.
- T024 and T025 are `[P]` (smoke test vs docs, different files).

## Implementation Strategy

- **MVP = Phase 1 + 2 + US1** (open a file by browsing). Ship-able and independently testable.
- Then US2 (save), then US3 (mouse parity + scrolling), then Polish (remove old dialogs + docs +
  full gate).
