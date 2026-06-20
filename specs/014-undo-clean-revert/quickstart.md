# Quickstart: Undo-to-clean and Revert

Validation guide for feature 014. See [spec.md](./spec.md) and
[contracts/clean-state-and-revert.md](./contracts/clean-state-and-revert.md).

## Prerequisites

```sh
make   # builds ./target/debug/edit
```

## Manual validation

1. **Undo-to-clean (US1)** — open a file: `./target/debug/edit Cargo.toml`. Type a character — the
   status bar shows `[Modified]`. Press Undo (`Ctrl+Z`) — `[Modified]` disappears. Press Redo
   (`Ctrl+Y`) — `[Modified]` returns.
2. **No false-clean (US2)** — save (`Ctrl+S`). Type `A`, type `B`, Undo once (back to A), then type `C`.
   The buffer stays `[Modified]` and cannot be returned to clean by undo/redo alone.
3. **Revert (US3)** — with unsaved edits, open File ▸ Revert (or `Alt+F` then `r`). Confirm at the
   prompt — the buffer reloads from disk and shows clean. Repeat and cancel — nothing changes.
4. **Revert with no file** — start `./target/debug/edit` (no file), type something, File ▸ Revert —
   a status notice says there is nothing to revert; the buffer is unchanged.

## Automated validation

```sh
cargo test --lib undo            # UndoStack saved-marker unit tests (incl. divergent no-false-clean)
cargo test --test undo_clean_revert   # end-to-end modified-flag + Revert
make ci-local                    # full gate
```

## Expected outcomes

- Undoing to the saved/opened content clears `[Modified]`; redoing/diverging restores it.
- The buffer is never shown clean unless its content equals the saved baseline.
- File ▸ Revert restores the on-disk version (with confirmation when there are unsaved changes) and is a
  safe no-op for never-saved buffers.
- All pre-existing undo/redo, save, and editing tests stay green.
