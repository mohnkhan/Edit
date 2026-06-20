# Contract: Scroll affordances + dialog button polish

Behavioral contract the tests assert against.

## Shared scrollbar helper (`src/ui/scrollbar.rs`)

| Input | Behavior |
|---|---|
| `content_len <= viewport_len` | draw nothing (no bar; no content hidden) |
| `content_len > viewport_len` | draw a `Scrollbar` on the view's reserved edge; thumb size ∝ `viewport_len/content_len`, thumb top ∝ `position/content_len` |
| any | never panic on tiny areas; clamp `position` to `[0, content_len]` |

## Editor (`src/ui/editor.rs` + `src/ui/mod.rs` + `src/app.rs`)

- Vertical bar on the rightmost reserved column; present whenever total lines (or total visual rows in
  soft-wrap) exceed visible rows. Thumb tracks `scroll_offset.0`.
- Horizontal bar on the bottom reserved row (non-wrap only); present when a visible line is wider than
  the content width or `scroll_offset.1 > 0`. Thumb tracks `scroll_offset.1`. **Never in soft-wrap.**
- Reservation invariant: `viewport_height()`, the horizontal content-width helper, and
  `handle_mouse_click` all account for the reserved column/row; a click on a reserved bar cell does not
  move the cursor.
- Works with the line-number gutter and in both split panes (each pane gets its own bars; no overlap with
  the divider).

## File browser (`src/ui/file_browser.rs`)

- Vertical bar in the list area's rightmost interior column when `entries.len() > list_rows`. Thumb
  tracks `scroll`. The entry name budget reserves that column so names never draw under the bar.
- `hit_test` unchanged (the bar column is inert); existing entry click/double-click behavior preserved.

## Help / About (`src/ui/mod.rs` + `src/app.rs`)

- A bordered **Close (Esc)** button is drawn on both Help and About.
- Click on the Close button dismisses the overlay (same effect as `Esc`).
- `Esc` / Enter / printable still dismiss as before.
- Vertical scrollbar shown when the content overflows the body rows; thumb tracks `help_scroll`.

## Encoding select / plugin manager (`src/ui/dialog.rs` / `src/ui/plugin_manager.rs`)

- Vertical scrollbar shown when the list overflows the visible rows; thumb tracks the list scroll.
- All feature-020 button/focus-ring behavior preserved.

## Key-hint button labels (app-wide)

| Dialog / button | Displayed label (example) |
|---|---|
| Save prompt | `Save (Enter)` · `Discard (D)` · `Cancel (Esc)` |
| Session restore | `Restore (Enter)` · `Decline (Esc)` |
| External change | `Reload (Enter)` · `Keep (Esc)` |
| Revert confirm | `Revert (Enter)` · `Cancel (Esc)` |
| Plugin consent | `Allow (Enter)` · `Deny (Esc)` |
| Encoding select | `OK (Enter)` · `Cancel (Esc)` |
| Plugin manager | `Close (Esc)` |
| Find/Replace | `Find (Enter)` · `Replace` · `Replace All (Ctrl+A)` · `Close (Esc)` |
| File browser | `Open (Enter)` / `Save (Enter)` · `Cancel (Esc)` |
| Help / About | `Close (Esc)` |

- Pressing the shown key runs the same action as before (label is informational).
- Click/focus activation maps to the same action regardless of the displayed text (keyed on index).
- The exact hint wording is a presentation detail; tests assert the key appears on the label and that the
  action/dispatch is unchanged — not the precise punctuation.

## No-regression (all surfaces)

- Scrolling (arrows, PgUp/PgDn, cursor movement), list navigation, dialog actions, and dismissal keys
  behave exactly as before this feature.
- No panic / no layout corruption across terminal sizes, resize, split view, and line-number mode.
