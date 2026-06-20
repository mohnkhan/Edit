//! Feature 020 — plugin-manager dialog: Close button + focus ring.
//!
//! Verifies the [List, Close] ring, mouse activation, and that the existing list
//! keys still work. Uses an empty registry (the default in tests) which must
//! still let the user reach and activate Close.

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;

fn app() -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    a.pending_plugin_manager = true;
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

// ── T018 [US2] ring ─────────────────────────────────────────────────────────

#[test]
fn ring_is_list_close_and_wraps() {
    let mut a = app();
    assert_eq!(a.interactive_button_labels(), vec!["Close (Esc)"]);
    a.handle_action(Action::FocusNextField).unwrap(); // 0 (list) -> 1 (Close)
    assert_eq!(a.dialog_focus, 1);
    assert_eq!(a.interactive_focus_is_button(), Some(0));
    a.handle_action(Action::FocusNextField).unwrap(); // wraps -> list
    assert_eq!(a.dialog_focus, 0);
    assert_eq!(a.interactive_focus_is_button(), None);
}

// ── T019 [US1] mouse ────────────────────────────────────────────────────────

#[test]
fn click_close_button_closes_manager() {
    let mut a = app();
    let rect = a.interactive_dialog_rect().unwrap();
    let labels = a.interactive_button_labels();
    let rects = edit::ui::buttons::button_rects(rect, &labels);
    let close = rects[0];
    click(&mut a, close.x + 1, close.y + 1);
    assert!(!a.pending_plugin_manager, "Close closes the manager");
}

// ── T020 [US3] no regression ────────────────────────────────────────────────

#[test]
fn esc_closes_from_any_focus() {
    let mut a = app();
    a.handle_action(Action::FocusNextField).unwrap(); // focus Close
    a.handle_action(Action::MenuClose).unwrap();
    assert!(!a.pending_plugin_manager, "Esc closes from a button");
}

#[test]
fn enter_on_close_button_closes() {
    let mut a = app();
    a.handle_action(Action::FocusNextField).unwrap(); // focus Close
    a.handle_action(Action::InsertNewline).unwrap();
    assert!(!a.pending_plugin_manager, "Enter activates Close");
}

#[test]
fn empty_registry_still_reaches_close() {
    let mut a = app();
    assert!(
        a.plugin_host.registry.instances.is_empty(),
        "registry is empty in this test"
    );
    // Ring still has the Close stop reachable by Tab.
    a.handle_action(Action::FocusNextField).unwrap();
    assert_eq!(a.interactive_focus_is_button(), Some(0));
}

#[test]
fn space_while_list_focused_does_not_close() {
    let mut a = app();
    // Focus is on the list (stop 0). With no plugins, Space toggles nothing and
    // must NOT close the dialog (only the Close button does).
    a.handle_action(Action::InsertChar(' ')).unwrap();
    assert!(a.pending_plugin_manager, "list-focused Space keeps it open");
}
