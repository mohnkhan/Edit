// Integration tests for Feature 004: Save-As Encoding Selection UI.
//
// These tests drive the App library API directly (not the binary) to verify
// that the encoding-select dialog writes the correct bytes to disk and that
// cancellation, persistence, and error-revert behaviour match the spec.
//
// Run with:
//   cargo test --test encoding_select

use std::env;
use std::fs;
use std::path::PathBuf;

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn temp_path(name: &str) -> PathBuf {
    env::temp_dir().join(format!("edit_enc_sel_{}_{}", name, std::process::id()))
}

fn make_app_with_file(path: PathBuf) -> App {
    App::new(Config::default(), vec![path], EncodingId::Utf8, None, None)
}

fn make_unnamed_app() -> App {
    App::new(Config::default(), vec![], EncodingId::Utf8, None, None)
}

// ── T023: UTF-16 LE round-trip ────────────────────────────────────────────────

#[test]
fn test_save_utf8_file_as_utf16le() {
    let path = temp_path("utf16le");
    fs::write(&path, b"Hello, World!").unwrap();

    let mut app = make_app_with_file(path.clone());

    // Open dialog → pre-selects UTF-8 at index 0.
    app.handle_action(Action::SaveAsEncoding).unwrap();
    assert_eq!(app.encoding_select_row(), Some(0));

    // Navigate one step down → UTF-16 LE at index 1.
    app.handle_action(Action::MoveDown).unwrap();
    assert_eq!(app.encoding_select_row(), Some(1));

    // Confirm selection.
    app.handle_action(Action::InsertNewline).unwrap();

    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        &bytes[0..2],
        &[0xFF, 0xFE],
        "expected UTF-16 LE BOM; got {:02X?}",
        &bytes[0..2]
    );
    assert_eq!(
        app.active_buffer().encoding,
        EncodingId::Utf16Le,
        "buffer encoding must be Utf16Le after save"
    );

    let _ = fs::remove_file(&path);
}

// ── T024: UTF-16 BE round-trip ────────────────────────────────────────────────

#[test]
fn test_save_utf8_file_as_utf16be() {
    let path = temp_path("utf16be");
    fs::write(&path, b"Hello, World!").unwrap();

    let mut app = make_app_with_file(path.clone());

    app.handle_action(Action::SaveAsEncoding).unwrap();
    // Navigate two steps → UTF-16 BE at index 2.
    app.handle_action(Action::MoveDown).unwrap();
    app.handle_action(Action::MoveDown).unwrap();
    assert_eq!(app.encoding_select_row(), Some(2));

    app.handle_action(Action::InsertNewline).unwrap();

    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        &bytes[0..2],
        &[0xFE, 0xFF],
        "expected UTF-16 BE BOM; got {:02X?}",
        &bytes[0..2]
    );
    assert_eq!(app.active_buffer().encoding, EncodingId::Utf16Be);

    let _ = fs::remove_file(&path);
}

// ── T025: Cancel leaves file unchanged ────────────────────────────────────────

#[test]
fn test_cancel_leaves_file_unchanged() {
    let path = temp_path("cancel");
    let original = b"Unchanged content";
    fs::write(&path, original).unwrap();

    let mut app = make_app_with_file(path.clone());

    let pre_bytes = fs::read(&path).unwrap();
    let pre_encoding = app.active_buffer().encoding;

    // Open dialog then cancel immediately.
    app.handle_action(Action::SaveAsEncoding).unwrap();
    app.handle_action(Action::MenuClose).unwrap();

    assert_eq!(
        app.encoding_select_row(),
        None,
        "dialog must be closed after cancel"
    );
    assert_eq!(
        app.active_buffer().encoding,
        pre_encoding,
        "encoding must not change on cancel"
    );

    let post_bytes = fs::read(&path).unwrap();
    assert_eq!(
        pre_bytes, post_bytes,
        "file must be byte-for-byte identical after cancel"
    );

    let _ = fs::remove_file(&path);
}

// ── T026: Encoding persists for subsequent saves ──────────────────────────────

#[test]
fn test_encoding_persists_on_regular_save() {
    let path = temp_path("persist");
    fs::write(&path, b"Persist me").unwrap();

    let mut app = make_app_with_file(path.clone());

    // Select UTF-16 LE via dialog.
    app.do_save_as_encoding(EncodingId::Utf16Le);

    // Now trigger an ordinary Save — must still use UTF-16 LE.
    app.handle_action(Action::Save).unwrap();

    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        &bytes[0..2],
        &[0xFF, 0xFE],
        "second save must still write UTF-16 LE BOM"
    );

    let _ = fs::remove_file(&path);
}

// ── T027: I/O error reverts encoding ─────────────────────────────────────────

#[test]
fn test_io_error_reverts_encoding() {
    // Point the buffer at a path inside a non-existent directory.
    // The write will fail because the parent directory doesn't exist,
    // which is a cross-platform way to guarantee an I/O error.
    let bad_path: PathBuf = env::temp_dir()
        .join("edit_nonexistent_dir_abc123xyz")
        .join("cannot_create.txt");

    // We need a real file to open so App can populate the buffer.
    // Use an existing temp file as the source, then manually override
    // the buffer path to the bad location.
    let src = temp_path("ioerr_src");
    fs::write(&src, b"Test content").unwrap();

    let mut app = make_app_with_file(src.clone());
    // Redirect the buffer's path to the bad (unwritable) location.
    app.buffers[app.active_idx].path = Some(bad_path.clone());

    // Attempt to save as UTF-16 BE — must fail and revert encoding.
    app.do_save_as_encoding(EncodingId::Utf16Be);

    assert_eq!(
        app.active_buffer().encoding,
        EncodingId::Utf8,
        "encoding must revert to Utf8 after a failed write"
    );
    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.contains("Save failed"),
        "status must contain 'Save failed'; got: {msg:?}"
    );

    let _ = fs::remove_file(&src);
}

// ── T028: Unnamed buffer pending encoding held until filename confirm ──────────

#[test]
fn test_new_buffer_pending_encoding_held() {
    let path = temp_path("pending");
    let mut app = make_unnamed_app();

    // Verify buffer has no path.
    assert!(
        app.active_buffer().path.is_none(),
        "buffer must be unnamed (no path)"
    );

    // Trigger encoding dialog and navigate to CP437 (index 3).
    app.handle_action(Action::SaveAsEncoding).unwrap();
    app.handle_action(Action::MoveDown).unwrap(); // 0 → 1 (UTF-16 LE)
    app.handle_action(Action::MoveDown).unwrap(); // 1 → 2 (UTF-16 BE)
    app.handle_action(Action::MoveDown).unwrap(); // 2 → 3 (CP437)
    app.handle_action(Action::InsertNewline).unwrap();

    // Case B: pending encoding must be held, dialog closed.
    assert_eq!(
        app.encoding_select_row(),
        None,
        "dialog must be closed after confirm"
    );
    assert_eq!(
        app.pending_save_as_encoding,
        Some(EncodingId::Cp437),
        "pending encoding must be Cp437"
    );

    // Simulate user providing a filename → encoding must be applied.
    let result = app.handle_save_as(path.clone());
    assert_eq!(
        app.pending_save_as_encoding, None,
        "pending encoding must be consumed after handle_save_as"
    );
    assert_eq!(
        app.active_buffer().encoding,
        EncodingId::Cp437,
        "buffer encoding must be Cp437 after handle_save_as"
    );

    let _ = result; // write may succeed or fail depending on buffer content
    let _ = fs::remove_file(&path);
}
