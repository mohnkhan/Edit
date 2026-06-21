# Roadmap

## Deferred Features

### Finish active-buffer accessor standardization (follow-up to feature 039)
- **Issue**: #68 (`follow-up`)
- **Status**: Deferred (feature 039 converted the reference-taking sites; ~71 field-level index
  accesses remain).
- **Description**: Route the remaining `self.buffers[self.active_idx].<field>` reads/writes through
  `active_buffer()`/`active_buffer_mut()` (FR-008). Deferred from 039 to avoid borrow-checker churn in
  a behavior-preserving refactor; the remainder is cosmetic. Effort: small, compiler-guided.

### Mouse interaction inside dialogs (follow-up to feature 029)
- **Issue**: #53 (`follow-up`)
- **Status**: List-row clicks **shipped** 2026-06-20 (feature 030 — encoding & plugin-manager dialogs
  are now click-to-select). The remaining half — caret-on-click inside dialog text fields — is split to
  **#58** (`follow-up`), deferred because it requires reverse-mapping clicks through the right-anchored
  field rendering.

### Caret-on-click inside dialog text fields (follow-up to feature 030)
- **Issue**: #58 (`follow-up`)
- **Status**: Complete as of 2026-06-21 (feature 031, branch `031-dialog-field-caret-click`).
- **Description**: Clicking inside the Find/Replace, Go-to-Line, and file-browser Name fields now moves
  the caret to the clicked grapheme (shared `ui::width::field_caret_at`); the Name and Go-to-Line inputs
  also gained caret editing (Left/Right/Home/End, mid-string insert/delete).

### Double-click word / triple-click line selection (follow-up to feature 029)
- **Issue**: #54 (`follow-up`)
- **Status**: Complete as of 2026-06-20 (feature 030, branch `030-interaction-completeness`).
- **Description**: The editor now selects the word on double-click and the line on triple-click.

### Right-click context menu (follow-up to feature 029)
- **Issue**: #55 (`follow-up`)
- **Status**: Complete as of 2026-06-20 (feature 030). Cut/Copy/Paste/Select All popup, mouse + keyboard.

### Additional DOS-standard F-key bindings (follow-up to feature 029)
- **Issue**: #56 (`follow-up`)
- **Status**: Complete as of 2026-06-20 (feature 030). F6/Shift+F6 buffer switch; F8/F9/F11 cut/copy/paste.

### Syntax highlighting beyond the baseline 5 (Constitution Principle VI)
- **Status**: Rust / JSON / TOML shipped 2026-06-20 (feature 026, branch
  `026-highlight-rust-json-toml`). The constitution defers "additional syntax-highlighting languages
  beyond the baseline 5" pending a spec + accepted user story; `specs/026-highlight-rust-json-toml/`
  is that spec. Further languages can follow the same pattern (`src/highlight/languages/`), each with
  its own spec.

### Bordered-box styling for Find/Replace fields (follow-up to feature 018)
- **Issue**: #41 (`follow-up`)
- **Status**: Complete as of 2026-06-20 (feature 019, branch `019-find-replace-field-boxes`)
- **Description**: Feature 018 gave the file-browser fields a bordered, labeled input box with a caret
  (and made the Open-mode path field visible). The Find/Replace fields already showed a label + caret
  inline; styling them as the same bordered boxes was a consistency follow-up.
- **Implementation**: The Find/Replace overlay in `src/ui/mod.rs` now renders each field as a labeled,
  bordered 3-row input box with a `▏` caret (only in the focused field) and right-anchored horizontal
  scroll, reusing `truncate_to_width`/`grapheme_width` from `src/ui/file_browser.rs`. Behavior
  (editing, `Tab`, option toggles, match count, `Esc`) unchanged; no focus-ring/buttons (still #38).
  Spec: `specs/019-find-replace-field-boxes/`.

### Boxed dialog buttons for interactive/list dialogs (follow-up to feature 016)
- **Issue**: #38 (`follow-up`)
- **Status**: Complete as of 2026-06-20 (feature 020, branch `020-interactive-dialog-buttons`)
- **Description**: Feature 016 gave boxed, focusable, mouse-clickable buttons + tab order to the
  confirm/dismiss dialogs (save prompt, session restore, external change, revert, plugin consent).
  The interactive/list dialogs — encoding select, plugin manager, Find/Replace, and the file browser
  — were deferred; they need a combined field/list + button focus-ring (their `Enter`/`Space`/`Tab`
  already carry list/field meaning).
- **Implementation**: `dialog_focus` was generalized into a per-dialog focus ring where stop 0 (and
  stop 1 for Find/Replace in replace mode) is the primary control and later stops are boxed buttons
  (encoding OK/Cancel, plugin-manager Close, Find/Replace Find/[Replace/Replace All]/Close, file
  browser Open|Save/Cancel). `Tab`/`Shift+Tab` cycle the ring, `Enter`/`Space`/click activate buttons,
  and every pre-existing key is preserved while the primary control is focused. Each dialog shares one
  outer-`Rect` source between its renderer and `handle_mouse_event` (drawn == clickable). Reused
  `src/ui/buttons.rs`. Spec: `specs/020-interactive-dialog-buttons/`. **No dialog-button deferrals
  remain.**

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
