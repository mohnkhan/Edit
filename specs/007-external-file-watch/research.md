# Research: External File Modification Detection (Feature 007)

## Decision 1: Filesystem Watch Crate — `notify` v6.x

**Decision**: Use the `notify` crate (v6.x; API-compatible with v8.x) as the sole filesystem watcher dependency.

**Rationale**:
- Cross-platform: `recommended_watcher` selects inotify on Linux, kqueue on FreeBSD/macOS, FSEvents on macOS automatically — matching the constitution's Principle III target platforms.
- Zero-cost fallback: `notify::PollWatcher` is a drop-in replacement via the `Watcher` trait; activated automatically on filesystems without native event support.
- Minimal: one crate, no tokio/async runtime dependency. Aligns with Principle IV (minimal footprint).
- Well-maintained: actively used by cargo-watch, watchexec, and other major Rust tooling.

**Alternatives considered**:
- `inotify` crate (Linux-only): rejected — violates Principle III (must build on BSD/macOS).
- `kqueue` crate (BSD/macOS only): rejected — same reason.
- Manual `inotify` syscall via libc: rejected — fragile, platform-specific, not worth the maintenance burden.
- `notify-debouncer-mini` / `notify-debouncer-full`: not needed — we implement our own 1-second debounce inside `FileWatcher` using `HashMap<PathBuf, Instant>`. This avoids an extra crate (Principle IV).

---

## Decision 2: Integration Pattern — Non-Blocking `mpsc` Channel Drain

**Decision**: Bridge notify events to the editor's main event loop via `std::sync::mpsc::channel`. The watcher callback sends events into `tx`; the main loop calls `rx.try_recv()` inside `handle_tick()`.

**Rationale**:
- `crossterm::event::poll(timeout)` already blocks up to `TICK_MS` (50ms). Draining `rx` with `try_recv()` after each poll cycle adds zero latency overhead.
- No second thread is needed for the drain; notify creates its own internal OS-event thread automatically.
- `try_recv()` returns `Err(TryRecvError::Empty)` immediately if no events are queued — no blocking, no spin-loop.

**Pitfall avoided**: Never call `rx.recv()` on the main thread — it would block the render loop.

---

## Decision 3: Watch Parent Directory, Not the File Directly

**Decision**: For each open buffer whose path is `Some(p)`, watch `p.parent()` (the containing directory) with `RecursiveMode::NonRecursive` rather than the file itself.

**Rationale**:
- Most editors and build tools save atomically: `write(tmp) → rename(tmp, dest)`. inotify fires `RenameMode::To` on the destination directory, **not** a file-level `Modify`. Watching only the file inode misses these events.
- On macOS (FSEvents), both the from and to sides of a rename emit `RenameMode::Any` on the parent directory; file-inode watching is unreliable.
- Watching the directory also catches the case where a file is replaced by a newly-created file with the same name (e.g., `cp new old`).

**Filtering**: Only events whose `event.paths` contain the exact buffer path are forwarded. All other directory entries are silently ignored.

**Pitfall noted**: Multiple buffers in the same directory → one directory watch covers all. Track a `HashMap<PathBuf, usize>` refcount to unwatch the directory only when the last buffer from that directory is closed.

---

## Decision 4: Self-Write Suppression — Grace-Window Timestamp

**Decision**: Track `self_write_times: HashMap<PathBuf, Instant>` in `App`. Any time the editor writes to a path (auto-save, `Ctrl+S`, Save As), record `self_write_times[path] = Instant::now()`. When a watcher event arrives for a path, suppress it if `self_write_times[path].elapsed() < SELF_WRITE_GRACE = Duration::from_secs(2)`.

**Rationale**:
- `Buffer::write_to()` uses atomic rename (`tmp → dest`). The inotify `RenameMode::To` event for `dest` arrives within milliseconds. A 2-second grace window (much larger than needed) ensures reliable suppression even on slow filesystems.
- No need to hash file contents or compare mtimes — the timestamp approach is simpler and sufficient.

**Pitfall avoided**: A 2-second window is short enough that a genuine external change immediately after an editor save still fires within a few seconds.

---

## Decision 5: Debounce — 1-Second Coalescing Window

**Decision**: In `FileWatcher::try_recv_event()`, maintain `last_emitted: HashMap<PathBuf, Instant>`. Only emit a `WatchEvent` for a given path if `last_emitted[path].elapsed() > Duration::from_secs(1)`. This satisfies FR-008 (10 rapid writes → 1 prompt).

**Rationale**:
- Build tools (cargo, make) often trigger multiple rapid writes to the same file. Without debouncing, the user would see repeated prompts.
- 1 second is the minimum; the spec requires debounce within a 1-second window (FR-008).

---

## Decision 6: `--no-watch` CLI Flag and `no_watch` Config Field

**Decision**: Add `no_watch: bool` to `Config` (default `false`). Add `--no-watch` CLI flag that sets `config.no_watch = true`. When true, `FileWatcher` is never created and `app.file_watcher = None`.

**Rationale**: FR-009 requires this escape hatch. Implementing it as a `Config` field (rather than a runtime toggle) keeps the logic simple — the watcher is either started at launch or not.

---

## Decision 7: Reload Clears Undo History

**Decision**: `App::reload_from_disk()` replaces the buffer content by creating a new `Buffer::open()` call for the same path (reusing the existing encoding detection). The undo history is implicitly cleared because the `Buffer` struct is replaced.

**Rationale**: After a full external replacement, the undo history is meaningless (it references character offsets in the old content). Clearing it is the correct behavior and matches the behavior of most text editors. The spec documents this assumption explicitly.

---

## Decision 8: Deletion Handling — Non-Blocking Status-Bar Notice

**Decision**: File deletion events (`EventKind::Remove`) produce a one-time status-bar message (e.g., "[file.txt deleted on disk — buffer kept in memory]") stored as `App::watcher_notice: Option<String>`. No modal dialog. The buffer remains editable.

**Rationale**: US3 specifies a "non-blocking notification". The existing status-bar rendering path already handles temporary messages; this avoids adding a second dialog type.

---

## notify Crate API Summary (Key types)

```rust
// Watcher creation (mpsc bridge)
let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
let mut watcher = notify::recommended_watcher(tx)?;
watcher.watch(dir_path, notify::RecursiveMode::NonRecursive)?;
watcher.unwatch(dir_path)?;

// Non-blocking drain in event loop
while let Ok(Ok(event)) = rx.try_recv() {
    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) => { /* check event.paths */ }
        EventKind::Remove(_) => { /* deletion */ }
        _ => {}
    }
}

// Event structure
pub struct Event {
    pub kind: EventKind,
    pub paths: Vec<PathBuf>,
    // ...
}
```

**PollWatcher fallback** (drop-in, same `Watcher` trait):
```rust
let mut watcher = notify::PollWatcher::new(tx, notify::Config::default()
    .with_poll_interval(Duration::from_secs(5)))?;
```

`recommended_watcher` returns `PollWatcher` automatically if the platform lacks native events (e.g., NFS mounts). No special handling needed on the consumer side.
