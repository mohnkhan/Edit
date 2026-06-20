# Contract: Clean state tracking and Revert

## Clean-state contract

| Situation | `[Modified]` shown? |
|---|---|
| Freshly opened file (no edits) | No |
| Brand-new empty buffer (no edits) | No |
| After any edit that diverges from the saved baseline | Yes |
| After undo/redo lands the content exactly on the saved baseline | No |
| After redo away from the saved baseline | Yes |
| After "save → undo part-way → new edit" (divergent) | Yes (saved point unreachable) |
| Immediately after a successful Save / Save As | No (current point becomes the new baseline) |

- The buffer is clean **iff** its current content equals the saved baseline (last write, or open-time
  content for an unsaved-but-opened file). Implemented via `UndoStack::is_at_saved()`.
- A divergent edit that discards a redo branch containing the saved marker MUST invalidate it
  (`saved = None`), so the buffer is never falsely shown clean.

## Revert contract (`Action::Revert`, File ▸ Revert, menu-only)

| Precondition | Effect |
|---|---|
| Buffer has no file path (never saved) | No-op; status notice "Nothing to revert (never saved)". |
| File-backed, **no** unsaved changes | Reload from disk (clean reload); no confirmation. |
| File-backed, **has** unsaved changes | Show confirm modal. Enter/Y → reload from disk; Esc/N → no change. |
| Reload fails (missing/unreadable file) | Status notice with the error; buffer and Modified state unchanged. |

- After a confirmed/clean Revert: buffer content equals the on-disk file (via the existing
  encoding/line-ending-aware open path), the buffer is clean, undo history is reset, and the cursor is at
  a valid position.
- Revert reuses `App::reload_from_disk`; it does not introduce a second file-reading path.

## Confirmation modal contract

- Shown only when reverting a modified buffer. While shown it is modal (intercepts input): Enter/`Y`
  confirm, Esc/`N` cancel. Reuses existing modal precedence (sits with the other confirm dialogs).

## Invariants / non-regression

- Ordinary edits still set Modified (FR-011).
- Existing undo/redo, save, autosave, and external-change-reload behavior are unchanged except that
  undo/redo now clear Modified when returning to the saved baseline.
- No file is read except the active buffer's own path, through the existing sanitized open path.
