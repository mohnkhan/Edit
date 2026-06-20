//! Feature 029 — UX completeness hardening (round 2).
//!
//! End-to-end regression tests driving `App::handle_action` for the round-2 fixes:
//! SavePrompt Esc, save feedback, copy/read-only feedback, Ctrl+W close, and the
//! gutter-aware click mapping. Crash/width/encoding fixes are unit-tested inline.

use edit::app::App;
use edit::buffer::rope::EditorRope;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;

fn app() -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    a
}

#[test]
fn save_prompt_esc_cancels() {
    let mut a = app();
    a.pending_save_prompt = true;
    a.handle_action(Action::MenuClose).unwrap();
    assert!(!a.pending_save_prompt);
    assert!(a.running);
}

#[test]
fn ctrl_w_close_via_action_closes_buffer() {
    let mut a = app();
    // Two buffers → closing one leaves one.
    a.buffers.push(edit::buffer::Buffer::new_empty());
    a.active_idx = 1;
    a.handle_action(Action::Close).unwrap();
    assert_eq!(a.buffers.len(), 1);
}

#[test]
fn readonly_typing_reports_message() {
    let mut a = app();
    a.buffers[0].rope = EditorRope::from_str("abc\n");
    a.buffers[0].readonly = true;
    a.handle_action(Action::InsertChar('z')).unwrap();
    assert_eq!(a.status_message.as_deref(), Some("Buffer is read-only"));
    assert_eq!(a.buffers[0].rope.line_slice(0), "abc");
}

#[test]
fn click_with_line_numbers_lands_under_pointer() {
    let mut a = app();
    a.config.line_numbers = true;
    a.buffers[0].rope = EditorRope::from_str("abcdefghij\n");
    // Gutter = 4; click terminal col 7 → text column 3.
    a.handle_mouse_click(7, 1);
    assert_eq!(a.buffers[0].cursor.grapheme_col, 3);
}

#[test]
fn wide_chars_use_shared_width() {
    // The shared width function: CJK = 2, combining = 0.
    assert_eq!(edit::ui::width::display_width("世"), 2);
    assert_eq!(edit::ui::width::str_width("e\u{0301}"), 1);
}
