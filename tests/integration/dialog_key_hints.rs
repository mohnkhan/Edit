//! Feature 021 — key-hint button labels.
//!
//! Verifies that dialog button labels advertise their activating key, and that
//! adding the hint does not change the action (dispatch keys on index, not text).

use std::fs;
use std::path::PathBuf;

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;
use edit::ui::file_browser::{BrowseMode, FileBrowser};

fn app() -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    a
}

fn tree(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("edit_keyhint_{tag}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

fn has_key(labels: &[&str], substr: &str) -> bool {
    labels.iter().any(|l| l.contains(substr))
}

#[test]
fn save_prompt_labels_carry_keys() {
    let mut a = app();
    a.handle_action(Action::InsertChar('x')).unwrap();
    a.handle_action(Action::Quit).unwrap(); // opens save prompt
    let labels = a.dialog_button_labels();
    assert!(has_key(&labels, "(S)"), "Save shows its key: {labels:?}");
    assert!(has_key(&labels, "(D)"), "Discard shows its key");
    assert!(has_key(&labels, "(Esc)"), "Cancel shows Esc");
}

#[test]
fn save_prompt_letter_shortcut_still_works() {
    let mut a = app();
    a.handle_action(Action::InsertChar('x')).unwrap();
    a.handle_action(Action::Quit).unwrap();
    // The 'C' shortcut still cancels (label change is informational only).
    a.handle_action(Action::InsertChar('c')).unwrap();
    assert!(!a.is_save_prompt_open(), "Cancel shortcut still works");
    assert!(a.running);
}

#[test]
fn encoding_labels_carry_keys() {
    let path = tree("enc").join("f.txt");
    fs::write(&path, b"hi").unwrap();
    let mut a = App::new(Config::default(), vec![path], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    a.handle_action(Action::SaveAsEncoding).unwrap();
    let labels = a.interactive_button_labels();
    assert!(
        has_key(&labels, "(Enter)") && has_key(&labels, "(Esc)"),
        "{labels:?}"
    );
}

#[test]
fn find_replace_labels_carry_keys_and_dispatch_by_index() {
    let mut a = app();
    a.handle_action(Action::InsertChar('a')).unwrap();
    a.handle_action(Action::InsertChar('a')).unwrap();
    a.handle_action(Action::FindReplace).unwrap(); // replace mode
    let labels = a.interactive_button_labels();
    assert!(
        has_key(&labels, "(Ctrl+A)"),
        "Replace All shows its key: {labels:?}"
    );
    assert!(has_key(&labels, "(Enter)") && has_key(&labels, "(Esc)"));
    // Dispatch is by index: button 3 (Close) closes regardless of label text.
    a.activate_interactive_button(3);
    assert!(a.pending_find_replace.is_none(), "index-3 button is Close");
}

#[test]
fn file_browser_labels_carry_keys() {
    let base = tree("fb");
    let mut a = app();
    a.file_browser = Some(FileBrowser::open(base, BrowseMode::Open));
    let labels = a.interactive_button_labels();
    assert!(has_key(&labels, "Open (Enter)") || has_key(&labels, "(Enter)"));
    assert!(has_key(&labels, "(Esc)"));
}

#[test]
fn help_close_label_carries_key() {
    assert!(edit::ui::buttons::HELP_CLOSE_LABEL.contains("Esc"));
}
