//! Integration tests for Feature 017: visible text selection.
//!
//! Verifies Shift-select keyboard selection, clear-on-plain-move, typing
//! replacing a selection, and mouse drag selection, by driving the `edit`
//! library `App` and inspecting `buffer.selection`.

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;

fn app_with(text: &str) -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    for c in text.chars() {
        a.handle_action(Action::InsertChar(c)).unwrap();
    }
    a
}

/// (line, gcol) ordered selection bounds, or None.
fn sel_bounds(app: &App) -> Option<((usize, usize), (usize, usize))> {
    let s = app.active_buffer().selection.as_ref()?;
    let a = (s.anchor.line, s.anchor.grapheme_col);
    let b = (s.active.line, s.active.grapheme_col);
    Some(if a <= b { (a, b) } else { (b, a) })
}

fn mouse(app: &mut App, kind: MouseEventKind, col: u16, row: u16) {
    app.handle_mouse_event(MouseEvent {
        kind,
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
}

// ── US1: Select All highlights whole buffer (state) ──────────────────────────

#[test]
fn select_all_selects_entire_buffer() {
    let app = {
        let mut a = app_with("hello");
        a.handle_action(Action::SelectAll).unwrap();
        a
    };
    assert_eq!(sel_bounds(&app), Some(((0, 0), (0, 5))));
}

// ── US2: Shift-select + clear + replace ──────────────────────────────────────

#[test]
fn shift_left_extends_selection() {
    let mut a = app_with("hello"); // cursor at (0,5)
    a.handle_action(Action::SelectLeft).unwrap();
    a.handle_action(Action::SelectLeft).unwrap();
    assert_eq!(sel_bounds(&a), Some(((0, 3), (0, 5))), "two chars selected");
}

#[test]
fn plain_move_clears_selection() {
    let mut a = app_with("hello");
    a.handle_action(Action::SelectLeft).unwrap();
    assert!(sel_bounds(&a).is_some());
    a.handle_action(Action::MoveLeft).unwrap(); // plain move
    assert!(sel_bounds(&a).is_none(), "plain move clears selection");
}

#[test]
fn shift_home_selects_to_line_start() {
    let mut a = app_with("hello"); // cursor (0,5)
    a.handle_action(Action::SelectLineStart).unwrap();
    assert_eq!(sel_bounds(&a), Some(((0, 0), (0, 5))));
}

#[test]
fn typing_replaces_selection() {
    let mut a = app_with("hello");
    a.handle_action(Action::SelectLineStart).unwrap(); // select all of "hello"
    a.handle_action(Action::InsertChar('Z')).unwrap();
    assert_eq!(a.active_buffer().rope.to_string(), "Z");
    assert!(sel_bounds(&a).is_none());
}

#[test]
fn backspace_deletes_selection() {
    let mut a = app_with("hello");
    a.handle_action(Action::SelectLeft).unwrap(); // select "o"
    a.handle_action(Action::SelectLeft).unwrap(); // select "lo"
    a.handle_action(Action::Backspace).unwrap();
    assert_eq!(a.active_buffer().rope.to_string(), "hel");
    assert!(sel_bounds(&a).is_none());
}

// ── US3: Mouse drag selection ────────────────────────────────────────────────

#[test]
fn mouse_drag_selects_range() {
    let mut a = app_with("hello world");
    // Press at col 0 (line start), drag to col 5.
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), 0, 1);
    assert!(sel_bounds(&a).is_none(), "press alone makes no selection");
    mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), 5, 1);
    assert_eq!(sel_bounds(&a), Some(((0, 0), (0, 5))), "drag selects 0..5");
}

#[test]
fn single_click_clears_selection() {
    let mut a = app_with("hello world");
    a.handle_action(Action::SelectAll).unwrap();
    assert!(sel_bounds(&a).is_some());
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), 2, 1); // single click, no drag
    assert!(sel_bounds(&a).is_none(), "single click clears selection");
}
