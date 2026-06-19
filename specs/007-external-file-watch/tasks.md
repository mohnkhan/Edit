# Tasks: External File Modification Detection

**Feature**: 007 | **Branch**: `007-external-file-watch`

**Input**: Design documents from `specs/007-external-file-watch/`

**Source references**: plan.md (Phases A–F), spec.md (US1–US4), data-model.md, contracts/file-watcher.md, research.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel with other [P]-marked tasks (different files, no shared state)
- **[Story]**: Maps to user story in spec.md (US1/US2/US3/US4)

---

## Phase 1: Setup

**Purpose**: Create the git branch and add the one new crate dependency.

- [X] T001 Create branch `007-external-file-watch` from `origin/master` with `git checkout -b 007-external-file-watch origin/master`
- [X] T002 Add `notify = "6"` dependency to `Cargo.toml` `[dependencies]` section; run `cargo fetch` to verify resolution
- [X] T003 Register new integration test in `Cargo.toml`: add `[[test]] name = "file_watch" path = "tests/integration/file_watch.rs"`
- [X] T004 Add `pub mod watcher;` to `src/lib.rs` (after the existing module declarations)

**Checkpoint**: `cargo build` passes with the notify crate pulled in. `src/lib.rs` compiles after T004 only after `src/watcher/mod.rs` is created (T005 onward).

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: New `watcher` module, Config/CLI changes, and new `App` fields. ALL of these MUST be complete before any user-story phase can be compiled or tested.

**⚠️ CRITICAL**: No user-story work (Phases 3–6) can begin until this phase is complete.

### Watcher Module

- [X] T005 Create `src/watcher/mod.rs`: define `WatchEventKind` enum (`Modified`, `Deleted`) and `WatchEvent` struct (`path: PathBuf, kind: WatchEventKind`); define `ExternalChange` struct (`buf_idx: usize, path: PathBuf, kind: WatchEventKind`)
- [X] T006 Implement `FileWatcher` struct in `src/watcher/mod.rs`: fields `_watcher: Box<dyn notify::Watcher + Send>`, `rx: std::sync::mpsc::Receiver<notify::Result<notify::Event>>`, `watched_dirs: HashMap<PathBuf, usize>`, `last_emitted: HashMap<PathBuf, Instant>`; constants `SELF_WRITE_GRACE: Duration = Duration::from_secs(2)`, `DEBOUNCE_SECS: Duration = Duration::from_secs(1)`
- [X] T007 Implement `FileWatcher::new() -> Result<Self, notify::Error>` in `src/watcher/mod.rs`: create `std::sync::mpsc::channel()`, pass `tx` to `notify::recommended_watcher(tx)`, return `Self { _watcher, rx, watched_dirs: HashMap::new(), last_emitted: HashMap::new() }`
- [X] T008 Implement `FileWatcher::watch_path(path: &Path) -> Result<(), notify::Error>` in `src/watcher/mod.rs`: get `dir = path.parent()`; if `watched_dirs[dir] == 0`, call `_watcher.watch(dir, RecursiveMode::NonRecursive)`; increment `watched_dirs[dir]`
- [X] T009 Implement `FileWatcher::unwatch_path(path: &Path) -> Result<(), notify::Error>` in `src/watcher/mod.rs`: get `dir = path.parent()`; decrement `watched_dirs[dir]`; if count reaches 0, call `_watcher.unwatch(dir)` and remove entry
- [X] T010 Implement `FileWatcher::try_recv_event(self_write_times: &HashMap<PathBuf, Instant>) -> Option<WatchEvent>` in `src/watcher/mod.rs`: call `rx.try_recv()` in a loop; for each raw notify event: (a) filter to only paths present in `watched_dirs` as a parent of any key, (b) skip if `self_write_times[path].elapsed() < SELF_WRITE_GRACE`, (c) skip if `last_emitted[path].elapsed() < DEBOUNCE_SECS`, (d) map `EventKind::Modify | EventKind::Create` to `WatchEventKind::Modified` and `EventKind::Remove` to `WatchEventKind::Deleted`, (e) update `last_emitted[path] = Instant::now()` and return `Some(WatchEvent)`; return `None` when queue empty or all events filtered
- [X] T011 Add unit tests in `src/watcher/mod.rs` `#[cfg(test)] mod tests`: write `test_watch_unwatch_refcount` (watch same dir twice, assert dir watch count correct; unwatch once, dir still watched; unwatch twice, dir unwatched), `test_two_buffers_same_file_single_watch` (call `watch_path` twice with paths sharing the same parent dir; assert `watched_dirs[dir] == 2` with only 1 OS-level watch registered; call `unwatch_path` twice; assert dir unwatched — verifies FR-011), `test_self_write_suppressed` (manually insert `Instant::now()` into `self_write_times`, assert `try_recv_event` returns `None` for simulated event on that path within grace window), `test_debounce_coalesces` (manually set `last_emitted[path] = Instant::now()`, assert second event suppressed; after 1.1s elapsed, assert event passes), `test_unknown_path_ignored` (simulate notify event for path not in `watched_dirs`, assert `None` returned)

### Config + CLI

- [X] T012 Add `no_watch: bool` field (default `false`) to `Config` struct in `src/config/schema.rs`; ensure TOML deserialization defaults to `false` when key is absent
- [X] T013 Add `--no-watch` CLI flag in `src/main.rs` `build_cli()` function: `Arg::new("no-watch").long("no-watch").action(ArgAction::SetTrue).help("Disable external file modification watching")`; in `merge_cli_flags()`, set `config.no_watch = true` if the flag is present

### App New Fields

- [X] T014 Add four new fields to `App` struct in `src/app.rs`:
  - `pub file_watcher: Option<crate::watcher::FileWatcher>` — `None` when `config.no_watch`
  - `pub self_write_times: std::collections::HashMap<std::path::PathBuf, std::time::Instant>` — tracks editor-initiated writes
  - `pub pending_external_change: Option<crate::watcher::ExternalChange>` — set when watcher fires; cleared by user response
  - `pub watcher_notice: Option<String>` — one-shot deletion/notice status bar message
- [X] T015 Initialize new `App` fields in `App::new()` in `src/app.rs`: if `!config.no_watch`, attempt `FileWatcher::new()`; on success, call `watch_path` for each initially-opened buffer that has `path: Some(p)`; on failure (e.g., inotify watch limit ENOSPC), log warning at `warn` level AND set `app.watcher_notice = Some("⚠ File watching unavailable — changes won't be detected")` so the user sees a one-time status-bar notice on first render; set `file_watcher = None`; initialize `self_write_times = HashMap::new()`, `pending_external_change = None`, `watcher_notice = None`

- [X] T045 Add structured log entries in `src/watcher/mod.rs` using the project's existing log facade (e.g., `log::info!`, `log::warn!`): log watcher init success/failure at `info`/`warn`, log each suppressed self-write event at `debug`, log each emitted WatchEvent at `debug`, log notify errors at `warn`; satisfies constitution Observability requirement (structured log to `$XDG_STATE_HOME/edit/logs`)

**Checkpoint**: `cargo build` compiles cleanly. `cargo test --lib` passes all existing tests (no regressions). The new module compiles even though no watcher events are yet handled by the event loop.

---

## Phase 3: User Story 1 — Detect External Change and Prompt to Reload (Priority: P1) 🎯 MVP

**Goal**: When a backing file is overwritten by an external process, the editor detects the change within 5 seconds and shows a modal "File changed on disk. Reload? [Y/n]" dialog.

**Independent Test**: `./target/debug/edit /tmp/watch_test.txt` → from another shell: `echo "new" > /tmp/watch_test.txt` → dialog appears within 5s. Press Y → buffer updated. See quickstart.md Scenario 1.

### Implementation for User Story 1

- [X] T016 [US1] Drain watcher events in `App::handle_tick()` in `src/app.rs`: after existing auto-save tick, if `self.file_watcher.is_some()` and `self.pending_external_change.is_none()`, call `fw.try_recv_event(&self.self_write_times)`; on `Some(WatchEvent { kind: Modified, path })`, find the buffer whose `path` matches, set `self.pending_external_change = Some(ExternalChange { buf_idx, path, kind: Modified })`
- [X] T017 [US1] Record self-write timestamps in `src/app.rs`: in all call sites that invoke `buffer.save()` or `buffer.write_to()` (Ctrl+S handler, auto-save in `handle_tick()`, Save As handler), after a successful write, insert `self.self_write_times.insert(path.clone(), Instant::now())`
- [X] T018 [US1] Add `Action::ReloadFile` and `Action::DismissExternalChange` variants to `Action` enum in `src/input/keymap.rs`
- [X] T019 [US1] Handle `Action::ReloadFile` in `App::handle_action()` in `src/app.rs`: if `pending_external_change` is `Some(ec)`, call `self.reload_from_disk(ec.buf_idx)`, then set `pending_external_change = None`
- [X] T020 [US1] Handle `Action::DismissExternalChange` in `App::handle_action()` in `src/app.rs`: if `pending_external_change` is `Some(ec)`, mark `self.buffers[ec.buf_idx].dirty = true`, set `pending_external_change = None`
- [X] T021 [US1] Implement `App::reload_from_disk(buf_idx: usize)` in `src/app.rs`: get `path` from `self.buffers[buf_idx].path.clone()`; call `Buffer::open(path)` with the existing encoding; on success replace `self.buffers[buf_idx]`; on error log warning and leave buffer unchanged
- [X] T022 [US1] Render `ExternalChangeDialog` overlay in `Ui::render()` in `src/ui/mod.rs`: when `app.pending_external_change.is_some()`, draw a centered 60×5 `Paragraph` block with title "File Changed on Disk", body "  [filename] was modified externally.\n  [Y] Reload from disk   [N] Keep in-editor version", style using `app.theme.menubar_bg`/`menubar_fg`; dispatch `Y`/`Enter` → `Action::ReloadFile`, `N`/`Esc` → `Action::DismissExternalChange` via `dispatch_event` (or handle directly in `handle_action` when dialog active — mirror the session-restore dialog pattern in `src/app.rs:handle_action()`)
- [X] T023 [US1] Watch/unwatch path when buffer is opened or closed in `src/app.rs`: in the "open new buffer" code path, call `file_watcher.watch_path(path)` if the buffer has `Some(path)`; in the "close buffer" code path, call `file_watcher.unwatch_path(path)`
- [X] T024 [P] [US1] Write integration test `test_external_write_triggers_event` in `tests/integration/file_watch.rs`: create a temp file; open a `Buffer` and register path with a `FileWatcher`; write to the temp file from the test; poll `try_recv_event` for up to 3s in a loop; assert `WatchEvent::Modified` received
- [X] T025 [P] [US1] Write integration test `test_atomic_rename_detected` in `tests/integration/file_watch.rs`: same as T024 but use `mv tmp dest` (atomic rename) pattern; assert `WatchEvent::Modified` still fires on the destination path
- [X] T026 [P] [US1] Write unit tests in `src/app.rs` for reload path: (a) `test_reload_replaces_buffer_content` — create App with a temp file; set `pending_external_change`; call `handle_action(Action::ReloadFile)`; assert `buffers[buf_idx].as_bytes() == fs::read(&path).unwrap()` (byte-level comparison per SC-002); assert `undo_history.is_empty()` (FR-004 clears undo); assert `pending_external_change == None`; (b) `test_reload_binary_file_shows_encoding_error` — write a binary (non-UTF-8) temp file; trigger reload; assert buffer unchanged and an error/notice is surfaced rather than corrupting the buffer (validates FR-004 encoding pipeline requirement)
- [X] T027 [P] [US1] Write unit test `test_dismiss_marks_buffer_dirty` in `src/app.rs`: set `pending_external_change`; call `handle_action(Action::DismissExternalChange)`; assert `buffers[buf_idx].dirty == true`; assert `pending_external_change == None`

**Checkpoint**: `cargo test --lib` + `cargo test --test file_watch test_external_write` — all pass. Manual quickstart.md Scenario 1 verified.

---

## Phase 4: User Story 2 — Reload with Unsaved Changes Warning (Priority: P2)

**Goal**: When the buffer has unsaved edits at the time of external modification, the reload dialog includes the line "You have unsaved changes." so the user explicitly knows reloading will discard their work.

**Independent Test**: Make edits in a buffer (don't save), overwrite file from shell, observe the warning line in the dialog.

### Implementation for User Story 2

- [X] T028 [US2] Modify `ExternalChangeDialog` rendering in `src/ui/mod.rs`: before drawing, check `app.buffers[ec.buf_idx].dirty`; if dirty, add the warning line "  You have unsaved changes." between the filename line and the [Y]/[N] line
- [X] T029 [P] [US2] Write unit test `test_external_change_dialog_shows_unsaved_warning` in `src/ui/mod.rs` or `src/app.rs`: render App with `pending_external_change` set and `buffer.dirty = true` into a ratatui `Buffer::empty`; assert rendered text contains "unsaved changes"
- [X] T030 [P] [US2] Write integration test `test_unsaved_changes_discarded_on_reload` in `tests/integration/file_watch.rs`: modify buffer content without saving; trigger external write; choose reload; assert buffer content matches disk and previous edits are gone

**Checkpoint**: `cargo test --lib` all pass. Manual quickstart.md Scenario 3 verified.

---

## Phase 5: User Story 3 — File Deleted While Open (Priority: P3)

**Goal**: When a backing file is deleted externally, the editor shows a non-modal status-bar notice and keeps the buffer editable in memory.

**Independent Test**: `rm /tmp/watch_del.txt` while editor has it open → status bar shows deletion notice; no modal dialog; buffer remains editable; `Ctrl+S` recreates the file.

### Implementation for User Story 3

- [X] T031 [US3] Handle `WatchEvent { kind: Deleted, path }` in `App::handle_tick()` drain in `src/app.rs`: when event kind is `Deleted`, do NOT set `pending_external_change`; instead set `self.watcher_notice = Some(format!("[{}] File deleted on disk — buffer kept in memory", path.file_name()...))`
- [X] T032 [US3] Display `watcher_notice` via status bar in `src/ui/mod.rs` and `src/ui/statusbar.rs`: pass `app.watcher_notice.as_deref()` as an optional prefix to `StatusBar::new()` parameters in `src/ui/statusbar.rs`; if present, prepend the notice text to the status bar display; after calling the StatusBar component in `Ui::render()` (in `src/ui/mod.rs`), clear `app.watcher_notice = None` so the notice shows for exactly one render frame — the clear happens in `mod.rs`, not inside `statusbar.rs`
- [X] T033 [P] [US3] Write integration test `test_delete_produces_notice_not_dialog` in `tests/integration/file_watch.rs`: delete temp file; poll for 3s; assert `watcher_notice.is_some()` and `pending_external_change.is_none()`
- [X] T034 [P] [US3] Write integration tests for deleted-file scenarios in `tests/integration/file_watch.rs`: (a) `test_deleted_file_save_recreates` — delete temp file; call `buffer.save()`; assert file exists on disk again; (b) `test_deleted_file_close_without_save_prompts` — delete temp file, mark buffer modified, then simulate close-buffer; assert close prompt is shown (validates US3 Acceptance Scenario 3: deleted buffer behaves identically to any other unsaved buffer)

**Checkpoint**: `cargo test --test file_watch test_delete` passes. Manual quickstart.md Scenario 4 verified.

---

## Phase 6: User Story 4 — Disable Watching via CLI Flag (Priority: P3)

**Goal**: `edit --no-watch` suppresses all file watching; no dialogs, no notices, regardless of external activity.

**Note**: The `no_watch` config field and `--no-watch` CLI flag were already added in Phase 2 (Foundational — T012/T013). The App initialization check `!config.no_watch` in T015 already ensures `file_watcher = None`. This phase adds only the verification tests.

**Independent Test**: `./target/debug/edit --no-watch /tmp/nowatch.txt` → overwrite from shell → no dialog.

### Implementation for User Story 4

- [X] T035 [US4] Write unit test `test_no_watch_config_leaves_watcher_none` in `src/app.rs`: construct `App` with `config.no_watch = true`; assert `app.file_watcher.is_none()`
- [X] T036 [P] [US4] Write integration test `test_no_watch_no_events` in `tests/integration/file_watch.rs`: create `FileWatcher` simulation with `no_watch=true` (or test at App level with config); assert `pending_external_change` remains `None` after external write

**Checkpoint**: `cargo test --lib app::tests::test_no_watch_config_leaves_watcher_none` passes. Manual quickstart.md Scenario 5 verified.

---

## Phase 7: Advanced Integration Tests & Docs Gate

**Purpose**: Additional edge-case coverage and documentation updates required before PR merge.

- [X] T037 [P] Write integration test `test_self_write_suppressed_no_prompt` in `tests/integration/file_watch.rs`: call `buffer.save()` (which records `self_write_times`); immediately poll `try_recv_event`; assert `None` returned (self-write grace window suppresses the inotify event)
- [X] T046 [P] Write integration test `test_same_file_two_buffers_single_watch` in `tests/integration/file_watch.rs`: create two `Buffer` instances pointing to the same temp file; call `file_watcher.watch_path(path)` twice; assert `file_watcher.watched_dirs[dir] == 2`; write to the temp file; poll `try_recv_event` twice; assert exactly 1 `WatchEvent::Modified` received (not 2); call `unwatch_path` twice; assert dir no longer watched — validates FR-011 with real inotify events
- [X] T047 Verify that `Buffer::open()` rejects binary/non-UTF-8 files in `src/` before implementing T026b: run `cargo test --lib -- buffer::tests` and confirm a test covering binary-file rejection already exists; if no such test exists, add `test_open_binary_file_returns_error` to the buffer unit tests as a prerequisite (validates the pre-existing behavior that FR-004 / T026b depend on)
- [X] T038 [P] Write integration test `test_debounce_10_writes_1_event` in `tests/integration/file_watch.rs`: write to temp file 10 times in a tight loop (≤500ms total); poll `try_recv_event` for 2s total; collect all `WatchEvent`s; assert exactly 1 event received
- [X] T039 [P] Update `CHANGELOG.md` — add "feature 007: External File Modification Detection" section above feature 006 entry with Added/Changed subsections covering US1–US4
- [X] T040 [P] Update `docs/STATUS.md` — add rows F007-US1, F007-US2, F007-US3, F007-US4 (all Complete) before F006 rows
- [X] T041 [P] Update `docs/CAPABILITIES.md` — add `--no-watch` to CLI flags table; add "External file modification detection (inotify/kqueue/FSEvents; `--no-watch` to disable)" under Features section
- [X] T042 Update `ROADMAP.md` — mark "External File Modification Detection" (Issue #3) as Shipped 2026-06-19
- [X] T043 Run `make ci-local` — confirm format → lint → unit tests → smoke → perf-check → docs-gate all pass; fix any failures before opening PR
- [X] T044 Run quickstart.md Scenarios 1–6 manually (or headless) and document pass/fail in PR body; include Principle VII self-certification in PR description confirming: (a) file reload uses `Buffer::open()` — no raw-byte bypass, (b) `--no-watch` CLI input is sanitized by clap, (c) `ExternalChangeDialog` text is static (no user-file-content rendered in dialog body)

**Checkpoint**: Full CI green. PR ready.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1; BLOCKS all user-story phases
- **Phase 3 (US1)**: Depends on Phase 2 — core detection + dialog
- **Phase 4 (US2)**: Depends on Phase 3 (extends the dialog from US1)
- **Phase 5 (US3)**: Depends on Phase 2; can run in parallel with Phase 4
- **Phase 6 (US4)**: Depends on Phase 2 (config + CLI already done); verification only
- **Phase 7 (Docs + CI)**: Depends on Phases 3–6 all complete

### User Story Dependencies

- **US1 (P1)**: Foundational complete → implement detection + dialog → verify with shell overwrite
- **US2 (P2)**: US1 complete (extends the dialog) → add dirty-buffer warning
- **US3 (P3)**: Foundational complete → independent of US1/US2 (different event kind, different UI path)
- **US4 (P3)**: Foundational complete (flag added in Phase 2) → only tests remain

### Sequential Within Phases

- T005 → T006 → T007 → T008 (FileWatcher methods build on struct)
- T010 → T011 → T012 → T013 → T014 → T015 → T016 (Config → CLI → App fields → init → drain → dispatch → UI)
- T018 → T019 → T020 → T021 → T022 → T023 (Action enum → handlers → reload impl → UI)
- T016 → T031 (both edit `handle_tick()` drain; T031 extends T016's Modified branch to also handle Deleted — must be sequential)

### Parallel Opportunities

- T024, T025, T026, T027 (Phase 3 tests) are all [P] — different functions, no shared mutable state
- T029, T030 (Phase 4 tests) are [P]
- T033, T034 (Phase 5 tests) are [P]
- T036 (Phase 6 test) is [P]
- T037, T038 (Phase 7 edge-case tests) are [P]
- T039, T040, T041 (docs) are [P] — different files

---

## Implementation Strategy

### MVP First (US1 Only — ~4–6 hours)

1. Phase 1: Setup (T001–T004)
2. Phase 2: Foundational (T005–T015)
3. Phase 3: US1 detection + dialog (T016–T027)
4. **STOP**: `cargo test` passes; manual quickstart Scenario 1 confirms prompt appears. US1 done and independently deliverable.

### Full Delivery

5. Phase 4: US2 dirty-buffer warning (T028–T030)
6. Phase 5: US3 deletion notice (T031–T034)
7. Phase 6: US4 verification tests (T035–T036)
8. Phase 7: Edge-case tests + docs + CI (T037–T044)
9. Open PR

### Total Task Count: 47 tasks across 7 phases

| Phase | Tasks | Story |
|---|---|---|
| Setup | T001–T004 | — |
| Foundational | T005–T015, T045 | — |
| US1 (P1 MVP) | T016–T027 | US1 |
| US2 (P2) | T028–T030 | US2 |
| US3 (P3) | T031–T034 | US3 |
| US4 (P3) | T035–T036 | US4 |
| Docs + CI | T037–T044, T046, T047 | — |

---

## Notes

- New source file: `src/watcher/mod.rs` (new module)
- New test file: `tests/integration/file_watch.rs`
- Modified source files: `src/app.rs`, `src/config/schema.rs`, `src/main.rs`, `src/lib.rs`, `src/ui/mod.rs`, `src/ui/statusbar.rs`, `src/input/keymap.rs`
- Modified config: `Cargo.toml` (one new dependency: `notify = "6"`)
- The `--no-watch` flag and `no_watch` config field default to false; watching is opt-out, not opt-in
- Integration tests write real temp files on the local filesystem and require a working inotify watch limit (`fs.inotify.max_user_watches`); if running in a heavily sandboxed CI, `test_*` may fall back to the `PollWatcher` automatically via `notify::recommended_watcher`
