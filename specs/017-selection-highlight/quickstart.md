# Quickstart: Visible text selection

See [spec.md](./spec.md) and [contracts/selection.md](./contracts/selection.md).

## Build
```sh
make
```

## Manual validation
1. **Highlight / Select All** — open a file; `Ctrl+A` → the whole buffer is highlighted (reverse video).
2. **Shift-select** — place the cursor; hold Shift and press `→` a few times → a growing highlight; `Ctrl+C` then move and `Ctrl+V` pastes exactly that text. `Shift+End` selects to end of line.
3. **Clear** — press an arrow without Shift → selection disappears; type a char over a selection → it replaces the selection.
4. **Mouse drag** — press and drag across some text → it highlights; release, `Ctrl+X` cuts it. A single click clears the selection.

## Automated
```sh
cargo test --lib selection           # range math + editor render
cargo test --test selection          # shift-select + copy, mouse drag, clear-on-move
make ci-local
```

## Expected
- Select All highlights the whole buffer; Shift+arrows/Home/End select; mouse drag selects; non-shift
  move / typing / single click clears. Copy/Cut act on the selection. Search-match highlight stays
  distinct. No regression to movement/editing.
