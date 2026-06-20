---
description: "Task list for feature 021 — scroll affordances + dialog button polish"
---

# Tasks: Scroll affordances + dialog button polish

**Input**: Design documents from `specs/021-scroll-affordances/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/scroll-affordances.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V (Test-Gated Merges, NON-NEGOTIABLE). Test tasks come first
in each phase.

**Organization**: Setup → Foundational (shared scrollbar helper) → US1 (scrollbars across views) → US2
(Help/About Close button) → US3 (key-hint labels) → Polish. US1 is split per surface; the editor sub-phase
carries the geometry-reservation risk.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files, no dependency on an incomplete task)
- **[Story]**: US1 (scrollbars) / US2 (Help Close) / US3 (key hints)

## Path Conventions

Single-project Rust: source under `src/`, integration tests under `tests/integration/`, unit tests inline.

---

## Phase 1: Setup

- [x] T001 Run `make tmpfs-setup` then `make` to confirm a clean baseline build on branch `021-scroll-affordances`.
- [x] T002 Re-read `src/ui/buttons.rs`, `src/ui/wrap.rs` (`total_visual_rows`), the editor geometry consumers in `src/app.rs` (`viewport_height` ~487, horizontal content-width helper ~3657, `handle_mouse_click` ~3424), and confirm `ratatui::widgets::{Scrollbar, ScrollbarState, ScrollbarOrientation}` is available in ratatui 0.26. No code change.

---

## Phase 2: Foundational — shared scrollbar helper (Blocking for US1)

**Purpose**: One wrapper over ratatui `Scrollbar` used by every scrollable view.

**⚠️ CRITICAL**: Blocks all US1 surface tasks (Phase 3).

- [x] T003 [P] Unit tests in `src/ui/scrollbar.rs`: `vertical`/`horizontal` draw nothing when `content_len <= viewport_len`; render and position a thumb when content overflows; no panic on a 1×1 / zero area; `position` clamped to `[0, content_len]`.
- [x] T004 Create `src/ui/scrollbar.rs` with `pub fn vertical(buf, area, content_len, viewport_len, position)` and `pub fn horizontal(...)`, each building a `ScrollbarState` and rendering a `Scrollbar` (`VerticalRight` / `HorizontalBottom`) only on overflow; centralize begin/end symbols + theme style. Register `pub mod scrollbar;` in `src/ui/mod.rs`.

**Checkpoint**: helper compiles, unit tests green, nothing else changed.

---

## Phase 3: User Story 1 — scrollbars across views (Priority: P1) 🎯 MVP

**Goal**: Every overflowing view (editor V+H, file browser, Help/About, encoding, plugin) shows an
accurate scrollbar; no content is hidden behind a bar.

**Independent Test**: open a long directory / large file / overflowing dialog → a scrollbar with a
thumb tracking the scroll offset; content that fits → no bar; nothing drawn under a bar.

### File browser (smallest end-to-end proof)

- [x] T005 [P] [US1] Integration/unit test in `src/ui/file_browser.rs`: the list scrollbar is drawn only when `entries.len() > list_rows`, the thumb reflects `scroll`, and entry names never occupy the reserved bar column.
- [x] T006 [US1] In `src/ui/file_browser.rs`, reserve the rightmost interior list column and render the vertical scrollbar via `scrollbar::vertical(scroll, list_rows, entries.len())` when overflowing; shrink the entry name budget by 1 so names don't draw under the bar. Keep `hit_test` correct.

### Editor (vertical + horizontal; geometry reservation)

- [x] T007 [P] [US1] Unit tests in `src/app.rs`: `viewport_height()` subtracts the reserved horizontal-bar row in non-wrap mode; `handle_mouse_click` on a reserved bar cell does not move the cursor; the horizontal content-width helper subtracts the reserved vertical-bar column.
- [x] T008 [US1] In `src/ui/mod.rs`, reserve 1 right column (vertical bar) and, in non-wrap mode, 1 bottom row (horizontal bar) of each editor pane (single + both split panes) and pass the shrunk area to `EditorWidget`; render the bars in the reserved strip using `scrollbar::*` (V: lines or `total_visual_rows`; H non-wrap: visible-line max width vs content width).
- [x] T009 [US1] In `src/app.rs`, update `viewport_height()` (~487), the horizontal content-width helper (~3657), and `handle_mouse_click` (~3424) to account for the reserved column/row so cursor-visibility, paging, and click mapping match the drawn area. The bottom-row (horizontal-bar) reservation is **conditional on `!soft_wrap`** (no horizontal bar in soft-wrap), so `viewport_height()` only subtracts that row in non-wrap mode; the right-column reservation is unconditional.
- [x] T010 [US1] In `src/ui/editor.rs`, expose the max visual width of the currently visible lines (computed during the render walk) so `src/ui/mod.rs` can size the horizontal bar without a full-file scan; ensure no horizontal bar in soft-wrap mode.

### Help/About + list dialogs

- [x] T011 [P] [US1] In `src/ui/mod.rs` `render_help_overlay`, draw the vertical scrollbar via `scrollbar::vertical(help_scroll, body_rows, total_lines)` when the cheat sheet overflows (keep/adjust the "▼ more" cue).
- [x] T012 [P] [US1] In `src/ui/dialog.rs` (encoding select) and `src/ui/plugin_manager.rs`, draw a vertical scrollbar when the list overflows its visible rows; reserve the bar column so list text isn't hidden. NOTE: the encoding dialog has no `scroll` field (7 fixed entries, no windowing) — its bar derives content/viewport from the entry count vs visible rows and is therefore only shown on a terminal too short to fit all 7; the plugin manager has a real cursor/scroll. Do not invent a scroll field where none exists.

**Checkpoint**: all five surfaces show correct scrollbars; `make check` green; no content hidden, no panics.

---

## Phase 4: User Story 2 — Help/About Close button (Priority: P1)

**Goal**: Help and About each show a clickable, bordered **Close (Esc)** button.

**Independent Test**: open Help → click Close → closes; open About → press Esc → closes.

- [x] T013 [P] [US2] Integration test in `tests/integration/help_close_button.rs`: opening Help then clicking the Close button rect closes it; opening About then `Esc` closes it; the Close button label contains its key.
- [x] T014 [US2] In `src/ui/mod.rs` `render_help_overlay`, grow the box for a button row and render a boxed Close button via `buttons::{button_rects,render_buttons}` (mirror the feature-020 plugin-manager wiring), label `Close (Esc)`.
- [x] T015 [US2] In `src/app.rs` `handle_mouse_event`, add a `pending_help` branch that hit-tests the Close button (`buttons::hit_test_buttons` over the same rect) and clears `pending_help` on a click; keyboard dismissal (`Esc`/Enter/printable) unchanged.

**Checkpoint**: Help and About dismissable by mouse and keyboard; `make check` green.

---

## Phase 5: User Story 3 — key-hint button labels (Priority: P2)

**Goal**: Every dialog button label includes its activating key; behavior unchanged.

**Independent Test**: open each dialog → each button label contains its key; pressing/clicking still runs
the same action.

- [x] T016 [P] [US3] Integration test in `tests/integration/dialog_key_hints.rs`: for the save prompt, encoding select, Find/Replace, file browser, and Help — each button label contains the expected key; activating by key and by click runs the same action as before (no regression).
- [x] T017 [US3] In `src/app.rs`, append key hints in `dialog_button_labels` (confirm dialogs) and `interactive_button_labels` (interactive dialogs) per `contracts/scroll-affordances.md`; keep activation/focus/click mapping keyed on button index/identity, not the displayed text.
- [x] T018 [US3] Ensure the Help Close button (T014) and any other button label sources use the same key-hint convention; verify `button_rects` width handles the longer labels (no truncation/overlap).

**Checkpoint**: all dialog buttons advertise their key; all prior keys/actions behave identically.

---

## Phase 6: Polish & Cross-Cutting

- [x] T019 [P] Add headless render assertions (FR-012/SC-005): an overflowing editor and an overflowing file browser each draw a scrollbar; Help shows a Close button; and the editor renders its bars correctly **with line numbers on, in split view, and after a simulated resize** (no panic, no content hidden under a bar).
- [x] T020 [P] Update `CHANGELOG.md` (feature 021 under `[Unreleased]`), `docs/STATUS.md`, and `docs/CAPABILITIES.md` (scrollbars, Help Close button, key-hint labels are user-visible). Docs gate.
- [x] T021 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check) and fix findings; confirm editor render stays within the perf budget.
- [x] T022 Run the `specs/021-scroll-affordances/quickstart.md` manual walkthrough (editor V+H, soft-wrap, split view, file browser, Help/About, dialog key hints, resize).

---

## Dependencies & Execution Order

- **Setup (P1)** → no deps.
- **Foundational (P2)** → depends on Setup; **blocks US1 (Phase 3)**.
- **US1 (Phase 3)** → depends on Foundational. The file-browser sub-phase (T005–T006) is the MVP slice;
  the editor sub-phase (T007–T010) is the riskiest (geometry) and is self-contained; Help/list dialogs
  (T011–T012) are independent `[P]`.
- **US2 (Phase 4)** and **US3 (Phase 5)** → depend only on Setup (US2's button reuse is independent of
  scrollbars; US3 edits label builders). They may proceed in parallel with US1 after Setup, but US3's
  Help-Close label (T018) assumes US2's button (T014).
- **Polish (Phase 6)** → after the desired stories.

### Within each phase

Write the test(s) first (must fail) → implement → make green. Commit per phase.

### Parallel opportunities

- T003 (helper test) ∥ doc reading.
- Within US1: T011 and T012 are `[P]` (distinct files) once the helper exists; T005/T007 tests are `[P]`.
- Polish T019/T020 are `[P]`.

---

## Implementation Strategy

### MVP

Setup → Foundational (scrollbar helper) → US1 file-browser scrollbar (T005–T006): the smallest
end-to-end proof that the affordance works. Then the editor sub-phase, then Help/list dialogs.

### Incremental delivery

US1 (scrollbars) → US2 (Help Close) → US3 (key hints) → Polish. Each independently testable and
committable.

---

## Notes

- TDD mandatory (Constitution V): failing test first per behavior.
- No new actions / no scroll-behavior changes — affordance/label only.
- Reservation invariant: the editor scrollbar geometry has ONE source of truth shared by render + scroll
  math + mouse mapping.
- Keep AI attribution out of commits/PR/issues (project rule). Branch `021-scroll-affordances`, PR to
  `master`, merge via GitHub.
- Follow-up queued (NOT this feature): file-dialog glob filtering + richer file/folder detail columns →
  feature 022.
