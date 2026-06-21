//! Feature 031 — caret-on-click in dialog text fields (#58). End-to-end tests
//! driving `App::handle_mouse_event` / `handle_action`.

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;
use edit::ui::dialog::{DialogField, DialogMode, FindReplaceDialog};
use edit::ui::file_browser::{BrowseMode, FileBrowser};

fn app() -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    a
}

fn left_click(a: &mut App, col: u16, row: u16) {
    a.handle_mouse_event(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
}

// ── US1: Find/Replace click-to-position ─────────────────────────────────────

#[test]
fn click_in_find_field_positions_caret() {
    let mut a = app();
    a.open_find_replace(FindReplaceDialog::new(
        DialogMode::Find,
        "hello world".into(),
    ));
    let full = ratatui::layout::Rect::new(0, 0, 80, 24);
    let d = a.find_replace().unwrap();
    let rects = edit::ui::find_replace_field_rects(d, full);
    let (field, fr) = rects[0];
    assert_eq!(field, DialogField::Query);
    // Click 6 columns into the query text ("hello |world") → caret at grapheme 6.
    left_click(&mut a, fr.x + 6, fr.y);
    let d = a.find_replace().unwrap();
    assert_eq!(d.caret, 6);
    assert_eq!(d.focus, DialogField::Query);
    // Click past the end clamps to the value length.
    left_click(&mut a, fr.x + 50, fr.y);
    assert_eq!(a.find_replace().unwrap().caret, "hello world".len());
}

// ── US2: file-browser Name field caret + click ──────────────────────────────

#[test]
fn click_and_arrows_edit_name_field() {
    let mut a = app();
    let mut fb = FileBrowser::open(std::path::PathBuf::from("."), BrowseMode::Save);
    for c in "report.txt".chars() {
        fb.push_char(c);
    }
    a.open_file_browser(fb);
    // Arrow + insert mid-string via the app's key path.
    a.handle_action(Action::MoveLineStart).unwrap(); // caret to 0
    a.handle_action(Action::MoveRight).unwrap(); // caret 1 (editing, not activate)
    a.handle_action(Action::InsertChar('Z')).unwrap();
    assert_eq!(a.file_browser().unwrap().filename, "rZeport.txt");

    // Click into the field box → caret moves to the clicked grapheme.
    let area = ratatui::layout::Rect::new(0, 0, 80, 24);
    let fr = a.file_browser().unwrap().field_text_rect(area);
    left_click(&mut a, fr.x + 4, fr.y);
    assert_eq!(a.file_browser().unwrap().caret, 4);
}

// ── US3: Go-to-Line caret + click ───────────────────────────────────────────

#[test]
fn click_in_goto_line_positions_caret() {
    let mut a = app();
    a.pending_goto_line = Some("12345".to_string());
    a.pending_goto_line_caret = 5;
    // Box: dw = (19 + 5).clamp(20, 80) = 24; dx = (80-24)/2 = 28; dy = (24-3)/2 = 10.
    // Digits start at dx + 1 + 12 = 41, on row dy+1 = 11. Click 2 cols in → caret 2.
    let value_x = 28 + 1 + 12;
    left_click(&mut a, value_x + 2, 11);
    assert_eq!(a.pending_goto_line_caret, 2);
    // Insert mid-string via the key path.
    a.handle_action(Action::InsertChar('0')).unwrap();
    assert_eq!(a.pending_goto_line.as_deref(), Some("120345"));
}
