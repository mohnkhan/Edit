---
description: "Task list for feature 018: Editable-field affordance + Help redesign"
---

# Tasks: Editable-field affordance + Help redesign

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/field-and-help.md
**Tests**: INCLUDED (Principle V, TDD). **Stories**: US1 file-dialog input box, US2 Help table.

## Phase 1: Setup
- [x] T001 Add `help_scroll: usize` to `App` (init 0); create `tests/integration/help_overlay.rs`
  registered as a `[[test]]` in `Cargo.toml` (if a non-unit test is needed; otherwise unit tests only).

## Phase 2: US1 — File-dialog bordered input box (Priority: P1)
- [x] T002 [US1] In `src/ui/file_browser.rs::compute_layout`, reserve a label row + 3-row bordered box
  region above the footer in BOTH Open and Save modes; shrink `list_rows` accordingly.
- [x] T003 [US1] Render the bordered input box in both modes: label ("Name:" Save / "Go to path:" Open),
  Block::ALL border, field text from `filename` with an always-visible caret; grapheme-correct;
  long text truncated/scrolled within the box. Update footer hints to mention typing in Open mode.
- [x] T004 [P] [US1] Unit test (TestBackend, ≥80×24) in `src/ui/file_browser.rs`: Open-mode render
  contains the path-box label + a border char + caret; Save-mode contains the name-box; typing into
  `filename` shows in the box.

## Phase 3: US2 — Help Key|Action table (Priority: P1)
- [x] T005 [US2] Define `HELP_SECTIONS` (grouped `(key, action)` rows) and rebuild `render_help_overlay`
  in `src/ui/mod.rs` as aligned `KEY  ACTION` rows under section headings, built into `Vec<Line>`, with
  a scroll window (`app.help_scroll`) and a "more"/scroll cue; box fits the terminal, no truncation.
- [x] T006 [US2] In `src/app.rs` help intercept, reset `help_scroll=0` on open; handle `Up`/`Down`/
  `PageUp`/`PageDown` to scroll (clamped to the row count); Esc/dismiss keys still close.
- [x] T007 [P] [US2] Unit test (TestBackend) for Help: rendered content shows a section heading and an
  aligned key+action row; a unit test that `help_scroll` clamps and that scrolling reveals later rows.

## Phase 4: Polish
- [x] T008 File the deferral: GitHub `follow-up` issue for bordered-box styling of the Find/Replace
  fields (already have a caret), + a ROADMAP row.
- [x] T009 [P] Update `CHANGELOG.md` (feature 018), `docs/STATUS.md` (F018 rows), `docs/CAPABILITIES.md`
  (file-dialog typeable fields; Help is a scrollable Key|Action table).
- [x] T010 Run the gate: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; live
  tmux check (Ctrl+O shows a path box; Help shows the table).

## Dependencies
- Setup → US1 (file_browser) and US2 (help) are independent → Polish. T008 before merge.

## Strategy
- MVP = US1 + US2. Find/Replace box styling deferred (T008).
