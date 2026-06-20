# Quickstart / Validation: Buffer tab bar

## Build & test

```sh
make tmpfs-setup
make
make check        # cargo test — unit + integration (buffer_tab_bar)
make ci-local     # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Unit** (`src/ui/tabbar.rs`, `src/app.rs`): `tab_hit_regions` (label/`[x]` rects; overflow keeps the
  active tab visible; no panic at tiny width); `editor_top`/`viewport_height` with 1 vs 2+ buffers;
  `close_buffer_at` active-index adjustment.
- **Integration** (`tests/integration/buffer_tab_bar.rs`): 2 buffers → tab bar present, active highlighted;
  clicking another tab switches; clicking `[x]` on a clean buffer closes it (bar hides at 1 left); `[x]`
  on a modified buffer opens the CloseConfirm and Save/Discard/Cancel behave; a text click with the bar
  shown lands on the right line (tab row accounted for); a tab-row click never moves the cursor.

## Manual walkthrough

`./target/debug/edit src/app.rs Cargo.toml` (two files):

1. A tab bar shows `app.rs` and `Cargo.toml`; the active tab is highlighted. Click the other tab → the
   editor switches; `Ctrl+Tab` still cycles.
2. Edit a buffer → its tab shows the modified marker; save → marker clears.
3. Click a clean tab's `[x]` → it closes; with one file left the tab bar disappears.
4. Open two files, modify one, click its `[x]` → a Save/Discard/Cancel prompt appears; Cancel keeps it.
5. With the bar shown, click in the text → the cursor lands where you clicked; roll the wheel / drag the
   scrollbar → the editor scrolls correctly; clicking the tab row never moves the cursor.
6. Open many/long-named files → tabs truncate but the active one stays visible; resize → no corruption.

## Expected outcome

Open files are visible and switchable/closable via tabs (with an unsaved-changes prompt on close), the
editor geometry stays correct beneath the bar, and single-file editing is unchanged (SC-001..SC-005).
