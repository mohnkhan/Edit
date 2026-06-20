---
description: "Task list for feature 024 — interactive (clickable + draggable) scrollbars"
---

# Tasks: Interactive (clickable + draggable) scrollbars

**Input**: Design documents from `specs/024-draggable-scrollbar/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/scrollbar-interaction.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V (Test-Gated Merges, NON-NEGOTIABLE). Tests first.

**Organization**: Setup → Foundational (pure mapping math + geometry accessors) → US1 (track click) →
US2 (thumb drag) → US3 (no-regression) → Polish.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files / independent)
- **[Story]**: US1 (track click) / US2 (thumb drag) / US3 (no-regression)

## Path Conventions

Single-project Rust: `src/ui/scrollbar.rs`, `src/ui/mod.rs`, `src/ui/file_browser.rs`, `src/app.rs`;
integration tests under `tests/integration/`, units inline.

---

## Phase 1: Setup

- [x] T001 Confirm a clean baseline build on branch `024-draggable-scrollbar` (`make tmpfs-setup` then `make`).
- [x] T002 Re-read `src/ui/scrollbar.rs`, `editor_panes`/`render_editor_scrollbars` (src/ui/mod.rs), the file-browser/help/dialog bar rects, and `handle_mouse_event` (drag block, wheel block, Press/Left guard, feature-017 `drag_anchor`) in `src/app.rs`. No code change.

---

## Phase 2: Foundational — mapping math + geometry accessors (Blocking)

- [x] T003 [P] Unit tests in `src/ui/scrollbar.rs` for `thumb_span` (content≤viewport → full track; min len 1; start in `[0,track-len]`; monotonic in pos), `pos_to_offset` (0→0, end→max, clamped, monotonic), and `hit_zone` (Above/Thumb/Below).
- [x] T004 Implement the pure helpers in `src/ui/scrollbar.rs`: `thumb_span(track_len, content, viewport, pos)`, `pos_to_offset(track_len, content, viewport, click)`, `hit_zone(track_len, content, viewport, pos, click)`. No I/O; saturating/clamped arithmetic.
- [x] T005 Expose the bar geometry the mouse handler needs: make `editor_panes` (or a small `editor_bar_rects`) `pub(crate)` in `src/ui/mod.rs`, and add a `pub` accessor in `src/ui/file_browser.rs` for the list-bar rect + `(entries.len(), list_rows, scroll)`. No behavior change.

**Checkpoint**: helpers compile + unit tests green; geometry reachable from `src/app.rs`.

---

## Phase 3: User Story 1 — track click pages (Priority: P1) 🎯 MVP

**Goal**: Clicking a scrollbar track pages the view toward the click (one viewport), every surface.

**Independent Test**: synth a press on the editor v-bar track below the thumb → `scroll_offset.0`
increases ~one viewport (bounded), cursor unchanged; above → decreases.

### Tests for US1 (write first, must fail)

- [x] T006 [P] [US1] Integration test in `tests/integration/scrollbar_interaction.rs`: editor v-bar track click below the thumb pages down (cursor unchanged); above pages up; clamps at ends. Also (M1): with a long line in non-wrap mode, a click right of the horizontal thumb pages the horizontal offset (`scroll_offset.1`) right, bounded.
- [x] T007 [P] [US1] Integration test: a track click on the file-browser bar and the Help bar scrolls those surfaces; with a modal open the editor offset is unchanged.

### Implementation for US1

- [x] T008 [US1] In `src/app.rs`, add `fn scrollbar_regions(&self) -> Vec<ScrollbarRegion>` for the active surface (modal wins, else editor pane under cursor): each `{ rect, axis, content, viewport, offset, kind }`, only when the bar is drawn (content > viewport). For the editor include BOTH the vertical bar and (non-wrap) the **horizontal** bar (M1), so left/right track clicks page the horizontal view.
- [x] T009 [US1] In `handle_mouse_event`, add a left-**press** scrollbar check **before** the feature-017 drag-anchor/editor-click and the modal entry/button handlers: if the press is on a region's track (`hit_zone` Above/Below), page `offset ∓ viewport` (clamped) and apply per `kind`, then `return Ok(())`. Editor applies via the feature-023 viewport-only clamp (cursor untouched).

**Checkpoint**: track click pages every surface, bounded; `make check` green.

---

## Phase 4: User Story 2 — thumb drag (Priority: P1)

**Goal**: Dragging a thumb scrolls proportionally; editor drag is viewport-only and never selects text.

**Independent Test**: press on the editor thumb, move toward the bottom → `scroll_offset.0` rises
proportionally; release → further moves don't scroll; cursor + selection unchanged.

### Tests for US2 (write first, must fail)

- [x] T010 [P] [US2] Integration test in `tests/integration/scrollbar_interaction.rs`: press on the editor v-bar thumb then a Drag near the track bottom sets `scroll_offset.0` near max (proportional); a Release ends the drag (a later Drag does not scroll); cursor + `selection` unchanged. Also (L1): a Release at coordinates outside the track ends the drag cleanly (no panic; subsequent Drag does not scroll).

### Implementation for US2

- [x] T011 [US2] In `src/app.rs`, add `scrollbar_drag: Option<ScrollbarDrag>` (kind + axis + track_start + track_len + content + viewport) and `ScrollbarDrag`/`ScrollbarKind` types; initialize `None` in `App::new`.
- [x] T012 [US2] In the US1 press check, when `hit_zone` is `Thumb`, start a `scrollbar_drag` (record the region geometry) and `return Ok(())` (no immediate jump).
- [x] T013 [US2] In `handle_mouse_event`, add a Drag branch: **if `scrollbar_drag` is `Some`**, map the cursor along the stored track via `pos_to_offset` and apply per `kind` (editor viewport-only), then return — placed **before** the feature-017 selection drag so it suppresses selection. Clear `scrollbar_drag` on `Release` (and guard the feature-017 drag with `scrollbar_drag.is_none()`).

**Checkpoint**: thumb drag scrubs proportionally; editor cursor/selection untouched; `make check` green.

---

## Phase 5: User Story 3 — no regression (Priority: P1)

- [x] T014 [P] [US3] Integration test in `tests/integration/scrollbar_interaction.rs`: a press-drag that **starts in the text body** still selects (no scroll); a press **on the scrollbar** does not place the cursor or select; a left-click in text still places the cursor; the wheel (023) still scrolls. (Covers FR-006/FR-007.)

**Checkpoint**: no-regression test green.

---

## Phase 6: Polish & Cross-Cutting

- [x] T015 [P] Update `CHANGELOG.md` (feature 024 under `[Unreleased]`), `docs/STATUS.md`, and `docs/CAPABILITIES.md` (scrollbars are now clickable + draggable).
- [x] T016 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check) and fix findings.
- [x] T017 Run the `specs/024-draggable-scrollbar/quickstart.md` manual walkthrough (track click, thumb drag, editor cursor unaffected, text selection intact, file browser + Help bars, bounds/resize).

---

## Dependencies & Execution Order

- **Setup (P1)** → none. **Foundational (P2)** → blocks US1/US2 (math + geometry).
- **US1 (P3)** adds `scrollbar_regions` + the press track-click. **US2 (P4)** adds the drag state + thumb
  press + Drag branch (depends on US1's region/press scaffolding). **US3 (P5)** asserts no regression.
- **Polish (P6)** → after the stories.

### Parallel opportunities

- T003 (math test) `[P]`; T006/T007/T010/T014 are `[P]` test additions; T015 docs `[P]`.

---

## Implementation Strategy

### MVP

Setup → Foundational math/geometry → US1 track click (T006–T009): the bars become usable with a click.
Then US2 thumb drag, then US3 no-regression.

### Notes

- TDD mandatory (Constitution V). No new deps/config (Constitution IV/VI).
- The scrollbar press-check must run before feature-017 selection/editor-click; the Drag branch must run
  before the feature-017 selection drag; `scrollbar_drag` gates both so text selection is never triggered
  by a bar gesture.
- Keep AI attribution out of commits/PR/issues. Branch `024-draggable-scrollbar`, PR to `master`, merge
  via GitHub.
