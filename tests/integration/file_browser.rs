//! Integration tests for the file browser dialogs (Feature 012).
//!
//! Drives the `edit` library `App` (`handle_action` / `handle_mouse_event`) to
//! verify open-by-browsing, save-by-browsing, and mouse activation end-to-end.

use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;
use edit::ui::file_browser::{BrowseMode, FileBrowser};

fn make_app() -> App {
    let mut app = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    app.terminal_size = (80, 24);
    app
}

fn tree(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("edit_ib_{tag}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

fn select_named(app: &mut App, name: &str) {
    let fb = app.file_browser.as_mut().unwrap();
    fb.selected = fb
        .entries
        .iter()
        .position(|e| e.name == name)
        .unwrap_or_else(|| panic!("entry {name} not found"));
}

#[test]
fn open_by_browsing_two_levels_keyboard() {
    let base = tree("open_kbd");
    fs::create_dir_all(base.join("sub")).unwrap();
    fs::write(base.join("sub").join("inner.txt"), b"deep content\n").unwrap();

    let mut app = make_app();
    app.file_browser = Some(FileBrowser::open(base.clone(), BrowseMode::Open));

    // Descend into "sub".
    select_named(&mut app, "sub");
    app.handle_action(Action::InsertNewline).unwrap();
    assert!(app.file_browser.as_ref().unwrap().cwd.ends_with("sub"));

    // Open "inner.txt".
    let n_before = app.buffers.len();
    select_named(&mut app, "inner.txt");
    app.handle_action(Action::InsertNewline).unwrap();

    assert!(app.file_browser.is_none(), "browser closes after open");
    assert_eq!(app.buffers.len(), n_before + 1);
    assert!(app
        .active_buffer()
        .rope
        .to_string()
        .contains("deep content"));

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn open_cancel_leaves_state_unchanged() {
    let base = tree("open_cancel");
    fs::write(base.join("a.txt"), b"x\n").unwrap();
    let mut app = make_app();
    let n_before = app.buffers.len();
    app.file_browser = Some(FileBrowser::open(base.clone(), BrowseMode::Open));
    app.handle_action(Action::MenuClose).unwrap();
    assert!(app.file_browser.is_none());
    assert_eq!(app.buffers.len(), n_before);
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn save_by_browsing_writes_file() {
    let base = tree("save_kbd");
    let mut app = make_app();
    for c in "save me".chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
    app.file_browser = Some(FileBrowser::open(base.clone(), BrowseMode::Save));
    for c in "out.txt".chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
    app.handle_action(Action::InsertNewline).unwrap();

    assert!(app.file_browser.is_none(), "browser closes after save");
    let written = fs::read_to_string(base.join("out.txt")).expect("file written");
    assert!(written.contains("save me"));
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn save_empty_filename_is_noop() {
    let base = tree("save_empty");
    let mut app = make_app();
    app.handle_action(Action::InsertChar('z')).unwrap();
    app.file_browser = Some(FileBrowser::open(base.clone(), BrowseMode::Save));
    // No filename typed; selected is ".." → Enter navigates up, no save/no panic.
    app.handle_action(Action::InsertNewline).unwrap();
    // Browser is still open (navigated up) and nothing was written into base.
    let count = fs::read_dir(&base).unwrap().count();
    assert_eq!(count, 0, "no file written with empty filename");
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn mouse_double_click_opens_file() {
    let base = tree("mouse_open");
    fs::write(base.join("click.txt"), b"clicked\n").unwrap();

    let mut app = make_app();
    app.file_browser = Some(FileBrowser::open(base.clone(), BrowseMode::Open));
    // Layout for 80x24, Open mode (feature 020 grew the box to 64x24 at (8,0) to
    // fit the button row): list starts at row 2.
    // entries: index 0 = "..", index 1 = "click.txt" → row 3.
    let n_before = app.buffers.len();
    let me = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 3,
        modifiers: KeyModifiers::NONE,
    };
    // First click only selects (browser stays open, no file opened).
    app.handle_mouse_event(me).unwrap();
    assert!(
        app.file_browser.is_some(),
        "single click selects without opening"
    );
    assert_eq!(
        app.buffers.len(),
        n_before,
        "no buffer added on single click"
    );
    assert_eq!(
        app.file_browser.as_ref().unwrap().selected,
        1,
        "single click highlights the clicked row"
    );

    // Second click on the same row activates → opens the file, closes browser.
    app.handle_mouse_event(me).unwrap();
    assert!(
        app.file_browser.is_none(),
        "browser closes after double-click open"
    );
    assert_eq!(app.buffers.len(), n_before + 1);
    assert!(app.active_buffer().rope.to_string().contains("clicked"));
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn mouse_double_click_folder_enters_then_stays_open() {
    // Regression: double-clicking a folder must navigate into it and stay open,
    // not enter and immediately open a file under the cursor in the new listing.
    let base = tree("mouse_dir");
    fs::create_dir_all(base.join("sub")).unwrap();
    fs::write(base.join("sub").join("inner.txt"), b"inner\n").unwrap();

    let mut app = make_app();
    app.file_browser = Some(FileBrowser::open(base.clone(), BrowseMode::Open));
    // entries: index 0 = "..", index 1 = "sub/" → row 3 (box now 64x24 at (8,0)).
    let me = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 3,
        modifiers: KeyModifiers::NONE,
    };
    let n_before = app.buffers.len();
    app.handle_mouse_event(me).unwrap();
    app.handle_mouse_event(me).unwrap();

    let fb = app
        .file_browser
        .as_ref()
        .expect("browser stays open after entering a folder");
    assert!(fb.cwd.ends_with("sub"), "navigated into the folder");
    assert_eq!(
        app.buffers.len(),
        n_before,
        "no file opened by folder click"
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn mouse_click_outside_cancels() {
    let base = tree("mouse_cancel");
    let mut app = make_app();
    app.file_browser = Some(FileBrowser::open(base.clone(), BrowseMode::Open));
    let me = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    };
    app.handle_mouse_event(me).unwrap();
    assert!(app.file_browser.is_none(), "outside click cancels");
    let _ = fs::remove_dir_all(&base);
}
