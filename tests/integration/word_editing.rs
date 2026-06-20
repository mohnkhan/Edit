//! Feature 032 — word-wise navigation, selection, and deletion. End-to-end via
//! `App::handle_action` (the real keybinding → action path).

use edit::app::App;
use edit::buffer::rope::EditorRope;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;

fn app_with(text: &str) -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    a.buffers[0].rope = EditorRope::from_str(text);
    a.active_idx = 0;
    a
}

fn put(a: &mut App, line: usize, gcol: usize) {
    a.buffers[0].cursor = edit::buffer::CursorPos {
        line,
        grapheme_col: gcol,
        visual_col: gcol,
    };
}

// ── US1: move by word ───────────────────────────────────────────────────────

#[test]
fn move_word_actions_navigate_and_cross_lines() {
    let mut a = app_with("foo bar\nbaz\n");
    put(&mut a, 0, 0);
    a.handle_action(Action::MoveWordRight).unwrap();
    assert_eq!(a.buffers[0].cursor.grapheme_col, 4); // start of "bar"
    a.handle_action(Action::MoveWordRight).unwrap(); // end of line → ...
    a.handle_action(Action::MoveWordRight).unwrap(); // ... next line
    assert_eq!(a.buffers[0].cursor.line, 1);
    a.handle_action(Action::MoveWordLeft).unwrap();
    assert_eq!(
        a.buffers[0].cursor.line, 0,
        "Ctrl+Left crosses back to line 0"
    );
}

// ── US2: select by word ─────────────────────────────────────────────────────

#[test]
fn select_word_actions_build_selection() {
    let mut a = app_with("alpha beta gamma\n");
    put(&mut a, 0, 0);
    a.handle_action(Action::SelectWordRight).unwrap();
    a.handle_action(Action::SelectWordRight).unwrap();
    assert_eq!(a.selection_text().as_deref(), Some("alpha beta "));
}

// ── US3: delete by word ─────────────────────────────────────────────────────

#[test]
fn delete_word_actions_and_undo() {
    let mut a = app_with("alpha beta gamma\n");
    put(&mut a, 0, 11); // start of "gamma"
    a.handle_action(Action::DeleteWordLeft).unwrap(); // deletes "beta "
    assert_eq!(a.buffers[0].rope.line_slice(0), "alpha gamma");
    a.handle_action(Action::Undo).unwrap();
    assert_eq!(a.buffers[0].rope.line_slice(0), "alpha beta gamma");
    // Forward delete from start removes "alpha ".
    put(&mut a, 0, 0);
    a.handle_action(Action::DeleteWordRight).unwrap();
    assert_eq!(a.buffers[0].rope.line_slice(0), "beta gamma");
}
