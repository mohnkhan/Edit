# Quickstart / Validation: Interactive scrollbars

## Build & test

```sh
make tmpfs-setup
make
make check        # cargo test — unit + integration (scrollbar_interaction)
make ci-local     # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Unit** (`src/ui/scrollbar.rs`): `thumb_span` (full track when content fits; min len 1; in-bounds;
  monotonic), `pos_to_offset` (top→0, bottom→max, clamped, monotonic), `hit_zone` classification.
- **Integration** (`tests/integration/scrollbar_interaction.rs`): a press on the editor v-bar track below
  the thumb pages the view forward (cursor unchanged); a press on the thumb + drag scrolls proportionally;
  a press-drag in the text body still selects (no scroll); a press on the bar does not select; clicking
  the file-browser / Help bars scrolls them; with a modal open the editor offset is unchanged.

## Manual walkthrough

`./target/debug/edit <large-file>`:

1. Click the editor's right-edge scrollbar below the thumb → view pages down; above → pages up; the thumb
   tracks. Drag the thumb → the view scrubs proportionally; the text cursor doesn't move.
2. With a long line (non-wrap), click left/right of the bottom horizontal thumb → pages left/right.
3. `Ctrl+O` on a big directory → click/drag its scrollbar to move through entries.
4. `F1` Help (overflowing) → click/drag its bar to scroll.
5. Confirm text press-drag still selects, the wheel still scrolls, and dialog buttons still click.

## Expected outcome

Every drawn scrollbar is clickable (pages) and draggable (proportional), bounded and panic-free, with no
change to text selection, the wheel, keyboard scrolling, or dialog actions (SC-001..SC-004).
