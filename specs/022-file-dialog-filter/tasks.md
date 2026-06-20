---
description: "Task list for feature 022 — file dialog glob filtering + richer entry details"
---

# Tasks: File dialog — glob filtering + richer entry details

**Input**: Design documents from `specs/022-file-dialog-filter/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/file-dialog.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V (Test-Gated Merges, NON-NEGOTIABLE). Test tasks first.

**Organization**: Setup → Foundational (std-only helpers) → US1 (live filtering) → US2 (detail columns)
→ Polish. Both stories live almost entirely in `src/ui/file_browser.rs`.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files / independent)
- **[Story]**: US1 (filtering) / US2 (detail columns)

## Path Conventions

Single-project Rust: `src/ui/file_browser.rs`, integration tests under `tests/integration/`, units inline.

---

## Phase 1: Setup

- [x] T001 Confirm a clean baseline build on branch `022-file-dialog-filter` (`make tmpfs-setup` then `make`).
- [x] T002 Re-read `FileBrowser::reload`, the widget render loop, `hit_test`, `activate_open/activate_save`, `push_char`/`backspace`, and the feature-021 scrollbar reservation in `src/ui/file_browser.rs`. No code change.

---

## Phase 2: Foundational — std-only helpers (Blocking)

**Purpose**: Pure helpers used by both stories; no I/O, fully unit-testable.

- [x] T003 [P] Unit tests in `src/ui/file_browser.rs` for `glob_match(pattern, name)`: `*.log` matches `a.log` not `a.txt`; `te?t` matches `test`/`text` not `tt`; leading/trailing `*`; whole-name anchoring; case-insensitive.
- [x] T004 [P] Unit tests in `src/ui/file_browser.rs` for `human_size` (pin the form: 0→`0B`, 1023→`1023B`, 1024→`1.0K`, ~1.5M, ~2G boundaries) and `format_mtime` (a fixed epoch → expected `YYYY-MM-DD HH:MM` UTC). Use the `1.0K` style consistently (A1).
- [x] T005 Implement the three std-only helpers in `src/ui/file_browser.rs`: `glob_match` (two-pointer `*`/`?`, case-insensitive), `human_size(u64) -> String`, `format_mtime(secs: u64) -> String` (days-from-civil, UTC). No new crates.

**Checkpoint**: helpers compile, unit tests green.

---

## Phase 3: User Story 1 — live filtering (Priority: P1) 🎯 MVP

**Goal**: Typing filters the listing live (glob or substring, case-insensitive); dirs + `..` always
shown; absolute path still jumps; clearing restores.

**Independent Test**: in a dir with `a.log`,`b.txt`,`sub/` — `*.log` → `a.log`+`sub/`+`..`; `b` →
`b.txt`+`sub/`+`..`; clear → all; `/etc`+Enter → jumps.

### Tests for US1 (write first, must fail)

- [x] T006 [P] [US1] Unit test in `src/ui/file_browser.rs`: `apply_filter` with `*.log` keeps all dirs + `..` and only matching files; with plain `b` keeps substring matches; empty restores `all_entries`; an absolute-path field leaves the listing unfiltered.
- [x] T007 [P] [US1] Unit test in `src/ui/file_browser.rs`: when the filter hides the selected entry, `selected` is re-clamped to a visible row and `scroll` stays valid (no out-of-range).
- [x] T008 [P] [US1] Integration test in `tests/integration/file_dialog_filter.rs`: driving `App` with the browser open, typing `*.log` narrows `file_browser.entries`; typing then backspacing restores; an absolute path typed + Enter still navigates/opens (feature-012 behavior).

### Implementation for US1

- [x] T009 [US1] In `src/ui/file_browser.rs`, add `all_entries: Vec<Entry>` to `FileBrowser`; change `reload()` to build `all_entries` (sorted, with size/mtime via `std::fs::metadata`, best-effort `Option`s) then call `apply_filter()`.
- [x] T010 [US1] In `src/ui/file_browser.rs`, implement `apply_filter()` per `contracts/file-dialog.md` (empty/absolute → full; `*`/`?` → glob; else substring; always keep `Parent`/`Dir`; case-insensitive) and re-clamp `selected`/`scroll`.
- [x] T011 [US1] In `src/ui/file_browser.rs`, call `apply_filter()` after `push_char` and `backspace` so filtering is live; verify `activate_open` (absolute-path jump / open selected) and `activate_save` still operate correctly over the filtered `entries`. Save mode (U1): filtering still keys on `filename`, so typing a brand-new save name may narrow files to none — that is acceptable (dirs/`..` remain, and the confirm still saves the typed name); add a Save-mode test asserting confirm saves the typed filename even when no existing file matches.

**Checkpoint**: live filtering works; dirs/`..` always present; jump + Save unchanged; `make check` green.

---

## Phase 4: User Story 2 — detail columns (Priority: P1)

**Goal**: Each row shows size (files) / `<DIR>` (dirs) + modified date, aligned; name truncates.

**Independent Test**: a dir with a small + large file + sub-folder → file rows show size+date, folder
shows `<DIR>`, columns aligned, long names truncate with `…`.

### Tests for US2 (write first, must fail)

- [x] T012 [P] [US2] Unit test in `src/ui/file_browser.rs`: a rendered listing (via the existing `render_browser` test helper) contains a `<DIR>` marker for a directory row and a size/date for a file row; a long file name is truncated with `…` while the detail columns remain present; an entry with `None` size/mtime renders without panic (blank detail) (covers FR-011, C1).

### Implementation for US2

- [x] T013 [US2] In `src/ui/file_browser.rs` widget render loop, lay out fixed right-aligned size + date columns and give the name the remaining width (reserving the feature-021 scrollbar column when present); truncate the name with `truncate_to_width`. Dirs/`..` show `<DIR>` instead of a size; missing metadata renders blank.
- [x] T014 [US2] In `src/ui/file_browser.rs`, ensure the detail columns degrade gracefully on a narrow dialog (drop detail columns → name-only) without corrupting layout, and stay width-correct for multi-byte names.

**Checkpoint**: detail columns render and align; `make check` green.

---

## Phase 5: Polish & Cross-Cutting

- [x] T015 [P] [US1/US2 regression] Integration test in `tests/integration/file_dialog_filter.rs`: with a filter active, feature-020 Open/Cancel buttons + focus ring still work and the feature-021 scrollbar reflects the filtered count; a no-match filter still lists `..`/dirs.
- [x] T016 [P] Update `CHANGELOG.md` (feature 022 under `[Unreleased]`), `docs/STATUS.md`, and `docs/CAPABILITIES.md` (file-dialog filtering + detail columns are user-visible; correct the prior "absolute jump-path only" wording).
- [x] T017 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check) and fix findings.
- [x] T018 Run the `specs/022-file-dialog-filter/quickstart.md` manual walkthrough (glob, substring, clear, absolute-path jump, detail columns, resize, buttons/scrollbar).

---

## Dependencies & Execution Order

- **Setup (P1)** → none.
- **Foundational (P2)** → depends on Setup; **blocks US1/US2** (helpers used by both).
- **US1 (Phase 3)** and **US2 (Phase 4)** → both depend on Foundational; they touch the same file but
  largely different regions (model/filter vs widget render) — do US1 first (MVP), then US2.
- **Polish (Phase 5)** → after both stories.

### Within each phase

Tests first (must fail) → implement → green. Commit per phase.

### Parallel opportunities

- T003/T004 (helper tests) are `[P]`; T006/T007/T008 (US1 tests) are `[P]`; T016 docs `[P]`.

---

## Implementation Strategy

### MVP

Setup → Foundational helpers → US1 live filtering (T006–T011): directly fixes the "`*.log` just closes"
complaint. Then US2 detail columns.

### Notes

- TDD mandatory (Constitution V). No new crates (Constitution IV). Open/Save still validate via
  `validate_path` (Constitution VII).
- Keep AI attribution out of commits/PR/issues. Branch `022-file-dialog-filter`, PR to `master`, merge
  via GitHub.
- Related follow-up queued (NOT this feature): mouse-wheel scrolling app-wide → feature 023.
