// Unit tests T101: security::sanitize module.
//
// These tests exercise `strip_escape_sequences` and `validate_path` from the
// already-implemented `src/security/sanitize.rs`.
//
// Run with:
//   cargo test --test security_sanitize
//
// The module is part of the `edit` crate (lib target); the tests import via the
// public crate path.  `src/lib.rs` (or the equivalent `[lib]` entry) must
// re-export `security::sanitize` for the `use edit::security::sanitize::*` path
// to resolve.  If the crate is binary-only, add a thin `lib.rs` that exposes
// the required modules, or convert the relevant modules to a library crate.
//
// NOTE: Until a `[lib]` target is added to Cargo.toml, these tests will fail
// to compile — this is intentional (TDD: red phase).  The API contract they
// encode is:
//
//   pub fn strip_escape_sequences(s: &str) -> String;
//   pub fn validate_path(p: &Path) -> Result<PathBuf, PathError>;
//   pub enum PathError { Traversal, Io(std::io::Error) }

use std::path::Path;

use edit::security::sanitize::{strip_escape_sequences, validate_path, PathError};

// ── strip_escape_sequences ────────────────────────────────────────────────────

/// ANSI CSI red-colour sequence (`ESC [ 3 1 m`) must be removed.
#[test]
fn strip_ansi_csi_red_colour() {
    let raw = "\x1b[31mred text\x1b[0m";
    let clean = strip_escape_sequences(raw);
    assert_eq!(clean, "red text", "CSI SGR colour codes were not removed");
}

/// CSI with numeric parameter (bold) must be stripped.
#[test]
fn strip_ansi_csi_bold() {
    let raw = "\x1b[1mbold\x1b[0m";
    let clean = strip_escape_sequences(raw);
    assert_eq!(clean, "bold");
}

/// CSI cursor-position sequence (`ESC [ row ; col H`) must be removed.
#[test]
fn strip_ansi_csi_cursor_position() {
    let raw = "\x1b[10;20Hvisible";
    let clean = strip_escape_sequences(raw);
    assert_eq!(clean, "visible", "CSI cursor-position sequence was not removed");
}

/// Multiple CSI sequences within a single string must all be removed.
#[test]
fn strip_multiple_csi_sequences() {
    let raw = "\x1b[1mBold\x1b[0m and \x1b[32mgreen\x1b[0m";
    let clean = strip_escape_sequences(raw);
    assert_eq!(clean, "Bold and green");
}

/// OSC sequence terminated with BEL (`\x07`) must be removed.
#[test]
fn strip_osc_with_bel() {
    let raw = "\x1b]0;Terminal Title\x07visible";
    let clean = strip_escape_sequences(raw);
    assert_eq!(clean, "visible", "OSC+BEL sequence was not removed");
}

/// OSC sequence terminated with ST (`ESC \`) must be removed.
#[test]
fn strip_osc_with_st() {
    let raw = "\x1b]0;title\x1b\\visible";
    let clean = strip_escape_sequences(raw);
    assert_eq!(clean, "visible", "OSC+ST sequence was not removed");
}

/// Plain ASCII text with no escape sequences must pass through unchanged.
#[test]
fn plain_text_passes_through_unchanged() {
    let plain = "Hello, World! 123 no escapes here.";
    assert_eq!(strip_escape_sequences(plain), plain);
}

/// Unicode text without escape sequences must pass through unchanged.
#[test]
fn unicode_text_passes_through_unchanged() {
    let unicode = "αβγ Привет 世界 مرحبا";
    assert_eq!(strip_escape_sequences(unicode), unicode);
}

/// Empty string input must return an empty string (no panic).
#[test]
fn empty_string_returns_empty() {
    assert_eq!(strip_escape_sequences(""), "");
}

/// A string consisting entirely of an escape sequence must return empty.
#[test]
fn only_escape_sequence_returns_empty() {
    let raw = "\x1b[31m";
    let clean = strip_escape_sequences(raw);
    assert_eq!(clean, "", "a string of only an escape sequence should yield empty");
}

// ── validate_path ─────────────────────────────────────────────────────────────

/// A path starting with `../../` (two `..` components) must be rejected.
#[test]
fn validate_path_rejects_double_dotdot() {
    let result = validate_path(Path::new("../../etc/passwd"));
    assert!(
        matches!(result, Err(PathError::Traversal)),
        "expected PathError::Traversal, got: {:?}",
        result
    );
}

/// A path starting with `../` (one `..` component) must be rejected.
#[test]
fn validate_path_rejects_single_dotdot_prefix() {
    let result = validate_path(Path::new("../secret"));
    assert!(
        matches!(result, Err(PathError::Traversal)),
        "expected PathError::Traversal for '../secret', got: {:?}",
        result
    );
}

/// A bare `..` path must be rejected.
#[test]
fn validate_path_rejects_bare_dotdot() {
    let result = validate_path(Path::new(".."));
    assert!(
        matches!(result, Err(PathError::Traversal)),
        "expected PathError::Traversal for '..', got: {:?}",
        result
    );
}

/// A `..` embedded inside a longer path must be rejected.
#[test]
fn validate_path_rejects_embedded_dotdot() {
    let result = validate_path(Path::new("foo/../../etc/shadow"));
    assert!(
        matches!(result, Err(PathError::Traversal)),
        "expected PathError::Traversal for embedded '..', got: {:?}",
        result
    );
}

/// A valid relative path that does not exist on disk must return `PathError::Io`
/// (specifically `ErrorKind::NotFound`), not `PathError::Traversal`.
#[test]
fn validate_path_valid_relative_nonexistent_returns_io_not_traversal() {
    let result = validate_path(Path::new("some/file.txt"));
    assert!(
        matches!(result, Err(PathError::Io(_))),
        "expected PathError::Io for a non-existent relative path without traversal, got: {:?}",
        result
    );
}

/// `/tmp` is an absolute path with no `..` components and it exists — must succeed.
#[test]
fn validate_path_accepts_absolute_tmp() {
    let result = validate_path(Path::new("/tmp"));
    assert!(
        result.is_ok(),
        "expected Ok for '/tmp', got: {:?}",
        result
    );
}

/// A simple single-component relative path that does not exist must return
/// `PathError::Io`, not `PathError::Traversal`.
#[test]
fn validate_path_nonexistent_simple_path_returns_io() {
    let result = validate_path(Path::new("this_path_does_not_exist_xyz_abc_123"));
    assert!(
        matches!(result, Err(PathError::Io(_))),
        "expected PathError::Io for non-existent simple path, got: {:?}",
        result
    );
}

/// A `./`-prefixed path must not trigger the traversal guard.  A non-existent
/// `./no_such_file` returns `PathError::Io`, not `PathError::Traversal`.
#[test]
fn validate_path_dot_prefix_is_not_traversal() {
    let result = validate_path(Path::new("./no_such_file_xyzzy"));
    assert!(
        matches!(result, Err(PathError::Io(_))),
        "expected PathError::Io (not Traversal) for './no_such_file_xyzzy', got: {:?}",
        result
    );
}
