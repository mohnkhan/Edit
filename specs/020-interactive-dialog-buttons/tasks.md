---
description: "Task list for feature 020 — boxed buttons + focus ring for the interactive/list dialogs"
---

# Tasks: Boxed buttons + focus ring for the interactive/list dialogs

**Input**: Design documents from `specs/020-interactive-dialog-buttons/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/focus-ring.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V (Test-Gated Merges, NON-NEGOTIABLE) mandates a failing
test before implementation for every user-visible behavior. Test tasks are written first in each phase.

**Organization**: Setup → Foundational (shared focus-ring dispatch) → one phase per dialog (each an
independently testable increment) → Polish. Each spec user story is a *facet* that applies to every
dialog: **US1** = mouse-clickable buttons, **US2** = Tab/Shift+Tab focus ring, **US3** = zero regression
to existing keys. Per-dialog tasks are labeled with the facet they serve.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files / independent, no dependency on an incomplete task)
- **[Story]**: US1 (mouse) / US2 (focus ring) / US3 (no-regression) — the spec facet the task serves

## Path Conventions

Single-project Rust layout: source under `src/`, integration tests under `tests/`, unit tests inline
(`#[cfg(test)]`) in the module they cover.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the build/test environment and re-read the reused button machinery.

- [x] T001 Run `make tmpfs-setup` then `make` to confirm a clean baseline build on branch `020-interactive-dialog-buttons` (target/ on tmpfs per project memory).
- [x] T002 Re-read `src/ui/buttons.rs` (`button_rects`/`render_buttons`/`hit_test_buttons`/`next`/`prev`) and the confirm-dialog wiring in `src/app.rs` (`dialog_focus`, `ensure_dialog_focus`, `button_dialog_rect`, `activate_dialog_button`, the `handle_action` Tab/Enter intercept, and the `handle_mouse_event` button hit-test) to confirm the patterns this feature mirrors. No code change.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the generic interactive-dialog focus-ring dispatch that every dialog phase plugs into.
This is additive scaffolding that delegates to per-dialog helpers (each returns `None` until its phase
implements it), so it compiles and is safe before any dialog is wired.

**⚠️ CRITICAL**: No dialog phase (3–6) can begin until this phase is complete.

- [x] T003 In `src/app.rs`, add an `enum InteractiveDialog { EncodingSelect, PluginManager, FindReplace, FileBrowser }` and `fn interactive_dialog(&self) -> Option<InteractiveDialog>` derived from the existing `pending_encoding_select` / `pending_plugin_manager` / `pending_find_replace` / `file_browser` state (mutually exclusive; document precedence).
- [x] T004 In `src/app.rs`, add per-dialog ring descriptor helpers that `match interactive_dialog()`: `fn interactive_field_stops(&self) -> usize`, `fn interactive_button_labels(&self) -> Vec<&'static str>`, and `fn interactive_ring_len(&self) -> usize` (= field_stops + labels.len()). Return defaults (0/empty) when no interactive dialog is open. Bodies for each dialog are filled by their phases; start with safe empty/`None` defaults (no `todo!()`) so it compiles.
- [x] T005 In `src/app.rs`, extend `ensure_dialog_focus()` so that when an interactive dialog is open it resets `dialog_focus = 0` once (default = primary control), reusing the existing `dialog_focus_init` guard.
- [x] T006 In `src/app.rs` `handle_action()`, add a generic ring intercept active when `interactive_dialog().is_some()` and `interactive_ring_len() > 1`: `Action::FocusNextField` → `dialog_focus = buttons::next(dialog_focus, ring_len)`; `Action::FocusPrevField` → `buttons::prev(...)`. Place it so each dialog's existing block can still consult `dialog_focus` (see per-dialog tasks). Add a `fn interactive_focus_is_button(&self) -> Option<usize>` returning `Some(dialog_focus - field_stops)` when `dialog_focus >= field_stops`.
- [x] T007 In `src/app.rs`, add `fn interactive_dialog_rect(&self) -> Option<Rect>` and `fn activate_interactive_button(&mut self, idx: usize)` that `match interactive_dialog()` and delegate to per-dialog rect/activate logic (filled by each phase; safe `None`/no-op defaults for now).
- [x] T008 In `src/app.rs` `handle_mouse_event()`, add a generic interactive-dialog button hit-test (before/independent of the confirm-dialog path): if `interactive_dialog().is_some()` and `interactive_dialog_rect()` is `Some(rect)`, compute `buttons::button_rects(rect, &labels)` and on `hit_test_buttons` hit call `activate_interactive_button(i)`. For the file browser, ensure this runs before the existing entry hit-test (buttons win).
- [x] T009 [P] Add inline unit tests in `src/app.rs` for the ring math with stubbed descriptors: `interactive_ring_len`, `interactive_focus_is_button` boundaries, and `next/prev` wrap over a ring (reuse `buttons::next/prev`). These pass once T004/T006 land.

**Checkpoint**: Generic dispatch compiles; `cargo test` green; no behavioral change yet (all per-dialog descriptors still default).

---

## Phase 3: Encoding-select dialog (Priority: P1) 🎯 MVP

**Goal**: The encoding selector gains OK/Cancel boxed buttons, a `[List, OK, Cancel]` focus ring, mouse
clicks, and unchanged list keys — the smallest end-to-end proof of the pattern.

**Independent Test**: Open the encoding dialog; `Tab` cycles List→OK→Cancel→List; `Up/Down` move the
selection only while the list is focused; clicking/Enter on OK applies the highlighted encoding; Cancel
and `Esc` close with no change.

### Tests for Encoding (write first, must fail)

- [x] T010 [P] [US2] Integration test in `tests/dialog_encoding_buttons.rs`: opening the dialog sets `dialog_focus = 0`; `FocusNextField` ×3 returns to 0; ring length is 3 (1 field + OK/Cancel).
- [x] T011 [P] [US1] Integration test in `tests/dialog_encoding_buttons.rs`: a simulated click on the OK button rect applies the selected encoding (assert via the same effect as `Enter` on the list); a click on Cancel closes with no encoding change.
- [x] T012 [P] [US3] Integration test in `tests/dialog_encoding_buttons.rs`: with focus on the list, `Up`/`Down`/`Enter`/`Esc` behave exactly as before this feature; with focus on a button, `Up`/`Down` are a no-op.

### Implementation for Encoding

- [x] T013 [US2] In `src/app.rs`, fill the encoding arms of `interactive_field_stops` (=1) and `interactive_button_labels` (=`["OK","Cancel"]`).
- [x] T014 [US1] In `src/app.rs`, implement the encoding arms of `interactive_dialog_rect` (the dialog's outer Rect, grown for the button row) and `activate_interactive_button` (0→apply `ENCODING_OPTIONS[selected]` + close, 1→close).
- [x] T015 [US2][US3] In `src/app.rs`, update the existing `pending_encoding_select` intercept block so `Up`/`Down`/`Enter` act only when `interactive_focus_is_button()` is `None` (list focused); when a button is focused, `Enter`/`Space` activate it (`Up`/`Down` no-op). `Esc` still closes from any focus.
- [x] T016 [US1][US2] In `src/ui/mod.rs` and/or `src/ui/dialog.rs`, grow the encoding dialog by the button row and render the buttons via `buttons::render_buttons` using the same Rect as T014, focusing the button only when `interactive_focus_is_button()` is `Some` (else keep the list's existing highlight).
- [x] T017 [P] [US1][US2] Inline unit test in `src/ui/dialog.rs` (or `mod.rs`): the encoding dialog's outer Rect grows to fit the button row and does not panic on a tiny terminal; button rects are non-empty at a normal size.

**Checkpoint**: Encoding dialog fully mouse+keyboard operable with no regressions; `make check` green. MVP demoable.

---

## Phase 4: Plugin-manager dialog (Priority: P2)

**Goal**: Plugin manager gains a Close button, a `[List, Close]` ring, mouse, unchanged list keys.

**Independent Test**: `Up/Down` move the cursor and `Space` toggles only while the list is focused;
`Tab` reaches Close; Enter/click on Close (or `Esc`) closes; empty plugin list still reaches Close.

### Tests for Plugin manager (write first, must fail)

- [x] T018 [P] [US2] Integration test in `tests/dialog_plugin_buttons.rs`: ring length 2; `Tab` cycles List→Close→List; default focus 0.
- [x] T019 [P] [US1] Integration test in `tests/dialog_plugin_buttons.rs`: click on the Close button rect closes the manager.
- [x] T020 [P] [US3] Integration test in `tests/dialog_plugin_buttons.rs`: with the list focused `Up`/`Down`/`Space`(toggle)/`Esc` behave as before; with Close focused `Space`/`Enter` close and `Up`/`Down` no-op; empty registry still allows Close.

### Implementation for Plugin manager

- [x] T021 [US2] In `src/app.rs`, fill the plugin-manager arms of `interactive_field_stops` (=1) and `interactive_button_labels` (=`["Close"]`).
- [x] T022 [US1] In `src/app.rs`, implement the plugin-manager arms of `interactive_dialog_rect` and `activate_interactive_button` (0→close).
- [x] T023 [US2][US3] In `src/app.rs`, update the `pending_plugin_manager` intercept so list keys (`Up`/`Down`/`Space`/`Enter` toggle) act only when the list is focused; when Close is focused, `Enter`/`Space` close and `Up`/`Down` no-op.
- [x] T024 [US1][US2] In `src/ui/mod.rs` / `src/ui/plugin_manager.rs`, grow the dialog by the button row and render the Close button with focus-aware highlight using the shared Rect.

**Checkpoint**: Plugin manager fully operable; `make check` green.

---

## Phase 5: File-browser dialog (Priority: P2)

**Goal**: File browser gains Open/Save + Cancel buttons, a `[Browser, Open|Save, Cancel]` ring, with its
existing mouse entry handling preserved (buttons hit-tested first).

**Independent Test**: `Tab` cycles Browser→Open/Save→Cancel→Browser; clicking Open/Save activates the
selection; Cancel/outside-click/`Esc` closes; entry single/double-click still works; nav keys unchanged.

### Tests for File browser (write first, must fail)

- [x] T025 [P] [US2] Integration test in `tests/dialog_browser_buttons.rs`: ring length 3; label is "Open" in open mode, "Save" in save-as mode; default focus 0.
- [x] T026 [P] [US1] Integration test in `tests/dialog_browser_buttons.rs`: click on the Open/Save button rect applies the same `BrowseOutcome` as `Enter`; click on Cancel closes; a click on the buttons takes precedence over the entry hit-test.
- [x] T027 [P] [US3] Integration test in `tests/dialog_browser_buttons.rs`: `Up`/`Down`/`Left`/`Right`/typing/`Backspace` and entry double-click behave as before; with a button focused `Up`/`Down` no-op; `Esc`/outside-click still close.

### Implementation for File browser

- [x] T028 [US2] In `src/app.rs`, fill the file-browser arms of `interactive_field_stops` (=1) and `interactive_button_labels` (mode-aware: `["Open","Cancel"]` or `["Save","Cancel"]`).
- [x] T029 [US1] In `src/ui/file_browser.rs`, extend `compute_layout()` to reserve the button row and add a button-rects accessor + a button hit-test; in `src/app.rs` implement the file-browser arms of `interactive_dialog_rect` and `activate_interactive_button` (0→apply `activate()` outcome, 1→close).
- [x] T030 [US2][US3] In `src/app.rs`, update the `file_browser` key intercept so nav/edit keys act only when the browser is focused; with a button focused `Enter`/`Space` activate it and `Up`/`Down` no-op; ensure the T008 mouse path hit-tests buttons before entries.
- [x] T031 [US1][US2] In `src/ui/file_browser.rs` / `src/ui/mod.rs`, render the Open/Save + Cancel buttons with focus-aware highlight using the layout from T029 (drawn == hit-tested).

**Checkpoint**: File browser fully operable incl. mouse precedence; `make check` green.

---

## Phase 6: Find/Replace dialog (Priority: P3)

**Goal**: Find/Replace gains a combined field+button ring — Find mode `[Query, Find, Close]`, Replace
mode `[Query, Replacement, Find, Replace, Replace All, Close]` — with all editing/options/match-nav keys
preserved and `FindReplaceDialog.focus` kept in sync with the ring's field stops.

**Independent Test**: In replace mode `Tab` cycles Query→Replacement→Find→Replace→Replace All→Close→Query;
typing/`Alt+C/A/R/W`/`F3`/`F2`/`Enter`-per-mode still work on field stops; clicking each button runs its
existing action; `Esc`/Close closes.

### Tests for Find/Replace (write first, must fail)

- [x] T032 [P] [US2] Integration test in `tests/dialog_findreplace_buttons.rs`: ring length 3 in Find mode and 6 in Replace mode; `Tab`/`Shift+Tab` visit each stop once and wrap; field stops 0/1 keep `FindReplaceDialog.focus` in sync (Query/Replacement).
- [x] T033 [P] [US1] Integration test in `tests/dialog_findreplace_buttons.rs`: clicking Find/Replace/Replace All/Close runs the same actions as `run_find_from_dialog`/`replace_current_from_dialog`/`replace_all_from_dialog`/`close_find_replace`.
- [x] T034 [P] [US3] Integration test in `tests/dialog_findreplace_buttons.rs`: with a field focused, typing, `Backspace`, `Left`/`Right`, `Alt+C/A/R/W`, `F3`/`F2`, `Enter` (per-mode), and `Ctrl+A` (`Action::SelectAll` → replace-all in Replace mode) behave exactly as before; with a button focused, text-editing keys do not mutate fields; `Esc` closes from any stop.

### Implementation for Find/Replace

- [x] T035 [US2] In `src/app.rs`, fill the Find/Replace arms of `interactive_field_stops` (1 in Find mode, 2 in Replace mode) and `interactive_button_labels` (mode-aware: `["Find","Close"]` / `["Find","Replace","Replace All","Close"]`).
- [x] T036 [US2] In `src/app.rs`, when `dialog_focus` lands on a field stop, set `FindReplaceDialog.focus` to the matching `DialogField` (stop 0→Query, stop 1→Replacement); remove the now-subsumed `switch_focus()`-only path so `Tab` drives the whole ring.
- [x] T037 [US1] In `src/app.rs`, implement the Find/Replace arms of `interactive_dialog_rect` (extract the rect currently computed inline in `src/ui/mod.rs` into a shared helper both render and mouse call) and `activate_interactive_button` (map indices to find/replace/replace-all/close).
- [x] T038 [US2][US3] In `src/app.rs`, update the `pending_find_replace` intercept so editing/option/match-nav keys (including `Ctrl+A`/`Action::SelectAll` → replace-all in Replace mode) apply when a field stop is focused; when a button is focused, `Enter`/`Space` activate it and text-editing keys are ignored; `Esc` always closes.
- [x] T039 [US1][US2] In `src/ui/mod.rs`, grow the dialog by the button row and render the Find/Replace buttons (mode-dependent set) with focus-aware highlight using the shared rect from T037; keep the focused field's caret only when a field stop is focused.

**Checkpoint**: All four dialogs fully operable by mouse + keyboard with no regressions.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [x] T040 [P] Add a smoke/headless assertion (extend the existing smoke suite) that each of the four dialogs renders a boxed button row with exactly one focused control.
- [x] T040b [P] Add a geometry test (inline unit or `tests/`) covering SC-005/FR-012: each dialog's outer Rect and button rects recompute correctly across a range of terminal sizes (small→full-screen) and after a simulated resize, with no panic and drawn==hit-tested; include a wide/UTF-8 label width check.
- [x] T041 [P] Update `CHANGELOG.md` (feature 020 entry under `[Unreleased]`) and `docs/STATUS.md`; update `docs/CAPABILITIES.md` (new mouse/Tab affordances on the four dialogs are user-visible). Docs gate.
- [x] T042 [P] Update `ROADMAP.md`: mark the issue-#38 row Complete (feature 020) and note no remaining dialog-button deferrals.
- [x] T043 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check) and fix any findings.
- [x] T044 Run the `specs/020-interactive-dialog-buttons/quickstart.md` manual walkthrough for all four dialogs (mouse + Tab + Esc + resize).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (P1)**: no dependencies.
- **Foundational (P2)**: depends on Setup; **blocks all dialog phases 3–6**.
- **Dialog phases (3–6)**: each depends only on Foundational; they are mutually independent and may be
  done in any order or in parallel (each touches distinct intercept blocks/render arms, plus its own test
  file). Risk-first order: Encoding (MVP) → Plugin manager → File browser → Find/Replace.
- **Polish (7)**: depends on all dialog phases intended for the release.

### Within Each Dialog Phase

- Write the three facet tests first (they must fail) → fill descriptor arms → wire keys → wire mouse →
  render → unit test geometry. Commit per phase.

### Parallel Opportunities

- T009 / the per-phase `[P]` test tasks are independent (separate test files).
- Dialog phases 3–6 can run in parallel once Foundational lands (distinct files, distinct match arms).
- Polish T040/T041/T042 are `[P]` (distinct files).

---

## Implementation Strategy

### MVP (Encoding dialog)

1. Phase 1 Setup → 2. Phase 2 Foundational (CRITICAL) → 3. Phase 3 Encoding → **validate independently**
   → demo. This proves the whole pattern (ring + buttons + mouse + no-regression) on the simplest dialog.

### Incremental delivery

Add Plugin manager → File browser → Find/Replace, each independently tested and committed, then Polish.
All four can also be parallelized across developers after Foundational.

---

## Notes

- TDD is mandatory (Constitution V): every behavior gets a failing test first.
- No new `Action`s or persisted state — buttons dispatch existing actions; `dialog_focus` is reused.
- Geometry invariant: the Rect feeding `render_buttons` must equal the Rect feeding `hit_test_buttons`.
- Keep AI attribution out of all commits/PR/issue text (project rule).
- Follow the project git workflow: branch `020-interactive-dialog-buttons`, PR to `master`, merge via GitHub.
