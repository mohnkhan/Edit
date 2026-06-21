# Phase 0 Research: Per-Tab Soft-Wrap

Codebase-internal decisions, verified against post-043 source.

## R1 — Where does the per-tab flag live?

**Decision**: Add `pub soft_wrap: bool` to `Buffer` (`src/buffer/mod.rs:180`). Remove `App::soft_wrap`.

**Rationale**: A tab == a `Buffer`; wrap is a per-file view choice (FR-001). `Buffer` already holds the
other per-tab view state (`cursor`, `scroll_offset`, `selection`), so wrap belongs there.
**Alternatives**: a parallel `Vec<bool>` keyed by buffer index — rejected (a second source of truth to
keep in sync with the buffer vector across open/close/reorder; the exact class of bug we keep hitting).

## R2 — Default for new/opened tabs

**Decision**: `Buffer::new_empty()`/`open()` default `soft_wrap = false`; the **App seeds it from
`config.soft_wrap`** immediately after creating any buffer (initial buffers in `App::new`, `new_buffer`,
`handle_open_file`, session-restore). The toggle no longer writes `config`.

**Rationale**: Preserves FR-005/FR-007 (a user with `config.soft_wrap = true` still gets wrapped new
tabs; the default-off user is unchanged). Buffer constructors have no `config` access, so seeding is an
App-layer concern. **Alternatives**: thread `config.soft_wrap` into `Buffer::open` as a param — rejected
(changes the constructor signature and every test call site for one bool). Keeping `config` as the live
runtime value — rejected (that's the global model we're removing).

**Verified creation sites**: `App::new` builds the initial buffer(s); `new_buffer` (actions.rs),
`handle_open_file` (actions.rs), `do_restore_session` (fileops.rs), and `close_buffer_at`'s
replace-with-empty path all create buffers.

## R3 — Readers (the ~27 sites)

**Decision**: Replace `self.soft_wrap` with the **relevant buffer's** flag:
- Single view + active-buffer geometry/scroll/mouse/cache-gate (`app.rs`, `softwrap.rs`, `editing.rs`,
  `mouse.rs`, event loop) → `self.active_buffer().soft_wrap`.
- Render `Ui::render` split view → left pane `app.buffers[0].soft_wrap`, right pane
  `app.buffers[right_idx].soft_wrap`; single view → `app.active_buffer().soft_wrap`.
- `StatusBar.soft_wrap` and the `View ▸ Soft Wrap` menu toggle-state → active buffer's flag (FR-004).

**Rationale**: FR-006 — geometry must use the wrap setting of the buffer it belongs to. `EditorWidget`
already takes `soft_wrap` + `wrap_starts` per render call, so per-pane rendering needs no widget change.
**Verified**: `editor.rs` handles `soft_wrap == true` with `wrap_starts == None` (the `if let Some(..)`
guard) — so a wrapped non-active split pane with no cache renders without panic (best-effort), satisfying
the split-view edge case without a second cache.

## R4 — Wrap cache & invalidation

**Decision**: Keep the single `wrap_cache` for the active buffer. Compute it only when
`active_buffer().soft_wrap` is on (event-loop gate at app.rs:1116 → read active buffer). It is already
invalidated on every buffer switch by `activate_buffer` (feature 043), so it always matches the active
buffer.

**Rationale**: Minimal change; the 043 fix already guarantees cache↔active-buffer correspondence
(FR-003). A second cache for the non-active split pane is out of scope (spec Assumptions).

## R5 — Toggle behavior

**Decision**: `handle_toggle_soft_wrap` flips `active_buffer().soft_wrap`, then invalidates the wrap
cache (so the active buffer re-wraps/un-wraps on the next frame). It no longer mutates `self.config`.
The existing "too-narrow content guard" (refuse toggle if content width < 10) stays, read from the
active buffer's geometry.

**Rationale**: FR-002 (toggle only the active tab). Dropping the config write matches the spec
Assumption that config is a default seed, not live state.

## Open questions

None. No `NEEDS CLARIFICATION` remain.
