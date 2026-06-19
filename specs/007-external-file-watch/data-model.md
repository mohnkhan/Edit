# Data Model: External File Modification Detection (Feature 007)

## Entities

### `FileWatcher` (`src/watcher/mod.rs`)

Wraps the `notify` watcher and the `mpsc::Receiver`. Owned by `App` (as `Option<FileWatcher>`).

**Fields**:
- `_watcher: Box<dyn notify::Watcher + Send>` — the OS-native or poll watcher; kept alive for its Drop
- `rx: std::sync::mpsc::Receiver<notify::Result<notify::Event>>` — receives raw events from notify's internal thread
- `watched_dirs: HashMap<PathBuf, usize>` — refcount of how many open buffers share a parent directory; the directory is unwatched when the count drops to zero
- `last_emitted: HashMap<PathBuf, Instant>` — debounce tracker: stores the last time a `WatchEvent` was emitted for each path (1-second minimum between emissions)

**Methods**:
- `FileWatcher::new() -> Result<Self, notify::Error>` — creates the recommended watcher with a channel bridge
- `watch_path(path: &Path) -> Result<(), notify::Error>` — registers the parent directory, increments refcount
- `unwatch_path(path: &Path) -> Result<(), notify::Error>` — decrements refcount; unregisters parent dir at zero
- `try_recv_event() -> Option<WatchEvent>` — non-blocking drain; returns at most one coalesced, debounced event per call; returns `None` if no event is ready

---

### `WatchEvent` (`src/watcher/mod.rs`)

A normalized filesystem event with debounce and dedup already applied.

**Fields**:
- `path: PathBuf` — the exact buffer path that was affected (after filtering from the raw directory event)
- `kind: WatchEventKind` — `Modified` or `Deleted`

**Enum `WatchEventKind`**:
- `Modified` — file content was changed or replaced externally
- `Deleted` — file was deleted or moved away from its original location

---

### `ExternalChange` (`src/app.rs` or `src/watcher/mod.rs`)

Pending-state struct stored in `App` when an external modification is detected and the user has not yet responded.

**Fields**:
- `buf_idx: usize` — which buffer index is affected
- `path: PathBuf` — display path for the dialog
- `kind: WatchEventKind` — whether this was a modification or deletion

---

### `App` — new fields (`src/app.rs`)

| Field | Type | Description |
|---|---|---|
| `file_watcher` | `Option<FileWatcher>` | `None` when `--no-watch` is set; `Some` otherwise |
| `self_write_times` | `HashMap<PathBuf, Instant>` | Tracks when the editor last wrote each path; used for self-write suppression (2-second grace window) |
| `pending_external_change` | `Option<ExternalChange>` | Set when a watcher event fires; cleared when user responds to the dialog |
| `watcher_notice` | `Option<String>` | One-shot non-blocking status-bar message (e.g., file-deleted notice); cleared after one render frame |

---

### `Config` — new field (`src/config/schema.rs`)

| Field | Type | Default | Description |
|---|---|---|---|
| `no_watch` | `bool` | `false` | When true, skip all file watching for the session |

---

## State Transitions

```
Buffer opened (has path)
    │
    ▼
FileWatcher::watch_path(path.parent())   ← refcounted dir watch registered
    │
    ├─── external write detected ──► pending_external_change = Some(ExternalChange { Modified, ... })
    │                                     │
    │                                     ├─ user presses Y/Enter ──► reload_from_disk() → buffer replaced, undo cleared
    │                                     └─ user presses N/Esc   ──► buffer marked "modified" (dirty=true)
    │
    ├─── external delete detected ──► watcher_notice = Some("… deleted …"), no modal dialog
    │
    └─── editor writes (save / auto-save)
             │
             └─► self_write_times[path] = Instant::now()
                   │
                   └─► next watcher event within 2s for same path → suppressed (not queued)

Buffer closed
    │
    ▼
FileWatcher::unwatch_path(path.parent())  ← refcount decremented; dir unwatched at zero
```

---

## File Layout (Source Code)

```
src/
├── watcher/
│   └── mod.rs         ← NEW: FileWatcher, WatchEvent, WatchEventKind
├── app.rs             ← MODIFIED: new fields, handle_tick drain, reload_from_disk()
├── config/
│   └── schema.rs      ← MODIFIED: add no_watch field
├── main.rs            ← MODIFIED: add --no-watch CLI flag
├── lib.rs             ← MODIFIED: pub mod watcher
└── ui/
    └── mod.rs         ← MODIFIED: render ExternalChangeDialog overlay

tests/integration/
└── file_watch.rs      ← NEW: integration tests (write temp file, verify event received)
```
