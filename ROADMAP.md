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
- **Issue**: #5 (closed — implemented in feature 002)
- **Status**: Shipped in v0.2.0 (branch `002-utf16-transcoding`)
- **Description**: Auto-detect + forced-decode/encode of UTF-16 LE/BE files, full round-trip,
  BOM handling, surrogate-pair support, `--encoding` CLI aliases.

### Save-As Encoding Selection UI (UTF-16 follow-up)
- **Issue**: #9
- **Status**: Deferred from feature 002
- **Description**: Interactive Save As... dialog that lets the user pick the output encoding
  (e.g. UTF-16 LE, UTF-8, CP437) from within the editor, rather than relying on CLI flags.
- **Why deferred**: The ratatui encoding-picker dialog does not yet exist; building it in-scope
  would have inflated the feature 002 PR beyond its stated goal. The transcoding plumbing is
  already present — only the interactive UI layer is missing.
- **Suggested approach**: Modal listbox dialog wired to a new `Action::SaveAsEncoding`; bind to
  File > Save As Encoding... and/or F12. See `specs/002-utf16-transcoding/plan.md` §US4.
- **Effort**: Small–Medium (~3 days)
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
