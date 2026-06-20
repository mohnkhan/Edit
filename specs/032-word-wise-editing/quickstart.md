# Quickstart / Validation: Word-wise navigation, selection, and deletion

## Build & test

```sh
make tmpfs-setup
make
make check        # cargo test — unit + integration (word_editing)
make ci-local     # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Unit** (`app.rs`): `next_word_pos` — mid-word, in-whitespace, at line ends (crossing), at buffer ends
  (no-op), multibyte/wide; `move_word` clears selection; `move_word_selecting` builds the selection;
  `delete_word` removes the expected range in one undo step, deletes an active selection, no-ops at ends,
  and is blocked (with a message) in a read-only buffer.
- **Unit** (`keymap.rs`): Ctrl+Left/Right, Ctrl+Shift+Left/Right, Ctrl+Backspace, Ctrl+Delete map to the
  six new actions; existing F-keys/Ctrl bindings unchanged.
- **Integration** (`tests/integration/word_editing.rs`): drive the actions end-to-end; assert cursor,
  `selection_text()`, and rope content (incl. undo).

## Manual walkthrough

1. Open a file; place the cursor mid-line. Ctrl+Right / Ctrl+Left jump by word, crossing line ends.
2. Ctrl+Shift+Right twice selects two words; Ctrl+C copies exactly them.
3. Ctrl+Backspace deletes the word before the cursor; Ctrl+Delete the word after; Ctrl+Z restores it.
4. Open a read-only file (`--readonly`): Ctrl+Backspace shows "Buffer is read-only" and changes nothing.

## Expected outcome

Word-wise move/select/delete work consistently with double-click word boundaries, multibyte-safe, with no
regression and no new dependencies (SC-001..SC-005).
