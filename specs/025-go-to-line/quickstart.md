# Quickstart / Validation: Go to Line

## Build & test

```sh
make tmpfs-setup
make
make check        # cargo test — unit + integration (go_to_line)
make ci-local     # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Unit** (`src/app.rs`): clamp of a parsed number to `[1, line_count]` (`0`→1, `>count`→count,
  overflow→count).
- **Integration** (`tests/integration/go_to_line.rs`): `Ctrl+G` opens the prompt; typing digits + Enter
  moves the cursor to that line's start and scrolls it into view; over-range clamps to the last line;
  `0`/below clamps to the first; Esc leaves the cursor unchanged; empty/non-numeric Enter does not move;
  while the prompt is open, a letter keystroke does not modify the buffer.

## Manual walkthrough

`./target/debug/edit <large-file>`:

1. Press `Ctrl+G` → a small "Go to line:" prompt appears. Type `120`, Enter → the cursor jumps to line
   120 (column 1) and the view scrolls to show it.
2. `Ctrl+G`, type a number bigger than the file, Enter → cursor lands on the last line.
3. `Ctrl+G`, type `0`, Enter → cursor lands on line 1.
4. `Ctrl+G`, press Esc → nothing moves. `Ctrl+G`, Enter on an empty field → nothing moves.
5. Open the Search menu → "Go to Line" is listed and does the same thing.
6. While the prompt is open, type letters → the buffer is not edited.

## Expected outcome

The cursor can be moved to any line by number via keyboard or the Search menu, with out-of-range clamped
and invalid input ignored, the target line always visible, and no change to editing or other dialogs
(SC-001..SC-004).
