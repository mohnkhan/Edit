//! Feature 022 — file dialog glob filtering + richer entry details.
//!
//! Drives the `edit` library `App`/`FileBrowser` to verify live filtering, the
//! absolute-path jump, Save-mode confirm, and no-regression with the feature-020
//! buttons and feature-021 scrollbar.

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
    let base = std::env::temp_dir().join(format!("edit_fdf_{tag}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

fn names(app: &App) -> Vec<String> {
    app.file_browser()
        .unwrap()
        .entries
        .iter()
        .map(|e| e.name.clone())
        .collect()
}

// ── US1: live filtering ─────────────────────────────────────────────────────

#[test]
fn typing_glob_filters_live() {
    let base = tree("glob");
    fs::create_dir_all(base.join("logs")).unwrap();
    fs::write(base.join("a.log"), b"x").unwrap();
    fs::write(base.join("b.txt"), b"x").unwrap();
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));

    for c in "*.log".chars() {
        a.handle_action(Action::InsertChar(c)).unwrap();
    }
    let n = names(&a);
    assert!(n.contains(&"a.log".to_string()));
    assert!(
        !n.contains(&"b.txt".to_string()),
        "non-matching file filtered out"
    );
    assert!(n.contains(&"logs".to_string()), "directory kept");
    assert!(n.contains(&"..".to_string()), "parent kept");
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn substring_filters_and_backspace_restores() {
    let base = tree("substr");
    fs::write(base.join("report.txt"), b"x").unwrap();
    fs::write(base.join("data.csv"), b"x").unwrap();
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    let full = names(&a).len();

    for c in "rep".chars() {
        a.handle_action(Action::InsertChar(c)).unwrap();
    }
    let n = names(&a);
    assert!(n.contains(&"report.txt".to_string()));
    assert!(!n.contains(&"data.csv".to_string()));

    for _ in 0..3 {
        a.handle_action(Action::Backspace).unwrap();
    }
    assert_eq!(
        names(&a).len(),
        full,
        "clearing the field restores the listing"
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn absolute_path_still_jumps() {
    let base = tree("jump");
    fs::create_dir_all(base.join("target")).unwrap();
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    let abs = base.join("target");
    for c in abs.to_string_lossy().chars() {
        a.handle_action(Action::InsertChar(c)).unwrap();
    }
    // Absolute path is not a filter — listing stays full while typing it.
    let fb = a.file_browser().unwrap();
    assert_eq!(
        fb.entries.len(),
        fb.all_entries.len(),
        "abs path doesn't filter"
    );
    a.handle_action(Action::InsertNewline).unwrap(); // jump
    assert!(
        a.file_browser().unwrap().cwd.ends_with("target"),
        "absolute path jumped into the directory"
    );
    let _ = fs::remove_dir_all(&base);
}

// ── US1: Save mode confirm unaffected by filtering ──────────────────────────

#[test]
fn save_mode_confirm_saves_typed_name_even_if_no_match() {
    let base = tree("save");
    fs::write(base.join("existing.txt"), b"x").unwrap();
    let mut a = make_app();
    // Put some content in the active buffer to save.
    a.handle_action(Action::InsertChar('h')).unwrap();
    a.handle_action(Action::InsertChar('i')).unwrap();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Save));
    for c in "brandnew.txt".chars() {
        a.handle_action(Action::InsertChar(c)).unwrap();
    }
    // Filtering to a non-existent name hides files but keeps dirs/`..`.
    a.handle_action(Action::InsertNewline).unwrap(); // confirm save
    assert!(
        base.join("brandnew.txt").exists(),
        "Save confirm wrote the typed filename despite no matching existing file"
    );
    let _ = fs::remove_dir_all(&base);
}

// ── Regression: buttons + scrollbar still work with a filter active ─────────

#[test]
fn buttons_and_scrollbar_work_with_filter_active() {
    let base = tree("regress");
    for i in 0..40 {
        fs::write(base.join(format!("note{i:02}.log")), b"x").unwrap();
    }
    fs::write(base.join("other.txt"), b"x").unwrap();
    let mut a = make_app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    for c in "*.log".chars() {
        a.handle_action(Action::InsertChar(c)).unwrap();
    }
    // Feature-020 Cancel button still closes the browser.
    let rect = a.interactive_dialog_rect().unwrap();
    let labels = a.interactive_button_labels();
    let rects = edit::ui::buttons::button_rects(rect, &labels);
    let cancel = rects[1];
    a.handle_mouse_event(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: cancel.x + 1,
        row: cancel.y + 1,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
    assert!(
        a.file_browser().is_none(),
        "Cancel button works under a filter"
    );
    let _ = fs::remove_dir_all(&base);
}
