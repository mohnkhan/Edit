# Quickstart / Validation: Interactive-dialog buttons + focus ring

## Build & test

```sh
make tmpfs-setup          # keep build writes off the SSD (per project memory)
make                      # cargo build (debug)
make check                # cargo test — unit + integration (focus-ring + no-regression)
make ci-local             # full gate: fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Ring math / labels / sizing** (unit, inline `#[cfg(test)]` in `src/app.rs` / `src/ui/`):
  ring length and `field_stops` per dialog/mode; button-label tables; outer-`Rect` grows for the button
  row and never panics on a tiny terminal; `hit_test` maps a click to the right button.
- **Per-dialog activation + no-regression** (integration, `tests/`): for each of the four dialogs, drive
  `Tab` around the whole ring (visits each stop once, wraps), activate each button by `Enter`/`Space`
  and by a simulated click, and assert the existing action ran; drive every legacy key with the primary
  control focused and assert unchanged behavior; `Esc` closes from any focus.

## Manual walkthrough (DOS-faithful affordance check)

Run `./target/debug/edit somefile.txt` and verify each dialog:

1. **Encoding select** — File › Save As Encoding (or `F12`):
   - List shows; `Up`/`Down` move selection. Press `Tab` → focus moves to **OK**, then **Cancel**, then
     back to the list. Click **OK** → selected encoding applied + dialog closes. Reopen, click **Cancel**
     (or press `Esc`) → closes, no change.
2. **Plugin manager** — Options › Plugins:
   - `Up`/`Down` move the cursor; `Space` toggles enabled. `Tab` → **Close** focused; `Enter`/click on
     **Close** (or `Esc`) closes.
3. **Find/Replace** — `Ctrl+F` (find) and `Ctrl+H` (replace):
   - Type a query. In replace mode `Tab` cycles Query → Replacement → **Find** → **Replace** →
     **Replace All** → **Close** → Query. `Enter` on a field still runs the per-mode action; clicking
     **Find/Replace/Replace All** runs each; `Alt+C/A/R/W` toggles still work; `F3`/`F2` still navigate
     matches; **Close**/`Esc` closes.
4. **File browser** — File › Open and File › Save As:
   - `Up`/`Down`/`Left`/`Right` navigate; type to filter/enter a path. `Tab` → **Open**/**Save** →
     **Cancel** → back to the list. Click **Open**/**Save** activates the selection; **Cancel**/outside
     click/`Esc` closes. Double-clicking an entry still activates it.

In every dialog: exactly one control is highlighted at a time; resizing the terminal keeps clicks
landing on the drawn buttons; a terminal too narrow for all buttons still lets `Tab` reach them.

## Expected outcome

All four interactive dialogs are fully operable by mouse and by keyboard, with zero change to any
previously working keystroke (SC-001..SC-005 in `spec.md`).
