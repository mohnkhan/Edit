# Internal Contract: Per-Tab Soft-Wrap

No external/public API. Behavioral + structural guarantees.

## Structural

- C-1: `Buffer` gains `pub soft_wrap: bool`; `App::soft_wrap` is removed. Every former reader of
  `App::soft_wrap` reads the relevant buffer's flag (active buffer, or the specific pane buffer in split
  view).
- C-2: `wrap_cache`/`wrap_text_gen` remain on `App`; the event-loop cache gate reads
  `active_buffer().soft_wrap`; the cache is invalidated on buffer switch (feature 043 `activate_buffer`).

## Behavioral

- B-1 (FR-002): toggling wrap changes only `active_buffer().soft_wrap`; all other buffers' flags are
  unchanged.
- B-2 (FR-003): after a tab switch, the active tab renders for its own `soft_wrap`; the wrap cache
  corresponds to the active buffer (no ghost wrap / gutter from another tab).
- B-3 (FR-004): the `View ▸ Soft Wrap` check mark and status-bar wrap indicator equal
  `active_buffer().soft_wrap`.
- B-4 (FR-005/FR-007): a new/opened buffer's `soft_wrap` equals `config.soft_wrap` at creation;
  single-tab and untouched-default behavior is identical to before.
- B-5 (FR-008): no panic or corruption for any tab/pane/wrap combination (extends 043; covered by the
  042 fuzz sweep + new tests).

## Test contract

- T-1: existing soft-wrap tests pass with assertions retargeted from `app.soft_wrap` to
  `app.active_buffer().soft_wrap` (no behavior change).
- T-2: new tests — per-tab independence (toggle A, B unchanged, switch round-trip preserves both); and
  indicator/toggle act on the active buffer.
- T-3: full suite incl. 043 wrap-cache tests and the 042 fuzz sweep stays green; `fmt` + `clippy -D
  warnings` clean.
