//! Feature 020 — encoding-select dialog: OK/Cancel buttons + focus ring.
//!
//! Drives the `edit` library `App` to verify the [List, OK, Cancel] ring, mouse
//! activation, and that existing list keys still work with zero regression.

use std::env;
use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;

fn temp_path(name: &str) -> PathBuf {
    env::temp_dir().join(format!("edit_enc_btn_{}_{}", name, std::process::id()))
}

/// App with a real backing file so `SaveAsEncoding` opens the dialog directly.
fn app_with_file(name: &str) -> (App, PathBuf) {
    let path = temp_path(name);
    fs::write(&path, b"hello").unwrap();
    let mut a = App::new(
        Config::default(),
        vec![path.clone()],
        EncodingId::Utf8,
        None,
        None,
    );
    a.terminal_size = (80, 24);
    a.handle_action(Action::SaveAsEncoding).unwrap();
    assert!(a.pending_encoding_select.is_some(), "encoding dialog open");
    (a, path)
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

// ── T010 [US2] focus ring ──────────────────────────────────────────────────

#[test]
fn ring_is_list_ok_cancel_and_wraps() {
    let (mut a, _p) = app_with_file("ring");
    assert_eq!(
        a.interactive_button_labels(),
        vec!["OK (Enter)", "Cancel (Esc)"]
    );
    // ensure_dialog_focus runs at the top of handle_action; first call inits to 0.
    a.handle_action(Action::FocusNextField).unwrap(); // 0 (list) -> 1 (OK)
    assert_eq!(a.dialog_focus, 1);
    assert_eq!(a.interactive_focus_is_button(), Some(0));
    a.handle_action(Action::FocusNextField).unwrap(); // -> 2 (Cancel)
    assert_eq!(a.interactive_focus_is_button(), Some(1));
    a.handle_action(Action::FocusNextField).unwrap(); // wraps -> 0 (list)
    assert_eq!(a.dialog_focus, 0);
    assert_eq!(a.interactive_focus_is_button(), None, "back on the list");
}

// ── T011 [US1] mouse activation ─────────────────────────────────────────────

#[test]
fn click_ok_applies_selected_encoding() {
    let (mut a, path) = app_with_file("ok");
    // Highlight UTF-16 LE (index 1) via the list.
    a.handle_action(Action::MoveDown).unwrap();
    assert_eq!(a.pending_encoding_select, Some(1));
    let rect = a.interactive_dialog_rect().unwrap();
    let labels = a.interactive_button_labels();
    let rects = edit::ui::buttons::button_rects(rect, &labels);
    let ok = rects[0];
    click(&mut a, ok.x + 1, ok.y + 1);
    assert!(a.pending_encoding_select.is_none(), "dialog closed by OK");
    // OK == Enter on the list: UTF-16 LE BOM written.
    let bytes = fs::read(&path).unwrap();
    assert_eq!(&bytes[0..2], &[0xFF, 0xFE], "UTF-16 LE BOM written");
    let _ = fs::remove_file(&path);
}

#[test]
fn click_cancel_closes_without_change() {
    let (mut a, path) = app_with_file("cancel");
    let before = fs::read(&path).unwrap();
    let rect = a.interactive_dialog_rect().unwrap();
    let labels = a.interactive_button_labels();
    let rects = edit::ui::buttons::button_rects(rect, &labels);
    let cancel = rects[1];
    click(&mut a, cancel.x + 1, cancel.y + 1);
    assert!(
        a.pending_encoding_select.is_none(),
        "dialog closed by Cancel"
    );
    assert_eq!(fs::read(&path).unwrap(), before, "no re-encode on Cancel");
    let _ = fs::remove_file(&path);
}

// ── T012 [US3] no regression ────────────────────────────────────────────────

#[test]
fn list_keys_unchanged_while_list_focused() {
    let (mut a, _p) = app_with_file("listkeys");
    a.handle_action(Action::MoveDown).unwrap();
    a.handle_action(Action::MoveDown).unwrap();
    assert_eq!(a.pending_encoding_select, Some(2));
    a.handle_action(Action::MoveUp).unwrap();
    assert_eq!(a.pending_encoding_select, Some(1));
}

#[test]
fn arrows_are_noop_while_button_focused() {
    let (mut a, _p) = app_with_file("btnnoop");
    a.handle_action(Action::MoveDown).unwrap(); // select index 1
    a.handle_action(Action::FocusNextField).unwrap(); // focus OK
    assert!(a.interactive_focus_is_button().is_some());
    a.handle_action(Action::MoveDown).unwrap(); // should NOT move the list
    assert_eq!(a.pending_encoding_select, Some(1), "selection unchanged");
}

#[test]
fn esc_closes_from_any_focus() {
    let (mut a, _p) = app_with_file("esc");
    a.handle_action(Action::FocusNextField).unwrap(); // focus a button
    a.handle_action(Action::MenuClose).unwrap();
    assert!(a.pending_encoding_select.is_none(), "Esc closes");
}

#[test]
fn space_on_focused_ok_activates() {
    let (mut a, path) = app_with_file("spaceok");
    a.handle_action(Action::FocusNextField).unwrap(); // focus OK (selection 0 = UTF-8)
    a.handle_action(Action::InsertChar(' ')).unwrap();
    assert!(a.pending_encoding_select.is_none(), "Space activates OK");
    let _ = fs::remove_file(&path);
}
