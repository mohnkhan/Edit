//! Feature 021 — Help/About Close button.
//!
//! Verifies the Help and About overlays each expose a clickable Close button
//! (mouse-dismissable) while keyboard dismissal (Esc) still works, and that the
//! button label advertises its key.

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
fn close_button_label_carries_its_key() {
    assert!(
        edit::ui::buttons::HELP_CLOSE_LABEL.contains("Esc"),
        "Help Close button label advertises its key"
    );
}

#[test]
fn help_closes_on_close_button_click() {
    let mut a = app();
    a.handle_action(Action::Help).unwrap();
    assert!(a.help_screen().is_some(), "Help opened");
    let rects = edit::ui::help_close_button_rects(ratatui::layout::Rect::new(0, 0, 80, 24));
    assert!(!rects.is_empty(), "Close button is laid out");
    let b = rects[0];
    click(&mut a, b.x + 1, b.y + 1);
    assert!(a.help_screen().is_none(), "clicking Close dismisses Help");
}

#[test]
fn about_closes_on_close_button_click() {
    let mut a = app();
    a.handle_action(Action::About).unwrap();
    assert!(a.help_screen().is_some(), "About opened");
    let rects = edit::ui::help_close_button_rects(ratatui::layout::Rect::new(0, 0, 80, 24));
    let b = rects[0];
    click(&mut a, b.x + 1, b.y + 1);
    assert!(a.help_screen().is_none(), "clicking Close dismisses About");
}

#[test]
fn esc_still_closes_help_and_about() {
    let mut a = app();
    a.handle_action(Action::Help).unwrap();
    a.handle_action(Action::MenuClose).unwrap();
    assert!(a.help_screen().is_none(), "Esc still closes Help");

    a.handle_action(Action::About).unwrap();
    a.handle_action(Action::MenuClose).unwrap();
    assert!(a.help_screen().is_none(), "Esc still closes About");
}

#[test]
fn click_outside_close_button_keeps_help_open() {
    let mut a = app();
    a.handle_action(Action::Help).unwrap();
    click(&mut a, 0, 0); // far corner, not the button
    assert!(
        a.help_screen().is_some(),
        "non-button click keeps Help open"
    );
}
