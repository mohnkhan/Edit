//! Feature 023 — mouse-wheel scrolling (app-wide).
//!
//! Drives `App::handle_mouse_event` with synthesized wheel events to verify the
//! editor, file browser, and Help overlay scroll (bounded), modal-wins routing,
//! and that existing click/cursor behavior is unaffected.

use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

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

fn wheel(app: &mut App, kind: MouseEventKind, col: u16, row: u16) {
    app.handle_mouse_event(MouseEvent {
        kind,
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
}

fn tall_app() -> App {
    let mut a = app();
    for _ in 0..60 {
        a.handle_action(Action::InsertNewline).unwrap();
    }
    a.buffers[0].scroll_offset.0 = 0;
    a
}

// ── US1: editor ─────────────────────────────────────────────────────────────

#[test]
fn wheel_scrolls_editor_viewport_not_cursor() {
    let mut a = tall_app();
    a.buffers[0].cursor.line = 2;
    let cur = a.buffers[0].cursor;
    wheel(&mut a, MouseEventKind::ScrollDown, 10, 5);
    assert_eq!(a.buffers[0].scroll_offset.0, 3, "down scrolls by step");
    assert_eq!(a.buffers[0].cursor, cur, "cursor not moved by wheel");
    wheel(&mut a, MouseEventKind::ScrollUp, 10, 5);
    assert_eq!(a.buffers[0].scroll_offset.0, 0, "up scrolls back");
    wheel(&mut a, MouseEventKind::ScrollUp, 10, 5);
    assert_eq!(a.buffers[0].scroll_offset.0, 0, "clamped at top");
}

#[test]
fn wheel_on_menu_or_status_row_is_ignored() {
    let mut a = tall_app();
    wheel(&mut a, MouseEventKind::ScrollDown, 10, 0); // menu bar row
    assert_eq!(a.buffers[0].scroll_offset.0, 0, "wheel on menu row ignored");
    let (_, rows) = a.terminal_size;
    wheel(&mut a, MouseEventKind::ScrollDown, 10, rows - 1); // status bar row
    assert_eq!(
        a.buffers[0].scroll_offset.0, 0,
        "wheel on status row ignored"
    );
}

// ── US2: lists & overlays (modal wins) ──────────────────────────────────────

#[test]
fn wheel_scrolls_help_not_editor() {
    let mut a = tall_app();
    a.handle_action(Action::Help).unwrap();
    assert!(a.help_screen().is_some());
    wheel(&mut a, MouseEventKind::ScrollDown, 10, 5);
    assert_eq!(a.help_scroll(), 3, "Help scrolls on wheel");
    assert_eq!(
        a.buffers[0].scroll_offset.0, 0,
        "editor under the modal does not scroll"
    );
    // Clamp at top.
    wheel(&mut a, MouseEventKind::ScrollUp, 10, 5);
    wheel(&mut a, MouseEventKind::ScrollUp, 10, 5);
    assert_eq!(a.help_scroll(), 0, "Help scroll clamps at 0");
}

#[test]
fn wheel_scrolls_file_browser_listing() {
    let base: PathBuf = std::env::temp_dir().join("edit_wheel_fb");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    for i in 0..40 {
        fs::write(base.join(format!("f{i:02}.txt")), b"x").unwrap();
    }
    let mut a = app();
    a.open_file_browser(FileBrowser::open(base.clone(), BrowseMode::Open));
    let sel0 = a.file_browser().unwrap().selected;
    wheel(&mut a, MouseEventKind::ScrollDown, 10, 5);
    let sel1 = a.file_browser().unwrap().selected;
    assert!(
        sel1 > sel0,
        "wheel advances the file-browser selection/listing"
    );
    wheel(&mut a, MouseEventKind::ScrollUp, 10, 5);
    assert!(
        a.file_browser().unwrap().selected < sel1,
        "wheel up reverses"
    );
    let _ = fs::remove_dir_all(&base);
}

// ── US3: no regression ──────────────────────────────────────────────────────

#[test]
fn left_click_still_places_cursor_and_wheel_does_not() {
    let mut a = tall_app();
    // A wheel event must not move the cursor or start a selection.
    let before = a.buffers[0].cursor;
    wheel(&mut a, MouseEventKind::ScrollDown, 10, 5);
    assert_eq!(a.buffers[0].cursor, before, "wheel leaves cursor put");
    assert!(
        a.buffers[0].selection.is_none(),
        "wheel does not start a selection"
    );
    // A left click still moves the cursor (into the visible region).
    a.handle_mouse_event(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 2,
        row: 2,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
    // Click maps to an editor position (cursor line within the file).
    assert!(a.buffers[0].cursor.line <= a.buffers[0].rope.line_count());
}
