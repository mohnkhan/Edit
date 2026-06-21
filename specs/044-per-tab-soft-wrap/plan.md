# Implementation Plan: Per-Tab Soft-Wrap

**Branch**: `044-per-tab-soft-wrap` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/044-per-tab-soft-wrap/spec.md`

## Summary

Move soft-wrap from a single global `App::soft_wrap` flag to a per-buffer `Buffer::soft_wrap` field, so
each tab keeps its own wrap setting. The toggle (`View ▸ Soft Wrap` / Ctrl+W) acts on the active
buffer; switching tabs shows that tab's setting; new/opened tabs seed from `config.soft_wrap`. All ~27
readers (render panes, status bar, menu check state, editor geometry, scroll/mouse math, wrap-cache
gate) switch to reading the relevant buffer's flag — the active buffer for the single view, each pane's
buffer in split view. The single wrap cache stays for the active buffer (already invalidated on switch
by feature 043). Behavior-preserving for the single-tab / default-setting case.

## Technical Context

**Language/Version**: Rust 2021, MSRV 1.74
**Primary Dependencies**: `ratatui` + `crossterm`, `ropey`. No new deps.
**Storage**: N/A (in-memory; wrap state not persisted to disk — out of scope).
**Testing**: `cargo test` (unit/inline + integration), `make smoke`, `make perf-check`.
**Target Platform**: Linux TUI.
**Project Type**: Single-project desktop TUI app.
**Performance**: No regression (a bool moves from one struct to another; same cache logic).
**Constraints**: Behavior-preserving for single-tab & untouched-default (FR-007); no panic/corruption
in any tab/pane/wrap combo (FR-008); the `clippy::unwrap_used` guardrail (042) on the app tree still
holds.
**Scale/Scope**: One field move + ~27 reader updates across `src/app*`, `src/ui/*`; toggle + seeding;
status bar + menu indicator; tests.

## Constitution Check

- **I. DOS-Faithful UI** — PASS. Wrap toggle/indicators unchanged in look; now per-tab.
- **II. UTF-8** — PASS / N/A. No new decoding path; wrap layout already grapheme/width aware.
- **III. Portable Build** — PASS. Pure Rust. (ncurses mention in constitution is stale; live stack is
  ratatui/crossterm per CLAUDE.md.)
- **IV. Minimal Footprint** — PASS. No new dependency.
- **V. Test-Gated (NON-NEGOTIABLE)** — PASS. New per-tab tests; existing soft-wrap tests adjusted only
  to read the flag's new location; full suite + 042 fuzz + 043 cache tests stay green.
- **VI. Simplicity / YAGNI** — PASS. Smallest change: a field move + reader updates. Session
  persistence and a second split-view cache are explicitly deferred.
- **VII. Security** — PASS / N/A.

**Result**: All gates pass. Complexity Tracking empty.

## Project Structure

```text
src/
├── buffer/mod.rs        # ADD `pub soft_wrap: bool` to Buffer; default false in new_empty()/open().
├── app.rs               # App::new seeds initial buffers' soft_wrap from config; geometry/scrollbar/
│                        #   event-loop cache gate read active_buffer().soft_wrap (+ per-pane in split).
├── app/
│   ├── softwrap.rs      # handle_toggle_soft_wrap → flip active_buffer().soft_wrap + invalidate cache
│   │                    #   (no longer rewrites config); content_width etc. read active buffer.
│   ├── actions.rs       # new_buffer / handle_open_file seed soft_wrap from config.
│   ├── editing.rs       # the `self.soft_wrap && wrap_cache` gate → active_buffer().soft_wrap.
│   ├── mouse.rs         # wheel/hit-test wrap checks → active_buffer().soft_wrap; menu toggle state.
│   ├── fileops.rs       # session-restore buffers seed soft_wrap from config; cache gate.
│   └── tests.rs         # update existing soft-wrap tests to per-buffer; add per-tab tests.
└── ui/
    ├── mod.rs           # render: single view uses active buffer's flag; split panes use each pane
    │                    #   buffer's flag; StatusBar + menu toggle-state use active buffer's flag.
    ├── editor.rs        # EditorWidget already takes soft_wrap per call — pass the pane buffer's flag.
    └── statusbar.rs     # StatusBar.soft_wrap fed from the active buffer.
```

**Structure Decision**: Unchanged layout. The field's home moves `App` → `Buffer`; readers updated in
place.

## Key design decisions (detail in research.md)

- `Buffer::soft_wrap` defaults `false`; **App seeds it from `config.soft_wrap`** at every buffer-creation
  site (initial buffers in `App::new`, `new_buffer`, `handle_open_file`, session-restore). Centralized
  enough to not be forgotten; covered by tests.
- Toggle flips the **active** buffer and invalidates the wrap cache; it no longer mutates `config`
  (config becomes a default seed, per spec Assumptions).
- The single wrap cache remains the active buffer's, computed only when `active_buffer().soft_wrap` is
  on, invalidated on switch (043). Split-view non-active wrapped pane renders best-effort with no cache
  (must not corrupt/crash) — `EditorWidget` already handles `soft_wrap=true` + `wrap_starts=None`.
- Indicators (status bar, `View ▸ Soft Wrap` check) read the active buffer's flag.

## Phased Approach (one PR, ordered commits)

1. **Field move**: add `Buffer::soft_wrap`; remove `App::soft_wrap`; compiler enumerates every reader.
2. **Readers + seeding + toggle**: update all readers to the right buffer's flag; seed at creation;
   retarget the toggle and indicators to the active buffer.
3. **Tests**: adjust existing soft-wrap tests; add per-tab independence + indicator tests.

## Complexity Tracking

*No violations. Table intentionally empty.*
