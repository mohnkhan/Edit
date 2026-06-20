# Quickstart / Validation: Scroll affordances + dialog button polish

## Build & test

```sh
make tmpfs-setup          # keep build writes off the SSD (project memory)
make                      # cargo build (debug)
make check                # cargo test — unit + integration
make ci-local             # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Scrollbar helper** (unit, `src/ui/scrollbar.rs`): draws nothing when content fits; renders and
  positions the thumb when content overflows; no panic on tiny areas.
- **Editor geometry** (unit, `src/app.rs`): `viewport_height()` reflects the reserved horizontal-bar row
  (non-wrap); a click on a reserved bar cell does not move the cursor.
- **File browser** (unit, `src/ui/file_browser.rs`): the list scrollbar appears only when
  `entries.len() > list_rows`, and entry names never occupy the reserved bar column.
- **Help Close** (integration, `tests/integration/help_close_button.rs`): Help and About each close on a
  click on the Close button and on `Esc`.
- **Key hints** (integration, `tests/integration/dialog_key_hints.rs`): each dialog's button labels
  include the activating key; pressing/clicking still runs the same action (no regression).

## Manual walkthrough

Run `./target/debug/edit <large-file>` (a file taller than the terminal, with some long lines):

1. **Editor** — a vertical scrollbar appears on the right; its thumb moves as you scroll/PgDn. Move the
   cursor along a long line → a horizontal scrollbar appears on the bottom and tracks the column. Toggle
   soft-wrap (Alt+Z) → the horizontal bar disappears, vertical remains. Confirm no text is hidden under
   the bars and clicks still land on the right character. Try a split view (F6) → each pane has its bars.
2. **File browser** — `Ctrl+O` on a directory with many entries → a vertical scrollbar tracks the
   selection as you arrow down; entry names aren't clipped by the bar.
3. **Help / About** — `F1` / Help menu → a **Close (Esc)** button is shown; click it to dismiss; reopen
   and press `Esc`. If the cheat sheet overflows, a scrollbar is shown.
4. **Dialog buttons** — open the encoding dialog (`F12`), Find/Replace (`Ctrl+F`/`Ctrl+H`), and trigger
   the save prompt (edit + quit): every button shows its key (e.g. `Cancel (Esc)`, `OK (Enter)`), and
   pressing the key or clicking the button does the same thing as before.

## Expected outcome

Every overflowing view shows an accurate scrollbar, Help/About are mouse-dismissable via Close, and all
dialog buttons advertise their shortcut — with zero change to scrolling, navigation, or actions
(SC-001..SC-005).
