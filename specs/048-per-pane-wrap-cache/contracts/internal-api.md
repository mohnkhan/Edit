# Internal Contract: Per-Pane Wrap Cache
- C-1: `content_width()` returns the ACTIVE pane's content width (full in single, half in vertical split).
- C-2: `pane_wrap_starts(buf_idx)` returns the wrap visual-starts for that visible buffer (active→primary,
  non-active visible→alt, else None).
- C-3: render passes each split pane `pane_wrap_starts(its_buffer_idx)`; the per-pane vertical scrollbar
  uses the same cache for its content-row total.
- B-1: a wrapped non-active split pane renders WRAPPED (FR-002).
- B-2: each pane wraps at its own width (FR-001) and its scrollbar total matches (FR-003).
- B-3: single-view output unchanged (FR-004).
- B-4: tab switch / edits never leave a stale cross-pane cache (FR-005).
- T: render tests (TestBackend) for both-wrapped split + single-unchanged; fuzz stays green; clippy/fmt.
