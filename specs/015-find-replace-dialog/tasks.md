---
description: "Task list for feature 015: Interactive Find and Replace dialogs"
---

# Tasks: Interactive Find and Replace dialogs

**Input**: Design documents from `specs/015-find-replace-dialog/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/find-replace-interaction.md

**Tests**: INCLUDED — Constitution Principle V; TDD mandatory.

**Organization**: by user story (US1 Find, US2 Next/Prev, US3 Replace, US4 Options).

## Path Conventions

Single Rust project. Primary files: `src/search/mod.rs`, `src/app.rs`, `src/ui/dialog.rs`,
`src/ui/editor.rs`, `src/ui/mod.rs`, `src/input/keymap.rs`.

---

## Phase 1: Setup

- [x] T001 Create integration test file `tests/integration/find_replace.rs` (scaffold mirroring
  `tests/integration/undo_clean_revert.rs`) and register a `[[test]]` target in `Cargo.toml`.

---

## Phase 2: Foundational (dialog state + input routing skeleton)

- [x] T002 Define the dialog model in `src/ui/dialog.rs` (or `src/app.rs`): `DialogMode` (Find|Replace),
  `Field` (Query|Replacement), and a `FindReplaceDialog` struct (query, replacement, focus, caret,
  case_sensitive, wrap, regex, whole_word) per data-model.md, with grapheme-aware field-edit helpers
  (insert char, backspace, move caret left/right, switch focus).
- [x] T003 [P] Unit tests for the field-edit helpers in `src/ui/dialog.rs`: insert at caret, backspace,
  caret clamping, Tab focus switch, all grapheme/UTF-8-correct (e.g. multibyte input).
- [x] T004 Add `pending_find_replace: Option<FindReplaceDialog>` to `App` (init `None`); add it to the
  mouse modal-guard list in `handle_mouse_event` so clicks are ignored while open.
- [x] T005 Add the dialog keyboard intercept in `App::handle_action` (with the other modal guards):
  printable→insert, Backspace, Left/Right, Tab (Replace), Esc→close+clear highlights; route Enter /
  Ctrl+A / F3 / F2 / Alt+C/W/G/O to the per-story handlers added below. Consumes all input; returns early.

**Checkpoint**: dialog opens/closes and edits its fields (T003 green); no search behavior yet.

---

## Phase 3: User Story 1 — Find and jump (Priority: P1)

- [x] T006 [US1] Wire `Action::Find` (Ctrl+F / Search ▸ Find) to open the Find dialog (seed query from
  last search), replacing the current stub in `src/app.rs`.
- [x] T007 [US1] Implement "run search" on Enter: copy dialog query+options into `search_state`, run
  `SearchEngine::find_all`, set `active_match` to the first match at/after the cursor, `scroll_to_match`;
  empty query → no-op; no matches → "not found" status, document unchanged.
- [x] T008 [US1] Render the Find dialog interactively in `src/ui/dialog.rs` + dispatch in `src/ui/mod.rs`:
  show the editable query field with a visible caret, the option toggle states, and the "X of Y" / "not
  found" indicator.
- [x] T009 [US1] Integrate match highlighting: add `match_ranges` + `active_match` to `EditorWidget`
  (`src/ui/editor.rs`), overlay match background on cells within ranges (current match distinct via
  `collect_match_spans` styles), and feed them from `src/ui/mod.rs` only while a search is active.
- [x] T010 [P] [US1] Integration test in `tests/integration/find_replace.rs`: open Find, type a present
  term, Enter → `search_state.matches` populated, `active_match` set, cursor moved to first match; type a
  missing term → no matches, cursor unchanged; Esc → dialog closed.

**Checkpoint**: US1 — type a term, find it, see highlights + count.

---

## Phase 4: User Story 2 — Next/Prev with wrap (Priority: P1)

- [x] T011 [P] [US2] Integration test: after a search with N matches, F3 advances current match through
  all N and wraps; F2 goes backward and wraps; the "X of Y" current index updates correctly.
- [x] T012 [US2] Ensure F3/F2 while the dialog is open route to `find_next`/`find_prev` and update the
  dialog indicator; confirm keymap has `F3`→FindNext and `F2`→FindPrev (already mapped) and that these
  also work after closing the dialog (existing behavior preserved).

**Checkpoint**: US2 — stepping/wrapping works with correct indicator.

---

## Phase 5: User Story 3 — Replace (Priority: P2)

- [x] T013 [US3] Wire `Action::FindReplace` (Ctrl+H / Search ▸ Find Replace) to open the Replace dialog
  (query + replacement fields), replacing the stub in `src/app.rs`.
- [x] T014 [US3] Implement "Replace current" (Enter in Replace mode): replace the current match via the
  normal edit path (undoable; marks modified — integrates with feature 014), recompute matches, advance
  to the next; report via status. Implement "Replace All" (Ctrl+A): run `replace_all`, report count,
  recompute.
- [x] T015 [US3] Render the Replace dialog interactively (two fields with focus indicator + caret,
  toggles, count indicator) in `src/ui/dialog.rs` + `src/ui/mod.rs`.
- [x] T016 [P] [US3] Integration test: Replace All replaces every occurrence and reports the count
  (zero remaining); single Replace replaces the current and advances; Undo restores the pre-replace
  document; replacing with no matches reports zero and changes nothing.

**Checkpoint**: US3 — replace current/all, undoable, correct counts.

---

## Phase 6: User Story 4 — Options incl. whole-word (Priority: P3)

- [x] T017 [P] [US4] Unit tests in `src/search/mod.rs` for whole-word matching: "cat" matches the word
  "cat" but not "category"/"scatter"; boundaries at start/end of document count as non-word; works with
  case-insensitive and with regex candidates; UTF-8 safe.
- [x] T018 [US4] Add `whole_word: bool` to `SearchState` and word-boundary filtering to
  `SearchEngine::find_all` (and its callers/signature) in `src/search/mod.rs`.
- [x] T019 [US4] Add four `Action::ToggleSearch{Case,Wrap,Regex,WholeWord}` variants + keymap bindings
  `Alt+C` / `Alt+A` / `Alt+R` / `Alt+W` (all currently free; Alt+O stays Options). Handle them in the
  dialog intercept (toggle the dialog option + re-run search); render their on/off state; they are inert
  no-ops outside an open dialog.
- [x] T020 [P] [US4] Integration test: case-sensitive on/off changes matches; whole-word on excludes
  substrings; wrap off (if exposed) stops at the last match; toggling + re-run updates results.

**Checkpoint**: US4 — all four options work.

---

## Phase 7: Polish & Cross-Cutting

- [x] T021 [P] Update `CHANGELOG.md` (`[Unreleased]` → feature 015: interactive Find + Replace dialogs).
- [x] T022 [P] Update `docs/STATUS.md` (F015 rows) and `docs/CAPABILITIES.md` (Search menu + Find/Replace
  keys, options, highlight).
- [x] T023 [P] Add a tmux smoke test `tests/smoke/find_replace.exp` (open Find, type, Enter, Esc, quit
  clean) and register it if the smoke harness enumerates files.
- [x] T024 Run the full gate: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
  smoke; then live-verify in tmux per `quickstart.md` (Ctrl+F find + highlight; Ctrl+H replace-all).

---

## Dependencies & Execution Order

- Setup (T001) → Foundational (T002–T005) blocks all stories.
- US1 (T006–T010) depends on Foundational. US2 (T011–T012) depends on US1 (needs a result set).
  US3 (T013–T016) depends on Foundational + US1 (operates on matches) and integrates with feature 014
  undo. US4 (T017–T020) depends on Foundational; whole-word engine work (T018) is independent of the UI.
- Polish (T021–T024) last.

## Parallel Opportunities

- T003, T017 (unit tests, different files); T010, T011, T016, T020 (integration scenarios);
  T021, T022 (docs).

## Implementation Strategy

- **MVP = Foundational + US1 + US2** (P1): working interactive Find with highlight + navigation.
- **Increment = US3** (P2): Replace.
- **Increment = US4** (P3): options incl. whole-word.
- Each phase green before the next (test-gated).
