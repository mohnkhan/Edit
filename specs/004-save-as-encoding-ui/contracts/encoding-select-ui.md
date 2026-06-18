# UI Contract: Encoding Select Dialog

**Feature**: 004-save-as-encoding-ui | **Date**: 2026-06-19

---

## Dialog Appearance

```
╔══════════════════════════════════════╗
║       Save As Encoding               ║
╠══════════════════════════════════════╣
║  > UTF-8                             ║   ← highlighted row (inverse video)
║    UTF-16 LE                         ║
║    UTF-16 BE                         ║
║    CP437                             ║
║    CP850                             ║
║    ISO-8859-1                        ║
║    Windows-1252                      ║
║                                      ║
║  [↑↓] Select  [Enter] Save  [Esc]   ║
╚══════════════════════════════════════╝
```

**Dimensions**: 40 columns × 11 rows (clamped to terminal dimensions if smaller).

**Colors**: `theme.menubar_fg` on `theme.menubar_bg` (same as other modal dialogs —
blue background, white text in the default DOS theme). Highlighted row uses
`Modifier::REVERSED` for the selection indicator.

**Positioning**: Centered in the terminal frame (same `centered_rect()` helper used
by all other dialogs).

**Overlay**: Rendered on top of the editor area after `Clear` is applied to the dialog
rect — editor content is preserved behind it.

---

## Keyboard Contract

| Key         | Effect                                                   |
|-------------|----------------------------------------------------------|
| `↑`         | Move selection up; wraps from first item to last         |
| `↓`         | Move selection down; wraps from last item to first       |
| `Enter`     | Confirm selected encoding; save file; close dialog       |
| `Esc`       | Cancel; close dialog without saving                      |

All other keys are silently consumed while the dialog is open (no text insertion,
no cursor movement in the editor).

---

## File Menu Contract

The File pull-down menu includes the new item between "Save As" and "Exit":

```
File
├── New
├── Open
├── Save         (F5 / Ctrl+S)
├── Save As
├── Save As Encoding...  (F12)     ← new
└── Exit
```

The label `"Save As Encoding..."` uses the trailing ellipsis convention indicating
a dialog will follow (consistent with "Save As" in other text editors).

---

## Status Bar Contract

After a successful encoding-select save, the status bar shows the transient message:

```
Saved as <label>
```

Where `<label>` is the human-readable name from `ENCODING_OPTIONS` (e.g. `"UTF-16 LE"`).

The encoding indicator in the status bar (permanently displayed) reflects the new
encoding immediately after the save.

---

## F12 Key Binding Contract

```
F12  →  Action::SaveAsEncoding  →  opens encoding-select dialog
```

F12 must not conflict with any existing binding. As of feature 003, F12 is unbound.

---

## Error Behavior Contract

| Scenario                       | Behavior                                                            |
|--------------------------------|---------------------------------------------------------------------|
| Write fails (permission/disk)  | Dialog closes; status bar shows `"Save failed: <io_error>"`; `buf.encoding` reverts to pre-dialog value |
| Terminal too narrow for dialog | Dialog clamped to terminal width; text truncated with `…` if needed |
| New buffer (no path)           | Filename-input dialog opens first; encoding applied after path is confirmed |
| Filename prompt cancelled      | Encoding selection discarded; no file written                       |
