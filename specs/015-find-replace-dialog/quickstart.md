# Quickstart: Interactive Find and Replace

Validation guide for feature 015. See [spec.md](./spec.md) and
[contracts/find-replace-interaction.md](./contracts/find-replace-interaction.md).

## Prerequisites

```sh
make   # builds ./target/debug/edit
```

## Manual validation

1. **Find (US1)** — open a file with repeated words. `Ctrl+F`, type a word, `Enter`. Matches highlight,
   the view jumps to the first, and the dialog shows "1 of N".
2. **Next/Prev (US2)** — press `F3` repeatedly to cycle forward (wraps at the end); `F2` goes backward.
   The current-match highlight and the "X of Y" indicator update each step.
3. **Replace (US3)** — `Ctrl+H`, type a term, `Tab`, type a replacement. `Enter` replaces the current
   match and advances; `Ctrl+A` replaces all and reports the count. `Ctrl+Z` undoes.
4. **Options (US4)** — toggle `Alt+C` (case), `Alt+W` (wrap), `Alt+G` (regex), `Alt+O` (whole-word) and
   re-run; results change accordingly. Whole-word: searching "cat" does not match "category".
5. **Esc** closes the dialog and clears highlights. With no dialog open, `Ctrl+A` still selects all.

## Automated validation

```sh
cargo test --lib search          # engine incl. new whole-word matching
cargo test --test find_replace   # end-to-end find / next / prev / replace / replace-all / options
make ci-local                    # full gate
```

## Expected outcomes

- A term can be typed and found from the menu or `Ctrl+F`; matches highlight; the view jumps to the
  current match with a correct "X of Y".
- Next/Prev visit every match and wrap; the current match is visually distinct.
- Replace and Replace All change the document, report counts, and are undoable in one step.
- Case-sensitive, wrap, regex, and whole-word toggles change results as specified.
- No regression to ordinary editing or other dialogs when no Find/Replace dialog is open.
