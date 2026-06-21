# Phase 0 Research: Per-Pane Wrap Cache
## R1 — Why two issues
`content_width()` = full terminal width − gutter − 1 (single-view geometry). In vertical split each pane
is ~half, so (a) the active pane's cache wraps at full width (wrong points + scrollbar), and (b) the
non-active pane has no cache → editor.rs falls through to the UNWRAPPED render (return at editor.rs:303
is inside `if let Some(wrap_starts)`, so None ⇒ unwrapped). Decision: fix both.
## R2 — Cache model
Keep `wrap_cache` as the ACTIVE buffer's (cursor_visual_row, scrollbar content_v, wheel-scroll, editing
all assume that). Add `wrap_cache_alt` + `wrap_alt_for: Option<usize>` for the non-active VISIBLE pane in
split. Max two visible panes ⇒ two caches suffice (no general map). Render selects per pane via a
`pane_wrap_starts(buf_idx)` accessor (active→wrap_cache, alt buffer→wrap_cache_alt, else None).
## R3 — Widths
`content_width()` becomes split-aware: Single ⇒ full; Vertical ⇒ active pane half. `alt_pane_content_width`
= the sibling half. Panes equal ±1; each cache uses its own width. `editor_panes` already derives the
exact text rect in render; the event-loop widths mirror the half-split.
## R4 — Invalidation
`activate_buffer` (043) bumps `wrap_text_gen` (both caches stale → recompute) and now also drops
`wrap_cache_alt`. Edits bump gen too. So no stale cross-pane cache (FR-005).
## R5 — Single-view preservation
With split=Single, content_width=full and no alt cache → identical to today (FR-004/SC-003).
## Open questions: none.
