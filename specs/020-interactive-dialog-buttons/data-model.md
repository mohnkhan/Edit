# Phase 1 Data Model: Focus-ring for the interactive/list dialogs

This feature is UI-state only — no persisted entities, no config, no file formats. The "data" is the
in-memory focus-ring state and the per-dialog descriptors derived from existing state.

## Reused state (no new fields expected)

| Field (in `src/app.rs`) | Type | Role in this feature |
|---|---|---|
| `dialog_focus` | `usize` | Now also the **ring index** for the open interactive dialog (0 = first primary-control stop). |
| `dialog_focus_init` | `bool` | Unchanged — guards one-time focus reset when a dialog opens. |
| `pending_encoding_select` | `Option<usize>` | Encoding list selection (primary control). |
| `pending_plugin_manager` / `plugin_manager_cursor` | `bool` / `usize` | Plugin list open + cursor. |
| `pending_find_replace` | `Option<FindReplaceDialog>` | Find/Replace fields, mode, options, focus field. |
| `file_browser` | `Option<FileBrowser>` | File browser list + typed path. |

`dialog_focus` is already reset by `ensure_dialog_focus()`; that function is extended to also recognize
an open interactive dialog and reset `dialog_focus = 0` (default = primary control).

## Focus-ring descriptor (derived, not stored)

For the currently-open interactive dialog, the implementation derives:

```
field_stops : usize        // primary-control focus stops
labels      : Vec<&str>    // ordered button labels
ring_len    : usize = field_stops + labels.len()
```

- `dialog_focus < field_stops`  ⇒ **primary control focused** (sub-stop = `dialog_focus`).
- `dialog_focus >= field_stops` ⇒ **button focused**, button index = `dialog_focus - field_stops`.
- `Tab` ⇒ `dialog_focus = buttons::next(dialog_focus, ring_len)`.
- `Shift+Tab` ⇒ `dialog_focus = buttons::prev(dialog_focus, ring_len)`.

## Per-dialog stop tables

### Encoding select
- `field_stops = 1` (the encoding list).
- Ring: `[ List, OK, Cancel ]` (`labels = ["OK","Cancel"]`).
- Default focus: `0` (list).

### Plugin manager
- `field_stops = 1` (the plugin list; may be empty).
- Ring: `[ List, Close ]` (`labels = ["Close"]`).
- Default focus: `0` (list).

### Find/Replace — Find mode (`DialogMode::Find`)
- `field_stops = 1` (Query field).
- Ring: `[ Query, Find, Close ]` (`labels = ["Find","Close"]`).
- Default focus: `0` (Query).

### Find/Replace — Replace mode (`DialogMode::Replace`)
- `field_stops = 2` (Query field = stop 0, Replacement field = stop 1).
- Ring: `[ Query, Replacement, Find, Replace, Replace All, Close ]`
  (`labels = ["Find","Replace","Replace All","Close"]`).
- Default focus: `0` (Query).
- The existing `FindReplaceDialog.focus: DialogField` is kept in sync with `dialog_focus` for stops 0/1
  so field rendering/editing targets the right field.

### File browser
- `field_stops = 1` (the browser primary control: entry list + typed path together).
- Ring: `[ Browser, Open|Save, Cancel ]` (`labels = ["Open"|"Save","Cancel"]` by mode).
- Default focus: `0` (browser).

## Invariants

- Exactly one stop is focused at all times a dialog is open (`0 <= dialog_focus < ring_len`).
- `ring_len >= 1` always (every dialog has at least one primary stop; all have ≥1 button).
- Geometry invariant: the `Rect` passed to `buttons::button_rects` for rendering is the same `Rect` used
  for `buttons::hit_test_buttons` in the mouse handler (drawn position == clickable region).
- Activating a button performs an action the dialog already supports — no new actions, no new state.
