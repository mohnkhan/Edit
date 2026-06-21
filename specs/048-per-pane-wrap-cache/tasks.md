# Tasks: Per-Pane Wrap Cache for Split View (#80)
**Branch**: 048-per-pane-wrap-cache. Behavior-preserving for single view (FR-004).
## Setup
- [ ] T001 Baseline `make check` (note 1284/0/11).
## Foundational
- [ ] T002 `content_width()` split-aware (active pane half in Vertical); add `alt_pane_content_width()` and
  `alt_visible_buffer()` in `src/app/softwrap.rs`.
- [ ] T003 Add `wrap_cache_alt: Option<WrapCache>` + `wrap_alt_for: Option<usize>` to App (`src/app.rs`),
  init None; clear them in `activate_buffer` (`src/app/actions.rs`).
## US1 (P1)
- [ ] T004 Event loop (`src/app.rs`): after the active-cache compute, in Vertical split compute
  `wrap_cache_alt` for `alt_visible_buffer()` when that buffer is soft_wrapped, at `alt_pane_content_width()`
  (stale check via is_stale + wrap_text_gen + width); else clear alt.
- [ ] T005 Add `pane_wrap_starts(buf_idx)` accessor (`src/app.rs`): active→wrap_cache, alt→wrap_cache_alt, else None.
- [ ] T006 `Ui::render` split (`src/ui/mod.rs`): each pane uses `app.pane_wrap_starts(buf_idx)` for the
  EditorWidget; `render_editor_scrollbars` uses the matching cache for content_v (replace the ptr::eq
  active-only check with pane_wrap_starts-based total).
## Tests
- [ ] T007 Render test: split, both buffers wrapped long lines → non-active pane wrapped (not unwrapped);
  each wraps at half width. Single-view wrap render unchanged.
## Ship
- [ ] T008 `make ci-local`; fuzz green; count == baseline + new test.
- [ ] T009 Docs: CHANGELOG + STATUS (no CAPABILITIES change). PR `feat(048): per-pane wrap cache`, Closes #80, merge.
