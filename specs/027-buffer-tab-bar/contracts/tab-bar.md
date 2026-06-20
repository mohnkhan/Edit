# Contract: Buffer tab bar

Behavioral contract the tests assert against.

## Visibility & content

- `buffers.len() > 1` ⇒ a one-row tab bar is shown directly below the menu bar; `== 1` ⇒ no tab bar.
- Each tab shows the buffer's file name (or "[No Name]"); the active tab is highlighted; a modified
  buffer's tab shows a modified marker.
- Overflow: tabs that don't fit truncate/scroll so the **active** tab stays visible; the row never
  corrupts.

## Click behavior (tab row)

| Click target | Effect |
|---|---|
| a tab's label region | switch: `active_idx = idx` (same as keyboard select) |
| a tab's `[x]` region | close that buffer: clean → close immediately; modified → open CloseConfirm |
| tab row, outside any tab | no-op |

A tab-row click never places the text cursor.

## Close-confirm (modified buffer via `[x]`)

- Buttons **Save / Discard / Cancel**; Cancel is the default focus.
- Save → save the buffer, then close it. Discard → close without saving. Cancel → dismiss, nothing closes.
- Reuses the feature-016 boxed-button focus ring (Tab/Shift+Tab, Enter/Space, click) + `Esc` = Cancel.
- Closing down to a single buffer hides the tab bar.

## Geometry (with the tab bar shown)

- The editor area starts one row lower and is one row shorter; `viewport_height` reflects this.
- A click in the editor text lands on the correct line/column (the tab row is accounted for).
- Paging and cursor-visibility use the reduced height; the wheel (023) and scrollbars (021/024) act on
  the reduced editor area.

## No-regression

- Single-buffer editing is unchanged (no tab bar; full-height editor; identical geometry).
- `Ctrl+Tab` / `Ctrl+Shift+Tab` switching is unchanged.
- Opening buffers and all editing behavior are unchanged; the feature adds only the tab-bar UI + its
  click/close interaction.
- No panic on any terminal size or buffer count (including very long names / many buffers).
