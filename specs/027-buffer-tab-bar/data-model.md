# Phase 1 Data Model: Buffer tab bar

UI-state only — no persistence/config.

## New state (in `src/app.rs`)

| Field | Type | Role |
|---|---|---|
| `pending_close_confirm` | `Option<usize>` | `Some(idx)` while the close-confirm for buffer `idx` is open |

(The tab list itself is derived from the existing `buffers` / `active_idx` — no stored tab model.)

## Derived geometry

- `tab_bar_visible() = buffers.len() > 1`.
- `editor_top() -> u16 = 1 + if tab_bar_visible { 1 } else { 0 }` (menu row + optional tab row).
- `viewport_height()` = `terminal_rows - editor_top() - 1(status) - hbar(non-wrap)`.
- Editor area (render + wheel + scrollbar) = `Rect::new(0, editor_top(), w, h - editor_top() - 1)`.
- `handle_mouse_click`: editor row = `row - editor_top()`; rows above `editor_top()` are not editor.

## Tab geometry (`src/ui/tabbar.rs`)

`tab_hit_regions(area, buffers, active) -> Vec<TabRegion>` where:

| Field | Meaning |
|---|---|
| `idx` | buffer index |
| `label_rect` | clickable region that switches to the buffer |
| `close_rect` | clickable `[x]` region that closes the buffer |

- Each tab = ` <name><modified?> [x] ` styled; active tab highlighted. Layout left→right with a small
  separator; on overflow, the visible window scrolls so the **active** tab is included; names truncate by
  display width. The renderer draws from the same `tab_hit_regions`, so drawn == clickable.

## Close-confirm (reusing feature-016 `ButtonDialog`)

| Aspect | Value |
|---|---|
| variant | `ButtonDialog::CloseConfirm` (active when `pending_close_confirm.is_some()`) |
| buttons | `Save (Enter)` / `Discard (D)` / `Cancel (Esc)` |
| default focus / cancel | Cancel (index 2) |
| activate | 0 → `buffers[idx].save()` then `close_buffer_at(idx)`; 1 → `close_buffer_at(idx)`; 2 → dismiss |
| body | "Save changes to `<name>` before closing?" |

`close_buffer_at(idx)`: remove `buffers[idx]`; if `idx < active_idx` → `active_idx -= 1`; clamp
`active_idx` to `[0, len-1]`. (Tab bar only shows with 2+ buffers, so a close always leaves ≥1.)

## Behavior mapping (tab-row click)

```
click on tab row (row == editor_top()-1, when tab_bar_visible):
  for region in tab_hit_regions:
    if click in region.close_rect:
      if buffers[idx].modified { pending_close_confirm = Some(idx) } else { close_buffer_at(idx) }
      return
    if click in region.label_rect: active_idx = idx; return
  // outside any tab → no-op
```

## Invariants

- Single source of truth for the editor top/height; every consumer uses `editor_top()`.
- Drawn tab/`[x]` geometry == clicked geometry; the active tab is always visible.
- `[x]` on a modified buffer never closes without the confirm (no silent data loss).
- Tab-row clicks never move the text cursor; single-buffer layout unchanged.
- No panic on any terminal size or buffer count.
