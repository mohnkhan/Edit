# Quickstart / Validation: Caret-on-click in dialog text fields

## Build & test

```sh
make tmpfs-setup
make
make check        # cargo test — unit + integration (field_caret)
make ci-local     # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Unit** (`ui::width`): `field_caret_at` — fits/overflow/right-anchored, multibyte & wide, clamp to end,
  empty value.
- **Unit** (`ui::mod`): `find_replace_field_rects` rows match `render_find_field` (query `dy+3`,
  replacement `dy+7`, x `dx+2`, width `dw-4`).
- **Unit** (`file_browser`): filename caret insert/delete/move (Left/Right/Home/End) + field text rect.
- **Unit** (`app`): Go-to-Line caret keys (insert at caret, digits-only, Backspace, Left/Right/Home/End).
- **Integration** (`tests/integration/field_caret.rs`): a click in each field (Find, Name, Go-to-Line)
  positions the caret; mid-string insert after a click works.

## Manual walkthrough

1. Find (Ctrl+F): type `hello world`, click on the `w` → caret lands there; type → inserts mid-string.
2. Save As: type a filename, press Left twice, type → inserts mid-string; click earlier → caret moves;
   Home/End jump.
3. Go to Line (Ctrl+G): type `123`, click between digits → caret moves; type a digit → inserts there;
   letters are still rejected; Enter jumps.

## Expected outcome

All three dialog fields support click-to-position and (for Name and Go-to-Line, newly) caret editing,
with no regression and no new dependencies (SC-001..SC-004). Closes #58.
