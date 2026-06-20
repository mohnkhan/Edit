# Quickstart: Field affordance + Help redesign

See [spec.md](./spec.md) and [contracts/field-and-help.md](./contracts/field-and-help.md).

## Build
```sh
make
```

## Manual validation
1. **Open dialog field** — `Ctrl+O`; a bordered "Go to path:" input box with a caret is visible; type a
   path and it appears in the box; `Enter` jumps/opens it as before.
2. **Save dialog field** — File ▸ Save As on a new buffer; a bordered "Name:" box with a caret is shown;
   type a name; `Enter` saves.
3. **Help** — open Help ▸ Help; shortcuts appear as aligned `Key   Action` rows under section headings
   (File/Edit/Search/Selection/View/Menus/Dialogs); nothing is cut off. On a short terminal,
   `↑`/`↓`/`PageUp`/`PageDown` scroll; `Esc` closes.

## Automated
```sh
cargo test --lib file_browser    # field renders boxed in both modes
cargo test --lib help            # help table builds + scroll clamps
cargo test
make ci-local
```

## Expected
- A clearly-bordered, labeled, caret'd input box in both file-dialog modes (Open field no longer hidden).
- A readable, grouped, scrollable Key|Action Help table with no truncation. No regression.
