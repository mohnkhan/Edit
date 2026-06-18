//! Session save/restore — Feature 003.
//!
//! Writes `$XDG_STATE_HOME/edit/session.toml` on a clean exit and reads it
//! back on the next no-arg startup, allowing the editor to reopen every buffer
//! at its saved cursor position and split layout.

#![allow(dead_code)]

use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ── Data model ───────────────────────────────────────────────────────────────

/// A single open buffer recorded in the session file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BufferEntry {
    /// File path — stored as-opened (absolute if originally opened with an
    /// absolute path, relative otherwise).
    pub path: String,
    /// Cursor line, 1-based.
    pub cursor_line: u32,
    /// Cursor column, 1-based.
    pub cursor_col: u32,
}

/// How the editor area was split when the session was saved.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitLayoutKind {
    #[default]
    None,
    Horizontal,
    Vertical,
}

/// Top-level session data written to `session.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionData {
    /// Schema version — currently always 1.
    pub version: u32,
    /// Index of the active buffer within `buffers` (0-based).
    pub active_buffer: usize,
    /// Split layout at the time of save.
    pub split_layout: SplitLayoutKind,
    /// Active pane: 0 for left/only pane, 1 for right pane.  MUST be 0 when
    /// `split_layout` is `None`.
    pub active_pane: u32,
    /// Buffers in visual tab order (left to right).
    pub buffers: Vec<BufferEntry>,
}

// ── Path resolution ──────────────────────────────────────────────────────────

/// Returns the canonical path for the session file.
///
/// Resolution order:
/// 1. `$XDG_STATE_HOME/edit/session.toml`
/// 2. `$HOME/.local/state/edit/session.toml` (fallback when `dirs::state_dir()` returns `None`)
pub fn session_path() -> PathBuf {
    let base = dirs::state_dir().unwrap_or_else(|| {
        let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        p.push(".local");
        p.push("state");
        p
    });
    base.join("edit").join("session.toml")
}

// ── Persistence ───────────────────────────────────────────────────────────────

/// Atomically write `data` to the session file.
///
/// Serialises via `toml::to_string_pretty`, creates the parent directory with
/// `fs::create_dir_all`, writes to a `.session.toml.tmp` sibling, then renames
/// it into place.  Callers log any returned error as `warn!`.
pub fn save_session(data: &SessionData) -> io::Result<()> {
    let path = session_path();
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("/tmp"));

    std::fs::create_dir_all(parent)?;

    let toml_str = toml::to_string_pretty(data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let tmp_path = path.with_extension("toml.tmp");
    std::fs::write(&tmp_path, toml_str.as_bytes())?;
    std::fs::rename(&tmp_path, &path)?;

    log::info!("session saved to {:?}", path);
    Ok(())
}

/// Load and validate the session file.
///
/// Three-valued return:
/// - `Ok(None)` — file absent (`NotFound`); caller shows no prompt or warning.
/// - `Ok(Some(data))` — file loaded and validated successfully.
/// - `Err(msg)` — file existed but is corrupt/invalid; caller shows a
///   status-bar warning.  The file will be silently overwritten on the next
///   clean exit.
///
/// Corrupt conditions (per FR-010):
/// (a) TOML parse failure
/// (b) `version != 1`
/// (c) missing required fields (detected by serde)
/// (d) `active_buffer >= buffers.len()`
/// (e) any `cursor_line` or `cursor_col` less than 1
pub fn load_session() -> Result<Option<SessionData>, String> {
    let path = session_path();

    // Delete any orphaned .tmp file from a crashed previous write.
    let tmp_path = path.with_extension("toml.tmp");
    if tmp_path.exists() {
        log::debug!("session: removing orphaned tmp file {:?}", tmp_path);
        let _ = std::fs::remove_file(&tmp_path);
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(e) => {
            let msg = format!("session: could not read {:?}: {}", path, e);
            log::warn!("{}", msg);
            return Err(msg);
        }
    };

    // (a) Parse TOML — lenient mode (unknown fields ignored by serde default).
    let data: SessionData = match toml::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("session: corrupt TOML in {:?}: {}", path, e);
            log::warn!("{}", msg);
            return Err(msg);
        }
    };

    // (b) Unknown schema version.
    if data.version != 1 {
        let msg = format!("session: unknown schema version {} in {:?}", data.version, path);
        log::warn!("{}", msg);
        return Err(msg);
    }

    // (d) active_buffer out of range.
    if !data.buffers.is_empty() && data.active_buffer >= data.buffers.len() {
        let msg = format!(
            "session: active_buffer {} >= buffers.len() {} in {:?}",
            data.active_buffer,
            data.buffers.len(),
            path
        );
        log::warn!("{}", msg);
        return Err(msg);
    }

    // (e) cursor coordinates below the 1-based minimum.
    for entry in &data.buffers {
        if entry.cursor_line < 1 || entry.cursor_col < 1 {
            let msg = format!(
                "session: invalid cursor ({}, {}) for {:?} in {:?}",
                entry.cursor_line, entry.cursor_col, entry.path, path
            );
            log::warn!("{}", msg);
            return Err(msg);
        }
    }

    log::info!("session loaded from {:?} ({} buffers)", path, data.buffers.len());
    Ok(Some(data))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Serialise/deserialise tests redirect the session path via XDG_STATE_HOME.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_state_dir<F: FnOnce()>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!("edit_session_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        env::set_var("XDG_STATE_HOME", &tmp);
        f();
        env::remove_var("XDG_STATE_HOME");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    fn sample_single() -> SessionData {
        SessionData {
            version: 1,
            active_buffer: 0,
            split_layout: SplitLayoutKind::None,
            active_pane: 0,
            buffers: vec![BufferEntry {
                path: "/tmp/test.txt".to_string(),
                cursor_line: 5,
                cursor_col: 10,
            }],
        }
    }

    #[test]
    fn test_round_trip_single_buffer() {
        with_temp_state_dir(|| {
            let original = sample_single();
            save_session(&original).unwrap();
            let loaded = load_session().unwrap().unwrap();
            assert_eq!(loaded, original);
        });
    }

    #[test]
    fn test_round_trip_split_vertical() {
        with_temp_state_dir(|| {
            let data = SessionData {
                version: 1,
                active_buffer: 1,
                split_layout: SplitLayoutKind::Vertical,
                active_pane: 1,
                buffers: vec![
                    BufferEntry { path: "/tmp/a.txt".to_string(), cursor_line: 1, cursor_col: 1 },
                    BufferEntry { path: "/tmp/b.txt".to_string(), cursor_line: 3, cursor_col: 7 },
                ],
            };
            save_session(&data).unwrap();
            let loaded = load_session().unwrap().unwrap();
            assert_eq!(loaded.split_layout, SplitLayoutKind::Vertical);
            assert_eq!(loaded, data);
        });
    }

    #[test]
    fn test_corrupt_toml_returns_err() {
        with_temp_state_dir(|| {
            let path = session_path();
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, b"this is not valid toml [[[").unwrap();
            assert!(matches!(load_session(), Err(_)));
        });
    }

    #[test]
    fn test_unknown_version_returns_err() {
        with_temp_state_dir(|| {
            let path = session_path();
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let toml = "version = 99\nactive_buffer = 0\nactive_pane = 0\nsplit_layout = \"none\"\n\n[[buffers]]\npath = \"/tmp/x.txt\"\ncursor_line = 1\ncursor_col = 1\n";
            std::fs::write(&path, toml).unwrap();
            assert!(matches!(load_session(), Err(_)));
        });
    }

    #[test]
    fn test_missing_file_returns_ok_none() {
        with_temp_state_dir(|| {
            // No file written — XDG_STATE_HOME/edit/session.toml does not exist.
            assert!(matches!(load_session(), Ok(None)));
        });
    }

    #[test]
    fn test_atomic_write_no_leftover_tmp() {
        with_temp_state_dir(|| {
            let data = sample_single();
            save_session(&data).unwrap();
            let tmp = session_path().with_extension("toml.tmp");
            assert!(!tmp.exists(), "tmp file should not remain after rename");
        });
    }
}
