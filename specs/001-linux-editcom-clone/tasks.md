---
description: "Task list for Linux EDIT.COM Clone implementation"
---

# Tasks: Linux EDIT.COM Clone

**Input**: Design documents from `specs/001-linux-editcom-clone/`

**Prerequisites**: plan.md ✅ | spec.md ✅ | research.md ✅ | data-model.md ✅ | contracts/ ✅

**Tests**: Smoke tests (expect scripts) and integration tests are included as they are
explicitly required by the spec (FR-001–FR-025, SC-006, SC-010) and constitution (Principle V).

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1–US8)
- Exact file paths included in every task description

## Path Conventions

Single Rust project at repository root:
- Source: `src/<subsystem>/<file>.rs`
- Tests: `tests/unit/`, `tests/integration/`, `tests/smoke/`
- Packaging: `packaging/`, `man/`
- Docs: `docs/`

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Initialize the Rust project, dependency manifest, build tooling, and directory structure.

- [X] T001 Initialize Rust binary crate with `cargo init --name edit` and set `edition = "2021"` in `Cargo.toml`
- [X] T002 Add all dependencies to `Cargo.toml`: `ratatui`, `crossterm`, `unicode-width`, `unicode-segmentation`, `encoding_rs`, `oem-cp`, `ropey`, `chardetng`, `regex`, `serde`+`toml`, `clap`, `dirs`, `log`, `env_logger`, `signal-hook`, `arboard`
- [X] T003 [P] Create full source directory tree: `src/buffer/`, `src/ui/`, `src/input/`, `src/encoding/`, `src/search/`, `src/highlight/languages/`, `src/config/`, `src/security/`, `src/diagnostics/`
- [X] T004 [P] Create `Makefile` with targets: `build`, `release`, `check`, `smoke`, `perf-check`, `static`, `package`, `docs-gate`, `ci-local`, `help`; update `.PHONY`, file-header comment, and `help:` body
- [X] T005 [P] Create `tests/unit/`, `tests/integration/`, `tests/smoke/`, `benches/`, `docs/`, `man/`, `packaging/` directories and add placeholder `.gitkeep` files

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before any user story can be implemented.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T006 [P] Implement `src/diagnostics/logging.rs`: `env_logger` init, XDG log path construction (`$XDG_STATE_HOME/edit/logs/edit-<date>.log`), `init_logging(level: LevelFilter)` function
- [X] T007 [P] Implement `src/diagnostics/crash.rs`: `install_panic_hook()` that writes crash report to `$XDG_STATE_HOME/edit/crash-<timestamp>.log` with backtrace; `install_signal_handler()` for SIGSEGV via `signal-hook`
- [X] T008 [P] Implement `src/security/sanitize.rs`: `strip_escape_sequences(s: &str) -> String` (removes ANSI CSI/OSC/DCS sequences); `validate_path(p: &Path) -> Result<PathBuf>` (rejects `../` traversal above cwd)
- [X] T009 [P] Implement `src/config/schema.rs`: `Config` struct with all fields from `contracts/config.md`; derive `serde::Deserialize`; `Default` impl with documented defaults
- [X] T010 Implement `src/config/mod.rs`: `load_config() -> Config` (read TOML from XDG path, log warnings for unknown keys, log errors for type mismatches, never panic); `merge_cli_flags(config: &mut Config, matches: &ArgMatches)`
- [X] T011 [P] Implement `src/encoding/detect.rs`: `detect_encoding(bytes: &[u8]) -> EncodingId` (BOM detection for UTF-8/UTF-16; heuristic fallback using `chardetng`); `EncodingProfile` struct and `ENCODING_REGISTRY` constant array
- [X] T012 [P] Implement `src/encoding/transcode.rs`: `decode(bytes: &[u8], enc: EncodingId) -> Result<String>` using `encoding_rs` for `Iso8859_1`/`Windows1252` and `oem-cp` for `Cp437`/`Cp850`; `encode(s: &str, enc: EncodingId) -> Result<Vec<u8>>`
- [X] T013 Implement `src/encoding/mod.rs`: re-export `EncodingId`, `EncodingProfile`, `detect_encoding`, `decode`, `encode`; `EncodingId` enum with all five variants
- [X] T014 Implement `src/buffer/rope.rs`: `EditorRope` wrapper around `ropey::Rope` with methods: `insert_str(char_idx, s)`, `delete_range(range)`, `line_count()`, `line_slice(line_idx)`, `graphemes_on_line(line_idx) -> Vec<&str>`, `char_to_byte(char_idx)`, `byte_to_char(byte_idx)`
- [X] T015 Implement `src/buffer/undo.rs`: `EditOp` enum (`Insert`, `Delete`, `Composite`); `UndoStack` struct with `push(op)`, `undo(rope) -> Option<EditOp>`, `redo(rope) -> Option<EditOp>`, `truncate_redo()`
- [X] T016 [P] Implement `src/ui/theme.rs`: `Theme` struct with all color fields from data-model; `CLASSIC` constant instance only; `theme_by_name(name: &str) -> &'static Theme` stub returning `CLASSIC` (HIGH_CONTRAST and PLAIN are implemented in T079 — not here, to avoid duplication)
- [X] T017 Implement `src/input/keymap.rs`: `Action` enum (all action variants from `contracts/config.md`); `KeybindingMap` struct; `default_map() -> KeybindingMap` with EDIT.COM defaults; `apply_user_overrides(map: &mut KeybindingMap, overrides: &HashMap<String,String>)` with conflict logging
- [X] T018 [P] Implement `src/input/mouse.rs`: `MouseEvent` normalization from crossterm's `MouseEvent` to `(col, row, MouseButton)` in terminal cell coordinates; `handle_mouse(event, app_state) -> Option<Action>`
- [X] T098 Implement `src/input/mod.rs`: event dispatcher skeleton — `dispatch_event(event: crossterm::event::Event, keymap: &KeybindingMap) -> Option<Action>`; re-exports `Action`, `KeybindingMap`; stub `handle_resize(w: u16, h: u16) -> Action` (needed by `src/main.rs` T019 and `src/app.rs` T020 before the full US3 menu wiring in T048–T049)
- [X] T019 Implement `src/main.rs`: `clap` CLI definition matching `contracts/cli.md` (all flags, positional FILE args, `--help`, `--version`); locale detection + UTF-8 enforcement + warning; call `init_logging`, `install_panic_hook`, `load_config`, `merge_cli_flags`, `App::run()`
- [X] T020 Implement `src/app.rs`: `App` struct skeleton (config, keymap, buffer list, active buffer index, running flag); `App::run()` entering crossterm raw mode + alternate screen; main event loop dispatching `crossterm::event::read()` to `handle_key`, `handle_mouse`, `handle_tick`

**Checkpoint**: Foundation complete — user story implementation can begin in parallel.

---

## Phase 3: User Story 1 — Basic File Editing (Priority: P1) 🎯 MVP

**Goal**: Open a file, navigate, edit text, save, and quit with unsaved-changes prompt.

**Independent Test**: `cargo run -- /tmp/test.txt` → type text → Ctrl+S → Ctrl+Q → `cat /tmp/test.txt` shows changes. See `quickstart.md` Scenario A–D.

**TDD gate** (write these tests FIRST — they will fail until T021–T034 + T099–T107 are implemented):

- [X] T099 [P] [US1] Write `tests/smoke/basic_edit.exp`: expect script — launch editor, type text, Ctrl+S, Ctrl+Q, verify file content; exit non-zero on failure (TDD — must exist before T021)
- [X] T100 [P] [US1] Write `tests/integration/file_io.rs`: open file → edit → save → read back and assert content; test new-file creation; test read-only enforcement (TDD — must exist before T023)
- [X] T101 [P] [US1] Write `tests/unit/security_sanitize.rs`: assert `strip_escape_sequences` removes ANSI CSI; assert `validate_path` rejects `../../etc/passwd` and accepts valid relative/absolute paths (TDD — must exist before implementing T008 callers)

- [X] T021 [P] [US1] Implement `CursorPos` struct in `src/buffer/mod.rs`: `line`, `grapheme_col`, `visual_col` fields; `visual_col_from_grapheme_col(rope, line, gcol) -> usize` using `unicode-width`
- [X] T022 [P] [US1] Implement `Selection` struct in `src/buffer/mod.rs`: `anchor` + `active` CursorPos; `ordered_range() -> (CursorPos, CursorPos)`; `is_empty() -> bool`
- [X] T023 [US1] Implement `Buffer` struct in `src/buffer/mod.rs`: all fields from data-model; `Buffer::open(path, encoding) -> Result<Buffer>` (read file bytes → transcode → load rope); `Buffer::new_empty() -> Buffer`
- [X] T024 [US1] Implement `Buffer::save(&self) -> Result<()>` in `src/buffer/mod.rs`: encode rope to bytes via `EncodingProfile`, write atomically (tmp file → rename), clear `modified` flag
- [X] T025 [US1] Implement cursor movement actions in `src/app.rs`: `move_cursor(dir: Direction)` for Up/Down/Left/Right; `move_line_start()`, `move_line_end()`, `move_page_up()`, `move_page_down()`, `move_doc_start()`, `move_doc_end()` — all updating `CursorPos` and `scroll_offset`
- [X] T026 [US1] Implement text insertion in `src/app.rs`: `insert_char(c: char)` — validate UTF-8, call `rope.insert_str`, push `EditOp::Insert` to undo stack, set `modified = true`
- [X] T027 [US1] Implement Backspace and Delete in `src/app.rs`: `delete_backward()` and `delete_forward()` — grapheme-aware removal, push `EditOp::Delete`, set `modified = true`
- [X] T028 [US1] Implement `src/ui/editor.rs`: `EditorWidget` implementing ratatui `Widget`; renders rope lines within scroll viewport; draws cursor cell; left gutter for line numbers (when enabled); lines wider than terminal width use **horizontal scroll** (not soft-wrap) — matching EDIT.COM behavior; horizontal position is tracked by `Buffer.scroll_offset.1` (the existing tuple's column element — see data-model.md "Long-line handling"); note: "title bar" from spec is implemented as the ratatui block title at row 0 of the editor area, not a separate widget
- [X] T029 [US1] Implement `src/ui/statusbar.rs`: `StatusBar` widget rendering filename (or `[No Name]`), `[Modified]`/`[Read Only]` flags, row/col position, encoding name — updates on every render tick
- [X] T030 [US1] Implement `src/ui/menubar.rs` label row (non-interactive): render "File  Edit  Search  View  Options  Help" in theme `menubar_bg`/`menubar_fg` colors at row 0
- [X] T031 [US1] Implement `src/ui/dialog.rs`: `SavePromptDialog` — modal overlay "Save changes to <filename>? [S]ave / [D]iscard / [C]ancel" with keyboard dispatch
- [X] T032 [US1] Implement `src/ui/mod.rs`: `Ui::render(frame, app)` — compose menubar (row 0) + editor area (rows 1..height-1) + statusbar (last row); handle dialog overlay rendering
- [X] T033 [US1] Wire quit flow in `src/app.rs`: `handle_quit()` — if `buffer.modified`, show `SavePromptDialog`; on Save → call `buffer.save()`; on Discard/Save-success → exit event loop; on Cancel → return to editing
- [X] T034 [US1] Implement read-only enforcement in `src/app.rs`: `insert_char`, `delete_backward`, `delete_forward` are no-ops when `buffer.readonly = true`; statusbar shows `[Read Only]`
- [X] T102 [US1] Implement clipboard cut/copy/paste in `src/app.rs`: `cut_selection()` → copy selected text to system clipboard via `arboard::Clipboard`, delete selection, push `EditOp::Delete`; `copy_selection()` → clipboard write only; `paste_clipboard()` → read clipboard, insert at cursor, push `EditOp::Insert`; no-op when `buffer.readonly`
- [X] T103 [P] [US1] Implement `SaveAsDialog` in `src/ui/dialog.rs`: single-line path input modal; on confirm, call `security::validate_path`; if the target path already exists, show an Overwrite/Cancel confirmation first (per FR-028); on confirm, call `buffer.save_as(new_path)` (write to new path + update `buffer.path` + clear `modified`); wire `Action::SaveAs` through `handle_save_as()` in `src/app.rs`; File > Save As... menu item dispatches this action
- [X] T104 [US1] Detect and preserve line endings in `Buffer::open` and `Buffer::save`: scan first 512 bytes for `\r\n`; store dominant ending in `Buffer.line_ending: LineEnding` enum (`LF` / `CRLF`); strip `\r` on load; re-insert `\r` before `\n` on save when `line_ending == CRLF`
- [X] T105 [US1] Handle terminal resize in `src/app.rs` event loop: on `crossterm::event::Event::Resize(w, h)`, update `App.terminal_size`; re-clamp both elements of `scroll_offset` (vertical line + horizontal `.1`) so the cursor stays visible; force full re-render on next tick; no crash or visual corruption; reject sizes below the documented minimum (see spec FR-026) by painting a "Terminal too small (min 80×24)" notice instead of the editor
- [X] T106 [US1] Detect binary/null-byte content in `Buffer::open`: scan first 512 bytes for null bytes (`\x00`); if found, return `Err(BufferError::BinaryContent)` and show a `ErrorDialog` "Binary file — cannot edit" instead of opening; editor remains open with no active buffer change
- [X] T107 [US1] Implement `SaveErrorDialog` in `src/ui/dialog.rs`: modal "Cannot save: <OS error>. Retry / Cancel"; wire to `Err` return from `Buffer::save()` in `handle_save()` and `handle_quit()` so disk-full and permission-denied errors surface visibly instead of silently failing

**Checkpoint**: User Story 1 complete — open/edit/save/quit independently functional and testable.

---

## Phase 4: User Story 2 — UTF-8 and Unicode Display (Priority: P1)

**Goal**: Correct rendering and navigation for multibyte, wide, combining, and emoji characters.

**Independent Test**: Open `tests/fixtures/unicode_sample.txt` (Japanese + emoji + combining chars); verify no misalignment; round-trip CP437 file. See `quickstart.md` US2 scenarios.

**TDD gate** (write before implementing T035–T040):

- [X] T108 [P] [US2] Write `tests/smoke/unicode_display.exp`: expect script — open `unicode_sample.txt`, send cursor-right key sequence for Japanese line, capture column output, assert cursor column matches expected visual width (TDD)
- [X] T109 [P] [US2] Write `tests/integration/encoding_roundtrip.rs`: for CP437, CP850, ISO-8859-1, Windows-1252 — read fixture bytes, transcode to UTF-8 via `decode()`, transcode back via `encode()`, assert byte-for-byte identical to fixture (TDD)

- [X] T035 [P] [US2] Create test fixture `tests/fixtures/unicode_sample.txt` containing Japanese kanji, Arabic text, combining characters (e + ̀), and emoji (😀) as a UTF-8 file
- [X] T036 [P] [US2] Create test fixture `tests/fixtures/cp437_box.bin` — raw CP437 bytes for a box-drawing frame (0xC9 0xCD 0xCD 0xBB header line)
- [X] T037 [US2] Implement grapheme-cluster-aware cursor navigation in `src/buffer/rope.rs`: `grapheme_count_on_line(line_idx) -> usize` via `unicode-segmentation`; `grapheme_at(line_idx, gcol) -> &str`; cursor Left/Right advance by one grapheme cluster, updating both `grapheme_col` and `visual_col`
- [X] T038 [US2] Implement wide-char scroll correction in `src/ui/editor.rs`: when rendering a line, accumulate visual column width per grapheme via `UnicodeWidthChar::width()`; skip rendering into the second cell of a wide char; fill it with a space at the same background color
- [X] T039 [US2] Implement non-UTF-8 locale warning in `src/main.rs`: after `setlocale`, check if resolved locale is UTF-8; if not, log warning and show `LocaleWarningDialog` in `src/ui/dialog.rs` before entering the editor
- [X] T040 [US2] Wire `--encoding` flag to `EncodingProfile` selection in `src/main.rs` and thread through `Buffer::open(path, encoding)` in `src/app.rs`

**Checkpoint**: User Story 2 complete — multilingual content displays and navigates correctly.

---

## Phase 5: User Story 3 — DOS-Style Menu and Keyboard Navigation (Priority: P1)

**Goal**: Full pull-down menu system with Alt+letter activation, arrow navigation, Escape to close, and all F-key bindings.

**Independent Test**: `expect tests/smoke/menu_nav.exp` exits 0; all 6 menus open and close correctly; F1/F3/F5/F10 trigger correct actions.

**TDD gate** (write before implementing T041–T051):

- [X] T110 [P] [US3] Write `tests/smoke/menu_nav.exp`: expect script — open all 6 menus via Alt+key, navigate items with arrow keys, close each with Escape; verify no crash and status bar restored (TDD)

- [X] T041 [P] [US3] Implement pull-down menu state machine in `src/ui/menubar.rs`: `MenuState` enum (`Inactive`, `TopActive(usize)`, `DropDown { top_idx, item_idx }`); `open_menu(idx)`, `close_menu()`, `navigate_down()`, `navigate_up()`, `select_item() -> Option<Action>` methods
- [X] T042 [US3] Implement File menu items in `src/ui/menubar.rs`: New / Open... / Save / Save As... / Exit — each maps to an `Action` variant; render highlighted item in `menu_selected_bg` color
- [X] T043 [US3] Implement Edit menu items in `src/ui/menubar.rs`: Undo / Redo / Cut / Copy / Paste / Select All — mapped to corresponding `Action` variants
- [X] T044 [US3] Implement Search menu items in `src/ui/menubar.rs`: Find... / Find Next / Find Previous / Replace... — mapped to `Action` variants
- [X] T045 [US3] Implement View menu items in `src/ui/menubar.rs`: Split View / Next Buffer / Previous Buffer / Toggle Line Numbers — mapped to `Action` variants
- [X] T046 [US3] Implement Options menu items in `src/ui/menubar.rs`: Theme / Default Encoding / Autosave On/Off / Syntax Highlight On/Off — mapped to `Action` variants
- [X] T047 [US3] Implement Help menu and built-in help screen in `src/ui/menubar.rs` and `src/ui/dialog.rs`: Help > Help Contents shows a `HelpDialog` with F-key reference and menu key guide; F1 shortcut triggers same dialog
- [X] T048 [US3] Implement Alt+letter menu activation in `src/input/mod.rs`: map `Alt+F/E/S/V/O/H` to `Action::MenuOpen(idx)`; dispatch to `MenuState`; Escape from menu → `Action::MenuClose`
- [X] T049 [US3] Implement F-key bindings in `src/input/mod.rs`: F1 → `help`, F3 → `find_next`, F5 → `save`, F10 → `menu`; wire through `KeybindingMap`
- [X] T050 [US3] Implement no-color terminal fallback in `src/ui/theme.rs`: `terminal_supports_color() -> bool` via crossterm `supports_ansi!`; if false, `CLASSIC` theme replaces colors with `Modifier::REVERSED` on ALL chrome — menu bar, status bar, dialog boxes, and selected-item highlights (per FR-008)
- [X] T051 [US3] Implement mouse click-to-menu in `src/input/mouse.rs`: detect click on row 0 within a menu label's column range → open corresponding top-level menu; click on dropdown item → select it
- [X] T111 [US3] Implement mouse click-in-editor-area cursor reposition in `src/app.rs`: for `MouseEvent` clicks with `row >= 1 && row <= height - 2` (editor rows), compute `line = scroll_offset.0 + row - 1` and `grapheme_col` by scanning visual widths of graphemes from `scroll_offset.1` up to the clicked `col`; update `Buffer.cursor`; bring clicked line into scroll view

**Checkpoint**: User Story 3 complete — full DOS-style menu navigation independently functional.

---

## Phase 6: User Story 4 — Search and Replace (Priority: P1)

**Goal**: Incremental find with regex, F3 next-match with wrap, find-and-replace with replace-all as undoable Composite op.

**Independent Test**: Open file with repeated terms; Ctrl+F, search, F3 cycles all matches; Replace All substitutes all; Ctrl+Z undoes all replacements in one step. See `quickstart.md` US4.

**TDD gate** (write before implementing T052–T057):

- [X] T112 [P] [US4] Write `tests/smoke/search_replace.exp`: expect script — search for known term, F3 next-match, replace-all, save, quit; assert output file has replacements (TDD)

- [X] T052 [P] [US4] Implement `SearchState` struct in `src/search/mod.rs`: all fields from data-model; `SearchEngine::find_all(rope, query, regex_mode, case_sensitive) -> Vec<CharRange>`; invalidate matches on buffer edit
- [X] T053 [P] [US4] Implement `src/search/highlight.rs`: `collect_match_spans(rope, matches, active_match) -> Vec<(CharRange, Style)>` for rendering highlighted matches in the editor widget
- [X] T054 [US4] Implement find dialog `FindDialog` in `src/ui/dialog.rs`: single-line query input; checkboxes for Regex and Case Sensitive; Escape to close; Enter/F3 to find first match
- [X] T055 [US4] Implement `find_next()` and `find_prev()` in `src/app.rs`: advance `SearchState::active_match`; scroll editor to show the match; display "Not found" or "Search wrapped" in status bar
- [X] T056 [US4] Implement `ReplaceDialog` in `src/ui/dialog.rs`: query + replacement inputs; Replace (single, at active match) and Replace All buttons; display replacement count in status bar
- [X] T057 [US4] Implement Replace All as `EditOp::Composite` in `src/app.rs`: collect all match ranges (reverse order to preserve indices), apply deletions + insertions, push one `Composite` op — single Ctrl+Z undoes all substitutions

**Checkpoint**: User Story 4 complete — full search and replace independently functional.

---

## Phase 7: User Story 5 — Auto-Save and Crash Recovery (Priority: P1)

**Goal**: Silently write recovery file every 30 seconds; detect stale lock on next open; offer and apply recovery.

**Independent Test**: `kill -9` the editor PID after 35 s; reopen same file; recovery dialog appears; accept → edits restored. See `quickstart.md` US5.

**TDD gate** (write before implementing T058–T064):

- [X] T113 [P] [US5] Write `tests/integration/recovery.rs`: spawn editor process, type text, wait 35 s (or parameterized `EDIT_AUTOSAVE_INTERVAL` env var for fast testing), SIGKILL, reopen, assert recovery dialog triggers, accept recovery, assert content matches pre-kill state (TDD)

- [X] T058 [P] [US5] Implement `src/buffer/autosave.rs`: `AutosaveState` struct; `recovery_path_for(abs_path: &Path) -> PathBuf` (SHA-256 of path, hex-encoded filename, under XDG_RUNTIME_DIR or TMPDIR fallback); dir created with mode 0700
- [X] T059 [US5] Implement `create_lock(state: &AutosaveState)` and `release_lock(state: &AutosaveState)` in `src/buffer/autosave.rs`: write PID to `.lock` file; delete on release; check_stale_lock returns `LockStatus::{OtherSessionActive(pid), StaleRecovery, Clean}`
- [X] T060 [US5] Implement `write_recovery(buffer: &Buffer)` in `src/buffer/autosave.rs`: serialize recovery file header + rope content per `contracts/recovery.md` format; atomic write (tmp → rename); update `AutosaveState::last_save_at`
- [X] T061 [US5] Implement `read_recovery(path: &Path) -> Result<RecoveryData>` in `src/buffer/autosave.rs`: parse header fields, validate `content_len`, return content string
- [X] T062 [US5] Wire autosave timer tick in `src/app.rs`: on each `handle_tick()` call, check elapsed since `last_save_at`; if ≥ `interval_secs` and `buffer.modified` and `autosave.enabled` → call `write_recovery`
- [X] T063 [US5] Implement stale-lock check on startup in `src/app.rs`: before opening buffer, call `check_stale_lock`; on `StaleRecovery` → show `RecoveryDialog`; on accept → load recovery content into buffer; on decline → delete recovery file; always (re)create lock with current PID
- [X] T064 [US5] Implement `RecoveryDialog` in `src/ui/dialog.rs`: modal "Recovery file found from <timestamp>. Recover? [Y/N]"; wire Y/N keys to accept/decline actions; clean up lock on exit

**Checkpoint**: User Story 5 complete — auto-save and crash recovery independently functional.

---

## Phase 8: User Story 6 — Multi-File Editing (Priority: P2)

**Goal**: Open multiple buffers, switch between them, edit independently, split-view two panels.

**Independent Test**: `edit fileA.txt fileB.txt` → switch via View menu → edit each → save each → verify both files correct. See `quickstart.md` US6.

- [X] T065 [P] [US6] Implement buffer manager in `src/app.rs`: change `App.buffer` from single `Buffer` to `Vec<Buffer>` + `active_idx: usize`; update ALL call sites that assumed a single buffer — specifically: `handle_key` (T025–T027), `insert_char`/`delete_backward`/`delete_forward` (T026–T027), `handle_save` (T024, T033), `handle_quit` (T033), `find_next`/`find_prev` (T055), `write_recovery`/autosave tick (T062), `handle_save_as` (T103), `handle_clipboard` (T102), `Buffer::open` calls (T023, T040); all must use `self.buffers[self.active_idx]` after this task
- [X] T066 [US6] Implement `next_buffer()` and `prev_buffer()` actions in `src/app.rs`: cycle `active_idx` with wrapping; update title bar to show active buffer's filename
- [X] T067 [US6] Implement split-view rendering in `src/ui/mod.rs`: `SplitMode::{Single, Vertical}` state; when `Vertical`, divide editor area into two equal panels; left panel renders `buffers[0]`, right panel `buffers[active_idx.max(1)]`; mouse click in a panel sets `active_idx`
- [X] T068 [US6] Update `src/ui/statusbar.rs` for multi-buffer: show `filename [N/M]` where N is active buffer index and M is total; disambiguate same-name files by showing parent directory
- [X] T069 [US6] Implement `OpenFileDialog` in `src/ui/dialog.rs`: single-line path input (no full file browser — per spec assumption); validate path via `security::sanitize::validate_path`; open as new buffer appended to `App.buffers`

**Checkpoint**: User Story 6 complete — multi-file editing independently functional.

---

## Phase 9: User Story 7 — Syntax Highlighting (Priority: P2)

**Goal**: Auto-detect file type by extension; apply keyword/string/comment highlighting for 5 languages; disableable.

**Independent Test**: Open `.py`, `.c`, `.sh`, `.yaml`, `.md` files — verify distinct colors for keywords, strings, comments; `--no-highlight` renders plain. See `quickstart.md` US7.

- [X] T070 [P] [US7] Define `Highlighter` trait in `src/highlight/mod.rs`: `fn highlight(line: &str) -> Vec<Span>`; `Span { start: usize, end: usize, style: Style }` (byte offsets within the line); `detect_highlighter(path: &Path) -> Option<Box<dyn Highlighter>>`
- [X] T071 [P] [US7] Implement C highlighter in `src/highlight/languages/c.rs`: keyword list (`int`, `void`, `if`, `for`, `while`, `return`, `struct`, `typedef`, …); string literal regex (`"[^"]*"`); comment regex (`//.*` and `/*…*/` spans); line-scoped regex scan returning `Vec<Span>`
- [X] T072 [P] [US7] Implement Python highlighter in `src/highlight/languages/python.rs`: keywords (`def`, `class`, `if`, `for`, `while`, `return`, `import`, `from`, `with`, `as`, …); string literals (`'...'`, `"..."`, triple-quoted detection at line level); comments (`#.*`)
- [X] T073 [P] [US7] Implement Shell highlighter in `src/highlight/languages/shell.rs`: keywords (`if`, `then`, `else`, `fi`, `for`, `do`, `done`, `while`, `function`, `case`, `esac`); string literals (`"..."`, `'...'`); comments (`#.*`); variables (`\$[A-Za-z_]+`)
- [X] T074 [P] [US7] Implement YAML highlighter in `src/highlight/languages/yaml.rs`: keys (text before `:`); string values; comments (`#.*`); `true`/`false`/`null` keywords; numeric values
- [X] T075 [P] [US7] Implement Markdown highlighter in `src/highlight/languages/markdown.rs`: headings (`^#{1,6}\s`); bold (`**…**`); italic (`*…*`); code spans (`` `…` ``); fenced code block indicators
- [X] T076 [US7] Wire highlighter into `src/ui/editor.rs`: on each line render, call `buffer.syntax.as_ref().map(|h| h.highlight(line_str))`, merge returned `Vec<Span>` into the ratatui `Line` cells with appropriate `theme` colors; cache spans per visible line, invalidate on edit
- [X] T077 [US7] Implement highlight toggle in `src/app.rs`: `toggle_highlight()` sets `buffer.syntax = None` (off) or re-detects via `detect_highlighter`; `--no-highlight` flag sets initial state to `None`; Options > Syntax Off menu item calls same action

**Checkpoint**: User Story 7 complete — syntax highlighting independently functional.

---

## Phase 10: User Story 8 — Configurable Keybindings and Themes (Priority: P2)

**Goal**: User config overrides keybindings and selects theme; conflicts warn; bad config falls back to defaults.

**Independent Test**: Add `"Ctrl+W" = "save"` to `config.toml`; restart; verify Ctrl+W saves and original Ctrl+S is still mapped (not removed). See `quickstart.md` US8.

- [X] T078 [P] [US8] Implement user keybinding override loading in `src/input/keymap.rs`: `apply_user_overrides(map, overrides)` — parse each `"Key" = "action"` pair; log `WARN` for unknown action names; log `WARN` for conflicts naming both the key and shadowed action; user entry wins
- [X] T079 [P] [US8] Implement `high-contrast` and `plain` theme palettes in `src/ui/theme.rs`: `HIGH_CONTRAST` uses black background, bright-white text, yellow keywords, green strings, grey comments; `PLAIN` inherits terminal defaults (no color override, use `Color::Reset`)
- [X] T080 [US8] Implement `--theme` CLI flag handling in `src/main.rs`: call `theme_by_name(flag)`, log `ERROR` and fall back to `CLASSIC` if unknown name; propagate active theme through `App` to all UI widgets
- [X] T081 [US8] Implement Options > Theme selection in `src/app.rs`: `set_theme(name: &str)` updates `App.theme` in-memory (takes effect immediately on next render); write selection back to `config.toml` for persistence
- [X] T082 [US8] Implement config file validation completeness in `src/config/mod.rs`: after `toml::from_str`, run `validate_config(c: &Config) -> Vec<ConfigError>` checking: `autosave_interval` clamped 10–300; `theme` in known list; `log_level` in known list; `default_encoding` in known list; log each error and apply default

**Checkpoint**: User Story 8 complete — configurable keybindings and themes independently functional.

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Observability, packaging, documentation, security hardening, CI, and performance.

- [X] T083 [P] Write `man/edit.1` groff man page: SYNOPSIS, DESCRIPTION, OPTIONS (all flags from `contracts/cli.md`), FILES (config/log/recovery paths), EXAMPLES (5+ from quickstart), SEE ALSO
- [X] T084 [P] Implement `--debug` verbose logging in `src/diagnostics/logging.rs`: when `--debug` set, override log level to `Debug`; log resolved locale, ncurses capabilities, active theme, config path, recovery dir path on startup
- [X] T085 [P] Configure `cargo-deb` in `Cargo.toml` `[package.metadata.deb]`: binary name, version, description, maintainer, license, man page path, config dir creation in `postinst`; `make package-deb` target
- [X] T086 [P] Write `packaging/edit.spec` RPM spec (hand-authored — intentional per research.md §10; `cargo-generate-rpm` not used because hand-authored spec provides finer control over `%post`/`%preun` scriptlets): Name/Version/Summary/License/BuildRequires; `%build` runs `cargo build --release`; `%install` copies binary + man page; `%files` lists `/usr/bin/edit`, `/usr/share/man/man1/edit.1.gz`; `make package-rpm` target
- [X] T087 [P] Run and validate `tests/smoke/basic_edit.exp` (file written in Phase 3 as T099); confirm exit 0 against the built binary; update script if UI changed since TDD authoring
- [X] T088 [P] Run and validate `tests/smoke/menu_nav.exp` (file written in Phase 5 as T110); confirm exit 0; update script if menu layout changed
- [X] T089 [P] Run and validate `tests/smoke/unicode_display.exp` (file written in Phase 4 as T108); confirm column assertions still match rendering
- [X] T090 [P] Run and validate `tests/smoke/search_replace.exp` (file written in Phase 6 as T112); confirm replacement count and file content assertions still pass
- [X] T091 Run and validate `tests/integration/file_io.rs` (file written in Phase 3 as T100); `cargo test --test file_io` must pass; add edge-case tests discovered during implementation
- [X] T092 Run and validate `tests/integration/recovery.rs` (file written in Phase 7 as T113); `cargo test --test recovery -- --nocapture` must pass with `EDIT_AUTOSAVE_INTERVAL=5` for fast CI runs
- [X] T093 [P] Run and validate `tests/integration/encoding_roundtrip.rs` (file written in Phase 4 as T109); `cargo test --test encoding_roundtrip` must pass for all 4 legacy encodings
- [X] T094 [P] Run and validate `tests/unit/security_sanitize.rs` (file written in Phase 3 as T101); `cargo test --lib security` must pass; add privilege-escalation assertion: confirm `Buffer::save` does not call `setuid`/`setgid`
- [X] T095 Configure CI matrix in `Makefile` `ci-local` target: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `make smoke`, `make perf-check`; document in `docs/STATUS.md`: (a) cross-platform CI steps for Linux ARM64 / macOS / FreeBSD, (b) the smoke-test runtime dependencies that MUST be installed on the CI image — `expect` and `tmux` — and (c) the minimum test-coverage standard (all FR-001–FR-029 have ≥ 1 automated test; the `make smoke` + `cargo test` suites collectively satisfy this — enforced by the per-phase TDD gates)
- [X] T096 [P] Configure musl static build in `Cargo.toml` and `Makefile`: add `[profile.release-static]` with `opt-level = 3`, `lto = true`, `strip = true`; `make static` runs `cargo build --target x86_64-unknown-linux-musl --profile release-static`
- [X] T097 Update `CHANGELOG.md`, `docs/STATUS.md`, `docs/CAPABILITIES.md` for v0.1.0: list all 25 FRs as implemented; list all 8 user stories as complete; list binary size, supported platforms, encoding support
- [X] T114 [P] Implement criterion benchmark suite in `benches/`: `benches/startup.rs` (measure cold-start time with `criterion::black_box`; assert ≤ 2 s), `benches/large_file.rs` (open 100 MB UTF-8 file; assert ≤ 3 s), `benches/keystroke.rs` (measure insert_char → render round-trip; assert ≤ 50 ms); add `make perf-check` target that runs `cargo bench` and extracts the three baselines
- [X] T115 [P] Implement `tests/integration/stress.rs` continuous-editing stress test (SC-008): parameterized loop duration via `EDIT_STRESS_DURATION_SECS` env var (default 259200 = 72h for manual; 300 for CI); insert/delete/search/undo operations in a loop; assert no allocation growth > 5 MB above baseline and no panics; add `make stress-test` Makefile target running with `EDIT_STRESS_DURATION_SECS=300`
- [X] T116 Create `ROADMAP.md` at repo root: add entry for **Plugin/Extension API** (deferred per constitution §VI; problem: no sandboxed plugin mechanism; why deferred: v1.x scope, no user story filed; suggested approach: WASI-based sandbox, opt-in user consent, requires separate spec; effort: L; issue: link to GitHub issue; label: `follow-up`); add entry for **External File Modification Detection** (deferred per constitution §VI; problem: no inotify/kqueue watcher; why deferred: complexity vs. user value; suggested approach: poll on focus regain; effort: S; label: `follow-up`)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories; T098 (`input/mod.rs`) must complete before T019/T020
- **US1 (Phase 3)**: Depends on Phase 2; TDD tasks T099–T101 must be written before T021–T034 + T102–T107
- **US2 (Phase 4)**: Depends on Phase 2; TDD tasks T108–T109 before T035–T040; parallel with US1
- **US3 (Phase 5)**: Depends on US1; TDD task T110 before T041–T051; T111 (mouse-cursor) after T051
- **US4 (Phase 6)**: Depends on US1; TDD task T112 before T052–T057
- **US5 (Phase 7)**: Depends on US1; TDD task T113 before T058–T064
- **US6 (Phase 8)**: Depends on US1 (T065 migration touches all US1 buffer call sites)
- **US7 (Phase 9)**: Depends on US1 (highlighter wired into editor widget)
- **US8 (Phase 10)**: Depends on Phase 2 config + US1 app bootstrap; T079 completes HIGH_CONTRAST + PLAIN (T016 only creates CLASSIC)
- **Polish (Phase N)**: Depends on all user stories complete; T114 benchmarks, T115 stress test, T116 ROADMAP

### User Story Dependencies

- **US1 (P1)**: Can start after Foundation — no story dependencies
- **US2 (P1)**: Can start after Foundation — parallel with US1 (different files: rope.rs, main.rs locale)
- **US3 (P1)**: Depends on US1 (menubar.rs renders over editor.rs widgets)
- **US4 (P1)**: Depends on US1 (SearchState operates on Buffer.rope)
- **US5 (P1)**: Depends on US1 (autosave wraps Buffer open/save)
- **US6 (P2)**: Depends on US1 (extends App.buffer to Vec<Buffer>)
- **US7 (P2)**: Depends on US1 (Highlighter trait plugs into editor.rs render)
- **US8 (P2)**: Depends on Phase 2 config + US1 app (theme wires through App)

### Within Each User Story

- Foundation → buffer models → UI widgets → event wiring → actions → integration
- [P] tasks within a story can run in parallel on different files
- Commit after each independently testable checkpoint

### Parallel Opportunities

```bash
# Phase 1+2 can batch [P] tasks:
# T003, T004, T005 in parallel
# T006, T007, T008, T009, T011, T012, T016, T018 in parallel (Phase 2)

# US1 + US2 run concurrently after Foundation:
# US1: T021→T022→T023→T024 (buffer structs) in parallel with US2: T035, T036

# US7 language highlighters all run in parallel:
# T071, T072, T073, T074, T075 in parallel
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL)
3. Complete Phase 3: US1 → Basic File Editing ← **first shippable binary**
4. Complete Phase 4: US2 → UTF-8 correctness
5. **STOP and VALIDATE**: open multilingual file, edit, save, verify
6. Run `cargo test && make smoke` → all P1 US1+US2 tests green

### Incremental Delivery (Recommended)

1. Setup + Foundation → working skeleton
2. US1 → MVP binary (open/edit/save/quit)
3. US2 → Unicode-correct binary (core project identity)
4. US3 → DOS-faithful UI (menus, F-keys)
5. US4 → Search and replace
6. US5 → Auto-save + recovery
7. US6 → Multi-file → Demo/release candidate
8. US7 → Syntax highlighting → Feature-complete
9. US8 → Customization → Polish → v1.0.0

### Parallel Team Strategy

- Developer A: US1 (buffer, editor widget, save/quit) + US5 (autosave)
- Developer B: US2 (unicode) + US4 (search)
- Developer C: US3 (menus, F-keys) + US6 (multi-buffer)
- Developer D: US7 (syntax highlighting languages in parallel) + US8 (config)

---

## Notes

- `[P]` = safe to run in parallel with other `[P]` tasks in the same phase
- Story label `[USN]` maps to User Story N in `spec.md`
- Each story phase ends with a **Checkpoint** — validate independently before next story
- **TDD gate**: each story phase opens with test-writing tasks (T099–T101, T108–T110, T112–T113); these must be committed before any implementation in that phase
- Recovery integration test (T092/T113): use `EDIT_AUTOSAVE_INTERVAL=5` env var for fast CI runs; full 35 s in integration, 72 h via `make stress-test` with `EDIT_STRESS_DURATION_SECS=259200`
- Musl static build (T096) requires `cross` or `musl-cross` toolchain — see `docs/STATUS.md`
- SC-001 (first-use task completion in 60 s) is **manual validation only** — no automated test is feasible; see `quickstart.md` US1 Scenario A for the procedure
- Clipboard support (T102) uses `arboard` crate; on headless/SSH systems without a display, `arboard` may fail — `paste_clipboard()` should degrade gracefully with a status-bar warning, not a panic
- Total tasks: 116 (T001–T097 original + T098–T116 analysis remediation)
