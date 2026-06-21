//! Recent-files list — Feature 049.
//!
//! Tracks recently opened files (most-recent first), persisted to
//! `$XDG_STATE_HOME/edit/recent.toml`, and surfaced in the File menu. The list is
//! de-duplicated and capped at `config.recent_files_limit`.

#![allow(dead_code)]

use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The persisted recent-files list, most-recent first.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentFiles {
    #[serde(default)]
    pub paths: Vec<String>,
}

impl RecentFiles {
    /// Record `path` as the most-recently-used: remove any existing equal entry,
    /// push it to the front, then cap to `limit`. `limit == 0` clears the list
    /// (the feature is disabled).
    pub fn record(&mut self, path: &str, limit: usize) {
        if limit == 0 {
            self.paths.clear();
            return;
        }
        self.paths.retain(|p| p != path);
        self.paths.insert(0, path.to_owned());
        self.paths.truncate(limit);
    }

    /// Drop a specific path (e.g. when it no longer exists on disk).
    pub fn remove(&mut self, path: &str) {
        self.paths.retain(|p| p != path);
    }
}

/// Path to the recent-files store. Mirrors [`crate::session::session_path`]'s
/// resolution (`$XDG_STATE_HOME/edit/` with a `$HOME/.local/state` fallback).
pub fn recent_path() -> PathBuf {
    let base = dirs::state_dir().unwrap_or_else(|| {
        let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        p.push(".local");
        p.push("state");
        p
    });
    base.join("edit").join("recent.toml")
}

/// Load the recent-files list. An absent or corrupt file loads as an empty list
/// (never an error — a missing/garbled store should not disrupt startup).
pub fn load() -> RecentFiles {
    let path = recent_path();
    match std::fs::read_to_string(&path) {
        Ok(s) => toml::from_str(&s).unwrap_or_else(|e| {
            log::warn!("recent: corrupt {:?} ({e}); starting empty", path);
            RecentFiles::default()
        }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => RecentFiles::default(),
        Err(e) => {
            log::warn!("recent: could not read {:?}: {e}", path);
            RecentFiles::default()
        }
    }
}

/// Atomically persist the recent-files list (tmp-write + rename), mirroring the
/// session writer. Best-effort: callers log any error.
pub fn save(recent: &RecentFiles) -> io::Result<()> {
    let path = recent_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let toml_str = toml::to_string_pretty(recent)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, toml_str.as_bytes())?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_dedups_moves_to_front_and_caps() {
        let mut r = RecentFiles::default();
        r.record("/a", 3);
        r.record("/b", 3);
        r.record("/c", 3);
        assert_eq!(r.paths, ["/c", "/b", "/a"]);
        // Re-record /a → moves to front, no duplicate.
        r.record("/a", 3);
        assert_eq!(r.paths, ["/a", "/c", "/b"]);
        // Cap: a 4th distinct entry drops the oldest.
        r.record("/d", 3);
        assert_eq!(r.paths, ["/d", "/a", "/c"]);
    }

    #[test]
    fn limit_zero_disables() {
        let mut r = RecentFiles::default();
        r.record("/a", 0);
        assert!(r.paths.is_empty());
        r.paths = vec!["/x".into()];
        r.record("/y", 0);
        assert!(r.paths.is_empty(), "limit 0 clears the list");
    }
}
