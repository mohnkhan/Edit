//! Feature 028 — UX crash-safety and keyboard navigation hardening.
//!
//! Drives `App::handle_action` to verify the keyboard-navigation fixes end to end:
//! Save-As typing reaches the field, arrow keys move between dialog buttons, Help
//! scrolls/closes from the keyboard, Home/End move the cursor, and lists page.

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;
use edit::ui::file_browser::{BrowseMode, FileBrowser};

fn app() -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    a
}

// ── US3: Save-As typing reaches the field ───────────────────────────────────

#[test]
fn save_browser_typing_accumulates_in_filename() {
    let mut a = app();
    // Simulate stale focus left on a button by a previously-used dialog.
    a.dialog_focus = 2;
    a.dialog_focus_init = false;
    a.open_file_browser(FileBrowser::open(
        std::path::PathBuf::from("."),
        BrowseMode::Save,
    ));

    for c in "report.txt".chars() {
        a.handle_action(Action::InsertChar(c)).unwrap();
    }

    assert_eq!(
        a.file_browser().unwrap().filename,
        "report.txt",
        "typed characters must reach the filename field"
    );
}

// ── US4: arrow keys move between dialog buttons ──────────────────────────────

#[test]
fn arrow_keys_navigate_confirm_dialog_buttons() {
    let mut a = app();
    // Open the save-before-quit prompt the real way (modify, then quit):
    // Save / Discard / Cancel; default focus = Cancel(2).
    a.handle_action(Action::InsertChar('x')).unwrap();
    a.handle_action(Action::Quit).unwrap();
    a.handle_action(Action::MoveRight).unwrap();
    assert_eq!(a.dialog_focus, 0);
    a.handle_action(Action::MoveLeft).unwrap();
    assert_eq!(a.dialog_focus, 2, "wraps to the last button");
}

// ── US5: Help scrolls and dismisses from the keyboard ───────────────────────

#[test]
fn help_overlay_scrolls_and_closes_from_keyboard() {
    let mut a = app();
    a.handle_action(Action::Help).unwrap();
    assert!(a.pending_help.is_some());
    a.handle_action(Action::MovePageDown).unwrap();
    assert!(a.help_scroll > 0, "PageDown scrolls Help");
    a.handle_action(Action::MoveLineStart).unwrap();
    assert_eq!(a.help_scroll, 0, "Home returns to the top");
    a.handle_action(Action::MenuClose).unwrap();
    assert!(a.pending_help.is_none(), "Esc closes Help");
}

// ── US6: Home/End move the editor cursor ────────────────────────────────────

#[test]
fn home_end_move_editor_cursor() {
    use edit::buffer::rope::EditorRope;
    let mut a = app();
    a.buffers[0].rope = EditorRope::from_str("abcdef\n");
    a.handle_action(Action::MoveLineEnd).unwrap();
    assert_eq!(a.buffers[0].cursor.grapheme_col, 6);
    a.handle_action(Action::MoveLineStart).unwrap();
    assert_eq!(a.buffers[0].cursor.grapheme_col, 0);
}
