//! Feature 030 — interaction completeness (#53–#56). End-to-end tests driving
//! `App::handle_mouse_event` / `handle_action`.

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

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

fn press(a: &mut App, btn: MouseButton, col: u16, row: u16) {
    a.handle_mouse_event(MouseEvent {
        kind: MouseEventKind::Down(btn),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
}

// ── US2: double / triple-click selection ────────────────────────────────────

#[test]
fn double_click_selects_word_triple_selects_line() {
    let mut a = app();
    a.buffers[0].rope = EditorRope::from_str("foo bar baz\n");
    // Editor row 0 is terminal row 1 (single buffer). Click on "bar" (cols 4-6).
    press(&mut a, MouseButton::Left, 5, 1); // single → position
    press(&mut a, MouseButton::Left, 5, 1); // double → word
    assert_eq!(a.selection_text().as_deref(), Some("bar"));
    press(&mut a, MouseButton::Left, 5, 1); // triple → line
    assert_eq!(a.selection_text().as_deref(), Some("foo bar baz"));
}

#[test]
fn single_click_after_multiclick_clears_selection() {
    let mut a = app();
    a.buffers[0].rope = EditorRope::from_str("foo bar baz\n");
    press(&mut a, MouseButton::Left, 5, 1);
    press(&mut a, MouseButton::Left, 5, 1);
    assert!(a.buffers[0].selection.is_some());
    // A click on a different cell is a fresh single click → clears.
    press(&mut a, MouseButton::Left, 0, 1);
    assert!(a.buffers[0].selection.is_none());
}

// ── US1: in-dialog mouse — clicking an encoding list row selects it ──────────

#[test]
fn clicking_encoding_row_selects_it() {
    use edit::ui::dialog::encoding_dialog_rect;
    let mut a = app();
    a.set_encoding_select(0);
    let rect = encoding_dialog_rect(ratatui::layout::Rect::new(0, 0, 80, 24));
    // Click the third option row (index 2) inside the dialog interior.
    let row = rect.y + 1 + 2;
    let col = rect.x + 3;
    press(&mut a, MouseButton::Left, col, row);
    assert_eq!(a.encoding_select_row(), Some(2));
    // Focus moved to the list (primary control), not a button.
    assert!(a.interactive_focus_is_button().is_none());
}

// ── US3: right-click context menu ───────────────────────────────────────────

fn key(a: &mut App, action: Action) {
    a.handle_action(action).unwrap();
}

#[test]
fn right_click_opens_menu_and_copy_runs_and_esc_dismisses() {
    let mut a = app();
    a.buffers[0].rope = EditorRope::from_str("hello world\n");
    // Right-click in the editor opens the menu.
    press(&mut a, MouseButton::Right, 5, 5);
    assert!(a.context_menu().is_some());
    // Navigate to Copy (index 1) and activate via keyboard.
    key(&mut a, Action::MoveDown); // focus 1 = Copy
    key(&mut a, Action::InsertNewline); // activate
    assert!(a.context_menu().is_none(), "menu closes after activation");

    // Re-open and dismiss with Esc.
    press(&mut a, MouseButton::Right, 5, 5);
    assert!(a.context_menu().is_some());
    key(&mut a, Action::MenuClose);
    assert!(a.context_menu().is_none());

    // Re-open and dismiss with an outside left-click.
    press(&mut a, MouseButton::Right, 5, 5);
    assert!(a.context_menu().is_some());
    press(&mut a, MouseButton::Left, 79, 23);
    assert!(a.context_menu().is_none());
}

#[test]
fn right_click_does_not_open_over_a_modal() {
    let mut a = app();
    a.open_help(edit::app::HelpScreen::Help);
    press(&mut a, MouseButton::Right, 5, 5);
    assert!(a.context_menu().is_none(), "no context menu over a modal");
}
