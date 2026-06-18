//! Security utilities: escape-sequence stripping and path validation (Task T008).
//!
//! # Escape-sequence stripping
//!
//! `strip_escape_sequences` removes terminal control sequences from untrusted
//! strings before they are displayed in the status bar or written to the log.
//! Covered families:
//! - CSI sequences  `ESC [ <params> <final-byte>` (SGR colour codes, cursor moves, …)
//! - OSC sequences  `ESC ] <text> BEL` or `ESC ] <text> ESC \`
//! - DCS sequences  `ESC P <text> ESC \`
//! - SS2/SS3        `ESC N` / `ESC O` followed by one byte
//! - Bare ESC       any remaining lone `ESC` followed by one byte
//!
//! # Path validation
//!
//! `validate_path` rejects paths that attempt to escape the current working
//! directory via `..` components and resolves any symlinks before returning
//! the canonical path.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

// ── PathError ────────────────────────────────────────────────────────────────

/// Errors returned by [`validate_path`].
#[derive(Debug)]
pub enum PathError {
    /// The path attempts to traverse above the current working directory.
    Traversal,
    /// An I/O error occurred while resolving the path.
    Io(std::io::Error),
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::Traversal => write!(f, "path traversal attempt rejected"),
            PathError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for PathError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PathError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PathError {
    fn from(e: std::io::Error) -> Self {
        PathError::Io(e)
    }
}

// ── Regex for terminal escape sequences ──────────────────────────────────────

/// Compiled regex that matches the ANSI/VT escape sequence families listed in
/// the module documentation.
///
/// Pattern breakdown (each alternative is anchored to `\x1b`):
///
/// 1. `\x1b\[[^@-~]*[@-~]`   — CSI sequences: `ESC [` + optional params + final byte
/// 2. `\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)` — OSC: `ESC ]` + text + BEL or ST
/// 3. `\x1b[P][^\x1b]*\x1b\\` — DCS: `ESC P` + text + ST (`ESC \`)
/// 4. `\x1b[NO].`             — SS2 / SS3: `ESC N` or `ESC O` + one byte
/// 5. `\x1b.`                 — catch-all: any other `ESC` + one byte
static ESCAPE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        // 1. CSI  ESC [ params final-byte
        r"\x1b\[[^@-~]*[@-~]",
        r"|",
        // 2. OSC  ESC ] text (BEL or ST)
        r"\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)",
        r"|",
        // 3. DCS  ESC P text ST
        r"\x1bP[^\x1b]*\x1b\\",
        r"|",
        // 4. SS2/SS3  ESC N/O + one byte
        r"\x1b[NO].",
        r"|",
        // 5. Catch-all lone ESC + one byte
        r"\x1b.",
    ))
    .expect("ESCAPE_RE is a valid pattern")
});

// ── Public API ───────────────────────────────────────────────────────────────

/// Strips ANSI/VT terminal escape sequences from `s` and returns a clean string.
///
/// The function is allocation-cheap when no sequences are found (`Cow::Borrowed`
/// would be ideal, but we return `String` for simplicity and API stability).
///
/// # Examples
///
/// ```
/// # use edit::security::sanitize::strip_escape_sequences;
/// let raw = "\x1b[1;31mhello\x1b[0m world";
/// assert_eq!(strip_escape_sequences(raw), "hello world");
/// ```
pub fn strip_escape_sequences(s: &str) -> String {
    ESCAPE_RE.replace_all(s, "").into_owned()
}

/// Validates a path for safe use within the editor.
///
/// A path is accepted if:
/// - It contains no `..` components (neither a leading `..` nor an embedded
///   `../`).
/// - On success, the path is canonicalised (symlinks resolved, `.` removed)
///   via [`std::fs::canonicalize`].
///
/// # Errors
///
/// Returns [`PathError::Traversal`] if the path contains `..` components.
/// Returns [`PathError::Io`] if [`std::fs::canonicalize`] fails (e.g. the
/// path does not exist on disk — callers that need to accept non-existent
/// paths should handle `ErrorKind::NotFound` appropriately).
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use edit::security::sanitize::{validate_path, PathError};
/// let good = validate_path(Path::new("src/main.rs"));
/// assert!(good.is_ok());
///
/// let bad = validate_path(Path::new("../../etc/passwd"));
/// assert!(matches!(bad, Err(PathError::Traversal)));
/// ```
pub fn validate_path(p: &Path) -> Result<PathBuf, PathError> {
    // ── 1. Reject any `..` component ─────────────────────────────────────
    for component in p.components() {
        if component == std::path::Component::ParentDir {
            log::warn!("validate_path: traversal attempt rejected for path {:?}", p);
            return Err(PathError::Traversal);
        }
    }

    // ── 2. Canonicalise (resolves symlinks and `.`) ───────────────────────
    let canonical = std::fs::canonicalize(p).map_err(PathError::Io)?;

    Ok(canonical)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // -- strip_escape_sequences -----------------------------------------------

    #[test]
    fn strips_sgr_colour_code() {
        let input = "\x1b[1;31mRed bold\x1b[0m";
        assert_eq!(strip_escape_sequences(input), "Red bold");
    }

    #[test]
    fn strips_cursor_move() {
        // ESC [ 2 ; 5 H  (cursor position)
        let input = "\x1b[2;5Hhello";
        assert_eq!(strip_escape_sequences(input), "hello");
    }

    #[test]
    fn strips_osc_with_bel() {
        // OSC 0 ; title BEL  (xterm window title)
        let input = "\x1b]0;My Terminal\x07visible";
        assert_eq!(strip_escape_sequences(input), "visible");
    }

    #[test]
    fn strips_osc_with_st() {
        let input = "\x1b]0;title\x1b\\visible";
        assert_eq!(strip_escape_sequences(input), "visible");
    }

    #[test]
    fn strips_dcs_sequence() {
        let input = "\x1bPsome dcs data\x1b\\text";
        assert_eq!(strip_escape_sequences(input), "text");
    }

    #[test]
    fn strips_ss2_ss3() {
        // SS3 O (often seen as ESC O A for cursor-up in application mode)
        let input = "\x1bOAtext";
        assert_eq!(strip_escape_sequences(input), "text");
    }

    #[test]
    fn strips_bare_esc_plus_byte() {
        let input = "\x1bctext"; // ESC c (RIS — reset)
        assert_eq!(strip_escape_sequences(input), "text");
    }

    #[test]
    fn plain_text_unchanged() {
        let input = "Hello, world! 123 αβγ";
        assert_eq!(strip_escape_sequences(input), input);
    }

    #[test]
    fn empty_string() {
        assert_eq!(strip_escape_sequences(""), "");
    }

    #[test]
    fn multiple_sequences() {
        let input = "\x1b[1mBold\x1b[0m and \x1b[32mgreen\x1b[0m";
        assert_eq!(strip_escape_sequences(input), "Bold and green");
    }

    // -- validate_path --------------------------------------------------------

    #[test]
    fn traversal_single_dotdot() {
        let result = validate_path(Path::new(".."));
        assert!(matches!(result, Err(PathError::Traversal)));
    }

    #[test]
    fn traversal_embedded_dotdot() {
        let result = validate_path(Path::new("foo/../../etc/passwd"));
        assert!(matches!(result, Err(PathError::Traversal)));
    }

    #[test]
    fn traversal_leading_dotdot_slash() {
        let result = validate_path(Path::new("../secret"));
        assert!(matches!(result, Err(PathError::Traversal)));
    }

    #[test]
    fn valid_existing_path() {
        // Use a path that definitely exists.
        let result = validate_path(Path::new("/tmp"));
        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    #[test]
    fn non_existent_path_returns_io_error() {
        let result = validate_path(Path::new("this_path_does_not_exist_xyz_abc_123"));
        assert!(matches!(result, Err(PathError::Io(_))));
    }

    #[test]
    fn dot_component_is_not_traversal() {
        // A path like `./foo` should not trigger the traversal guard.
        // It will return Io(NotFound) for a non-existent path, not Traversal.
        let result = validate_path(Path::new("./no_such_file_xyzzy"));
        assert!(matches!(result, Err(PathError::Io(_))));
    }
}
