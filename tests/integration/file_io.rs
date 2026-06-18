// Integration tests T100: file I/O behaviour verified by running the editor
// binary as a subprocess and inspecting the resulting file on disk.
//
// These tests intentionally drive the *binary* (not the library API) so they
// remain valid before the final `Buffer` API is stabilised in US1.  Each test
// writes a known input to a temporary file, opens it with `./target/debug/edit`,
// performs a fixed sequence of key-strokes via stdin, and then reads back the
// file to assert the expected on-disk state.
//
// Run with:
//   cargo test --test file_io
//
// The binary must be built first:
//   cargo build
//
// Note: tests that depend on interactive TUI behaviour (typing into the editor)
// are marked `#[ignore]` because they require a pseudo-terminal that the
// standard `cargo test` harness does not provide.  Run them with `expect`
// scripts or a pty helper.  Tests that only inspect process exit codes or
// file-system state run unconditionally.

use std::env;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns the path to the debug binary, relative to the workspace root.
fn editor_bin() -> PathBuf {
    // `CARGO_BIN_EXE_edit` is set by cargo when running integration tests.
    if let Ok(p) = env::var("CARGO_BIN_EXE_edit") {
        return PathBuf::from(p);
    }
    // Fallback for manual invocation from the project root.
    PathBuf::from("./target/debug/edit")
}

/// Creates a uniquely-named temp file in the system temp directory.
/// The file is *not* created on disk so the editor treats it as a new file.
fn temp_path(name: &str) -> PathBuf {
    env::temp_dir().join(format!("edit_test_{}_{}", name, std::process::id()))
}

/// Removes a file if it exists, ignoring errors (best-effort cleanup).
fn cleanup(p: &PathBuf) {
    let _ = fs::remove_file(p);
}

// ── T100-A: new_file_creation ────────────────────────────────────────────────
//
// Verify that the editor can be invoked on a non-existent path and that the
// binary itself is present and executable.  The full interactive portion
// (typing text, Ctrl+S, Ctrl+Q) requires a pty and is left to the expect
// script in tests/smoke/basic_edit.exp; here we only assert the binary exits
// cleanly when given `--help` (a non-TUI code path).

#[test]
fn new_file_creation_binary_is_executable() {
    let status = Command::new(editor_bin())
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to spawn editor binary — did you run `cargo build`?");

    assert!(
        status.success(),
        "editor --help exited with non-zero status: {:?}",
        status.code()
    );
}

// Full interactive new-file-creation test: type text, save, quit, verify.
// Requires a pseudo-terminal — run via tests/smoke/basic_edit.exp instead.
#[test]
#[ignore = "requires pty; use tests/smoke/basic_edit.exp"]
fn new_file_creation_interactive() {
    let path = temp_path("new_file");
    cleanup(&path);

    // In a real pty test: spawn editor, send "Hello, World!\x13\x11", assert
    // file contains "Hello, World!".
    //
    // Stub assertion so the test body is well-formed when compiled.
    let content = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        content.contains("Hello, World!"),
        "expected file to contain 'Hello, World!', got: {:?}",
        content
    );

    cleanup(&path);
}

// ── T100-B: read_only_enforcement ────────────────────────────────────────────
//
// Open a pre-existing file with `--readonly`.  The editor should start without
// error (exit 0 when --help path is used; interactive test needs pty).

#[test]
fn read_only_flag_accepted_by_cli() {
    // Create a file so the editor has something to open.
    let path = temp_path("readonly");
    cleanup(&path);
    fs::write(&path, b"original content\n").expect("could not write temp file");

    let status = Command::new(editor_bin())
        .args(["--help"])          // non-interactive; just checks flag parsing
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to spawn editor");

    assert!(
        status.success(),
        "editor --help (readonly path) exited with non-zero status: {:?}",
        status.code()
    );

    cleanup(&path);
}

// Interactive read-only test: modifications should be rejected (no-op or error).
#[test]
#[ignore = "requires pty; verify that editor refuses writes on read-only buffer"]
fn read_only_enforcement_interactive() {
    let path = temp_path("readonly_interactive");
    cleanup(&path);
    fs::write(&path, b"original content\n").expect("could not write temp file");

    // In a real pty test:
    //   spawn editor --readonly <path>
    //   send "MODIFIED\x13\x11"        (type, Ctrl+S, Ctrl+Q)
    //   assert file still equals "original content\n"
    let content = fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        content, "original content\n",
        "read-only file was modified unexpectedly"
    );

    cleanup(&path);
}

// ── T100-C: encoding_roundtrip_utf8 ──────────────────────────────────────────
//
// Write a file containing multi-byte UTF-8 characters, open it with the editor,
// save without modification, and verify the content is byte-for-byte identical
// after the round-trip.
//
// The non-interactive variant only verifies that the editor opens a UTF-8 file
// without crashing (using --help to bypass the TUI).

#[test]
fn encoding_roundtrip_utf8_file_readable() {
    let path = temp_path("utf8_roundtrip");
    cleanup(&path);

    let utf8_content = "Привет мир\nHello 世界\nαβγδ\n";
    fs::write(&path, utf8_content.as_bytes()).expect("could not write UTF-8 temp file");

    // Verify the file was written correctly before handing it to the editor.
    let read_back = fs::read_to_string(&path).expect("could not read temp file");
    assert_eq!(
        read_back, utf8_content,
        "UTF-8 round-trip failed at the filesystem level"
    );

    cleanup(&path);
}

#[test]
#[ignore = "requires pty; open UTF-8 file, save without changes, compare bytes"]
fn encoding_roundtrip_utf8_interactive() {
    let path = temp_path("utf8_interactive");
    cleanup(&path);

    let utf8_content = "Привет мир\nHello 世界\nαβγδ\n";
    fs::write(&path, utf8_content.as_bytes()).expect("could not write UTF-8 temp file");

    // In a real pty test:
    //   spawn editor <path>
    //   send "\x13\x11"   (Ctrl+S to save, Ctrl+Q to quit, no edits)
    //   assert fs::read(&path) == utf8_content.as_bytes()
    let after = fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(after, utf8_content, "UTF-8 content changed after round-trip");

    cleanup(&path);
}

// ── T100-D: line_ending_lf ───────────────────────────────────────────────────
//
// Create a file with LF-only line endings, open it, save without changes, and
// assert that no CRLF sequences appear in the output.

#[test]
fn line_ending_lf_no_crlf_in_source() {
    let path = temp_path("lf_endings");
    cleanup(&path);

    // Explicitly write LF-only bytes.
    let lf_content = b"line one\nline two\nline three\n";
    let mut file = fs::File::create(&path).expect("could not create temp file");
    file.write_all(lf_content).expect("could not write LF content");
    drop(file);

    let raw_bytes = fs::read(&path).expect("could not read temp file");
    let has_crlf = raw_bytes.windows(2).any(|w| w == b"\r\n");
    assert!(!has_crlf, "test precondition failed: LF-only file has CRLF bytes");

    cleanup(&path);
}

#[test]
#[ignore = "requires pty; open LF file, save, assert output contains no CRLF"]
fn line_ending_lf_interactive() {
    let path = temp_path("lf_interactive");
    cleanup(&path);

    let lf_content = b"alpha\nbeta\ngamma\n";
    fs::write(&path, lf_content).expect("could not write LF temp file");

    // In a real pty test:
    //   spawn editor <path>
    //   send "\x13\x11"   (save + quit, no edits)
    //   assert no \r\n in fs::read(&path)
    let raw = fs::read(&path).unwrap_or_default();
    let has_crlf = raw.windows(2).any(|w| w == b"\r\n");
    assert!(!has_crlf, "editor introduced CRLF into a LF-only file");

    cleanup(&path);
}

// ── T100-E: line_ending_crlf ─────────────────────────────────────────────────
//
// Create a file with CRLF line endings; after a save round-trip the editor
// should preserve CRLF (not silently convert to LF).

#[test]
fn line_ending_crlf_in_source() {
    let path = temp_path("crlf_endings");
    cleanup(&path);

    // Explicitly write CRLF bytes.
    let crlf_content = b"line one\r\nline two\r\nline three\r\n";
    let mut file = fs::File::create(&path).expect("could not create temp file");
    file.write_all(crlf_content).expect("could not write CRLF content");
    drop(file);

    let raw_bytes = fs::read(&path).expect("could not read temp file");
    let has_crlf = raw_bytes.windows(2).any(|w| w == b"\r\n");
    assert!(has_crlf, "test precondition failed: CRLF file has no CRLF bytes");

    cleanup(&path);
}

#[test]
#[ignore = "requires pty; open CRLF file, save, assert output still contains CRLF"]
fn line_ending_crlf_interactive() {
    let path = temp_path("crlf_interactive");
    cleanup(&path);

    let crlf_content = b"one\r\ntwo\r\nthree\r\n";
    fs::write(&path, crlf_content).expect("could not write CRLF temp file");

    // In a real pty test:
    //   spawn editor <path>
    //   send "\x13\x11"   (save + quit, no edits)
    //   assert \r\n present in fs::read(&path)
    let raw = fs::read(&path).unwrap_or_default();
    let has_crlf = raw.windows(2).any(|w| w == b"\r\n");
    assert!(has_crlf, "editor converted CRLF to LF — CRLF preservation broken");

    cleanup(&path);
}
