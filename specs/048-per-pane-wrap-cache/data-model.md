# Phase 1 Data Model: Per-Pane Wrap Cache
## App fields
- `wrap_cache: Option<WrapCache>` — ACTIVE buffer's cache (unchanged role).
- `wrap_cache_alt: Option<WrapCache>` — NEW: the non-active VISIBLE pane's cache (split only).
- `wrap_alt_for: Option<usize>` — NEW: which buffer index `wrap_cache_alt` belongs to (else None).
## Width
- `content_width()` (split-aware): Single ⇒ full − gutter − 1; Vertical ⇒ activePaneHalf − gutter − 1.
- `alt_pane_content_width()` ⇒ siblingHalf − gutter − 1 (split only).
- `alt_visible_buffer()` ⇒ Some(idx) of the non-active visible pane in split (else None):
  active_idx==0 ⇒ right_idx (active.max(1)); active_idx>=1 ⇒ 0.
## Accessor
- `pane_wrap_starts(buf_idx) -> Option<&[Vec<u32>]>`: active buffer ⇒ wrap_cache; `wrap_alt_for==buf_idx`
  ⇒ wrap_cache_alt; else None.
## Event-loop compute (per frame)
1. active: if active_buffer().soft_wrap, (re)compute `wrap_cache` at content_width() (now pane-aware).
2. alt: if split && alt_visible_buffer()=Some(i) && buffers[i].soft_wrap, (re)compute `wrap_cache_alt`
   at alt_pane_content_width(), set `wrap_alt_for=Some(i)`; else clear alt (None).
## Invalidation
`activate_buffer` bumps `wrap_text_gen` and clears `wrap_cache_alt`/`wrap_alt_for`.
## No persistence/schema.
