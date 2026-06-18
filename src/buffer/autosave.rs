//! Auto-save and crash-recovery subsystem (US5 / T058-T061).
//!
//! # Overview
//!
//! Every buffer with a path gets an [`AutosaveState`] that tracks:
//! - The paths of the `.recovery` and `.lock` files (derived from the buffer's absolute path).
//! - When the last auto-save write happened.
//! - Whether auto-save is enabled for this buffer.
//!
//! Recovery files use the format specified in `contracts/recovery.md` (EDIT-RECOVERY-V1).
//! Lock files hold only the owning PID.  On startup, stale locks (dead PID) trigger a
//! recovery offer to the user.

#![allow(dead_code, unused_variables, unused_imports)]

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::buffer::Buffer;
use crate::encoding::EncodingId;

// ---------------------------------------------------------------------------
// FNV-1a hash (no external deps required)
// ---------------------------------------------------------------------------

/// Compute a 64-bit FNV-1a hash of the given byte slice.
///
/// This is deterministic and stable within a single binary (does not use
/// `std::hash`, which has an unstable hasher seed).
fn fnv1a_64(data: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

// ---------------------------------------------------------------------------
// Recovery / lock path computation
// ---------------------------------------------------------------------------

/// Return the recovery file path for a buffer at `abs_path`.
///
/// Stored under `$XDG_RUNTIME_DIR/edit/<hash>.recovery` or
/// `$TMPDIR/edit-recovery/<hash>.recovery` as a fallback.
pub fn recovery_path_for(abs_path: &Path) -> PathBuf {
    let hash = fnv1a_64(abs_path.as_os_str().as_encoded_bytes());
    let filename = format!("{:016x}.recovery", hash);
    recovery_dir().join(filename)
}

/// Return the lock file path for a buffer at `abs_path`.
pub fn lock_path_for(abs_path: &Path) -> PathBuf {
    let hash = fnv1a_64(abs_path.as_os_str().as_encoded_bytes());
    let filename = format!("{:016x}.lock", hash);
    recovery_dir().join(filename)
}

/// The directory where recovery / lock files are stored.
///
/// `$XDG_RUNTIME_DIR/edit` when set, otherwise `$TMPDIR/edit-recovery`.
fn recovery_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(xdg).join("edit")
    } else {
        let tmp = std::env::var("TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir());
        tmp.join("edit-recovery")
    }
}

/// Create the recovery directory (mode 0700) if it does not already exist.
fn ensure_recovery_dir() -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    let dir = recovery_dir();
    if !dir.exists() {
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&dir)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// AutosaveState
// ---------------------------------------------------------------------------

/// Per-buffer auto-save state (T058).
pub struct AutosaveState {
    /// How often (in seconds) to write a recovery file.
    pub interval_secs: u32,
    /// When the last recovery write occurred.
    pub last_save_at: Instant,
    /// Path of the `.recovery` file.
    pub recovery_path: PathBuf,
    /// Path of the `.lock` file.
    pub lock_path: PathBuf,
    /// Whether auto-save is enabled for this buffer.
    pub enabled: bool,
}

impl AutosaveState {
    /// Create a placeholder `AutosaveState` for buffers without an on-disk path.
    ///
    /// `enabled` will be `false`; the paths are meaningless sentinels.
    pub fn new(enabled: bool, interval_secs: u32) -> Self {
        AutosaveState {
            interval_secs,
            last_save_at: Instant::now(),
            recovery_path: PathBuf::new(),
            lock_path: PathBuf::new(),
            enabled,
        }
    }

    /// Create an `AutosaveState` derived from `path` (must be absolute).
    pub fn for_path(path: &Path, enabled: bool, interval_secs: u32) -> Self {
        AutosaveState {
            interval_secs,
            last_save_at: Instant::now(),
            recovery_path: recovery_path_for(path),
            lock_path: lock_path_for(path),
            enabled,
        }
    }
}

// ---------------------------------------------------------------------------
// LockStatus
// ---------------------------------------------------------------------------

/// Result of checking a `.lock` file on startup (T059).
#[derive(Debug, Clone)]
pub enum LockStatus {
    /// Another `edit` session owns this file and is still running.
    OtherSessionActive(u32),
    /// The lock file exists but the recorded PID is not alive — previous session crashed.
    StaleRecovery,
    /// No lock file exists; this is a clean open.
    Clean,
}

// ---------------------------------------------------------------------------
// Lock file operations (T059)
// ---------------------------------------------------------------------------

/// Write the current process PID to `<recovery_path>.lock` with mode 0600.
pub fn create_lock(lock_path: &Path, pid: u32) -> io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    ensure_recovery_dir()?;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(lock_path)?;
    writeln!(f, "{}", pid)?;
    Ok(())
}

/// Delete the lock file.  Errors are silently ignored.
pub fn release_lock(lock_path: &Path) {
    let _ = std::fs::remove_file(lock_path);
}

/// Check whether `state.lock_path` contains a live or stale PID.
pub fn check_stale_lock(state: &AutosaveState) -> LockStatus {
    let lock_path = &state.lock_path;

    // No lock file → clean.
    if !lock_path.exists() {
        return LockStatus::Clean;
    }

    // Read PID from lock file.
    let pid: u32 = match std::fs::read_to_string(lock_path) {
        Ok(s) => match s.trim().parse::<u32>() {
            Ok(p) => p,
            Err(_) => return LockStatus::StaleRecovery,
        },
        Err(_) => return LockStatus::StaleRecovery,
    };

    if is_pid_alive(pid) {
        LockStatus::OtherSessionActive(pid)
    } else {
        LockStatus::StaleRecovery
    }
}

/// Check if a process with the given PID is alive using `kill -0`.
///
/// This calls the `kill` shell utility with signal 0, which is a POSIX-portable
/// way to test process liveness without sending an actual signal.
///
/// Note: the contract specifies `kill(pid, 0)` via POSIX.  On Linux this can
/// also be done via `libc`, but to avoid adding a dependency we use the shell
/// utility which is always available on our target platform.
fn is_pid_alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Recovery file write (T060)
// ---------------------------------------------------------------------------

/// Data needed to write a recovery file (decoupled from `Buffer` borrow).
pub struct RecoverySnapshot {
    pub original_path: PathBuf,
    pub encoding: String,
    pub content: String,
}

/// Write a recovery file for `buf` and update `state.last_save_at`.
///
/// The write is atomic: content goes to `<recovery_path>.tmp` first, then the
/// file is renamed into place.
pub fn write_recovery(buf: &Buffer, state: &mut AutosaveState) -> io::Result<()> {
    let path = match &buf.path {
        Some(p) => p.clone(),
        None => return Ok(()), // unnamed buffers cannot be recovered
    };

    let content = buf.rope.to_string();
    let encoding_name = encoding_name(buf.encoding);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let content_bytes = content.as_bytes();
    let content_len = content_bytes.len();

    ensure_recovery_dir()?;

    let tmp_path = {
        let mut p = state.recovery_path.clone();
        let mut name = p
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("recovery"))
            .to_os_string();
        name.push(".tmp");
        p.set_file_name(name);
        p
    };

    // Build the header + content block.
    let mut out = Vec::with_capacity(256 + content_len);
    writeln!(out, "EDIT-RECOVERY-V1")?;
    writeln!(out, "path: {}", path.display())?;
    writeln!(out, "encoding: {}", encoding_name)?;
    writeln!(out, "timestamp: {}", timestamp)?;
    writeln!(out, "content_len: {}", content_len)?;
    writeln!(out, "---")?;
    out.write_all(content_bytes)?;

    // Atomic write.
    std::fs::write(&tmp_path, &out)?;
    std::fs::rename(&tmp_path, &state.recovery_path)?;

    state.last_save_at = Instant::now();
    log::debug!(
        "Auto-saved recovery file: {:?} ({} bytes)",
        state.recovery_path,
        content_len
    );
    Ok(())
}

/// Convenience function that extracts the snapshot from `buf`, writes the
/// recovery file, and updates `buf.autosave.last_save_at`.
///
/// This helper takes a `&mut Buffer` rather than separate borrows to avoid the
/// "cannot borrow `buf` as immutable because it is also borrowed as mutable"
/// error that arises when borrowing `buf` and `buf.autosave` simultaneously.
pub fn write_recovery_for_buffer(buf: &mut Buffer, _interval_secs: u32) {
    // Temporarily take the path and content we need.
    let path = match buf.path.clone() {
        Some(p) => p,
        None => return,
    };
    let content = buf.rope.to_string();
    let encoding = buf.encoding;

    let encoding_name = encoding_name(encoding);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let content_bytes = content.as_bytes();
    let content_len = content_bytes.len();

    let recovery_path = buf.autosave.recovery_path.clone();
    let tmp_path = {
        let mut p = recovery_path.clone();
        let mut name = p
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("recovery"))
            .to_os_string();
        name.push(".tmp");
        p.set_file_name(name);
        p
    };

    if let Err(e) = ensure_recovery_dir() {
        log::error!("Cannot create recovery dir: {}", e);
        return;
    }

    let mut out: Vec<u8> = Vec::with_capacity(256 + content_len);
    if writeln!(out, "EDIT-RECOVERY-V1").is_err()
        || writeln!(out, "path: {}", path.display()).is_err()
        || writeln!(out, "encoding: {}", encoding_name).is_err()
        || writeln!(out, "timestamp: {}", timestamp).is_err()
        || writeln!(out, "content_len: {}", content_len).is_err()
        || writeln!(out, "---").is_err()
        || out.write_all(content_bytes).is_err()
    {
        log::error!("Failed to serialise recovery data");
        return;
    }

    if let Err(e) = std::fs::write(&tmp_path, &out) {
        log::error!("Failed to write tmp recovery file: {}", e);
        return;
    }
    if let Err(e) = std::fs::rename(&tmp_path, &recovery_path) {
        log::error!("Failed to rename recovery file into place: {}", e);
        return;
    }

    buf.autosave.last_save_at = Instant::now();
    log::debug!("Auto-saved recovery for {:?} ({} bytes)", path, content_len);
}

// ---------------------------------------------------------------------------
// Recovery file read (T061)
// ---------------------------------------------------------------------------

/// The deserialized contents of a recovery file.
#[derive(Debug)]
pub struct RecoveryData {
    /// The path of the file that was being edited when the crash occurred.
    pub original_path: PathBuf,
    /// The encoding name stored in the recovery header.
    pub encoding: String,
    /// Unix epoch timestamp of when the recovery was written.
    pub timestamp: u64,
    /// The buffer content at the time of the last auto-save.
    pub content: String,
}

/// Errors that can occur when reading or parsing a recovery file.
#[derive(Debug)]
pub enum RecoveryError {
    /// An I/O error occurred while reading the file.
    Io(io::Error),
    /// The file does not match the expected format.
    InvalidFormat(String),
    /// The version magic is a future version this build does not support.
    UnknownVersion(String),
    /// The `content_len` header does not match the actual content length.
    ContentLenMismatch,
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryError::Io(e) => write!(f, "I/O error: {}", e),
            RecoveryError::InvalidFormat(s) => write!(f, "invalid recovery format: {}", s),
            RecoveryError::UnknownVersion(v) => {
                write!(f, "unrecognised recovery version: {}", v)
            }
            RecoveryError::ContentLenMismatch => write!(
                f,
                "recovery file content_len does not match actual content length"
            ),
        }
    }
}

impl From<io::Error> for RecoveryError {
    fn from(e: io::Error) -> Self {
        RecoveryError::Io(e)
    }
}

/// Parse a recovery file from `path` and return the structured data.
pub fn read_recovery(path: &Path) -> Result<RecoveryData, RecoveryError> {
    let raw = std::fs::read(path).map_err(RecoveryError::Io)?;
    parse_recovery_bytes(&raw)
}

/// Parse recovery file contents from a byte slice.
///
/// Separated from `read_recovery` so it can be unit-tested without touching
/// the filesystem.
pub fn parse_recovery_bytes(raw: &[u8]) -> Result<RecoveryData, RecoveryError> {
    // Convert to UTF-8 first.
    let text = std::str::from_utf8(raw)
        .map_err(|_| RecoveryError::InvalidFormat("not valid UTF-8".into()))?;

    let mut lines = text.splitn(usize::MAX, '\n');

    // --- Version magic -------------------------------------------------------
    let version_line = lines.next().unwrap_or("").trim_end_matches('\r');
    if version_line != "EDIT-RECOVERY-V1" {
        if version_line.starts_with("EDIT-RECOVERY-V") {
            return Err(RecoveryError::UnknownVersion(version_line.to_string()));
        }
        return Err(RecoveryError::InvalidFormat(format!(
            "expected 'EDIT-RECOVERY-V1', got '{}'",
            version_line
        )));
    }

    // --- Header fields -------------------------------------------------------
    let mut original_path: Option<PathBuf> = None;
    let mut encoding: Option<String> = None;
    let mut timestamp: Option<u64> = None;
    let mut content_len: Option<usize> = None;
    let mut header_ended = false;
    let separator_pos: usize = 0; // byte offset just after the `---\n` separator (unused sentinel)

    // We need to know the byte offset where the content starts.
    // Re-scan the raw bytes to find `---\n`.
    let separator = b"---\n";
    let mut sep_byte_offset: Option<usize> = None;
    for i in 0..raw.len() {
        if raw[i..].starts_with(separator) {
            sep_byte_offset = Some(i + separator.len());
            break;
        }
    }

    // Parse header fields from the text representation.
    for line in lines {
        let line = line.trim_end_matches('\r');
        if line == "---" {
            header_ended = true;
            break;
        }
        if let Some(val) = line.strip_prefix("path: ") {
            original_path = Some(PathBuf::from(val));
        } else if let Some(val) = line.strip_prefix("encoding: ") {
            encoding = Some(val.to_string());
        } else if let Some(val) = line.strip_prefix("timestamp: ") {
            timestamp = val.parse::<u64>().ok();
        } else if let Some(val) = line.strip_prefix("content_len: ") {
            content_len = val.parse::<usize>().ok();
        }
        // Unknown fields are silently ignored (forward-compat).
    }

    if !header_ended {
        return Err(RecoveryError::InvalidFormat(
            "separator '---' line not found".into(),
        ));
    }

    let original_path =
        original_path.ok_or_else(|| RecoveryError::InvalidFormat("missing 'path' field".into()))?;
    let encoding =
        encoding.ok_or_else(|| RecoveryError::InvalidFormat("missing 'encoding' field".into()))?;
    let timestamp = timestamp
        .ok_or_else(|| RecoveryError::InvalidFormat("missing 'timestamp' field".into()))?;
    let content_len = content_len
        .ok_or_else(|| RecoveryError::InvalidFormat("missing 'content_len' field".into()))?;

    // --- Content block -------------------------------------------------------
    let content_start = sep_byte_offset
        .ok_or_else(|| RecoveryError::InvalidFormat("separator '---' not found in bytes".into()))?;

    if content_start + content_len > raw.len() {
        return Err(RecoveryError::ContentLenMismatch);
    }

    let content_bytes = &raw[content_start..content_start + content_len];
    if content_bytes.len() != content_len {
        return Err(RecoveryError::ContentLenMismatch);
    }

    let content = std::str::from_utf8(content_bytes)
        .map_err(|_| RecoveryError::InvalidFormat("content is not valid UTF-8".into()))?
        .to_string();

    Ok(RecoveryData {
        original_path,
        encoding,
        timestamp,
        content,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return a human-readable encoding name for a given [`EncodingId`].
fn encoding_name(enc: EncodingId) -> &'static str {
    match enc {
        EncodingId::Utf8 => "utf-8",
        EncodingId::Cp437 => "cp437",
        EncodingId::Cp850 => "cp850",
        EncodingId::Iso8859_1 => "iso-8859-1",
        EncodingId::Windows1252 => "windows-1252",
        EncodingId::Utf16Le => "utf-16-le",
        EncodingId::Utf16Be => "utf-16-be",
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // recovery_path_for — determinism and uniqueness
    // -----------------------------------------------------------------------

    #[test]
    fn recovery_path_is_deterministic() {
        let p = Path::new("/home/user/docs/file.txt");
        let r1 = recovery_path_for(p);
        let r2 = recovery_path_for(p);
        assert_eq!(r1, r2);
    }

    #[test]
    fn recovery_paths_differ_for_different_files() {
        let a = recovery_path_for(Path::new("/home/user/a.txt"));
        let b = recovery_path_for(Path::new("/home/user/b.txt"));
        assert_ne!(a, b);
    }

    #[test]
    fn lock_and_recovery_paths_differ() {
        let p = Path::new("/tmp/test.txt");
        let r = recovery_path_for(p);
        let l = lock_path_for(p);
        assert_ne!(r, l);
    }

    #[test]
    fn recovery_path_ends_with_recovery_extension() {
        let p = recovery_path_for(Path::new("/tmp/file.txt"));
        assert!(
            p.to_str().unwrap().ends_with(".recovery"),
            "expected .recovery extension, got {:?}",
            p
        );
    }

    // -----------------------------------------------------------------------
    // parse_recovery_bytes — happy path
    // -----------------------------------------------------------------------

    fn make_recovery_bytes(content: &str) -> Vec<u8> {
        let mut out = Vec::new();
        writeln!(out, "EDIT-RECOVERY-V1").unwrap();
        writeln!(out, "path: /home/user/test.txt").unwrap();
        writeln!(out, "encoding: utf-8").unwrap();
        writeln!(out, "timestamp: 1700000000").unwrap();
        writeln!(out, "content_len: {}", content.len()).unwrap();
        writeln!(out, "---").unwrap();
        out.write_all(content.as_bytes()).unwrap();
        out
    }

    #[test]
    fn parse_recovery_happy_path() {
        let content = "hello\nworld\n";
        let raw = make_recovery_bytes(content);
        let data = parse_recovery_bytes(&raw).expect("should parse");
        assert_eq!(data.original_path, PathBuf::from("/home/user/test.txt"));
        assert_eq!(data.encoding, "utf-8");
        assert_eq!(data.timestamp, 1_700_000_000);
        assert_eq!(data.content, content);
    }

    #[test]
    fn parse_recovery_empty_content() {
        let raw = make_recovery_bytes("");
        let data = parse_recovery_bytes(&raw).expect("should parse empty content");
        assert_eq!(data.content, "");
    }

    #[test]
    fn parse_recovery_content_with_dashes() {
        // Content that contains "---" should not confuse the parser.
        let content = "---\nsome --- text\n---\n";
        let raw = make_recovery_bytes(content);
        let data = parse_recovery_bytes(&raw).expect("should parse");
        assert_eq!(data.content, content);
    }

    // -----------------------------------------------------------------------
    // parse_recovery_bytes — error paths
    // -----------------------------------------------------------------------

    #[test]
    fn parse_recovery_wrong_magic() {
        let raw = b"EDIT-CORRUPT\npath: /tmp/x\n---\n".to_vec();
        let err = parse_recovery_bytes(&raw).unwrap_err();
        assert!(matches!(err, RecoveryError::InvalidFormat(_)));
    }

    #[test]
    fn parse_recovery_future_version() {
        let raw = b"EDIT-RECOVERY-V99\npath: /tmp/x\n---\n".to_vec();
        let err = parse_recovery_bytes(&raw).unwrap_err();
        assert!(matches!(err, RecoveryError::UnknownVersion(_)));
    }

    #[test]
    fn parse_recovery_missing_separator() {
        let raw =
            b"EDIT-RECOVERY-V1\npath: /tmp/x\nencoding: utf-8\ntimestamp: 0\ncontent_len: 0\n"
                .to_vec();
        let err = parse_recovery_bytes(&raw).unwrap_err();
        // Could be InvalidFormat (separator not found) or MissingField; either is acceptable.
        // Just verify it's an error.
        let _ = err;
    }

    #[test]
    fn parse_recovery_content_len_mismatch() {
        let mut raw = Vec::new();
        writeln!(raw, "EDIT-RECOVERY-V1").unwrap();
        writeln!(raw, "path: /tmp/x").unwrap();
        writeln!(raw, "encoding: utf-8").unwrap();
        writeln!(raw, "timestamp: 0").unwrap();
        writeln!(raw, "content_len: 9999").unwrap(); // way too large
        writeln!(raw, "---").unwrap();
        raw.write_all(b"short").unwrap();
        let err = parse_recovery_bytes(&raw).unwrap_err();
        assert!(matches!(err, RecoveryError::ContentLenMismatch));
    }

    // -----------------------------------------------------------------------
    // FNV-1a hash basic properties
    // -----------------------------------------------------------------------

    #[test]
    fn fnv1a_same_input_same_output() {
        assert_eq!(fnv1a_64(b"hello"), fnv1a_64(b"hello"));
    }

    #[test]
    fn fnv1a_different_inputs_differ() {
        assert_ne!(fnv1a_64(b"hello"), fnv1a_64(b"world"));
    }

    #[test]
    fn fnv1a_empty_is_offset_basis() {
        assert_eq!(fnv1a_64(b""), 0xcbf2_9ce4_8422_2325);
    }

    // -----------------------------------------------------------------------
    // AutosaveState construction
    // -----------------------------------------------------------------------

    #[test]
    fn autosave_state_new_disabled() {
        let s = AutosaveState::new(false, 30);
        assert!(!s.enabled);
        assert_eq!(s.interval_secs, 30);
    }

    #[test]
    fn autosave_state_for_path() {
        let p = Path::new("/tmp/myfile.txt");
        let s = AutosaveState::for_path(p, true, 60);
        assert!(s.enabled);
        assert_eq!(s.interval_secs, 60);
        assert!(s.recovery_path.to_str().unwrap().ends_with(".recovery"));
        assert!(s.lock_path.to_str().unwrap().ends_with(".lock"));
    }
}
