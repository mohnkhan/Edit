# Quickstart: Focusable dialog buttons

Validation guide for feature 016. See [spec.md](./spec.md) and
[contracts/dialog-buttons.md](./contracts/dialog-buttons.md).

## Prerequisites

```sh
make   # builds ./target/debug/edit
```

## Manual validation

1. **Mouse + buttons (US1/US2)** — make an edit, press `Ctrl+Q`. The unsaved-changes dialog shows three
   boxed buttons (Save / Discard / Cancel) with one focused. Click **Cancel** → dialog closes, nothing
   lost. Re-trigger, click **Discard** → quits.
2. **Tab order (US3)** — re-trigger the prompt; `Tab` moves the focus highlight Save→Discard→Cancel→Save;
   `Shift+Tab` reverses; `Enter` activates the focused button. The letter keys (S/D/C) still work too.
3. **Help/About** — open Help ▸ About; it shows a **Close** button; click it or press Enter/Esc to
   dismiss.
4. **List dialog** — open File ▸ Save As Encoding; the encoding list still navigates with Up/Down, and
   there are **OK / Cancel** buttons reachable by Tab and clickable by mouse.
5. **Outside click** — open any cancelable dialog and click outside the box → it cancels.

## Automated validation

```sh
cargo test --lib buttons            # layout / hit-test / focus units
cargo test --test dialog_buttons    # activation by Tab+Enter and by click
make ci-local                       # full gate
```

## Expected outcomes

- Every in-scope dialog shows boxed buttons with exactly one focused; Tab/Shift+Tab cycle; Enter/Space
  activate; clicks activate the clicked button; outside-click cancels where safe.
- Choosing by button equals the old letter/key shortcut; Esc and list navigation still work.
- No panic at any terminal size; non-dialog editing and file-browser/menu mouse unchanged.

> Find/Replace and the file browser are deferred (already navigable) — tracked via a follow-up issue.
