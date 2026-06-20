//! Integration tests for Feature 016: focusable dialog buttons.
//!
//! Drives the `edit` library `App` through the unsaved-changes (save) prompt to
//! verify boxed-button focus default, Tab/Shift+Tab + Enter activation, letter
//! shortcuts coexisting, and mouse-click activation — using the same geometry the
//! renderer draws with (`App::button_dialog_rect` + `ui::buttons`).

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;

fn app() -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    a
}

/// Open the unsaved-changes prompt (edit + quit on a modified buffer).
fn open_save_prompt() -> App {
    let mut a = app();
    a.handle_action(Action::InsertChar('x')).unwrap();
    a.handle_action(Action::Quit).unwrap();
    assert!(a.pending_save_prompt, "save prompt should be open");
    a
}

fn click(app: &mut App, col: u16, row: u16) {
    app.handle_mouse_event(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
}

#[test]
fn save_prompt_buttons_and_safe_default() {
    let a = open_save_prompt();
    assert_eq!(a.dialog_button_labels(), vec!["Save", "Discard", "Cancel"]);
    assert_eq!(
        a.dialog_default_focus(),
        2,
        "default focus is the safe Cancel"
    );
}

#[test]
fn enter_activates_default_cancel() {
    let mut a = open_save_prompt();
    a.handle_action(Action::InsertNewline).unwrap(); // Enter on default (Cancel)
    assert!(!a.pending_save_prompt, "prompt closed");
    assert!(a.running, "Cancel keeps the editor running");
}

#[test]
fn shift_tab_then_enter_reaches_discard() {
    let mut a = open_save_prompt();
    // Default focus = Cancel(2); Shift+Tab → Discard(1).
    a.handle_action(Action::FocusPrevField).unwrap();
    a.handle_action(Action::InsertNewline).unwrap();
    assert!(!a.running, "Discard quits");
}

#[test]
fn tab_wraps_through_all_three() {
    let mut a = open_save_prompt();
    // First action initializes focus to default (2); Tab → 0 → 1 → 2.
    a.handle_action(Action::FocusNextField).unwrap(); // 2 -> 0
    a.handle_action(Action::FocusNextField).unwrap(); // 0 -> 1
    assert_eq!(a.dialog_focus, 1);
    a.handle_action(Action::FocusNextField).unwrap(); // 1 -> 2
    assert_eq!(a.dialog_focus, 2, "wraps back to start");
}

#[test]
fn letter_shortcuts_still_work() {
    let mut a = open_save_prompt();
    a.handle_action(Action::InsertChar('c')).unwrap(); // Cancel shortcut
    assert!(!a.pending_save_prompt);
    assert!(a.running);

    let mut b = open_save_prompt();
    b.handle_action(Action::InsertChar('d')).unwrap(); // Discard shortcut
    assert!(!b.running);
}

#[test]
fn click_discard_button_activates_it() {
    let mut a = open_save_prompt();
    let rect = a.button_dialog_rect().expect("dialog rect");
    let labels = a.dialog_button_labels();
    let rects = edit::ui::buttons::button_rects(rect, &labels);
    let d = rects[1]; // Discard
    click(&mut a, d.x + 1, d.y + 1);
    assert!(!a.running, "clicking Discard quits");
}

#[test]
fn click_cancel_button_closes_prompt() {
    let mut a = open_save_prompt();
    let rect = a.button_dialog_rect().unwrap();
    let labels = a.dialog_button_labels();
    let rects = edit::ui::buttons::button_rects(rect, &labels);
    let c = rects[2]; // Cancel
    click(&mut a, c.x + 1, c.y + 1);
    assert!(!a.pending_save_prompt);
    assert!(a.running);
}

#[test]
fn click_outside_cancels() {
    let mut a = open_save_prompt();
    click(&mut a, 0, 0); // far outside the centered dialog
    assert!(!a.pending_save_prompt, "outside click cancels");
    assert!(a.running);
}

#[test]
fn click_inside_not_on_button_is_inert() {
    let mut a = open_save_prompt();
    let rect = a.button_dialog_rect().unwrap();
    // Top-left interior cell (body area), not a button.
    click(&mut a, rect.x + 1, rect.y + 1);
    assert!(
        a.pending_save_prompt,
        "inside-not-on-button keeps prompt open"
    );
}
