# Quickstart / Validation: File Browser Dialogs

Prerequisites: `cargo` (MSRV 1.74+), `expect` for smoke tests. Build into the tmpfs `target/`
(SSD-saving active): confirm `readlink target` points under `/tmp/edit/`.

## Build & automated tests

```sh
cargo build
cargo test                           # unit + integration (incl. tests/integration/file_browser.rs)
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Expected: all green; `clippy --all-targets` clean.

## Unit-level validation (cargo test)

- **Navigation**: from a temp tree, `move_down`/`activate` into a subdir updates `cwd` and listing;
  `enter_parent` moves up; no-op at root.
- **Sort order**: entries come back `..` → dirs (alpha, case-insensitive) → files (alpha); dot-files
  present.
- **Hit-testing**: a `(col,row)` inside the box maps to the expected entry index; outside → cancel.
- **Save path**: `selected_save_path` rejects empty / separator / `..` filenames and returns
  `cwd.join(name)` otherwise.
- **UTF-8 truncation**: a long multi-byte name truncates on a grapheme boundary with `…`, never
  splitting a character.
- **App wiring**: `Action::Open`/`Action::SaveAs` set `file_browser`; `Ctrl+S` on an unnamed buffer
  opens the Save browser; activating a file `Outcome::OpenFile` opens a buffer; `Outcome::SaveFile`
  writes the file.

## End-to-end validation (live, deterministic)

Build the debug binary, then drive with `expect` (SGR mouse: `ESC [ < b ; col ; row M/m`, 1-based):

1. **Open by browsing (keyboard)**: launch, `Ctrl+O`, arrow down to a known sub-folder, `Enter`
   (enters it), arrow to a file, `Enter` → file loads (verify via `--debug` log line
   `Opened "<path>" as buffer N`).
2. **Open by browsing (mouse)**: `Ctrl+O`, click a folder row (enters it), click a file row → file
   loads. Deterministic check: navigate to and open a file, or click a directory then File-list
   change.
3. **Save by browsing**: type text, File ▸ Save As, navigate to a temp dir, type a filename, `Enter`
   → assert the file exists on disk with the buffer contents.
4. **Cancel**: open the browser, `Esc` → editor unchanged, no buffer added/written.
5. **Permission error**: point the browser at an unreadable dir → editor does not crash; error
   notice shown; previous listing retained.

A deterministic smoke check (no screen-scraping): `Ctrl+O`, navigate, and open a file whose load is
confirmed by the `--debug` log, mirroring the approach used for features 010/011.

## Contracts & data model

See [contracts/file-browser-interaction.md](./contracts/file-browser-interaction.md) for the full
keyboard/mouse/dispatch contract and [data-model.md](./data-model.md) for the `FileBrowser` types.
Implementation detail (widget rendering, exact methods) belongs in `tasks.md` / the implementation.
