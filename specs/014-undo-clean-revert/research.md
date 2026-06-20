# Phase 0 Research: Undo-to-clean and Revert

## R1. How to track "clean vs. modified" — saved marker in undo history

**Decision**: Add `saved: Option<usize>` to `UndoStack`, recording the value of `cursor` at the moment
of the last save (or open). The buffer is clean iff `saved == Some(cursor)` (`is_at_saved()`). `modified`
is derived as `!is_at_saved()`.

**Rationale**: The undo history is already linear with a `cursor` that uniquely identifies the current
content along the *current* branch. Comparing the cursor to a recorded saved cursor is O(1) and exactly
captures "are we back at the saved content" — including redo back to it. It avoids storing or hashing
content. This directly fixes the current bug where `apply_history_cursor` unconditionally sets
`modified = true` even when undo returns to the saved content.

**Alternatives considered**:
- *Per-edit boolean flag* (status quo): cannot represent "returned to saved", which is the whole point.
- *Content hash / snapshot compare*: correct but O(n) per check and more memory; unnecessary given the
  linear history. Rejected by YAGNI.
- *Counting edits since save*: breaks under divergent edits (same count, different content) — exactly the
  false-clean hazard. Rejected.

## R2. Avoiding false-clean after divergent edits (FR-004, safety-critical)

**Decision**: In `UndoStack::push` (which truncates the redo branch before appending), if the saved
marker points into the region being discarded — `saved == Some(s)` with `s > cursor` — set `saved = None`
(the saved point is now unreachable). Markers at `s <= cursor` remain valid because `ops[0..cursor]` are
retained unchanged, so undoing back to `s` reproduces the same content.

**Rationale**: After "save → undo part-way → type new edit", the old saved position lived in the
discarded redo branch. If we kept the numeric marker, a later cursor landing on the same index would show
a false clean despite different content. Dropping the marker when it falls in the discarded branch makes
`is_at_saved()` return false until the next save — never a false clean.

**Worked example**: save at cursor 2 (`saved=2`); undo to 1; type C → `push` sees `saved=2 > cursor=1` →
`saved=None`; truncate to `[A]`, push C → `[A,C]`, cursor 2. `is_at_saved()` = (`None == Some(2)`) =
false → Modified. Correct.

## R3. Deriving `modified` without touching every edit site

**Decision**: Ordinary edits always diverge from the saved content (a fresh `push` advances `cursor`
beyond any `saved <= old cursor`), so the existing `modified = true` at edit sites stays correct. Only
the undo/redo path can *return* to the saved content, so the single behavioral change is in
`apply_history_cursor` (the shared undo/redo post-step): set `modified = !undo_stack.is_at_saved()`
instead of unconditional `true`. Add a small `Buffer::refresh_modified()` helper for clarity and reuse.

**Rationale**: Minimal, low-risk change surface; the saved-marker invariant does the heavy lifting.

## R4. Revert implementation — reuse the existing reload path

**Decision**: Revert reuses `App::reload_from_disk(buf_idx)` (feature 007), which calls `Buffer::open`
and replaces the buffer — yielding `modified=false`, a fresh `UndoStack`, and (with R1) a clean buffer.
Revert flow: if the active buffer has no `path` → status message "Nothing to revert (never saved)"
(FR-009, no-op). Else if the buffer is modified → show a confirmation modal; on confirm → reload; on
cancel → unchanged (FR-007). A clean buffer reverts (reloads) without confirmation. Read failure surfaces
a status notice and leaves the buffer intact (FR-010), matching `reload_from_disk`'s existing error path.

**Rationale**: `reload_from_disk` already does exactly the right thing (encoding/line-ending-aware open,
error handling). Revert is "reload, but user-initiated and confirmed". No duplicate I/O logic.

**Alternatives considered**: In-place rope replacement preserving the same `Buffer` object — more code,
must manually reset undo/cursor/selection; `reload_from_disk` already handles it. Rejected.

## R5. Confirmation modal for destructive Revert

**Decision**: Add a lightweight `pending_revert_confirm: bool` (or `Option<usize>` buf index) modal,
intercepted like the other modals in `handle_action` (Enter/Y = confirm, Esc/N = cancel), reusing the
existing modal precedence and a simple dialog render. Only shown when the buffer is modified.

**Rationale**: Consistent with the editor's existing confirm dialogs (save prompt, external change);
prevents silent data loss (FR-007). Keeping it a dedicated flag avoids overloading unrelated dialogs.

## R6. Menu surface and keybinding

**Decision**: Add `File ▸ Revert` between "Save As Encoding..." and "Exit", `Action::Revert`, mnemonic
`r` (free in the File menu). No keyboard binding (clarified: menu-only).

**Rationale**: Matches the clarified scope; `r` is unused among File item accelerators
(n/o/s/a/e/x), so feature-013 mnemonic uniqueness holds.
