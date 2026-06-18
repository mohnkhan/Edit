# Data Model: Session Restore (Feature 003)

**Date**: 2026-06-18

---

## Entities

### SessionData

The top-level session record written to `$XDG_STATE_HOME/edit/session.toml`.

| Field | Type | Description |
|---|---|---|
| `version` | `u32` | Schema version; currently always `1`. Readers reject unknown versions. |
| `active_buffer` | `usize` | 0-based index into `buffers` of the buffer that was active at exit. |
| `split_layout` | `SplitLayoutKind` | The pane arrangement at exit (`"none"`, `"horizontal"`, `"vertical"`). |
| `active_pane` | `u32` | Which pane was active: `0` = left/only, `1` = right. |
| `buffers` | `Vec<BufferEntry>` | Ordered list of open buffer records, one per open file. |

**Invariants**:
- `active_buffer < buffers.len()` (validated on load; violation → treat file as corrupt)
- `active_pane ∈ {0, 1}` (values > 1 are clamped to 1 on restore)
- `version == 1` (other values → reject as unknown; return `None`)

---

### BufferEntry

One entry in the session's `[[buffers]]` array.

| Field | Type | Description |
|---|---|---|
| `path` | `String` | UTF-8 file path as recorded at exit (absolute preferred; relative allowed). |
| `cursor_line` | `u32` | 1-based line number of the cursor at exit. |
| `cursor_col` | `u32` | 1-based grapheme column of the cursor at exit. |

**Invariants**:
- `cursor_line >= 1` (0 is invalid; `.saturating_sub(1)` guard on restore converts to 0-based)
- `cursor_col >= 1` (same guard)
- `path` must not be empty and must survive `security::sanitize::validate_path`

---

### SplitLayoutKind

Enum representing the pane arrangement. Serialized as a TOML string.

| Variant | TOML string | Description |
|---|---|---|
| `None` | `"none"` | Single-pane view. Maps to `SplitMode::Single`. |
| `Horizontal` | `"horizontal"` | Top/bottom split. Not yet supported by the UI; restore falls back to `Single`. |
| `Vertical` | `"vertical"` | Left/right split. Maps to `SplitMode::Vertical`. |

---

## State Transitions

```
[editor running]
      │
      │ user quits (clean exit)
      ▼
[serialize App → SessionData]
      │
      │ session::save_session(data)
      │   ↳ write .tmp → rename → session.toml
      ▼
[editor exits]

[editor starts, no file args, no --no-session]
      │
      │ session::load_session()
      ▼
 file exists & valid TOML & version==1?
      │                         │
     YES                       NO
      │                         │
      ▼                         ▼
[App starts with          [App starts, blank buffer]
 pending_session_restore]
      │
      │ TUI renders restore dialog
      │
  user presses Y/Enter          user presses N/Escape
      │                               │
      ▼                               ▼
[do_restore_session()]         [clear pending; blank buffer]
  for each BufferEntry:
    path valid & exists? ──NO──→ skip + status-bar warning
         │ YES
         ▼
    Buffer::open → seek cursor → apply highlight
  all failed? ──→ blank buffer + status-bar warning
  else → replace buffers, restore split + active_idx
```

---

## Session File Example

```toml
version = 1
active_buffer = 1
split_layout = "vertical"
active_pane = 1

[[buffers]]
path = "/home/user/projects/foo/src/main.rs"
cursor_line = 42
cursor_col = 8

[[buffers]]
path = "/home/user/projects/foo/Cargo.toml"
cursor_line = 1
cursor_col = 1
```
