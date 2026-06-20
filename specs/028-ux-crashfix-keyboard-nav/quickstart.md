# Quickstart / Validation: UX crash-safety and keyboard navigation hardening

## Build & test

```sh
make tmpfs-setup
make
make check        # cargo test — unit + integration (ux_hardening)
make ci-local     # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Unit** (`src/ui/editor.rs`): rendering with a deliberately stale/oversized wrap cache and with
  empty lines does not panic.
- **Unit** (`src/app.rs`): `invalidate_wrap_cache` (or the switch/open/close/restore paths) bumps
  `wrap_text_gen`; interactive dialogs open with `dialog_focus == 0`; arrow keys move `dialog_focus`
  via the button ring; `copy_selection` on a degenerate range returns empty.
- **Unit** (`src/ui/file_browser.rs`): scroll arithmetic uses saturating subtraction; PageUp/Down move
  by a page and clamp.
- **Unit** (`src/diagnostics/crash.rs`): the panic-hook terminal-restore path runs best-effort without
  panicking; `write_report` still emits a full report.
- **Integration** (`tests/integration/ux_hardening.rs`): restore a session with soft-wrap on → no
  panic; Save browser typing accumulates + caret shown; arrow-key button movement per dialog family;
  Help keyboard scroll clamps (Up/Down/PageUp/PageDown/Home/End); Home/End move the editor cursor;
  list PageUp/Down move by a page.

## Manual walkthrough

1. **Session restore (the crash)**: with soft-wrap on, create a session (open a couple of files, quit),
   relaunch, choose "Restore previous session" → files load, no crash, layout correct.
2. **Crash → terminal usable**: in a debug build, induce a panic → the terminal returns to a normal
   prompt with a visible cursor and the crash message is readable; a crash log exists under
   `$XDG_STATE_HOME/edit/`.
3. **Save-As typing**: open a confirm dialog and dismiss it, then `Ctrl+S` on an unnamed buffer → type
   a filename → characters appear with a caret.
4. **Arrow keys on buttons**: open any multi-button dialog → Left/Right (and Up/Down) move the focus
   ring; Enter activates; Tab still works.
5. **Help keyboard**: open Help on a short terminal → Down/PageDown/End scroll to the bottom,
   Up/PageUp/Home back; Esc closes.
6. **Home/End + paging**: in the editor press Home/End → cursor to line start/end; in a long file
   browser listing press PageDown/PageUp → selection jumps a page.

## Expected outcome

Session restore (and every buffer switch) is crash-free with soft-wrap on; any panic leaves a usable
terminal; Save-As and all dialogs are fully keyboard-operable; no regression in existing behavior
(SC-001..SC-006).
