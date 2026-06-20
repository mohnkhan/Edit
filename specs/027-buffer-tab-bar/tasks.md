---
description: "Task list for feature 027 — buffer tab bar"
---

# Tasks: Buffer tab bar

**Input**: Design documents from `specs/027-buffer-tab-bar/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/tab-bar.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V (Test-Gated Merges, NON-NEGOTIABLE). Tests first.

**Organization**: Setup → Foundational (editor-geometry refactor) → US1 (tab bar + switch) → US2 (`[x]`
close + confirm) → US3 (geometry no-regression) → Polish.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files / independent)
- **[Story]**: US1 (tabs+switch) / US2 (close) / US3 (geometry no-regression)

## Path Conventions

Single-project Rust: `src/ui/tabbar.rs` (new), `src/ui/mod.rs`, `src/app.rs`; integration tests under
`tests/integration/`, units inline.

---

## Phase 1: Setup

- [X] T001 Confirm a clean baseline build on branch `027-buffer-tab-bar` (`make tmpfs-setup` then `make`).
- [X] T002 Re-read the layout in `src/ui/mod.rs`, the editor-geometry consumers in `src/app.rs` (`viewport_height`, `handle_mouse_click`, the wheel block, `scrollbar_regions`), `close_active_buffer`/`next_buffer`/`prev_buffer`, and the feature-016 `ButtonDialog` infra (`open_button_dialog`/`dialog_button_labels`/`activate_dialog_button`/`dialog_view_text`/`dialog_default_focus`/`dialog_cancel_index`). No code change.

---

## Phase 2: Foundational — tab-bar-aware editor geometry (Blocking)

**⚠️ CRITICAL**: every editor-geometry consumer must use one shared top/height before/at the same time the
tab row is added, or clicks/scroll desync.

- [X] T003 [P] Unit tests in `src/app.rs`: `editor_top()` is 1 with one buffer and 2 with 2+ buffers; `viewport_height()` subtracts the tab row when 2+ buffers (and still the hbar row in non-wrap).
- [X] T004 In `src/app.rs`, add `tab_bar_visible(&self) -> bool` (`buffers.len() > 1`) and `editor_top(&self) -> u16` (`1 + tab_rows`); update `viewport_height` to subtract `tab_rows`; update `handle_mouse_click` (editor row = `row - editor_top()`; ignore rows `< editor_top()`), the wheel block, and `scrollbar_regions` to compute the editor area as `Rect::new(0, editor_top(), w, h - editor_top() - 1)` and use `ev.row >= editor_top()` guards.

**Checkpoint**: with one buffer everything is byte-identical to today; `make check` green.

---

## Phase 3: User Story 1 — tab bar + switch (Priority: P1) 🎯 MVP

**Goal**: With 2+ buffers, a tab bar shows below the menu bar (active highlighted, modified marked);
clicking a tab switches.

**Independent Test**: open 2 buffers → tab bar lists both, active highlighted; click the other → switches.

### Tests for US1 (write first, must fail)

- [X] T005 [P] [US1] Unit tests in `src/ui/tabbar.rs`: `tab_hit_regions` yields a label + `[x]` rect per buffer; overflow with many/long names keeps the active tab's rects present (visible); no panic at width 1.
- [X] T006 [P] [US1] Integration test in `tests/integration/buffer_tab_bar.rs`: with 2 buffers a click on the second tab's label sets `active_idx = 1`; with 1 buffer no tab bar is shown (a click on row 1 is editor, not a tab).

### Implementation for US1

- [X] T007 [US1] Create `src/ui/tabbar.rs`: `tab_hit_regions(area, buffers, active)` (label + `[x]` rects; left→right layout; overflow scrolls to keep the active tab visible; width-correct truncation) and a render fn drawing each tab (name + modified marker + `[x]`, active highlighted) from the same geometry.
- [X] T008 [US1] In `src/ui/mod.rs`, add a conditional 1-row tab-bar chunk below the menu bar when `tab_bar_visible()`, render it via `tabbar`, and pass the resulting (shrunk) editor area to the editor widget(s).
- [X] T009 [US1] In `src/app.rs` `handle_mouse_event`, add tab-row handling (when `tab_bar_visible()` and `ev.row == editor_top()-1`): hit-test `tab_hit_regions`; a label hit sets `active_idx`; return so it never reaches the editor click. (`[x]` handled in US2.)

**Checkpoint**: tab bar shows + click-switch works; single buffer unchanged; `make check` green.

---

## Phase 4: User Story 2 — `[x]` close + unsaved confirm (Priority: P1)

**Goal**: Clicking a tab's `[x]` closes the buffer; a modified buffer triggers a Save/Discard/Cancel
confirm (no silent data loss).

**Independent Test**: `[x]` on a clean buffer closes it; `[x]` on a modified buffer opens the confirm.

### Tests for US2 (write first, must fail)

- [X] T010 [P] [US2] Unit test in `src/app.rs`: `close_buffer_at(idx)` removes the buffer and adjusts `active_idx` correctly (closing before/at/after the active index).
- [X] T011 [P] [US2] Integration test in `tests/integration/buffer_tab_bar.rs`: `[x]` on a clean buffer closes it (tab bar hides when one remains); `[x]` on a modified buffer opens `pending_close_confirm`; activating Save saves+closes, Discard closes, Cancel keeps the buffer.

### Implementation for US2

- [X] T012 [US2] In `src/app.rs`, add `close_buffer_at(idx)` (generalize `close_active_buffer`) and a `CloseConfirm` `ButtonDialog` variant + `pending_close_confirm: Option<usize>`; wire `open_button_dialog`/`dialog_button_labels` (`Save (S)`/`Discard (D)`/`Cancel (Esc)`)/`dialog_default_focus`/`dialog_cancel_index`/`dialog_view_text`/`activate_dialog_button`. **Operate on the stored idx, not `active_idx`** (M1): 0 → `buffers[idx].save()` then `close_buffer_at(idx)`; 1 → `close_buffer_at(idx)`; 2 → clear `pending_close_confirm`. Add a keyboard intercept consistent with the SavePrompt: `S`/`D`/`C` letter shortcuts + `Esc` = cancel (A1, so the `(S)`/`(D)` label hints are accurate).
- [X] T013 [US2] In `handle_mouse_event` tab-row handling, route an `[x]` hit: clean buffer → `close_buffer_at(idx)`; modified → `pending_close_confirm = Some(idx)`.

**Checkpoint**: `[x]` close + confirm works; `make check` green.

---

## Phase 5: User Story 3 — geometry no-regression (Priority: P1)

- [X] T014 [P] [US3] Integration test in `tests/integration/buffer_tab_bar.rs`: with 2 buffers (tab bar shown), a click in the editor text lands on the expected line (accounting for the tab row); a click on the tab row does not move the cursor; a wheel event scrolls the editor (cursor unchanged) against the reduced area; and `NextBuffer`/`PrevBuffer` still cycle the active buffer with the tab bar present (FR-006/L1).

**Checkpoint**: geometry correct beneath the bar; `make check` green.

---

## Phase 6: Polish & Cross-Cutting

- [X] T015 [P] Add a headless render assertion: with 2+ buffers the tab bar row renders the buffer names + active highlight + `[x]`; with 1 buffer it is absent; renders without panic at a tiny size.
- [X] T016 [P] Update `CHANGELOG.md` (feature 027 under `[Unreleased]`), `docs/STATUS.md`, and `docs/CAPABILITIES.md` (tab bar: click to switch, `[x]` to close, modified marker).
- [X] T017 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check) and fix findings.
- [X] T018 Run the `specs/027-buffer-tab-bar/quickstart.md` manual walkthrough (switch, modified marker, `[x]` close + confirm, geometry, overflow, single-buffer unchanged).

---

## Dependencies & Execution Order

- **Setup (P1)** → none. **Foundational (P2)** → blocks everything (shared geometry).
- **US1 (P3)** tab bar + switch (depends on geometry). **US2 (P4)** adds `[x]` close + confirm (depends on
  US1's tab-row handling). **US3 (P5)** asserts geometry no-regression.
- **Polish (P6)** → after the stories.

### Parallel opportunities

- T003/T005/T010 (unit tests) and T006/T011/T014 (integration tests) are `[P]`; T015/T016 polish `[P]`.

---

## Implementation Strategy

### MVP

Setup → Foundational geometry → US1 tab bar + switch (T005–T009): files visible + clickable. Then US2
`[x]` close + confirm, then US3 geometry no-regression.

### Notes

- TDD mandatory (Constitution V). No new crates (Constitution IV). Reuse the feature-016 confirm infra for
  the close prompt; centralize the editor top/height in `editor_top()` (the key risk).
- Keep AI attribution out of commits/PR/issues. Branch `027-buffer-tab-bar`, PR to `master`, merge via GitHub.
