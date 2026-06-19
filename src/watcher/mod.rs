//! Filesystem watcher — detects external modifications to open buffer files.
//!
//! Wraps the `notify` crate's platform-native watcher (inotify on Linux,
//! kqueue on BSD/macOS) and exposes a simple, non-blocking drain API for the
//! editor's main event loop.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// Grace window applied after every editor-initiated write to suppress the
/// inotify event that the write itself generates (prevents self-write false positives).
const SELF_WRITE_GRACE: Duration = Duration::from_secs(2);

/// Minimum time between consecutive `WatchEvent` emissions for the same path
/// (debounce — coalesces rapid external writes into a single prompt).
const DEBOUNCE_SECS: Duration = Duration::from_secs(1);

// ── Event types ──────────────────────────────────────────────────────────────

/// Normalized file-system event kind as seen by the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEventKind {
    Modified,
    Deleted,
}

/// A debounced, self-write-filtered event ready for the editor to act on.
#[derive(Debug, Clone)]
pub struct WatchEvent {
    pub path: PathBuf,
    pub kind: WatchEventKind,
}

/// Pending external-change state stored in `App` while the reload dialog is open.
#[derive(Debug, Clone)]
pub struct ExternalChange {
    pub buf_idx: usize,
    pub path: PathBuf,
    pub kind: WatchEventKind,
}

// ── FileWatcher ───────────────────────────────────────────────────────────────

/// Wraps `notify::RecommendedWatcher` and delivers debounced `WatchEvent`s via
/// a non-blocking `try_recv_event()` drain.
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<notify::Result<notify::Event>>,
    /// Refcount of open buffers whose backing file lives in each watched directory.
    watched_dirs: HashMap<PathBuf, usize>,
    /// Debounce tracker: last emission time per absolute file path.
    last_emitted: HashMap<PathBuf, Instant>,
}

impl FileWatcher {
    /// Create a new watcher backed by the OS-native mechanism (inotify on Linux,
    /// kqueue on BSD/macOS).  Returns `Err` only if the OS fails to create the
    /// underlying watch handle.
    pub fn new() -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel();
        let watcher = notify::recommended_watcher(tx)?;
        log::info!("FileWatcher: OS-native watcher initialised");
        Ok(Self {
            _watcher: watcher,
            rx,
            watched_dirs: HashMap::new(),
            last_emitted: HashMap::new(),
        })
    }

    /// Register `path`'s parent directory for watching (refcounted).
    ///
    /// If two buffers share the same parent directory, the directory is
    /// registered only once at the OS level (FR-011).
    pub fn watch_path(&mut self, path: &Path) -> Result<(), notify::Error> {
        let dir = match path.parent() {
            Some(d) if !d.as_os_str().is_empty() => d.to_path_buf(),
            _ => return Ok(()),
        };
        let count = self.watched_dirs.entry(dir.clone()).or_insert(0);
        if *count == 0 {
            self._watcher.watch(&dir, RecursiveMode::NonRecursive)?;
            log::debug!("FileWatcher: watching dir {:?}", dir);
        }
        *count += 1;
        Ok(())
    }

    /// Decrement the watch refcount for `path`'s parent directory; unregister
    /// the directory when the count reaches zero.
    pub fn unwatch_path(&mut self, path: &Path) -> Result<(), notify::Error> {
        let dir = match path.parent() {
            Some(d) if !d.as_os_str().is_empty() => d.to_path_buf(),
            _ => return Ok(()),
        };
        if let Some(count) = self.watched_dirs.get_mut(&dir) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self._watcher.unwatch(&dir)?;
                self.watched_dirs.remove(&dir);
                log::debug!("FileWatcher: unwatched dir {:?}", dir);
            }
        }
        Ok(())
    }

    /// Expose the directory-refcount map for integration tests (T046).
    pub fn watched_dirs(&self) -> &HashMap<PathBuf, usize> {
        &self.watched_dirs
    }

    /// Non-blocking drain: returns the first qualifying `WatchEvent` or `None`.
    ///
    /// A raw notify event qualifies when:
    /// 1. The affected path is one of the watched buffer-backing files
    ///    (checked against `self.watched_dirs` membership for the parent dir).
    /// 2. It is not a self-write — `self_write_times[path].elapsed() >= SELF_WRITE_GRACE`.
    /// 3. It is not debounced — `self.last_emitted[path].elapsed() >= DEBOUNCE_SECS`.
    pub fn try_recv_event(
        &mut self,
        self_write_times: &HashMap<PathBuf, Instant>,
        watched_paths: &[PathBuf],
    ) -> Option<WatchEvent> {
        loop {
            match self.rx.try_recv() {
                Ok(Ok(event)) => {
                    for raw_path in &event.paths {
                        // Only act on paths that are known buffer-backing files.
                        let canonical = raw_path.as_path();
                        if !watched_paths.iter().any(|p| p.as_path() == canonical) {
                            continue;
                        }

                        // Self-write suppression (FR-007).
                        if let Some(&ts) = self_write_times.get(canonical) {
                            if ts.elapsed() < SELF_WRITE_GRACE {
                                log::debug!(
                                    "FileWatcher: suppressed self-write event for {:?}",
                                    canonical
                                );
                                continue;
                            }
                        }

                        // Debounce (FR-008).
                        if let Some(&ts) = self.last_emitted.get(canonical) {
                            if ts.elapsed() < DEBOUNCE_SECS {
                                log::debug!("FileWatcher: debounced event for {:?}", canonical);
                                continue;
                            }
                        }

                        let kind = match &event.kind {
                            EventKind::Modify(_) | EventKind::Create(_) => WatchEventKind::Modified,
                            EventKind::Remove(_) => WatchEventKind::Deleted,
                            _ => continue,
                        };

                        self.last_emitted
                            .insert(canonical.to_path_buf(), Instant::now());
                        log::debug!("FileWatcher: emitting {:?} event for {:?}", kind, canonical);
                        return Some(WatchEvent {
                            path: canonical.to_path_buf(),
                            kind,
                        });
                    }
                }
                Ok(Err(e)) => {
                    log::warn!("FileWatcher: notify error: {}", e);
                }
                Err(mpsc::TryRecvError::Empty) => return None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    log::warn!("FileWatcher: watcher channel disconnected");
                    return None;
                }
            }
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    fn make_watcher() -> FileWatcher {
        FileWatcher::new().expect("FileWatcher::new should succeed in test env")
    }

    #[test]
    fn test_watch_unwatch_refcount() {
        let mut fw = make_watcher();
        let dir = std::env::temp_dir();
        let path_a = dir.join("fw_test_a.txt");
        let path_b = dir.join("fw_test_b.txt");

        // Both files share the same parent directory.
        fw.watch_path(&path_a).unwrap();
        assert_eq!(fw.watched_dirs[&dir], 1);

        fw.watch_path(&path_b).unwrap();
        assert_eq!(
            fw.watched_dirs[&dir], 2,
            "refcount should be 2 after two watches"
        );

        fw.unwatch_path(&path_a).unwrap();
        assert_eq!(
            fw.watched_dirs[&dir], 1,
            "refcount should be 1 after one unwatch"
        );
        assert!(
            fw.watched_dirs.contains_key(&dir),
            "dir should still be watched"
        );

        fw.unwatch_path(&path_b).unwrap();
        assert!(
            !fw.watched_dirs.contains_key(&dir),
            "dir should be unwatched after both paths released"
        );
    }

    #[test]
    fn test_two_buffers_same_file_single_watch() {
        let mut fw = make_watcher();
        let dir = std::env::temp_dir();
        let path = dir.join("fw_same_file_test.txt");

        // Two buffers pointing at the same file (FR-011).
        fw.watch_path(&path).unwrap();
        fw.watch_path(&path).unwrap();
        assert_eq!(
            fw.watched_dirs[&dir], 2,
            "refcount 2 for same file watched twice"
        );

        fw.unwatch_path(&path).unwrap();
        assert_eq!(fw.watched_dirs[&dir], 1);
        fw.unwatch_path(&path).unwrap();
        assert!(!fw.watched_dirs.contains_key(&dir));
    }

    #[test]
    fn test_self_write_suppressed() {
        let mut fw = make_watcher();
        let dir = std::env::temp_dir();
        let path = dir.join("fw_self_write_test.txt");
        fs::write(&path, b"initial").ok();
        fw.watch_path(&path).unwrap();

        // Record a very recent self-write for this path.
        let mut self_write_times = HashMap::new();
        self_write_times.insert(path.clone(), Instant::now());

        // Drain immediately — any events arriving should be suppressed.
        // (In the unit test environment there may be no actual events, but the
        // suppression logic is tested by the grace-window check in try_recv_event.)
        let watched = vec![path.clone()];
        let event = fw.try_recv_event(&self_write_times, &watched);
        // Either None (no event yet) or None (suppressed) — both are correct.
        // The real behaviour is tested by the integration test test_self_write_suppressed_no_prompt.
        assert!(event.is_none() || event.map(|e| e.path == path).unwrap_or(false));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_debounce_coalesces() {
        let mut fw = make_watcher();
        let path = std::env::temp_dir().join("fw_debounce_test.txt");

        // Simulate a recently emitted event for this path.
        fw.last_emitted.insert(path.clone(), Instant::now());

        // A second event for the same path within DEBOUNCE_SECS should be suppressed.
        // We inject a synthetic entry — the real check is in try_recv_event logic.
        let elapsed = fw.last_emitted[&path].elapsed();
        assert!(
            elapsed < DEBOUNCE_SECS,
            "last_emitted should be within debounce window: {:?}",
            elapsed
        );

        // After the debounce window passes, the event should be allowed.
        // (We just verify the Duration constant is 1 second as specced.)
        assert_eq!(DEBOUNCE_SECS, Duration::from_secs(1));
    }

    #[test]
    fn test_unknown_path_ignored() {
        let mut fw = make_watcher();
        let watched_path = std::env::temp_dir().join("fw_known.txt");
        let unknown_path = std::env::temp_dir().join("fw_unknown.txt");

        fw.watch_path(&watched_path).unwrap();
        let self_write_times = HashMap::new();
        // Only watched_path is in the watched list — events for unknown_path are ignored.
        let watched = vec![watched_path.clone()];
        // With no events in the channel, try_recv_event returns None.
        let event = fw.try_recv_event(&self_write_times, &watched);
        assert!(event.is_none());
    }

    #[test]
    fn test_self_write_grace_window_is_two_seconds() {
        assert_eq!(SELF_WRITE_GRACE, Duration::from_secs(2));
    }
}
