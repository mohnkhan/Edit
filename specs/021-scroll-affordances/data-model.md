# Phase 1 Data Model: Scroll affordances + dialog button polish

UI-state only — no persisted entities, config, or file formats. The "data" is the per-view scroll inputs
(already in state) and the composed button labels.

## Scrollbar indicator (derived, not stored)

Each scrollable view supplies three numbers to the shared `src/ui/scrollbar.rs` helper:

| Field | Meaning |
|---|---|
| `content_len` | total scrollable extent (lines / entries / visual columns) |
| `viewport_len` | visible extent along the same axis |
| `position` | current scroll offset (top/left of the viewport) |

Rule: the helper draws nothing when `content_len <= viewport_len` (FR-007); otherwise it renders a
ratatui `Scrollbar` whose thumb size/position derive from these three.

### Per-view sources

| View | Axis | content_len | viewport_len | position |
|---|---|---|---|---|
| Editor (non-wrap) | vertical | `rope.line_count()` | visible text rows | `scroll_offset.0` |
| Editor (soft-wrap) | vertical | `WrapCache::total_visual_rows()` | visible text rows | `scroll_offset.0` |
| Editor (non-wrap) | horizontal | max visual width of **visible** lines | content width (cols, minus gutter) | `scroll_offset.1` |
| File browser | vertical | `entries.len()` | `list_rows` | `scroll` |
| Help / About | vertical | total cheat-sheet/about lines | body rows | `help_scroll` (clamped) |
| Encoding select | vertical | `ENCODING_OPTIONS.len()` | visible list rows | list scroll offset |
| Plugin manager | vertical | plugin instance count | visible list rows | list scroll offset |

## Editor area reservation (single source of truth)

The editor text area is the pane area minus: menubar (top), statusbar (bottom), gutter (left, width 4
when line numbers on), **vertical scrollbar (right, 1 col)**, and **horizontal scrollbar (bottom, 1 row;
non-wrap only)**. Consumers kept in sync:

- `App::viewport_height()` — subtracts the horizontal-bar row in non-wrap mode.
- horizontal content-width helper (`src/app.rs:~3657`) — subtracts the vertical-bar column.
- `App::handle_mouse_click` — treats the reserved bar cells as inert (no cursor placement there).
- `EditorWidget::render` — receives the already-shrunk area from `src/ui/mod.rs`.

## Dialog button label

| Part | Source |
|---|---|
| action name | existing label ("OK", "Cancel", "Close", "Save", "Find", …) |
| key hint | the button's existing primary shortcut, in parentheses ("(Enter)", "(Esc)", …) |

Composition happens in the label builder; dispatch (click/focus/activate) keys on button **index /
identity**, never the displayed text (FR-010), so wider labels don't affect behavior.

## Invariants

- A scrollbar never overlaps content — its edge is reserved by the view.
- Exactly the reserved cells hold the bar; the rest of the view is unchanged.
- Scroll position shown == actual scroll offset at all times.
- Adding key hints changes only displayed text; every button maps to the same action as before.
