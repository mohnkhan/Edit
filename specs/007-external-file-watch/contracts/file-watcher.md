# Behavioral Contract: `FileWatcher`

## Overview

`FileWatcher` is a thin wrapper around a `notify` filesystem watcher. It delivers normalized, debounced `WatchEvent`s to the editor's main event loop via a non-blocking drain method. It is owned by `App` as `Option<FileWatcher>`.

---

## Terminology

| Term | Definition |
|---|---|
| **Backing file** | The on-disk file associated with a `Buffer`; `buffer.path = Some(p)` |
| **Parent directory** | The immediate parent of a backing file, watched with `RecursiveMode::NonRecursive` |
| **Self-write** | A write initiated by the editor itself (auto-save, `Ctrl+S`, Save As) |
| **External change** | Any file-system modification whose originator is NOT the editor (other processes, shells, build tools) |
| **Debounce window** | 1-second minimum between consecutive `WatchEvent` emissions for the same path |
| **Grace window** | 2-second suppression window applied to self-writes to prevent false reload prompts |

---

## Contract: `FileWatcher::new()`

**Signature**: `fn new() -> Result<FileWatcher, notify::Error>`

**Pre-conditions**: None (called at App startup).

**Post-conditions**:
- Returns `Ok(fw)` where `fw.watched_dirs` is empty and no paths are being watched.
- Returns `Err(e)` only if the OS fails to create the inotify/kqueue/FSEvents handle.

**Invariants**:
- `watched_dirs` refcounts are always ≥ 1 for any directory currently being watched at the OS level.
- `last_emitted` contains only entries for paths that have had at least one event emitted since the `FileWatcher` was created.

---

## Contract: `FileWatcher::watch_path(path)`

**Signature**: `fn watch_path(&mut self, path: &Path) -> Result<(), notify::Error>`

**Pre-conditions**: `path` is an absolute path to a regular file (not a directory, symlink, or pseudo-file).

**Post-conditions**:
- `path.parent()` is registered with the OS watcher (if not already) with `RecursiveMode::NonRecursive`.
- `watched_dirs[path.parent()]` is incremented by 1.
- Subsequent external modifications to `path` will eventually appear in `try_recv_event()`.

**Edge cases**:
- If `path.parent()` is already watched (from another buffer in the same directory), only the refcount is incremented — no duplicate OS-level watch is registered.
- If `path` has no parent (root path), the call is a no-op and returns `Ok(())`.
- Paths under pseudo-filesystems (`/proc`, `/sys`) are silently accepted; no events will arrive from them.

---

## Contract: `FileWatcher::unwatch_path(path)`

**Signature**: `fn unwatch_path(&mut self, path: &Path) -> Result<(), notify::Error>`

**Pre-conditions**: `path` was previously passed to `watch_path`.

**Post-conditions**:
- `watched_dirs[path.parent()]` is decremented by 1.
- If the count reaches 0, `path.parent()` is unregistered from the OS watcher.
- After successful unwatch (count reaches 0), no further events for files in `path.parent()` are delivered.

**Edge cases**:
- Calling `unwatch_path` for a path that was never watched is a no-op.

---

## Contract: `FileWatcher::try_recv_event()`

**Signature**: `fn try_recv_event(&mut self, self_write_times: &HashMap<PathBuf, Instant>) -> Option<WatchEvent>`

**Pre-conditions**: Called from the main event loop thread only (not from a background thread).

**Post-conditions (event returned)**:
- The returned `WatchEvent.path` belongs to a set of currently-watched paths.
- The returned event is NOT a self-write: for the event's path `p`, either `self_write_times[p]` is absent, or `self_write_times[p].elapsed() >= SELF_WRITE_GRACE` (2 seconds).
- The returned event respects the debounce window: `last_emitted[p].elapsed() >= DEBOUNCE_SECS` (1 second) since the previous emission for the same path.
- After returning `Some(event)`, `last_emitted[event.path] = Instant::now()`.

**Post-conditions (None returned)**:
- The OS-event queue was empty, or all pending events were either: self-writes (suppressed), debounced (too recent), or for unrelated paths.
- Calling `try_recv_event` is non-blocking in all cases.

**Guarantee**: A single call drains at most one event from the raw queue (to keep the main loop responsive). If multiple events are pending, subsequent calls will drain them on future ticks.

---

## Contract: Reload Dialog Behavior (App-level)

When `App::pending_external_change = Some(ExternalChange { buf_idx, path, kind: Modified })`:

- `Ui::render()` draws the `ExternalChangeDialog` as a modal overlay (blocking other input).
- The dialog shows the filename and two choices: `[Y] Reload` / `[N] Keep`.
- If the buffer has unsaved changes (`buffer.modified() == true`), the dialog additionally warns that unsaved edits will be lost.

**On Y / Enter**:
- `App::reload_from_disk(buf_idx)` is called.
- `Buffer::open(path)` re-reads and re-encodes the file through the full encoding pipeline.
- The old buffer is replaced; undo history is cleared.
- `pending_external_change = None`.

**On N / Esc**:
- Buffer is not reloaded.
- `buffer.dirty = true` (marked as modified / unsaved) even if no edits were made.
- `pending_external_change = None`.

---

## Anti-Regression Guarantees

- A self-write from `Ctrl+S` or auto-save MUST NOT produce a `WatchEvent` that reaches `pending_external_change`. Verified by unit test `test_self_write_suppressed`.
- Ten rapid external writes within 1 second MUST produce exactly 1 `WatchEvent`. Verified by unit test `test_debounce_coalesces`.
- With `config.no_watch = true`, `App.file_watcher` MUST be `None` and `try_recv_event` is never called. Verified by unit test `test_no_watch_flag`.
