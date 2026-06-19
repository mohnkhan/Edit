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
- **Issue**: #3 (closed — implemented in feature 007)
- **Status**: Complete as of 2026-06-19
- **Description**: Detect when a file opened in the editor is modified by an external process;
  prompt the user to reload or keep their in-editor version; show a one-shot notice on file
  deletion; suppress self-writes; debounce rapid changes; `--no-watch` flag.
- **Implementation**: `src/watcher/mod.rs` using `notify = "6"` (inotify on Linux).

### Soft-Wrap Mode
- **Issue**: #4 (closed — implemented in feature 005)
- **Status**: Complete as of 2026-06-19
- **Description**: Optional soft-wrap visual rendering (`»` continuation marker, Alt+Z / View menu,
  `soft_wrap` config key, `[WRAP]` status-bar indicator, visual-row-aware scrolling and mouse).
- **Follow-up shipped**: Menu check-indicator (#13) — implemented in feature 006.

### UTF-16 Transcoding
- **Issue**: #5 (closed — implemented in feature 002)
- **Status**: Shipped in v0.2.0 (branch `002-utf16-transcoding`)
- **Description**: Auto-detect + forced-decode/encode of UTF-16 LE/BE files, full round-trip,
  BOM handling, surrogate-pair support, `--encoding` CLI aliases.

### Save-As Encoding Selection UI (UTF-16 follow-up)
- **Issue**: #9 (closed — implemented in feature 004)
- **Status**: Complete as of 2026-06-19
- **Description**: Modal TUI listbox dialog (F12 / File › Save As Encoding...) to select
  output encoding (UTF-8, UTF-16 LE/BE, CP437, CP850, ISO-8859-1, Windows-1252). Encoding
  persists for subsequent saves. Filename prompt invoked for new (unsaved) buffers.
- **Implementation**: `src/ui/dialog.rs` (`EncodingSelectDialog`), wired via `Action::SaveAsEncoding`.

### Menu Item Checked-State Indicator
- **Issue**: #13 (closed — implemented in feature 006)
- **Status**: Shipped 2026-06-19 (branch `006-menu-check-state-indicator`)
- **Description**: View > "Soft Wrap (ext)" and any future toggleable menu items display a `✓`
  (U+2713) prefix when active via `MenuBarWidget::toggle_states: &[(Action, bool)]`. Implemented
  as a general mechanism (FR-007): no per-item bespoke code required for future toggleable items.

### Session Restore
- **Issue**: #6 (closed — implemented in feature 003)
- **Status**: Complete as of 2026-06-19
- **Description**: On clean exit, write `session.toml` to `$XDG_STATE_HOME/edit/`; on next
  startup without file arguments, offer a TUI restore dialog. `--no-session` flag suppresses
  the prompt. Explicit file arguments bypass restore.
- **Implementation**: `src/session/` module; `specs/003-session-restore/`.
