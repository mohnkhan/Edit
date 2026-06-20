# Quickstart / Validation: File dialog — glob filtering + richer entry details

## Build & test

```sh
make tmpfs-setup
make
make check        # cargo test — unit + integration
make ci-local     # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Unit** (`src/ui/file_browser.rs`): `glob_match` (`*.log`, `te?t`, anchoring, case-insensitive);
  substring fallback; `human_size` boundaries; `format_mtime` for a known epoch; `apply_filter` keeps
  dirs + `..`, filters files, and re-clamps the selection; detail-row name truncation is width-correct.
- **Integration** (`tests/integration/file_dialog_filter.rs`): typing `*.log` and a plain substring
  filters the listing (dirs/`..` retained); clearing restores; an absolute path still jumps/opens;
  Save-mode confirm still saves the typed name; the feature-020 buttons + feature-021 scrollbar still
  work with a filter active.

## Manual walkthrough

`./target/debug/edit` then `Ctrl+O`:

1. In a directory with mixed files + a sub-folder, type `*.log` → only `*.log` files remain, with the
   sub-folder and `..` still shown; the count/scrollbar reflect the filtered set.
2. Type a plain substring (e.g. `rep`) → entries whose name contains it remain (case-insensitive).
3. Clear the field → the full listing returns.
4. Type an absolute path (e.g. `/etc`) and Enter → it jumps there (not treated as a filter).
5. Observe each file row shows a size + modified date; folders show `<DIR>`; long names truncate with `…`
   while columns stay aligned. Resize the terminal → columns re-flow.
6. `Tab` to the Open/Cancel buttons and click them; confirm a long filtered list still scrolls with a
   visible scrollbar.

## Expected outcome

Typing a glob/substring filters the listing live (case-insensitive), directories stay reachable, every
row shows size/date detail, and all prior navigation/buttons/scrollbar behavior is unchanged
(SC-001..SC-005).
