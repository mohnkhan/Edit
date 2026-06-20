# Quickstart / Validation: Bordered-box Find/Replace fields

Prerequisites: a checkout on branch `019-find-replace-field-boxes`; Rust toolchain (MSRV 1.74);
`make tmpfs-setup` recommended to keep build artifacts off the SSD.

## Build & test

```sh
make            # cargo build (debug)
make check      # cargo test — unit + render + integration
```

Expected: all tests pass, including the new Find/Replace render tests in `src/ui/dialog.rs` (or
`src/ui/mod.rs`/`tests/`) asserting bordered boxes + caret.

## Manual validation

Launch with a small file and exercise both dialogs:

```sh
LC_ALL=C.UTF-8 LANG=C.UTF-8 ./target/debug/edit README.md
```

1. **Find box** — Press `Ctrl+F`.
   - Expected: the search term is entered inside a bordered, labeled box (`Find what:`) with a
     visible caret `▏`; matches the file-browser Open box (compare with `Ctrl+O`).
   - Type a word present in the file → matches highlight; the `i/N` count shows.
   - Edit with Backspace/Delete/Home/End/Left/Right → text and caret update inside the box.
   - `Esc` closes the dialog.

2. **Replace boxes** — Press `Ctrl+H`.
   - Expected: two bordered boxes — `Find what:` and `Replace with:`.
   - `Tab` moves the caret between the two boxes; the caret `▏` is only in the focused box.
   - Type a find term and a replacement; `Enter` replaces, `Ctrl+A` replaces all.
   - Toggle `Alt+C` (Case), `Alt+A` (Wrap), `Alt+R` (Regex), `Alt+W` (Word) → `[x]` reflects state.

3. **Long text** — In either box type a string wider than the box.
   - Expected: text scrolls horizontally within the box; the caret stays visible (right-anchored).

4. **Small terminal** — Shrink the terminal to a few rows/columns and reopen `Ctrl+H`.
   - Expected: the dialog clamps to the available size; no panic, no corruption, no drawing outside
     the frame. (Reproduces the feature-015 small-terminal guard.)

## References

- Visual contract: [contracts/render-contract.md](contracts/render-contract.md)
- State model (unchanged): [data-model.md](data-model.md)
- Reused helpers: `truncate_to_width` and the right-anchored caret in `src/ui/file_browser.rs`.
