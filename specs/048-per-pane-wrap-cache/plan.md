# Implementation Plan: Per-Pane Wrap Cache for Split View
**Branch**: `048-per-pane-wrap-cache` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md) | Issue #80

## Summary
Make split-view wrap correct per pane: (1) `content_width()` returns the active pane's width (half in
split) — fixing the active pane's wrap width; (2) add `wrap_cache_alt` (+ which buffer it's for) for the
non-active visible pane, computed in the event loop at that pane's width; (3) render hands each pane its
own cache; (4) the per-pane vertical scrollbar uses the matching cache. Single view unchanged.

## Technical Context
Rust 2021; ratatui/ropey. No new deps. Tests: ratatui TestBackend render checks + fuzz. Behavior-
preserving for single view (FR-004). 042/046 guards hold.

## Constitution Check
I PASS (correct split wrap, no UI control change). II PASS (grapheme-aware wrap unchanged). III/IV PASS.
V PASS (render tests + fuzz). VI PASS (two caches, not general N). VII N/A. All gates pass.

## Project Structure
- `src/app/softwrap.rs` — `content_width()` split-aware (active pane width); add `alt_pane_content_width()`
  + `alt_visible_buffer()` helpers.
- `src/app.rs` — fields `wrap_cache_alt: Option<WrapCache>`, `wrap_alt_for: Option<usize>`; event-loop
  computes the alt cache for the non-active visible buffer in split when it is soft_wrapped; a
  `pane_wrap_starts(buf_idx)` accessor returns the right cache for a pane.
- `src/ui/mod.rs` — split render hands each pane `app.pane_wrap_starts(buf_idx)`; scrollbar uses it.
- `activate_buffer` (043) already invalidates wrap_text_gen → both caches recompute on switch; clear
  `wrap_cache_alt` too.

## Key decisions (research.md)
- Keep `wrap_cache` = ACTIVE buffer's (all existing consumers rely on that); add `wrap_cache_alt` for the
  other visible pane only. Render/scrollbar select per buffer index via `pane_wrap_starts`.
- `content_width()` split-aware fixes the active pane width bug; the alt pane width is the sibling half.
- Invalidate alt cache on tab switch (extend `activate_buffer`) and when not in split / single buffer.

## Phases (one PR)
1. content_width split-aware + width/visible helpers. 2. alt cache field + event-loop compute +
   pane_wrap_starts + render/scrollbar wiring + invalidation. 3. render tests (both panes wrapped; single
   unchanged) + fuzz.

## Complexity Tracking: empty.
