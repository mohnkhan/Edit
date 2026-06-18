# Data Model: Save-As Encoding Selection UI

**Feature**: 004-save-as-encoding-ui | **Date**: 2026-06-19

---

## Entities

### EncodingId (existing — `src/encoding/detect.rs`)

The canonical encoding identifier enum. All seven variants are surfaced in the dialog.

| Variant       | Display Label   | BOM on write |
|---------------|-----------------|--------------|
| `Utf8`        | UTF-8           | No           |
| `Utf16Le`     | UTF-16 LE       | Yes (FF FE)  |
| `Utf16Be`     | UTF-16 BE       | Yes (FE FF)  |
| `Cp437`       | CP437           | No           |
| `Cp850`       | CP850           | No           |
| `Iso8859_1`   | ISO-8859-1      | No           |
| `Windows1252` | Windows-1252    | No           |

### Buffer (existing — `src/buffer/mod.rs`)

Extended field (already exists, semantics unchanged):

| Field      | Type         | Semantics                                                             |
|------------|--------------|-----------------------------------------------------------------------|
| `encoding` | `EncodingId` | Target encoding for all subsequent `save()` calls. Updated by the encoding-select dialog on confirm. |
| `path`     | `Option<PathBuf>` | File path on disk. `None` for new unnamed buffers. Used to determine whether to trigger the filename-prompt flow. |

No new fields are added to `Buffer`.

### App (existing — `src/app.rs`)

New field added to `App`:

| Field                     | Type          | Semantics                                                                    |
|---------------------------|---------------|------------------------------------------------------------------------------|
| `pending_encoding_select` | `Option<usize>` | `None` = encoding dialog is closed. `Some(idx)` = dialog is open; `idx` is the 0-based highlighted row in `ENCODING_OPTIONS`. |

### ENCODING_OPTIONS (new constant — `src/ui/dialog.rs`)

Static ordered list mapping row index → `(EncodingId, display_label)`:

```
[(Utf8, "UTF-8"), (Utf16Le, "UTF-16 LE"), (Utf16Be, "UTF-16 BE"),
 (Cp437, "CP437"), (Cp850, "CP850"), (Iso8859_1, "ISO-8859-1"),
 (Windows1252, "Windows-1252")]
```

Index 0 is always UTF-8 (the default). The ordering is deterministic and stable across
releases — changing the order would break any user muscle memory.

### Action (existing enum — `src/input/keymap.rs`)

New variant:

| Variant           | Trigger              | Semantics                                 |
|-------------------|----------------------|-------------------------------------------|
| `SaveAsEncoding`  | F12 / File menu item | Opens the encoding-select dialog          |

---

## State Transitions

### Encoding Dialog Lifecycle

```
[Dialog closed] pending_encoding_select = None
    │
    │  Action::SaveAsEncoding
    ▼
[Dialog open] pending_encoding_select = Some(current_enc_idx)
    │
    ├── Action::MoveUp   → Some((idx - 1 + N) % N)   [wrap around]
    ├── Action::MoveDown → Some((idx + 1) % N)         [wrap around]
    │
    ├── Action::InsertNewline / Enter → do_save_as_encoding(ENCODING_OPTIONS[idx].0)
    │                                   pending_encoding_select = None
    │
    └── Action::MenuClose / Escape   → pending_encoding_select = None
```

### Buffer Encoding Update (on confirm)

```
do_save_as_encoding(enc: EncodingId):
  if buf.path.is_some():
    buf.encoding = enc
    buf.save()        # atomic write via tmp-rename
    status_message = "Saved as <label>"
  else:
    // transition to filename-prompt (existing handle_save_as path)
    pending_save_as_encoding = Some(enc)   // held until filename confirmed
    handle_save_as()   // opens filename input dialog
    // on filename confirm: buf.encoding = enc; buf.save_as(path)
    // on filename cancel: pending_save_as_encoding = None
```

---

## Validation Rules

- `idx` in `pending_encoding_select` is always in `0 ..< ENCODING_OPTIONS.len()` (wrap arithmetic guarantees this).
- `buf.encoding` is updated **only** after a successful `buf.save()` call — on I/O error the encoding reverts to the pre-dialog value (status bar shows error).
- For unnamed buffers, the encoding is committed only if both the filename prompt **and** the write succeed.

---

## Key Relationships

```
App ──────────────────────► Buffer  (one active buffer per frame)
  │ pending_encoding_select: Option<usize>
  │
  │   on confirm ──────────► buf.encoding = EncodingId
  │                          buf.save()
  │
  └── ENCODING_OPTIONS[idx] ──► (EncodingId, &str label)
```
