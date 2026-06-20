# Quickstart / Validation: Mouse-wheel scrolling

## Build & test

```sh
make tmpfs-setup
make
make check        # cargo test — unit + integration (mouse_wheel)
make ci-local     # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Unit** (`src/app.rs`): the editor wheel helper increments/decrements `scroll_offset.0` by the step,
  clamps at 0 (top) and `content-1` (bottom), and leaves the cursor unchanged.
- **Integration** (`tests/integration/mouse_wheel.rs`): synthesized `ScrollUp`/`ScrollDown` events scroll
  the editor (cursor unchanged, bounded), the file-browser listing, and Help (`help_scroll`, bounded at
  0); with a modal open the wheel scrolls the modal not the editor; left-click and press-drag selection
  still behave (no regression).

## Manual walkthrough

`./target/debug/edit <large-file>`:

1. Roll the wheel over the editor → the view scrolls ~3 lines per notch; the vertical scrollbar tracks;
   the cursor stays put; rolling up at the top / down at the bottom does nothing.
2. `Ctrl+O` on a big directory → wheel scrolls the listing (highlight stays visible).
3. `F1` Help (overflowing) → wheel scrolls the cheat sheet; its scrollbar tracks; stops at the ends.
4. Open the encoding dialog (`F12`) / plugin manager → wheel moves through the list.
5. Confirm clicks, dialog buttons, and click-drag text selection still behave exactly as before.

## Expected outcome

Every scrollable surface responds to the wheel, bounded and panic-free, with no change to existing
click/drag/keyboard behavior (SC-001..SC-004).
