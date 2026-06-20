# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### feature 016: Focusable dialog buttons (borders, tab order, mouse)

#### Added

- **Boxed dialog buttons** on the confirm/dismiss dialogs (unsaved-changes Save/Discard/Cancel,
  session restore, external-change, revert, plugin consent): each choice is drawn in its own box
  border with one button focused (inverted + `▶` marker).
- **Mouse navigation of dialogs** — clicking a button activates it; clicking outside cancels (where a
  safe cancel exists). Dialogs previously ignored the mouse entirely.
- **Tab order** — `Tab`/`Shift+Tab` move button focus (wrapping); `Enter`/`Space` activate the focused
  button. Each dialog opens focused on its safe default (Cancel/No/Keep for destructive prompts). The
  existing letter shortcuts (S/D/C, Y/N) still work.
- A reusable button component (`src/ui/buttons.rs`) provides the shared layout/render/hit-test so a
  click always lands on the drawn button.
- Built via the full Spec Kit pipeline; artifacts under `specs/016-dialog-buttons/`.

#### Notes

- The interactive/list dialogs (encoding select, plugin manager, Find/Replace, file browser) keep
  their current navigation for now; boxed buttons for them are deferred (issue #38, ROADMAP) as they
  need a combined field/list + button focus ring.
- The Revert dialog now defaults focus to **Cancel** (press `Y` or Tab to Revert to confirm) — a safer
  default for a destructive action.

### feature 015: Interactive Find and Replace dialogs

#### Added

- **Find dialog** (`Ctrl+F` / Search ▸ Find): a modal input box to type a search term; `Enter` finds
  all matches, highlights them in the document (the current match shown distinctly), jumps the view to
  the first match at/after the cursor, and shows an "X of Y" indicator. `F3` / `F2` step to the
  next/previous match with wrap-around; `Esc` closes and clears the highlights.
- **Replace dialog** (`Ctrl+H` / Search ▸ Find Replace): find + replace-with fields (`Tab` switches
  focus); `Enter` replaces the current match and advances; `Ctrl+A` replaces all and reports the count.
  Replacements are a single undoable edit and mark the buffer modified.
- **Search-option toggles** in the dialog: case-sensitive (`Alt+C`), wrap-around (`Alt+A`), regex
  (`Alt+R`), and **whole-word** (`Alt+W`). Whole-word (word-boundary) matching was added to the search
  engine. All field input and match highlighting are UTF-8/grapheme-correct.
- Built via the full Spec Kit pipeline; artifacts under `specs/015-find-replace-dialog/`.

#### Fixed

- Search ▸ Find and Search ▸ Find Replace were stubs (reset state / logged, with no way to type a
  term). They now open working interactive dialogs.
- **Crash on a very small terminal**: scroll clamping computed `viewport_height − 1`, which underflowed
  and panicked when the visible height was 0 (possible now that the editor tracks the real frame size).
  The viewport height used for clamping is floored at 1, so editing never panics at any frame size.

### feature 014: Undo-to-clean state and Revert to saved

#### Added

- **File ▸ Revert** (menu-only) reloads the active buffer from its last saved version on disk,
  discarding in-editor changes. It asks for confirmation when there are unsaved changes, is a safe
  no-op (with a notice) for never-saved buffers, and leaves the buffer untouched if the file can't be
  read.

#### Fixed

- **Undo back to the saved version now clears `[Modified]`.** The dirty indicator is derived from the
  undo history (a saved-point marker) instead of being forced on by every edit, so undoing to the
  saved/opened content shows the buffer as clean and redoing marks it modified again — matching DOS
  EDIT and standard editors. The marker is invalidated on a divergent edit (undo, then retype), so the
  buffer is never *falsely* shown clean when its content differs from the saved version.

### Fixed

- **Mouse clicks in the file browser (and menus) on non-80×24 terminals.** `terminal_size` was only
  refreshed on a resize event, so on startup it stayed at the default 80×24 while the UI was drawn
  for the real frame size. Mouse hit-testing used the stale geometry, so a click inside the visible
  Open/Save browser box could map to "outside" and close the dialog. The render path now syncs
  `terminal_size` to the actual frame every frame, so clicks land on what is drawn at any size.

### feature 013: DOS-style menu mnemonic accelerators

#### Added

- **Underlined accelerator letters** on every top-level menu (File, Edit, Search, View, Options,
  Help) and every dropdown item, recreating the MS-DOS EDIT.COM look. The underlined letter is the
  key that activates the entry, and the two never drift.
- **Letter activation inside an open menu.** With a dropdown open, pressing an item's accelerator
  (case-insensitive) runs that item immediately — e.g. `Alt+F` then `N` makes a New buffer. A letter
  that matches no item is an inert no-op (the menu stays open and the buffer is never edited). While
  the menu bar is active without a dropdown, a top-level letter opens that menu.
- **Bare `Alt` activates the menu bar** (like `F10`) on terminals that report modifier-only keys
  (keyboard-enhancement protocol); on terminals without that support it is a no-op and `F10` /
  `Alt+letter` remain the entry path (graceful degradation).
- **Plugin menu accelerators** are assigned automatically — unique within their menu and
  non-colliding with built-in items — so plugin-contributed items and menus are letter-activatable
  too.
- Built via the full Spec Kit pipeline; spec/plan/tasks/checklists under `specs/013-menu-mnemonics/`.

---

### feature 012: Navigable file browser for Open / Save dialogs

#### Added

- **File browser** replaces the blind path-text Open and Save As dialogs. Both now show the current
  directory's folders and files in a navigable list: arrow keys move the selection, Enter enters a
  folder or picks a file, `←`/`Backspace` go to the parent, and the current-directory path is shown
  at the top. With the mouse, a single click selects a row and a double-click activates it (enters
  the folder / opens the file) — so double-clicking a folder browses into it instead of immediately
  opening a file underneath the cursor. Long listings scroll to keep the selection visible;
  folder/file names render UTF-8-correct and truncate (with `…`) without splitting multi-byte
  characters.
- **Save browser** includes an editable filename field; in Open mode an absolute path may be typed to
  jump straight to it. `Ctrl+S` on an unnamed (new) buffer now opens the Save browser to choose a
  destination, instead of silently failing.
- Built via the full Spec Kit pipeline; spec/plan/tasks/checklists under
  `specs/012-file-browser-dialogs/`.

#### Changed

- Removed the superseded single-line `OpenFileDialog` / `SaveAsFileDialog` widgets and the
  `pending_open` / `pending_save_as` state in favour of one `FileBrowser` model that drives both
  rendering and mouse hit-testing (so clicks always land on what is drawn). All chosen paths are
  validated through the existing path sanitizer before any read/write; unreadable directories surface
  a notice instead of crashing.

---

### feature 011: Mouse-operable menus and working menu actions

#### Added

- **Mouse menu interaction.** Clicking a top-level menu title opens (or toggles) its
  dropdown, and **clicking a dropdown item now activates it** — previously only the top row
  was hit-tested and dropdown items could only be reached with the arrow keys. Hit-testing
  shares the exact geometry the menu bar renders with (`hit_test_menu` / `dropdown_layout` in
  `ui/menubar.rs`), so it is correct for built-in *and* plugin menus and for the checkable
  View items. Clicking outside an open menu closes it; clicking in the editor repositions the
  cursor.
- **Help ▸ About** screen showing the program name, version, description, author
  (Mohiuddin Khan Inamdar), and copyright.
- **Help ▸ Help** screen with a key-binding cheat sheet.
- **`Ctrl+N` / File ▸ New** creates a fresh empty buffer.
- **File ▸ Save As** opens a path-entry dialog and writes the active buffer to the new path.

#### Fixed

- **Most Edit/View/File menu items (and their keyboard shortcuts) were no-ops.** `Undo, Redo,
  Cut, Copy, Paste, Select All, Save As, Toggle Line Nos` and `New` dispatched actions that
  had no arm in `handle_action`, so they fell through to a debug-log catch-all and did
  nothing — this also killed the bound `Ctrl+Z/Y/X/C/V/A` shortcuts. All are now wired to the
  existing buffer/undo/clipboard support. (Help did nothing for the same reason.)
- **Mouse clicks on menus past "File" / on dropdown items did nothing**, because mouse events
  were flattened to an action with no coordinates or menu state before the app saw them.
  Mouse events are now routed to `App::handle_mouse_event` with full state.

---

### feature 010: Working Escape key and File ▸ Open

#### Added

- **File ▸ Open is now functional.** Selecting *Open* from the File menu opens a modal
  path-entry dialog; typing a path and pressing `Enter` loads the file into a new buffer
  (path-validated via the existing sanitizer). Previously the menu item dispatched an
  `Open` action that no action handler matched, so it silently did nothing — the
  `OpenFileDialog` widget and `handle_open_file` were present but never wired up.
- **`Ctrl+O` now opens the File ▸ Open dialog.** This shortcut was documented in
  `CAPABILITIES.md` but had no keymap binding; it is now bound to the `Open` action.

#### Fixed

- **Escape key now works.** `Esc` was never bound to any action (no `"Esc"` entry in the
  default keymap and no fallback arm in the key dispatcher), so pressing it did nothing —
  it could not close an open menu/dropdown or cancel any modal dialog, despite the
  handling code (`Action::MenuClose`) already being in place. `Esc` is now bound to
  `MenuClose`, restoring DOS-faithful "Escape backs out" behavior across the menu bar and
  all modal dialogs (encoding select, plugin consent, plugin manager). This completes the
  `Esc` closes the menu behavior described under feature 009.

---

## [0.3.0] - 2026-06-19

### feature 009: Live menu-bar activation

#### Added

- **Keyboard menu navigation**: pull-down menus are now fully operable from the keyboard.
  `F10` activates the menu bar (top-level highlight); `Alt+<letter>` opens a menu's dropdown
  directly; `←`/`→` move between top-level menus (wrapping, opening the adjacent dropdown);
  `↑`/`↓` move within a dropdown (wrapping); `Enter` activates the highlighted item; `Esc`
  closes the menu. Works for both built-in and plugin menus.
- **Plugin-contributed top-level menus** now render in the menu bar, positioned **between
  Options and Help** (Help stays rightmost). Items from a plugin whose menu name matches a
  built-in menu are merged into that built-in dropdown. Activating a plugin item runs its
  sandboxed `menu_action` and shows the result in the status bar. Completes the deferred
  portion of the Plugin API (issue #19).

#### Fixed

- Open dropdowns were previously drawn *under* the editor content and never visible; the menu
  bar now renders on top of the editor area so dropdowns appear correctly.
- Transient status messages (search results, save confirmations, and plugin menu-action
  results) are now displayed in the status bar; previously `status_message` was set but never
  rendered.

---

### feature 008: Plugin API (Rhai)

#### Added

- **Plugin API**: third-party plugins can extend the editor with syntax highlighters, custom
  keybindings, and menu items without modifying the source. Plugins live in
  `$XDG_CONFIG_HOME/edit/plugins/<id>/` as a `plugin.toml` manifest plus (for highlighter/menu
  plugins) a `plugin.rhai` script. Engine: **Rhai** — pure-Rust embedded scripting, no C/C++
  dependencies, statically linkable, builds on every target including FreeBSD.
- **Syntax highlighter plugins**: a plugin highlighter integrates with the existing highlight
  pipeline and takes precedence over the built-in highlighter for its file extensions.
- **Keybinding plugins**: manifest `[keybindings]` entries merge into the keymap; plugin
  bindings take precedence over built-ins, except safety-critical actions (Save, Quit) which
  cannot be overridden.
- **Menu plugins**: manifest `[[menu_items]]` register plugin commands; `menu_action` is
  dispatched in the sandbox and may post a status-bar message.
- **One-time consent dialog**: each newly-installed plugin must be approved before it runs;
  decisions persist to `$XDG_CONFIG_HOME/edit/plugins.toml`.
- **Plugin manager**: Options > Plugins lists installed plugins and toggles them on/off
  (persisted).
- **Default-deny sandbox** (Constitution VII): scripts have no filesystem/process/network
  access; the only file access is the permission-gated `read_file` host function. A 50 ms
  per-call wall-clock limit and resource caps bound execution; a plugin that loops, errors,
  or repeatedly violates the sandbox is disabled for the session — the editor stays responsive.
- **`--no-plugins` CLI flag**: suppresses all plugin loading for the session without modifying
  persisted consent.
- **Reference plugins** under `examples/plugins/`: `lua-syntax`, `word-count`, `custom-keys`
  (each with a README), plus `infinite-loop` / `fs-violation` test fixtures.
- New dependencies: `rhai` (with `sync`) and `semver`. No `extism`/`wasm`/C++ runtime.

#### Notes

- Live keyboard activation of plugin-contributed top-level menu items via the menu bar is
  deferred (the editor's menu-bar item-selection event path is shared with built-in menus and
  is future work); the menu registry, dispatch, and consent/manager dialogs are complete. See
  ROADMAP and the issue tracker (`follow-up`).

---

### feature 007: External File Modification Detection

#### Added

- **External modification detection**: when a file open in the editor is written by another process
  (e.g. `git checkout`, a build tool, another editor), the editor detects the change via inotify
  (Linux) and prompts the user with a DOS-style overlay dialog: **[Y] Reload from disk / [N] Keep
  in editor**.
- **`--no-watch` CLI flag**: disables all filesystem watching for the session; no prompts or
  notices will appear.
- `no_watch = true` supported in `config.toml` (persisted option).
- **Unsaved-changes warning** (US2): the reload dialog grows to 7 rows and adds a
  `WARNING: You have unsaved changes.` line when the buffer has in-editor edits that
  would be lost on reload.
- **File-deleted notice** (US3): when the backing file is removed from disk the editor does
  *not* close the buffer; instead a one-shot status-bar notice appears:
  `[filename] File deleted on disk — buffer kept in memory`.  The buffer remains editable and
  saving it recreates the file.
- **Self-write suppression**: inotify events generated by the editor's own `Ctrl+S` / F5 saves
  are suppressed for a 2-second grace window, preventing spurious reload prompts.
- **1-second debounce**: rapid external writes (e.g. ten overwrites in 0.1 s) coalesce into a
  single prompt (FR-008).
- **Parent-directory watching**: the parent directory (not the file itself) is registered with
  inotify so atomic `mv`-style renames are detected (FR-011).
- **Refcounted directory watches**: when two buffers share the same parent directory, only one
  OS-level watch is registered; the watch is released when the last buffer in that directory is
  closed (FR-011).
- `src/watcher/mod.rs`: new `FileWatcher` struct with `watch_path()`, `unwatch_path()`, and
  non-blocking `try_recv_event()` drain; 6 unit tests.
- `tests/integration/file_watch.rs`: 13 integration tests covering all user stories plus edge
  cases (atomic rename, self-write suppression, debounce, no-watch, binary reload error).
- `StatusBar` now accepts an optional one-shot `notice: &str` parameter; notice is displayed
  centred in the status bar for exactly one render frame.

#### Changed

- `StatusBar::new()` signature: now takes a 6th parameter `notice: Option<&'a str>`.
- `App` struct gains four new fields: `file_watcher`, `self_write_times`, `pending_external_change`,
  `watcher_notice`.
- `Action` enum gains `ReloadFile` and `DismissExternalChange` variants.

---

### feature 006: Menu Check-State Indicator

#### Added

- **Check-state indicator** (non-DOS extension): toggleable View menu items now display a `✓`
  (U+2713) prefix when their associated toggle is active, and a 2-space filler when inactive,
  maintaining consistent label alignment across all items in the dropdown.
- `toggle_states: &'a [(Action, bool)]` field on `MenuBarWidget<'a>`: a zero-cost, zero-allocation
  runtime mapping from action to checked/unchecked boolean, read fresh every render frame from
  `App`'s authoritative state (never stale).
- `lookup_checked()` private helper in `src/ui/menubar.rs`: O(n) slice scan to resolve check state
  for a given action; n ≤ 8 items per dropdown.
- `has_checkable` per-dropdown flag: when `true`, expands `content_width` by 2 and shifts all item
  labels by 2 columns so the prefix column and label column are consistent across all items
  (FR-008 alignment guarantee).
- **"Soft Wrap (ext)"** in the View menu now reflects `App::soft_wrap` state: shows `✓ Soft Wrap
  (ext)` when soft-wrap is ON, plain `Soft Wrap (ext)` when OFF.
- General mechanism: any future toggleable item (in any menu) can participate by adding an entry to
  the `toggle_states` slice at the `Ui::render()` call site in `src/ui/mod.rs` — no further changes
  to `src/ui/menubar.rs` required (FR-007).
- 7 unit tests in `src/ui/menubar.rs` covering: checked/unchecked rendering, non-toggleable menu
  isolation, label alignment, action-agnostic generality (FR-007), empty-toggle-states regression,
  and config-persisted initial state (US3).
- Closes issue #13 (deferred from feature 005).

#### Changed

- `MenuBarWidget::new()` signature: accepts a third `toggle_states: &'a [(Action, bool)]` argument.
  Call site in `src/ui/mod.rs` updated to pass `&[(Action::ToggleSoftWrap, app.soft_wrap)]`.

---

### feature 005: Soft-Wrap Mode

#### Added

- **Soft-wrap rendering mode** (non-DOS extension): optional visual line wrapping at the terminal
  width, toggled via **Alt+Z** or the new "Soft Wrap (ext)" item in the View menu.
- `WrapCache` in `src/ui/wrap.rs`: per-logical-line byte-offset cache computed from grapheme
  clusters using `unicode-segmentation` + `unicode-width`; word-break heuristics for space, tab,
  comma, period, semicolon, colon, hyphen, slash; hard-break fallback at grapheme boundary.
- `»` (U+00BB) continuation marker rendered at the left of each visual continuation row.
- Visual/logical coordinate separation: cursor moves on logical lines; `scroll_offset.0` switches
  to visual-row units when wrap is active; horizontal scroll is zeroed while wrap is on.
- `App::wrap_cache: Option<WrapCache>` and `App::wrap_text_gen: u64` for cache lifecycle management;
  cache rebuilt on resize and after every buffer mutation.
- `App::save_config_to_disk()`: atomic tmp-rename persist of `soft_wrap` to
  `$XDG_CONFIG_HOME/edit/config.toml`; failure logs a warning and sets the status bar message
  without reverting the toggle.
- `soft_wrap: bool` field in `Config` (`src/config/schema.rs`) with TOML round-trip support.
- `[WRAP]` indicator in the status bar when soft-wrap is active.
- Mouse click mapping through `WrapCache::visual_to_logical()` for correct cursor placement in
  soft-wrap mode.
- 10-column viewport-width guard: toggling on below the minimum shows a status message and no-ops.
- 10 new unit tests (toggle cycle, cursor unchanged, Home/End, Up/Down, save byte-identity).
- 3 integration tests in `tests/integration/soft_wrap.rs`.

#### Deferred

- Menu check-indicator (✓ prefix next to "Soft Wrap (ext)" when active): tracked in issue TBD,
  ROADMAP.md. The `[WRAP]` status-bar indicator serves as a workaround for v1.

---

### feature 004: Save-As Encoding Selection UI

#### Added

- Save As Encoding dialog (F12 / File › Save As Encoding...): interactive TUI listbox
  for selecting the output encoding when saving a file (FR-001–FR-013)
- Supported encodings: UTF-8, UTF-16 LE, UTF-16 BE, CP437, CP850, ISO-8859-1, Windows-1252
- Dialog pre-selects the buffer's current encoding on open; wraps at list boundaries (FR-006)
- Confirmed encoding is written atomically (tmp-rename) and status bar shows e.g. "Saved as UTF-16 LE"
- Selected encoding persists in `buffer.encoding` for all subsequent Ctrl+S saves (FR-009)
- I/O failure reverts `buffer.encoding` to its pre-dialog value and shows "Save failed: …" (FR-012)
- Unnamed-buffer path: encoding dialog confirmation stores selection and chains into the
  existing filename-input flow (US4)
- `Action::SaveAsEncoding` variant added to the `Action` enum; `F12` bound in default keymap
- `ENCODING_OPTIONS` constant and `EncodingSelectDialog` widget added to `src/ui/dialog.rs`
- "Save As Encoding..." entry added to the File pull-down menu in `src/ui/menubar.rs`
- 7 unit tests in `src/ui/dialog.rs`; 9 unit tests + 2 integration-level tests in `src/app.rs`
- 6 integration tests in `tests/integration/encoding_select.rs` (UTF-16 LE/BE round-trips,
  cancel-unchanged, persistence, I/O error revert, unnamed-buffer flow)

---

### feature 003: Session Restore

#### Added

- `src/session/mod.rs` — new module: `BufferEntry`, `SplitLayoutKind`, `SessionData` types with
  serde round-trip support; `session_path()`, `save_session()`, `load_session()` functions
- Session file written atomically (`.session.toml.tmp` → rename) to
  `$XDG_STATE_HOME/edit/session.toml` on every clean exit (FR-001, FR-002)
- Session restore dialog: a TUI overlay rendered at startup when a valid session file exists and
  no explicit file arguments or `--no-session` flag were supplied (FR-003, FR-007)
- `Y`/`y`/`Enter` confirms restore; `N`/`n`/`Escape`/`Ctrl+Q` declines (FR-003, FR-007)
- Missing or unreadable files are silently skipped during restore with a status-bar warning;
  the editor falls back to a blank buffer when all files fail (FR-004, FR-005, FR-006)
- Corrupt or invalid session files are treated as absent and overwritten on next clean exit using
  the same atomic sequence; a status-bar warning is shown on startup (FR-010)
- Path traversal guard via `security::sanitize::validate_path` on every path loaded from the
  session file (FR-005, Constitution Principle VII)
- `--no-session` CLI flag suppresses the restore prompt entirely; editor opens a blank buffer
  regardless of session file state (FR-008)
- Explicit `FILE` arguments on the CLI bypass session restore completely (FR-009)
- `active_idx` is clamped when the active buffer was among the skipped/missing files to prevent
  out-of-bounds panics (remediation I1)
- Orphaned `.session.toml.tmp` files from a previous crash are silently removed at startup
- 6 unit tests in `src/session/mod.rs` (`#[cfg(test)]` block)
- 8 integration tests in `tests/integration/session.rs` registered as `[[test]] name = "session"`
- `no_session: bool` field added to `Config` (runtime-only, `#[serde(skip)]`)
- `pending_session_restore`, `default_encoding` fields added to `App` struct
- `App::new` signature extended with `session: Option<SessionData>` and
  `session_warning: Option<String>` parameters

#### Changed

- `App::new` now accepts two additional arguments; callers (`src/main.rs`) pass the
  session data resolved at startup

---

### feature 002: UTF-16 Transcoding

#### Added

- `EncodingId::Utf16Le` and `EncodingId::Utf16Be` variants in `src/encoding/detect.rs`
- UTF-16 LE/BE auto-detection via BOM sniffing (`0xFF 0xFE` / `0xFE 0xFF`) in `detect_encoding()`
- UTF-16 LE/BE decode via `encoding_rs` in `src/encoding/transcode.rs`, with BOM stripping and
  odd-byte-length guard
- UTF-16 LE/BE encode via `str::encode_utf16()` with automatic BOM prefix in `transcode.rs`
- Full round-trip support: file → decode → UTF-8 rope → encode → file (byte-identical)
- Surrogate-pair pass-through (SMP characters such as emoji correctly survive round-trips)
- `encoding_from_str()` aliases in `src/encoding/mod.rs`: `utf-16-le`, `utf16le`, `utf-16-be`,
  `utf16be`, `utf-16` (defaults to LE), case-insensitive
- Status bar displays "UTF-16 LE" / "UTF-16 BE" for open UTF-16 files
- Test fixtures: `tests/fixtures/utf16le_bom.bin`, `utf16be_bom.bin`, `utf16le_nobom.bin`,
  `utf16le_surrogate.bin`
- 20 new unit tests in `src/encoding/transcode.rs` and 7 integration tests in
  `tests/integration/encoding_roundtrip.rs`
- All four integration test suites (`encoding_roundtrip`, `file_io`, `recovery`, `stress`)
  registered in `Cargo.toml` so `cargo test` discovers them

#### Fixed

- FNV-1a 64-bit prime constant in `src/buffer/autosave.rs` corrected to
  `0x0000_0100_0000_01b3` (was `0x0000_0001_00000_01b3` — wrong grouping and wrong value)
- Pre-existing borrow-checker error in `tests/integration/recovery.rs` (`write_recovery` split
  borrow replaced with `write_recovery_for_buffer`)
- 11 pre-existing clippy warnings across `autosave.rs`, `rope.rs`, `buffer/mod.rs`,
  `search/mod.rs`, and `app.rs`

#### Deferred

- Save-As encoding selection UI (interactive dialog to choose output encoding at save time):
  tracked in issue #9, ROADMAP.md

---

## [0.1.0] - 2026-06-18

### Added

- DOS-faithful blue background UI with pull-down menus (US1)
- Full UTF-8/Unicode support with CP437/CP850/ISO-8859-1/Windows-1252 transcoding (US2)
- DOS-style pull-down menu bar with keyboard and mouse navigation (US3)
- Find and Replace with regex support and match highlighting (US4)
- Auto-save and crash recovery with EDIT-RECOVERY-V1 format (US5)
- Multi-file editing with split-view and buffer cycling (US6)
- Syntax highlighting for C, Python, Shell, YAML, Markdown (US7)
- Configurable themes: classic (DOS blue), high-contrast, plain (US8)
- Grapheme-aware cursor movement and text editing
- Undo/redo with composite operation support
- XDG-compliant config, log, and state directories
- Crash handler with panic hook and SIGSEGV recovery
- Man page (`man/edit.1`)
- RPM and Debian packaging configs
- Static musl binary build profile (`make static`, `profile.release-static`)
- Criterion benchmark suite (`benches/startup.rs`, `benches/large_file.rs`, `benches/keystroke.rs`)
- Stress test suite (`tests/integration/stress.rs`, opt-in with `--ignored`)
