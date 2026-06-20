# Phase 1 Data Model: Undo-to-clean and Revert

## `UndoStack` (extended) — `src/buffer/undo.rs`

| Field | Type | Notes |
|---|---|---|
| `ops` | `Vec<EditOp>` | unchanged — linear op history |
| `cursor` | `usize` | unchanged — position in `ops`; `ops[cursor..]` is the redo branch |
| `saved` | `Option<usize>` | **new** — cursor value at the last save/open; `None` = no reachable saved point (e.g. discarded by a divergent edit) |

### New methods

- `mark_saved(&mut self)` → `self.saved = Some(self.cursor)`. Called on open and after a successful save.
- `is_at_saved(&self) -> bool` → `self.saved == Some(self.cursor)`. True ⟺ current content equals the
  saved baseline along the current branch.

### Modified behavior

- `push(op)`: **before** truncating the redo branch, if `saved == Some(s)` and `s > cursor`, set
  `saved = None` (the marker was in the branch about to be discarded → unreachable). Then truncate +
  push as today. Markers with `s <= cursor` are preserved (retained ops are unchanged).
- `undo` / `redo`: unchanged mechanics; the buffer layer recomputes `modified` afterward.

### Invariants

- **No false-clean**: `is_at_saved()` is true only when the current content equals the saved baseline.
  Proof sketch: along an unbranched history, `cursor` ↔ content is a bijection, so `cursor == saved`
  ⟺ same content. A branch (truncating push) that discards the marker sets `saved = None`, so a
  coincidental future `cursor == old s` cannot report clean.
- **Determinism**: state depends only on the op sequence and save/open calls.

## `Buffer` (behavior) — `src/buffer/mod.rs`

- `open(...)`: after constructing with a fresh `UndoStack`, call `undo_stack.mark_saved()` (cursor 0) so
  a freshly opened/reverted buffer is clean. `modified` stays `false`.
- `new_empty()`: `mark_saved()` on its fresh stack so a brand-new empty buffer is clean; after the first
  edit it becomes modified, and undo back to empty is clean again.
- `refresh_modified(&mut self)` (**new helper**): `self.modified = !self.undo_stack.is_at_saved();`
  Used by the undo/redo path.

## `App` (behavior) — `src/app.rs`

- On successful `Save` / `Save As` / `Save As Encoding`: after setting content written, call
  `buffer.undo_stack.mark_saved()` (and keep `modified = false`). This makes the *current* point the new
  clean baseline.
- `apply_history_cursor(...)` (undo/redo post-step): replace `buf.modified = true` with
  `buf.refresh_modified()` (i.e. `modified = !is_at_saved()`).
- Ordinary edit sites: keep `modified = true` (a new edit always diverges from saved).
- **Revert** (`Action::Revert`):
  - no `path` → status "Nothing to revert (never saved)"; no change.
  - `path` + not modified → `reload_from_disk(active_idx)` (clean reload).
  - `path` + modified → set `pending_revert_confirm = Some(active_idx)`; modal asks to confirm.
- `pending_revert_confirm: Option<usize>` (**new App field**): when set, a confirm modal is shown and
  intercepts input (Enter/Y → `reload_from_disk` then clear; Esc/N → clear, no change).

## `Action` — `src/input/keymap.rs`

- New variant `Revert` (no default keybinding). Parsed in the string→Action map for completeness.

## Menu — `src/ui/menubar.rs`

- New `FILE_MENU` item `{ label: "Revert", action: Action::Revert, mnemonic: Some('r') }`, placed before
  `Exit`. Unique within the File menu (n/o/s/a/e/r/x).
