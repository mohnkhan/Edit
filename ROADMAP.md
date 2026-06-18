# Roadmap

## Deferred Features

### Plugin API
- **Issue**: #2
- **Status**: Deferred from v0.1.0
- **Description**: A plugin API allowing external tools to register custom syntax highlighters,
  key bindings, and menu items.
- **Why deferred**: Scope constraint — core editor stability takes priority. Plugin ABI requires
  stabilization of internal APIs first.
- **Suggested approach**: Expose a C FFI or WASM plugin interface via `dlopen` or a WASM runtime.
- **Effort**: Large (2–3 weeks)
- **Label**: `follow-up`

### External File Modification Detection
- **Issue**: #3
- **Status**: Deferred from v0.1.0
- **Description**: Detect when a file opened in the editor is modified by an external process
  (e.g. via `inotify`), and prompt the user to reload or keep their version.
- **Why deferred**: `inotify` integration adds complexity and Linux-specific code paths that
  require more careful design to avoid races with the auto-save subsystem.
- **Suggested approach**: Use `inotify` (Linux) via the `notify` crate; poll as fallback on
  other platforms.
- **Effort**: Medium (1 week)
- **Label**: `follow-up`

### Soft-Wrap Mode
- **Issue**: #4
- **Status**: Deferred from v0.1.0
- **Description**: Optional soft-wrap rendering as an alternative to horizontal scroll.
- **Why deferred**: DOS EDIT.COM does not support soft-wrap; implementing it faithfully requires
  significant editor widget changes to the line-rendering pipeline.
- **Suggested approach**: Add a `wrap_mode: bool` flag to `Config`; change `ui/editor_widget.rs`
  to pre-compute visual line breaks at render time based on terminal width.
- **Effort**: Medium (1 week)
- **Label**: `follow-up`

### UTF-16 Transcoding
- **Issue**: #5
- **Status**: Deferred from v0.1.0
- **Description**: Full read/write support for UTF-16 LE/BE files. Detection already flags UTF-16
  BOMs, but the transcoding layer currently returns the raw bytes rather than converting them.
- **Why deferred**: Uncommon on Linux; UTF-8 covers the vast majority of use cases.
- **Suggested approach**: Add `EncodingId::Utf16Le` and `EncodingId::Utf16Be` variants; wire up
  `encoding_rs`'s UTF-16 decoders in `src/encoding/transcode.rs`.
- **Effort**: Small (2–3 days)
- **Label**: `follow-up`

### Session Restore
- **Issue**: #6
- **Status**: Deferred from v0.1.0
- **Description**: On startup without file arguments, restore the previous editing session
  (open buffers, cursor positions, split layout).
- **Why deferred**: Requires a stable session-state serialization format; deferred to let the
  buffer and UI APIs stabilize first.
- **Suggested approach**: Write a `session.toml` to `$XDG_STATE_HOME/edit/` on clean exit;
  deserialize on startup.
- **Effort**: Small (2–3 days)
- **Label**: `follow-up`
