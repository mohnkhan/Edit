# Phase 1 Data Model: Per-Tab Soft-Wrap

No persisted/schema data. The change is the home of one boolean.

## Entity change: `Buffer`

```rust
pub struct Buffer {
    // … existing per-tab view state …
    pub cursor: CursorPos,
    pub scroll_offset: (usize, usize),
    pub selection: Option<Selection>,
    pub soft_wrap: bool,   // NEW (Feature 044): this tab's wrap setting
    // …
}
```

- Default in `Buffer::new_empty()` / `Buffer::open()`: `false`.
- Seeded by the App from `config.soft_wrap` at every creation site (see research R2).

## Entity removal: `App::soft_wrap`

`App::soft_wrap: bool` is **removed**. Its readers move to the relevant buffer's flag:

| Context | Before | After |
|---|---|---|
| active-buffer geometry / scroll / mouse / cache gate | `self.soft_wrap` | `self.active_buffer().soft_wrap` |
| render — single view | `app.soft_wrap` | `app.active_buffer().soft_wrap` |
| render — split left / right pane | `app.soft_wrap` | `app.buffers[0].soft_wrap` / `app.buffers[right_idx].soft_wrap` |
| status bar indicator | `app.soft_wrap` | active buffer's flag |
| `View ▸ Soft Wrap` menu check | `app.soft_wrap` | active buffer's flag |
| toggle target | `self.soft_wrap = !self.soft_wrap; self.config.soft_wrap = …` | `active_buffer().soft_wrap ^= true` + invalidate cache (no config write) |

`wrap_cache` / `wrap_text_gen` stay on `App` (single active-buffer cache; invalidated on switch — 043).

## Invariants

- INV-1: A tab's `soft_wrap` changes only via the toggle while that tab is active (or at creation).
- INV-2: The geometry/render for a given buffer uses that buffer's `soft_wrap`.
- INV-3: `wrap_cache` (when present) corresponds to the active buffer and is only consulted when the
  active buffer's `soft_wrap` is on.

## No migration / serialization

Wrap state is not written to the session file; restored buffers seed from `config.soft_wrap` (out of
scope to persist — spec Assumptions).
