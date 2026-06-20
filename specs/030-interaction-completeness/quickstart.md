# Quickstart / Validation: Interaction completeness

## Build & test

```sh
make tmpfs-setup
make
make check        # cargo test — unit + integration (interaction)
make ci-local     # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **US1**: `dialog`/`plugin_manager` row-hit helpers map a click to the right index; `field_caret_at`
  maps a click column to a caret grapheme (incl. multibyte + clamp); `handle_mouse_event` selects the
  clicked list row / positions the field caret and sets focus.
- **US2**: double-click selects the word, triple-click the line (Copy returns the expected text) over
  ASCII and multibyte; single-click clears; boundaries don't panic.
- **US3**: context menu opens on right-click, focus moves, items run the right action and close, Esc /
  outside-click dismiss, on-screen clamping, modal precedence.
- **US4**: keymap maps F6/Shift+F6/F8/F9/F11 to NextBuffer/PrevBuffer/Cut/Copy/Paste; existing F-keys
  unchanged.
- **Integration** (`tests/integration/interaction.rs`): the above driven through
  `handle_mouse_event`/`handle_action`.

## Manual walkthrough

1. Open the encoding dialog (Save As Encoding) → click a row → it's selected; click OK.
2. Open Find (Ctrl+F), type a query, click partway into it → caret moves there.
3. In the editor, double-click a word → it's selected; Ctrl+C; triple-click a line → whole line
   selected.
4. Right-click in the editor → Cut/Copy/Paste/Select All menu; pick Copy (mouse or ↓+Enter); Esc to
   dismiss.
5. Press F9 (copy), F11 (paste), F8 (cut), F6 / Shift+F6 (next/prev buffer). Confirm F1/F3/F5/F10/F12
   still behave as before.

## Expected outcome

Dialogs are fully mouse-operable (rows + fields), the editor supports double/triple-click selection and
a right-click menu, and the DOS F-keys work — with no regression and no new dependencies
(SC-001..SC-005). Closes #53, #54, #55, #56.
