# Project Status

**Project**: Linux EDIT.COM Clone (`edit`)
**Version**: 0.4.0 (features 001–035 complete; 038–046 unreleased)
**Last updated**: 2026-06-21

## Implementation Status

| User Story | Description | Status |
|---|---|---|
| F046 | Harden raw slice/index access (#78): checked .get() for selection/cursor-driven list + stored buffer-index lookups; content-bearing no-panic fuzz sweep. Behavior-preserving | Complete |
| F045 | Persist per-tab soft-wrap across restart: session `BufferEntry` gains `soft_wrap` (schema v2, v1 still loads); restore applies each tab's saved value | Complete |
| F044 | Per-tab soft-wrap: `soft_wrap` moved from a global `App` flag to per-`Buffer` state; toggle/indicators act on the active tab; new tabs seed from config; split panes honor each buffer's flag | Complete |
| F043 | Fix: soft-wrap cache leaking across tabs — tab-click/new_buffer switches now invalidate the wrap cache (via centralized `activate_buffer`), fixing ghost wrap + misaligned line numbers on other tabs | Complete |
| F042 | Harden error handling (#72): 24 guarded `unwrap()`s in input/dialog code → pattern matches; `clippy::unwrap_used` guardrail on the app tree; deterministic no-panic fuzz sweep; fixed 2 latent stale-index panics (`char_idx_for`, `line_slice`). Behavior-preserving | Complete |
| F041 | Split `app.rs` (7361→1395 lines) into focused `src/app/*.rs` submodules (dispatch/mouse/dialogs/search/fileops/actions/softwrap/tests). Pure relocation; behavior-preserving (#71) | Complete |
| F040 | Finish active-buffer accessor standardization (FR-008 / #68): all `self.buffers[self.active_idx]` field access routed through `active_buffer()`/`active_buffer_mut()`. Behavior-preserving | Complete |
| F039 | Centralize UI state: one `Modal` enum (illegal two-overlay states unrepresentable) + single `LAYER_PRECEDENCE` driving paint & hit-test + shared Go-to-Line geometry. Behavior-preserving refactor; no user-visible change | Complete |
| F038 | Fix: first dropdown menu item is clickable when the tab bar is open (tab-bar click interception skipped while a dropdown overlays row 1) | Complete |
| F035 | Animated demo GIF (`assets/demo.gif` via `examples/demo_cast.rs` + `make demo-gif`) + README revamp with corrected keybindings | Complete |
| F034 | Crash-safe line access (cursor clamped before render; `line_slice` panic-safe) + always-on crash backtraces + `make debug-run` | Complete |
| F033 | Fix: menu dropdown no longer hidden behind the tab bar (z-order) | Complete |
| F032-US1 | Word-wise cursor movement (Ctrl+Left/Right), grapheme/word-boundary aware, crosses lines | Complete |
| F032-US2 | Word-wise selection (Ctrl+Shift+Left/Right), usable by Copy/Cut | Complete |
| F032-US3 | Word-wise deletion (Ctrl+Backspace/Delete), one undo step, read-only respected | Complete |
| F031-US1 | Click to position the caret in the Find/Replace query & replacement fields (#58) | Complete |
| F031-US2 | File-browser Name field caret editing (Left/Right/Home/End, mid-string insert/delete) + click-to-position (#58) | Complete |
| F031-US3 | Go-to-Line input caret editing + click-to-position (digits-only preserved) (#58) | Complete |
| F030-US1 | Click a list row in the encoding/plugin dialogs to select it (closes #53 list-clicks; field caret → #58) | Complete |
| F030-US2 | Double-click selects the word, triple-click the line (Unicode-aware, multibyte-safe) (#54) | Complete |
| F030-US3 | Right-click context menu (Cut/Copy/Paste/Select All), mouse + keyboard, modal-aware (#55) | Complete |
| F030-US4 | DOS F-keys: F6/Shift+F6 buffer switch, F8/F9/F11 cut/copy/paste (additive) (#56) | Complete |
| F029-US1 | No-panic on real content: multibyte delete/cut, Unicode recovery path, byte→char, oversized-file guard | Complete |
| F029-US2 | Saving never silently loses data: "Saved" on success, error + retained modified on failure; autosave-failure notice | Complete |
| F029-US3 | Dialog consistency: save-before-quit prompt cancels on Esc; Go-to-Line respects modal precedence | Complete |
| F029-US4 | Save-As through the file browser keeps the chosen encoding | Complete |
| F029-US5 | Clicks account for gutter + horizontal scroll; one shared display-width (combining=0, wide=2) across all surfaces | Complete |
| F029-US6 | Action feedback: copy/cut/paste, read-only edits, and file-open failures all report a message | Complete |
| F029-US7 | Ctrl+W + File ▸ Close close the buffer; selected menu item legible in every color theme | Complete |
| F028-US1 | Session restore / buffer switch with soft-wrap never panics (renderer clamps slices; wrap cache invalidated on every active-buffer change) | Complete |
| F028-US2 | A panic restores the terminal (cooked mode, primary screen, cursor) before printing; crash log still written | Complete |
| F028-US3 | Interactive dialogs open focused on the input field, so Save-As typing works and the caret shows | Complete |
| F028-US4 | Arrow keys move focus between dialog buttons (016 + 020 rings), consistent with Tab | Complete |
| F028-US5 | Help/About scroll with Up/Down/PageUp/PageDown/Home/End (clamped) and dismiss from the keyboard | Complete |
| F028-US6 | Home/End move the editor cursor to line start/end; lists support PageUp/PageDown (clamped) | Complete |
| F027-US1 | Buffer tab bar (2+ buffers): one-row strip below the menu bar; click a tab to switch; active highlighted, modified marked | Complete |
| F027-US2 | Tab `✕` close box: clean buffer closes immediately; modified opens a Save/Discard/Cancel confirm (no silent data loss) | Complete |
| F027-US3 | Editor geometry accounts for the tab row (cursor/paging/wheel/scrollbars); single-buffer layout unchanged | Complete |
| F026-US1 | Rust (.rs) syntax highlighting (keywords, types, strings, numbers, comments, attributes, macros) | Complete |
| F026-US2 | JSON (.json) syntax highlighting (keys vs values, numbers, true/false/null, punctuation) | Complete |
| F026-US3 | TOML (.toml) syntax highlighting (headers, keys, strings, numbers, dates, booleans, comments) | Complete |
| F025-US1 | Go to Line (Ctrl+G / Search menu): jump the cursor to a typed 1-based line, scrolled into view | Complete |
| F025-US2 | Out-of-range clamps to first/last; Esc and empty/invalid input never move the cursor | Complete |
| F025-US3 | Prompt is modal (captures input, one at a time); editing/find-replace/other dialogs unchanged | Complete |
| F024-US1 | Clicking a scrollbar track pages the view toward the click (all surfaces) | Complete |
| F024-US2 | Dragging the thumb scrolls proportionally; editor drag is viewport-only (cursor unchanged) | Complete |
| F024-US3 | Scrollbar gestures don't disturb text selection (017), wheel (023), buttons, or keyboard | Complete |
| F023-US1 | Mouse wheel scrolls the editor viewport (3 lines/notch, cursor unchanged, bounded) | Complete |
| F023-US2 | Mouse wheel scrolls the file browser, Help/About, and encoding/plugin dialogs (modal wins) | Complete |
| F023-US3 | Wheel handling leaves click/drag-selection and keyboard scrolling unchanged | Complete |
| F022-US1 | File dialog: live glob/substring filtering (case-insensitive); dirs/`..` always kept; absolute path still jumps | Complete |
| F022-US2 | File dialog: per-entry size + modified-date columns (`<DIR>` for folders), aligned, name truncates | Complete |
| F021-US1 | Scrollbars on overflow: editor (vertical + horizontal), file browser, Help/About, encoding/plugin | Complete |
| F021-US2 | Help and About show a clickable, bordered Close button (mouse-dismissable; Esc still works) | Complete |
| F021-US3 | Every dialog button label advertises its activating key (no behavioral change) | Complete |
| F020-US1 | Interactive dialogs (encoding, plugin mgr, Find/Replace, file browser): mouse-clickable boxed buttons | Complete |
| F020-US2 | Combined list/field + button focus ring; Tab/Shift+Tab cycle; Enter/Space activate | Complete |
| F020-US3 | All existing dialog keys preserved (list nav, toggles, match nav, Esc) — zero regression | Complete |
| F019-US1 | Find dialog: search field rendered as a bordered, labeled input box with a caret | Complete |
| F019-US2 | Replace dialog: both fields as bordered boxes; caret only in the focused field | Complete |
| F019-US3 | All existing Find/Replace behavior (editing, Tab, toggles, count, Esc) preserved | Complete |
| F018-US1 | File dialog: bordered, labeled input box with caret (Open path field now visible) | Complete |
| F018-US2 | Help redesigned as a grouped, scrollable Key\|Action table (no truncation) | Complete |
| F017-US1 | Selected text rendered with a highlight (reverse video); Select All shows it | Complete |
| F017-US2 | Shift+Arrow/Home/End keyboard selection; typing/paste replaces; Copy/Cut act on it | Complete |
| F017-US3 | Mouse press-drag-release selects; single click clears | Complete |
| F016-US1 | Mouse-click dialog buttons (confirm/dismiss dialogs); outside-click cancels | Complete |
| F016-US2 | Boxed buttons with one focused (borders + ▶ marker) | Complete |
| F016-US3 | Tab/Shift+Tab order + Enter/Space activation; safe default focus; letter shortcuts intact | Complete |
| F016 (defer) | Boxed buttons for encoding/plugin-manager/Find-Replace/file-browser dialogs | Complete (feature 020) |
| F015-US1 | Interactive Find dialog (Ctrl+F): type term, Enter to search, highlight matches, jump to current, "X of Y" | Complete |
| F015-US2 | Find Next/Prev (F3/F2) cycle matches with wrap; current-match highlight distinct | Complete |
| F015-US3 | Replace dialog (Ctrl+H): Replace current (Enter) + Replace All (Ctrl+A), undoable, reports count | Complete |
| F015-US4 | Search options: case-sensitive, wrap, regex, whole-word (new engine support) | Complete |
| F014-US1 | Undo back to the saved/opened content clears `[Modified]`; redo restores it | Complete |
| F014-US2 | No false-clean after divergent edits (saved point invalidated when its branch is discarded) | Complete |
| F014-US3 | File ▸ Revert reloads from disk (confirm when dirty; no-op for never-saved; safe on read error) | Complete |
| F013-US1 | Underlined accelerator letter shown on every top-level menu and dropdown item | Complete |
| F013-US2 | Typing an item's accelerator while its dropdown is open activates it; non-match is inert | Complete |
| F013-US3 | Top-level letter / `Alt`+letter opens the matching menu; bare `Alt` activates the bar (terminal-permitting) | Complete |
| F013-US4 | Plugin menu items/menus get auto-assigned, unique, collision-free accelerators | Complete |
| F012-US1 | Open a file by browsing the directory tree (keyboard + mouse), no path typing required | Complete |
| F012-US2 | Save to a chosen folder/name by browsing; `Ctrl+S` on an unnamed buffer opens the Save browser | Complete |
| F012-US3 | Consistent keyboard & mouse navigation; long listings scroll; UTF-8-safe name display | Complete |
| F011-US1 | Mouse: click top-level menus and dropdown items; click-outside closes; editor click repositions cursor | Complete |
| F011-US2 | Edit/File/View menu actions wired (Undo/Redo/Cut/Copy/Paste/Select All/New/Save As/Toggle Line Nos) | Complete |
| F011-US3 | Help ▸ Help (key bindings) and Help ▸ About (version + copyright) screens | Complete |
| F010-US1 | File ▸ Open (and `Ctrl+O`) opens a modal path dialog that loads the file into a new buffer | Complete |
| F010-US2 | `Esc` bound to `MenuClose`: closes menus and cancels all modal dialogs (was a no-op) | Complete |
| F008-US1 | Syntax highlighter plugins (Rhai); plugin highlighter takes precedence over built-in | Complete |
| F008-US2 | Custom keybinding plugins; merged into keymap; Save/Quit non-overridable | Complete |
| F008-US3 | Menu item plugins; `menu_action` dispatched in sandbox; live menu-bar activation wired in feature 009 | Complete |
| F009-US1 | Keyboard navigation/activation of built-in pull-down menus (arrows/Enter/Esc; F10 + Alt+letter entry) | Complete |
| F009-US2 | Plugin-contributed top-level menus render (between Options and Help) and activate via keyboard | Complete |
| F009-US3 | DOS-faithful navigation semantics: wrap-around, Left/Right ring, modal precedence | Complete |
| F008-US4 | Plugin manager (Options > Plugins) + one-time consent dialog; decisions persisted | Complete |
| F008-US5 | Default-deny sandbox: 50 ms timeout, FS-violation denial, crash isolation | Complete |
| F007-US1 | Detect external file modification, prompt Y/N reload dialog | Complete |
| F007-US2 | Unsaved-changes warning shown in reload dialog when buffer is dirty | Complete |
| F007-US3 | File-deleted notice in status bar; buffer kept in memory | Complete |
| F007-US4 | `--no-watch` CLI flag / `no_watch` config option to disable watching | Complete |
| US1 | Basic File Editing (open, navigate, edit, save, quit) | Complete |
| US2 | UTF-8/Unicode support, CP437/CP850/ISO-8859-1/Windows-1252 transcoding | Complete |
| F002-US1 | UTF-16 LE/BE auto-detect (BOM sniffing) | Complete |
| F002-US2 | UTF-16 LE/BE decode/encode with full round-trip and surrogate-pair support | Complete |
| F002-US3 | `--encoding utf-16-le/be` CLI aliases via `encoding_from_str()` | Complete |
| F002-US4 | Save-As encoding selection UI (interactive dialog) | Complete (feat 004) |
| F006-US1 | View menu "Soft Wrap (ext)" shows `✓` prefix when soft-wrap is ON; no prefix when OFF | Complete |
| F006-US2 | Check-state mechanism general: any action/bool pair in `toggle_states` shows `✓` | Complete |
| F006-US3 | Check-state reflects config-persisted `soft_wrap=true` on first render (no toggle needed) | Complete |
| F005-US1 | Soft-wrap visual rendering with `»` continuation marker; Alt+Z / View menu | Complete |
| F005-US2 | Cursor, Home/End, mouse click work on logical lines in wrap mode | Complete |
| F005-US3 | Soft-wrap setting persisted to `config.toml` via atomic write | Complete |
| F005-US4 | `[WRAP]` status-bar indicator; "Soft Wrap (ext)" in View menu | Complete |
| F004-US1 | Save active buffer in chosen encoding via dialog (F12 / File menu) | Complete |
| F004-US2 | Cancel encoding dialog — file and encoding unchanged | Complete |
| F004-US3 | Selected encoding persists for subsequent Ctrl+S saves | Complete |
| F004-US4 | Unnamed buffer triggers filename prompt after encoding selection | Complete |
| F003-US1 | Session restore: write session on clean exit; TUI restore dialog on relaunch | Complete |
| F003-US2 | Handle missing/unreadable session files gracefully; status-bar warning | Complete |
| F003-US3 | `--no-session` CLI flag suppresses restore prompt | Complete |
| F003-US4 | Explicit file arguments bypass session restore | Complete |
| US3 | DOS-style pull-down menu bar, keyboard and mouse navigation | Complete |
| US4 | Find and Replace with regex support and match highlighting | Complete |
| US5 | Auto-save and crash recovery (EDIT-RECOVERY-V1 format) | Complete |
| US6 | Multi-file editing with split-view and buffer cycling | Complete |
| US7 | Syntax highlighting for C, Python, Shell, YAML, Markdown | Complete |
| US8 | Configurable themes: classic (DOS blue), high-contrast, plain | Complete |

## Feature Summary

- Grapheme-aware cursor movement and text editing
- Undo/redo with composite operation support
- XDG-compliant config, log, and state directories
- Crash handler with panic hook and SIGSEGV recovery via `signal-hook`
- Man page at `man/edit.1`
- RPM and Debian packaging configs
- Static musl binary support (`make static`)

## CI Matrix

### Target Platforms

| Target | Toolchain | Profile | Notes |
|---|---|---|---|
| `x86_64-unknown-linux-gnu` | stable 1.74.0+ | debug, release | Primary development target |
| `aarch64-unknown-linux-gnu` | stable 1.74.0+ | debug, release | Cross-compiled via cross |
| `x86_64-unknown-linux-musl` | nightly | release-static | Static binary, no glibc dependency |

### Rust Toolchain

- **Minimum supported**: stable 1.74.0 (required for `ratatui` 0.26 and `clap` 4)
- **Nightly**: used only for the `release-static` musl profile; not required for development
- **Edition**: 2021

### Test Suite

| Suite | Command | Description |
|---|---|---|
| Unit tests | `cargo test` | All `#[cfg(test)]` modules in `src/` |
| Integration tests | `cargo test --test '*'` | Files under `tests/integration/` |
| Smoke tests | `make smoke` | `expect`-based scripts in `tests/smoke/` (requires `expect` + `tmux`) |
| Stress tests | `cargo test --test stress -- --ignored` | Continuous-editing and encoding stress tests (slow, opt-in) |
| Benchmarks | `make perf-check` | Criterion benchmarks in `benches/` |

### Build Profiles

| Profile | Command | Output | Notes |
|---|---|---|---|
| `debug` | `make build` / `cargo build` | `target/debug/edit` | Debug symbols, no optimizations |
| `release` | `make release` / `cargo build --release` | `target/release/edit` | LTO, stripped, `-O3` |
| `release-static` | `make static` | `target/x86_64-unknown-linux-musl/release-static/edit` | musl, static linkage, requires musl target + nightly |

### CI Gate (`make ci-local`)

Runs in order:
1. `cargo fmt --check` — formatting
2. `cargo clippy -- -D warnings` — lints
3. `cargo test` — unit + integration tests
4. `make smoke` — expect smoke tests
5. `make perf-check` — benchmarks (non-regressing, results logged)

## Known Limitations
- Menu activation is keyboard-driven; mouse-click menu selection is not yet wired (keyboard
  navigation covers all menus). General mouse support requires a terminal emulator that reports
  mouse events in crossterm's supported protocol.
