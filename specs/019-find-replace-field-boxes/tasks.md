# Tasks: Bordered-box styling for Find/Replace fields

**Feature**: 019-find-replace-field-boxes | **Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

**Scope**: Render-layer restyle of the Find/Replace dialog fields into labeled, bordered input boxes
matching feature 018. No state-model changes; no focus-ring/buttons (issue #38).

**Tests**: Included — Constitution V requires an automated test for the visible behavior.

## Phase 1: Setup

- [ ] T001 Confirm build env: run `make tmpfs-setup` (per project convention) then `make check` to
  establish a green baseline before changes (repo root).

## Phase 2: Foundational (blocking prerequisites)

- [ ] T002 Review and confirm reuse surface for the shared input-box treatment: read
  `src/ui/file_browser.rs` (`truncate_to_width`, `grapheme_width`, the `field_box`/`field_label_row`
  layout in `compute_layout`, and the right-anchored caret render ~lines 588-640). Decide whether to
  call `truncate_to_width` directly or extract a small private `render_input_box` helper in
  `src/ui/mod.rs`. Record the choice as a code comment referencing feature 019.

## Phase 3: User Story 1 — Find field as a bordered input box (P1) 🎯 MVP

**Goal**: The Find dialog renders its search term inside a labeled, bordered box with a caret,
matching the file browser; all editing/search behavior preserved.

**Independent test**: `Ctrl+F`, type/edit a term, run search — box + caret visible, count works.

- [ ] T003 [P] [US1] Add a render test in `src/ui/dialog.rs` (or `src/ui/mod.rs` tests) using
  `ratatui` `TestBackend` that renders the **Find** overlay (build an `App`/`FindReplaceDialog` in
  Find mode) and asserts: border chars (`┌`,`└`,`│`) present, `Find what:` label present, caret `▏`
  present in the query box, and the four option labels present. (Mirror the `render_browser` pattern
  in `file_browser.rs`.) Test should fail before T005.
- [ ] T004 [US1] In `src/ui/mod.rs` (Find/Replace overlay block ~lines 205-287), replace the inline
  `Find:    text│` line for the query field with a labeled, bordered 3-row input box: a `Find what:`
  label row + a `Block::default().borders(Borders::ALL)` box whose middle row shows the query text
  with the `▏` caret, reusing `truncate_to_width` for right-anchored horizontal scroll. Keep the
  match-count indicator, options row, and hint row.
- [ ] T005 [US1] Recompute the Find-mode dialog height/width to fit the taller layout (label + 3-row
  box + options + hint) and clamp with `.min(size.width)`/`.min(size.height)` and `saturating_*`
  child rects so it degrades gracefully on small terminals (FR-009). Verify T003 passes.

**Checkpoint**: Find dialog is visually consistent with the file browser; `make check` green.

## Phase 4: User Story 2 — Replace dialog: both fields as boxes (P1)

**Goal**: Both Replace fields render as boxes; `Tab` switches focus; caret only in focused box.

**Independent test**: `Ctrl+H`, `Tab` between boxes, type in each, replace/replace-all.

- [ ] T006 [P] [US2] Add a render test for the **Replace** overlay asserting: two bordered boxes,
  `Find what:` and `Replace with:` labels present, caret `▏` present in the focused box and **absent**
  from the unfocused box (assert for both `focus == Query` and `focus == Replacement`). Place in
  `src/ui/dialog.rs`/`mod.rs` tests. Should fail before T007.
- [ ] T007 [US2] Extend the `src/ui/mod.rs` overlay so Replace mode draws the second `Replace with:`
  labeled bordered box below the first (reusing the same helper/logic from T004), drawing the caret
  only in the focused field. Recompute Replace-mode height accordingly (two boxes) with the same
  small-terminal clamping. Verify T006 passes.

**Checkpoint**: Replace dialog consistent; focus indication correct.

## Phase 5: User Story 3 — Behavior preserved (P2, regression guard)

**Goal**: Options, match count, navigation, and dismissal unchanged after restyle.

**Independent test**: toggle each option, run find/replace/replace-all, `Esc`.

- [ ] T008 [P] [US3] Verify existing field-editing/focus unit tests in `src/ui/dialog.rs`
  (`frd_*`) still pass unchanged; if the caret glyph assertion exists anywhere, update `│`→`▏` to
  match the new in-box caret (research Decision 3). Do not change behavior.
- [ ] T009 [US3] Add/confirm a test asserting the options row (`Case`,`Wrap`,`Regex`,`Word` with
  `[ ]`/`[x]`) and the hint row still render in the restyled overlay, and that **no** boxed buttons
  / focus ring were introduced (scope guard C-7 for issue #38).

## Phase 6: Polish & Cross-Cutting

- [ ] T010 [P] Run `cargo fmt --check` and `cargo clippy -D warnings`; fix any lint/format issues in
  touched files (`src/ui/mod.rs`, tests).
- [ ] T011 [P] Update docs: `CHANGELOG.md` (feature 019 entry under Unreleased), `docs/STATUS.md`,
  and `docs/CAPABILITIES.md` (Find/Replace fields now rendered as bordered input boxes — user-visible
  affordance change).
- [ ] T012 Manual validation per [quickstart.md](quickstart.md): Find box, Replace boxes + `Tab`,
  long-text scroll, small-terminal clamp. Run `make ci-local`.
- [ ] T013 Close-out: reference issue #41 in the PR; update `ROADMAP.md` (mark the #41 follow-up
  shipped under feature 019).

## Dependencies & Execution Order

- **Setup (T001)** → **Foundational (T002)** → **US1 (T003–T005)** → **US2 (T006–T007)** →
  **US3 (T008–T009)** → **Polish (T010–T013)**.
- US2 depends on US1 (reuses the box helper/logic from T004). US3 is a regression guard over US1+US2.
- **MVP** = Phase 3 (US1): Find dialog boxed — independently shippable and already satisfies the core
  consistency complaint.

## Parallel Opportunities

- T003 and T006 (test authoring) can be drafted in parallel `[P]` (different test fns).
- T010 and T011 `[P]` (lint vs docs, different files).
- Within a story, implementation tasks (T004→T005, T007) are sequential (same file `src/ui/mod.rs`).

## Format validation

All tasks use `- [ ] Txxx [P?] [USx?] description + file path`. Setup/Foundational/Polish carry no
story label; user-story tasks carry `[US1]`/`[US2]`/`[US3]`.
