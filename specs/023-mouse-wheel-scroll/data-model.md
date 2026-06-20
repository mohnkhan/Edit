# Phase 1 Data Model: Mouse-wheel scrolling

UI-state only — no persistence, config, or new types. The "data" is the routing of a wheel event to an
existing scroll offset.

## Wheel event (input)

| Field | Source | Use |
|---|---|---|
| direction | `NormalizedMouseKind::ScrollUp` / `ScrollDown` | sign of the step |
| `col` / `row` | normalized event | choose the editor pane (split view) under the cursor |

Constant: `WHEEL_STEP = 3` (lines/rows/items per notch).

## Routing target (modal-wins precedence)

| Open surface | Wheel effect |
|---|---|
| Help / About (`pending_help`) | `help_scroll = help_scroll ± step` (render clamps the top bound; saturating at 0) |
| Encoding select (`pending_encoding_select`) | cursor index `± step`, clamped `[0, n-1]` |
| File browser (`file_browser`) | `move_up`/`move_down(visible_rows)` ×`step` (scrolls via `ensure_visible`) |
| Plugin manager (`pending_plugin_manager`) | `plugin_manager_cursor ± step`, clamped `[0, n-1]` |
| Find/Replace (`pending_find_replace`) | ignored (nothing to scroll) |
| none (editor) | scroll the pane-under-cursor buffer's `scroll_offset.0` (see below) |

## Editor viewport scroll (helper)

```
wheel_scroll_editor(buf_idx, down, step):
  content_rows = if soft_wrap { wrap_cache.total_visual_rows() } else { buffers[buf_idx].rope.line_count() }
  max = content_rows.saturating_sub(1)
  off = buffers[buf_idx].scroll_offset.0
  buffers[buf_idx].scroll_offset.0 = if down { (off + step).min(max) } else { off.saturating_sub(step) }
  // cursor is NOT modified
```

Pane selection (split view): `buf_idx = 0` when `col < width/2`, else the right pane's buffer index
(`active_idx.max(1)` when more than one buffer, else 0) — mirrors the render in `src/ui/mod.rs`.

Non-editor rows (FR-009): when no modal is open, a wheel on the menu-bar row (`row == 0`) or the
status-bar row (`row == term_rows-1`) is **ignored** (no editor scroll).

## Invariants

- Wheel never moves the cursor (editor) — viewport only.
- Scroll offsets stay in range (`>= 0`, `<= content-1` / list `n-1`); at a limit the wheel is a no-op.
- Routing is mutually exclusive (one target per event); a modal is always preferred over the editor.
- No existing click/drag/keyboard path is entered for a wheel event (handled and returned early).
