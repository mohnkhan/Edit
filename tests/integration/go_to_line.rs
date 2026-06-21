//! Feature 025 — Go to Line.
//!
//! Drives the `edit` library `App` (`handle_action`) to verify the prompt opens,
//! jumps to a line (scrolled into view), clamps out-of-range, cancels, ignores
//! invalid input, and never edits the buffer while open.

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;

fn app_with_lines(n: usize) -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    for _ in 1..n {
        a.handle_action(Action::InsertNewline).unwrap();
    }
    // Reset to the top for deterministic assertions.
    a.buffers[0].cursor.line = 0;
    a.buffers[0].cursor.grapheme_col = 0;
    a.buffers[0].scroll_offset = (0, 0);
    a
}

fn typ(a: &mut App, s: &str) {
    for c in s.chars() {
        a.handle_action(Action::InsertChar(c)).unwrap();
    }
}

#[test]
fn ctrl_g_opens_and_jumps_scrolling_into_view() {
    let mut a = app_with_lines(100);
    a.handle_action(Action::GoToLine).unwrap();
    assert!(a.pending_goto_line.is_some(), "prompt opened");
    typ(&mut a, "50");
    a.handle_action(Action::InsertNewline).unwrap();
    assert!(a.pending_goto_line.is_none(), "prompt closed on Enter");
    assert_eq!(a.buffers[0].cursor.line, 49, "1-based line 50 → index 49");
    assert_eq!(a.buffers[0].cursor.grapheme_col, 0, "cursor at column 1");
    let scroll = a.buffers[0].scroll_offset.0;
    assert!(scroll > 0, "view scrolled to bring line 50 into view");
    assert!(scroll <= 49, "line 49 is at/after the scroll top");
}

#[test]
fn over_range_clamps_to_last_line() {
    let mut a = app_with_lines(100);
    a.handle_action(Action::GoToLine).unwrap();
    typ(&mut a, "999999");
    a.handle_action(Action::InsertNewline).unwrap();
    assert_eq!(a.buffers[0].cursor.line, 99, "clamped to the last line");
}

#[test]
fn zero_clamps_to_first_line() {
    let mut a = app_with_lines(100);
    a.buffers[0].cursor.line = 40;
    a.handle_action(Action::GoToLine).unwrap();
    typ(&mut a, "0");
    a.handle_action(Action::InsertNewline).unwrap();
    assert_eq!(a.buffers[0].cursor.line, 0, "0 clamps to line 1");
}

#[test]
fn esc_cancels_without_moving() {
    let mut a = app_with_lines(100);
    a.buffers[0].cursor.line = 12;
    a.handle_action(Action::GoToLine).unwrap();
    typ(&mut a, "80");
    a.handle_action(Action::MenuClose).unwrap();
    assert!(a.pending_goto_line.is_none(), "Esc closed the prompt");
    assert_eq!(a.buffers[0].cursor.line, 12, "Esc did not move the cursor");
}

#[test]
fn empty_enter_does_not_move() {
    let mut a = app_with_lines(100);
    a.buffers[0].cursor.line = 7;
    a.handle_action(Action::GoToLine).unwrap();
    a.handle_action(Action::InsertNewline).unwrap(); // empty field
    assert!(a.pending_goto_line.is_none());
    assert_eq!(a.buffers[0].cursor.line, 7, "empty entry → no movement");
}

#[test]
fn backspace_edits_and_non_digits_rejected_buffer_untouched() {
    let mut a = app_with_lines(100);
    let before = a.buffers[0].rope.to_string();
    a.handle_action(Action::GoToLine).unwrap();
    typ(&mut a, "9");
    typ(&mut a, "a"); // non-digit ignored by the field
    a.handle_action(Action::Backspace).unwrap(); // remove the '9'
    typ(&mut a, "3");
    assert_eq!(
        a.pending_goto_line.as_deref(),
        Some("3"),
        "field is digits-only"
    );
    assert_eq!(
        a.buffers[0].rope.to_string(),
        before,
        "buffer is not modified while the prompt is open"
    );
    a.handle_action(Action::InsertNewline).unwrap();
    assert_eq!(a.buffers[0].cursor.line, 2, "jumped to line 3");
}

#[test]
fn go_to_line_does_not_open_over_another_modal() {
    let mut a = app_with_lines(10);
    a.handle_action(Action::Find).unwrap(); // open Find dialog
    assert!(a.find_replace().is_some());
    a.handle_action(Action::GoToLine).unwrap();
    assert!(
        a.pending_goto_line.is_none(),
        "Go to Line does not open while another modal is open"
    );
}
