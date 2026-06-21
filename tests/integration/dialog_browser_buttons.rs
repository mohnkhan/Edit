//! Feature 020 — file-browser dialog: Open/Save + Cancel buttons + focus ring.
//!
//! Verifies the [Browser, Open|Save, Cancel] ring, mode-aware labels, mouse
//! activation (buttons take precedence over entry clicks), and that existing
//! navigation keys still work.

use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;
use edit::ui::file_browser::{BrowseMode, FileBrowser};

fn make_app() -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    a
}

fn tree(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("edit_brbtn_{tag}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

fn select_named(app: &mut App, name: &str) {
    let fb = app.file_browser_mut().unwrap();
    fb.selected = fb
        .entries
        .iter()
        .position(|e| e.name == name)
        .unwrap_or_else(|| panic!("entry {name} not found"));
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

// ── T025 [US2] ring + mode-aware label ──────────────────────────────────────

#[test]
fn ring_len_and_label_by_mode() {
    let base = tree("ring");
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    assert_eq!(
        a.interactive_button_labels(),
        vec!["Open (Enter)", "Cancel (Esc)"]
    );
    assert_eq!(a.dialog_focus, 0, "default focus on the browser");

    let mut b = make_app();
    b.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Save));
    assert_eq!(
        b.interactive_button_labels(),
        vec!["Save (Enter)", "Cancel (Esc)"]
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn tab_cycles_browser_open_cancel() {
    let base = tree("tab");
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    a.handle_action(Action::FocusNextField).unwrap(); // -> Open
    assert_eq!(a.interactive_focus_is_button(), Some(0));
    a.handle_action(Action::FocusNextField).unwrap(); // -> Cancel
    assert_eq!(a.interactive_focus_is_button(), Some(1));
    a.handle_action(Action::FocusNextField).unwrap(); // wraps -> browser
    assert_eq!(a.interactive_focus_is_button(), None);
    let _ = fs::remove_dir_all(&base);
}

// ── T026 [US1] mouse activation + precedence ────────────────────────────────

#[test]
fn click_open_button_opens_selected_file() {
    let base = tree("openbtn");
    fs::write(base.join("pick.txt"), b"picked\n").unwrap();
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    select_named(&mut a, "pick.txt");
    let n_before = a.buffers.len();

    let rect = a.interactive_dialog_rect().unwrap();
    let labels = a.interactive_button_labels();
    let rects = edit::ui::buttons::button_rects(rect, &labels);
    let open = rects[0];
    click(&mut a, open.x + 1, open.y + 1);

    assert!(a.file_browser().is_none(), "browser closed after Open");
    assert_eq!(a.buffers.len(), n_before + 1, "file opened");
    assert!(a.active_buffer().rope.to_string().contains("picked"));
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn click_cancel_button_closes_browser() {
    let base = tree("cancelbtn");
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    let rect = a.interactive_dialog_rect().unwrap();
    let labels = a.interactive_button_labels();
    let rects = edit::ui::buttons::button_rects(rect, &labels);
    let cancel = rects[1];
    click(&mut a, cancel.x + 1, cancel.y + 1);
    assert!(a.file_browser().is_none(), "Cancel closes the browser");
    let _ = fs::remove_dir_all(&base);
}

// ── T027 [US3] no regression ────────────────────────────────────────────────

#[test]
fn nav_keys_unchanged_while_browser_focused() {
    let base = tree("nav");
    fs::write(base.join("a.txt"), b"a").unwrap();
    fs::write(base.join("b.txt"), b"b").unwrap();
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    let sel0 = a.file_browser().unwrap().selected;
    a.handle_action(Action::MoveDown).unwrap();
    let sel1 = a.file_browser().unwrap().selected;
    assert_ne!(sel0, sel1, "Down still moves the selection");
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn typing_edits_path_while_browser_focused() {
    let base = tree("type");
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Save));
    a.handle_action(Action::InsertChar('x')).unwrap();
    a.handle_action(Action::InsertChar('y')).unwrap();
    assert_eq!(a.file_browser().unwrap().filename, "xy");
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn arrows_noop_while_button_focused() {
    let base = tree("btnnoop");
    fs::write(base.join("a.txt"), b"a").unwrap();
    fs::write(base.join("b.txt"), b"b").unwrap();
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    a.handle_action(Action::FocusNextField).unwrap(); // focus Open
    let sel = a.file_browser().unwrap().selected;
    a.handle_action(Action::MoveDown).unwrap(); // no-op while a button is focused
    assert_eq!(a.file_browser().unwrap().selected, sel);
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn esc_closes_from_any_focus() {
    let base = tree("esc");
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    a.handle_action(Action::FocusNextField).unwrap(); // focus a button
    a.handle_action(Action::MenuClose).unwrap();
    assert!(a.file_browser().is_none(), "Esc closes from any focus");
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn entry_double_click_still_activates() {
    let base = tree("dbl");
    fs::create_dir_all(base.join("sub")).unwrap();
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    // Find the row of "sub/" and double-click it; the browser should enter it.
    let rect = a.interactive_dialog_rect().unwrap();
    // Entry list starts at box.y + 2 (border + header). idx 0 = "..".
    let sub_idx = a
        .file_browser()
        .unwrap()
        .entries
        .iter()
        .position(|e| e.name == "sub")
        .unwrap();
    let row = rect.y + 2 + sub_idx as u16;
    let col = rect.x + 2;
    click(&mut a, col, row);
    click(&mut a, col, row);
    let fb = a.file_browser().expect("stays open after entering dir");
    assert!(fb.cwd.ends_with("sub"), "double-click entered the folder");
    let _ = fs::remove_dir_all(&base);
}
