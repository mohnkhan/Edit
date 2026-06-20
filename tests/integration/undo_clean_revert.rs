//! Integration tests for Feature 014: undo-to-clean Modified tracking + Revert.
//!
//! Drives the `edit` library `App` (`handle_action`) end-to-end to verify the
//! `[Modified]` state tracks the saved baseline through undo/redo (with no
//! false-clean after divergent edits) and that File ▸ Revert restores the
//! on-disk version with the right guards.

use std::path::{Path, PathBuf};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;

fn temp_file(tag: &str, contents: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "edit_rev_{}_{}_{}.txt",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&p, contents).unwrap();
    p
}

fn app_with(path: &Path) -> App {
    App::new(
        Config::default(),
        vec![path.to_path_buf()],
        EncodingId::Utf8,
        None,
        None,
    )
}

fn modified(app: &App) -> bool {
    app.active_buffer().modified
}

// ── US1: undo back to saved clears Modified ───────────────────────────────────

#[test]
fn undo_to_saved_clears_modified_redo_restores() {
    let path = temp_file("u1", "hello\n");
    let mut app = app_with(&path);
    assert!(!modified(&app), "freshly opened buffer is clean");

    app.handle_action(Action::InsertChar('x')).unwrap();
    assert!(modified(&app), "edit marks modified");

    app.handle_action(Action::Undo).unwrap();
    assert!(!modified(&app), "undo back to opened content is clean");

    app.handle_action(Action::Redo).unwrap();
    assert!(modified(&app), "redo away from saved is modified");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn save_then_undo_to_save_point_is_clean() {
    let path = temp_file("u2", "");
    let mut app = app_with(&path);
    app.handle_action(Action::InsertChar('a')).unwrap();
    app.handle_action(Action::Save).unwrap(); // baseline at "a"
    assert!(!modified(&app), "clean immediately after save");

    app.handle_action(Action::InsertChar('b')).unwrap();
    assert!(modified(&app));
    app.handle_action(Action::Undo).unwrap(); // back to "a" (saved)
    assert!(!modified(&app), "undo back to the save point is clean");
    app.handle_action(Action::Undo).unwrap(); // back to "" (before save)
    assert!(modified(&app), "undo before the save point is modified");

    let _ = std::fs::remove_file(&path);
}

// ── US2: no false clean after divergent edits ─────────────────────────────────

#[test]
fn divergent_edit_after_save_is_never_falsely_clean() {
    let path = temp_file("u3", "");
    let mut app = app_with(&path);
    app.handle_action(Action::InsertChar('a')).unwrap();
    app.handle_action(Action::InsertChar('b')).unwrap();
    app.handle_action(Action::Save).unwrap(); // baseline at "ab" (depth 2)
    app.handle_action(Action::Undo).unwrap(); // back to "a" (depth 1)
    assert!(modified(&app));
    app.handle_action(Action::InsertChar('c')).unwrap(); // divergent: discards "b" branch

    assert!(modified(&app), "divergent edit is modified");
    // The saved point lived in the discarded branch — it must be unreachable, so
    // no amount of undo/redo can show clean.
    for _ in 0..5 {
        app.handle_action(Action::Undo).unwrap();
        assert!(modified(&app), "never falsely clean after divergence");
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn undo_clean_then_different_edit_is_modified() {
    let path = temp_file("u4", "");
    let mut app = app_with(&path);
    app.handle_action(Action::InsertChar('a')).unwrap();
    app.handle_action(Action::Undo).unwrap(); // clean (empty == opened)
    assert!(!modified(&app));
    app.handle_action(Action::InsertChar('z')).unwrap();
    assert!(modified(&app), "a new edit after clean is modified");
    let _ = std::fs::remove_file(&path);
}

// ── US3: Revert ───────────────────────────────────────────────────────────────

#[test]
fn revert_confirm_restores_disk_and_clears_modified() {
    let path = temp_file("r1", "original\n");
    let mut app = app_with(&path);
    app.handle_action(Action::InsertChar('X')).unwrap();
    assert!(modified(&app));

    app.handle_action(Action::Revert).unwrap();
    assert_eq!(
        app.pending_revert_confirm,
        Some(0),
        "modified buffer asks to confirm"
    );
    app.handle_action(Action::InsertNewline).unwrap(); // confirm

    assert_eq!(app.pending_revert_confirm, None);
    assert!(!modified(&app), "buffer clean after revert");
    assert_eq!(app.active_buffer().rope.to_string(), "original\n");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn revert_cancel_keeps_changes() {
    let path = temp_file("r2", "original\n");
    let mut app = app_with(&path);
    app.handle_action(Action::InsertChar('X')).unwrap();
    app.handle_action(Action::Revert).unwrap();
    assert_eq!(app.pending_revert_confirm, Some(0));
    app.handle_action(Action::InsertChar('n')).unwrap(); // cancel

    assert_eq!(app.pending_revert_confirm, None);
    assert!(modified(&app), "edits preserved on cancel");
    assert!(app.active_buffer().rope.to_string().starts_with('X'));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn revert_clean_buffer_reloads_without_confirm() {
    let path = temp_file("r3", "data\n");
    let mut app = app_with(&path);
    // Change the file on disk behind the editor, then revert a clean buffer.
    std::fs::write(&path, "data2\n").unwrap();
    app.handle_action(Action::Revert).unwrap();
    assert_eq!(
        app.pending_revert_confirm, None,
        "no confirm for a clean buffer"
    );
    assert_eq!(app.active_buffer().rope.to_string(), "data2\n");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn revert_pathless_buffer_is_noop_with_notice() {
    let mut app = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    app.handle_action(Action::InsertChar('q')).unwrap();
    app.handle_action(Action::Revert).unwrap();
    assert_eq!(app.pending_revert_confirm, None);
    assert!(modified(&app), "pathless buffer unchanged");
    assert_eq!(app.active_buffer().rope.to_string(), "q");
    assert!(app
        .status_message
        .as_deref()
        .unwrap_or("")
        .to_lowercase()
        .contains("revert"));
}

#[test]
fn revert_reload_failure_leaves_buffer_unchanged() {
    let path = temp_file("r5", "keep\n");
    let mut app = app_with(&path);
    app.handle_action(Action::InsertChar('Z')).unwrap();
    std::fs::remove_file(&path).unwrap(); // make the on-disk file unreadable
    app.handle_action(Action::Revert).unwrap();
    assert_eq!(app.pending_revert_confirm, Some(0));
    app.handle_action(Action::InsertNewline).unwrap(); // confirm → reload fails

    // Buffer content is preserved on a failed reload.
    assert!(app.active_buffer().rope.to_string().starts_with('Z'));
}
