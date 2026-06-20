# Quickstart / Validation: UX completeness hardening (round 2)

## Build & test

```sh
make tmpfs-setup
make
make check        # cargo test — unit + integration (ux_round2)
make ci-local     # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Crash**: char-safe delete/cut over multibyte (`src/app.rs`); recovery-path truncation no-panic
  (`src/ui/dialog.rs`); `byte_to_char` on a non-boundary offset (`src/buffer/rope.rs`); file-size guard
  returns the error (`src/buffer/mod.rs`).
- **No silent data loss**: save success/failure status + retained modified flag; autosave-failure notice.
- **Dialog/correctness**: SavePrompt Esc cancels; Save-As via browser keeps the pending encoding; click
  maps with gutter + h-scroll; Go-to-Line not over a menu.
- **Width**: `ui::width::display_width` (combining=0, CJK=2) and that all surfaces use it.
- **Feedback**: copy/cut/paste/read-only/open status strings.
- **Reachability/legibility**: Ctrl+W bound to Close; File ▸ Close present; both themes legible selected
  item (headless render assertion).
- **Integration** (`tests/integration/ux_round2.rs`): the above driven through `handle_action`/render.

## Manual walkthrough

1. Open a file via a Unicode path → no crash; trigger the recovery prompt with a long path → no crash.
2. Select across accented/CJK/emoji text → cut → correct text removed, undo restores it.
3. `Ctrl+S` on a writable file → "Saved"; make it read-only and save → "Save failed: …", tab still ●.
4. Open the save-before-quit prompt → Esc cancels.
5. Save-As a non-UTF-8 encoding through the browser → file written in that encoding.
6. Enable line numbers (View menu) → click in the text → cursor lands under the pointer.
7. Render CJK/emoji/combining text → columns align (editor, file browser, tabs).
8. Copy/cut/paste → status feedback; paste empty clipboard → "Nothing to paste"; type in a read-only
   buffer → "Buffer is read-only".
9. `Ctrl+W` → current buffer closes; File ▸ Close does the same.
10. Switch to the light theme → the highlighted menu item is readable.
11. Try to open a multi-GB file → "file too large" message, no hang/OOM.

## Expected outcome

No operation panics on real-world content; saves never fail silently; dialogs and clicks behave
consistently and correctly; wide/combining/emoji text aligns under one width function; actions give
feedback; Close is reachable and every theme is legible (SC-001..SC-008). Deferred enhancements are
tracked as issues + ROADMAP rows.
