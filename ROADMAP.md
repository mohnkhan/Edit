# Roadmap

## Deferred Features

### Bordered-box styling for Find/Replace fields (follow-up to feature 018)
- **Issue**: #41 (`follow-up`)
- **Status**: Open (deferred from feature 018, branch `018-field-affordance-help`)
- **Description**: Feature 018 gave the file-browser fields a bordered, labeled input box with a caret
  (and made the Open-mode path field visible). The Find/Replace fields already show a label + caret
  inline; styling them as the same bordered boxes is a consistency follow-up. Spec:
  `specs/018-field-affordance-help/`.

### Boxed dialog buttons for interactive/list dialogs (follow-up to feature 016)
- **Issue**: #38 (`follow-up`)
- **Status**: Open (deferred from feature 016, branch `016-dialog-buttons`)
- **Description**: Feature 016 gave boxed, focusable, mouse-clickable buttons + tab order to the
  confirm/dismiss dialogs (save prompt, session restore, external change, revert, plugin consent).
  The interactive/list dialogs — encoding select, plugin manager, Find/Replace, and the file browser
  — were deferred; they need a combined field/list + button focus-ring (their `Enter`/`Space`/`Tab`
  already carry list/field meaning). They remain navigable today (file browser by mouse; the rest by
  keyboard).
- **Suggested approach**: reuse `src/ui/buttons.rs`; add a per-dialog focus ring. Spec:
  `specs/016-dialog-buttons/`.

### Plugin API
- **Issue**: #2 (implemented in feature 008)
- **Status**: Complete as of 2026-06-19 (branch `008-plugin-api`)
- **Description**: Plugin API allowing external plugins to register custom syntax highlighters,
  keybindings, and menu items, in a default-deny sandbox with one-time user consent.
- **Implementation**: `src/plugin/` using **Rhai** (pure-Rust embedded scripting — chosen over
  WASM/dlopen for minimal footprint, trivial static linking, and FreeBSD support per
  Constitution III/IV). Per-call 50 ms wall-clock limit via Rhai `on_progress`; `read_file` is
  the only host FS capability and is permission-gated; consent persisted to `plugins.toml`;
  manager at Options > Plugins; `--no-plugins` flag. Reference plugins in `examples/plugins/`.
  Spec: `specs/008-plugin-api/`.

### Plugin top-level menu activation (follow-up to feature 008)
- **Issue**: #19 (`follow-up`)
- **Status**: Complete as of 2026-06-19 (feature 009, branch `009-menu-bar-activation`)
- **Description**: Live keyboard activation of plugin-contributed top-level menu items (e.g.
  "Tools > Word Count") via the menu bar, plus the broader menu-interaction pass it depended on.
- **Implementation**: Keyboard navigation (`F10` top-level highlight, `Alt+<letter>` direct
  dropdown, Left/Right between menus, Up/Down within a dropdown, Enter activate, Esc close) is
  wired in `App::handle_action` for both built-in and plugin menus. A single resolved menu model
  (`resolve_menus()` in `src/ui/menubar.rs`) drives both rendering and navigation; plugin menus
  render between Options and Help (Help stays rightmost), merging into a built-in menu on name
  collision. Activation dispatches `Action::PluginMenuActivated` and shows the result in the
  status bar. Spec: `specs/009-menu-bar-activation/`.

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
