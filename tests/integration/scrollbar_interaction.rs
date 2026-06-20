//! Feature 024 — interactive (clickable + draggable) scrollbars.
//!
//! Drives `App::handle_mouse_event` with synthesized press/drag/release to verify
//! track-click paging, thumb-drag scrolling (editor viewport-only), and that text
//! drag-selection (feature 017) is unaffected.

use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

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

fn ev(app: &mut App, kind: MouseEventKind, col: u16, row: u16) {
    app.handle_mouse_event(MouseEvent {
        kind,
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
}

fn press(app: &mut App, col: u16, row: u16) {
    ev(app, MouseEventKind::Down(MouseButton::Left), col, row);
}
fn drag(app: &mut App, col: u16, row: u16) {
    ev(app, MouseEventKind::Drag(MouseButton::Left), col, row);
}
fn release(app: &mut App, col: u16, row: u16) {
    ev(app, MouseEventKind::Up(MouseButton::Left), col, row);
}

fn tall_app() -> App {
    let mut a = app();
    for _ in 0..60 {
        a.handle_action(Action::InsertNewline).unwrap();
    }
    a.buffers[0].scroll_offset.0 = 0;
    a.buffers[0].cursor.line = 2;
    a.buffers[0].cursor.grapheme_col = 0;
    a
}

// ── US1: track click pages ──────────────────────────────────────────────────

#[test]
fn track_click_pages_editor_vertical() {
    // 80x24: editor vbar is the rightmost column (79); track rows ~1..22.
    let mut a = tall_app();
    let cur = a.buffers[0].cursor;
    press(&mut a, 79, 21); // bottom of track, below the thumb → page down
    assert!(a.buffers[0].scroll_offset.0 > 0, "track click paged down");
    assert_eq!(a.buffers[0].cursor, cur, "cursor not moved by track click");
    let after_down = a.buffers[0].scroll_offset.0;
    press(&mut a, 79, 1); // top of track, above the thumb → page up
    assert!(
        a.buffers[0].scroll_offset.0 < after_down,
        "track click above paged up"
    );
}

#[test]
fn track_click_scrolls_file_browser() {
    let base: PathBuf = std::env::temp_dir().join("edit_sb_fb");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    for i in 0..40 {
        fs::write(base.join(format!("f{i:02}.txt")), b"x").unwrap();
    }
    let mut a = app();
    a.file_browser = Some(FileBrowser::open(base.clone(), BrowseMode::Open));
    let (rect, _c, _v, _o) = a
        .file_browser
        .as_ref()
        .unwrap()
        .list_scrollbar(Rect::new(0, 0, 80, 24))
        .expect("listing overflows → a bar exists");
    // Click near the bottom of the track → page down.
    press(&mut a, rect.x, rect.y + rect.height - 1);
    assert!(
        a.file_browser.as_ref().unwrap().scroll > 0,
        "file-browser track click scrolled the listing"
    );
    let _ = fs::remove_dir_all(&base);
}

// ── US2: thumb drag ─────────────────────────────────────────────────────────

#[test]
fn drag_editor_thumb_scrolls_proportionally_viewport_only() {
    let mut a = tall_app();
    let cur = a.buffers[0].cursor;
    // Thumb sits near the top at offset 0; press on it (row 2), then drag down.
    press(&mut a, 79, 2);
    drag(&mut a, 79, 20);
    assert!(
        a.buffers[0].scroll_offset.0 > 20,
        "thumb drag scrolled proportionally"
    );
    assert_eq!(
        a.buffers[0].cursor, cur,
        "editor drag did not move the cursor"
    );
    assert!(
        a.buffers[0].selection.is_none(),
        "no selection from a thumb drag"
    );
    // Release ends the drag; a later drag does not scroll.
    release(&mut a, 79, 20);
    let off = a.buffers[0].scroll_offset.0;
    drag(&mut a, 79, 1);
    assert_eq!(
        a.buffers[0].scroll_offset.0, off,
        "drag after release does not scroll"
    );
}

#[test]
fn release_outside_track_ends_drag_cleanly() {
    let mut a = tall_app();
    press(&mut a, 79, 2); // start a thumb drag
    release(&mut a, 5, 23); // release far outside the track — must not panic
    let off = a.buffers[0].scroll_offset.0;
    drag(&mut a, 79, 18);
    assert_eq!(a.buffers[0].scroll_offset.0, off, "no scroll after release");
}

// ── US3: no regression (feature 017 text selection) ─────────────────────────

#[test]
fn text_press_drag_still_selects_but_bar_press_does_not() {
    let mut a = tall_app();
    // Press-drag inside the text body → selection (not on the scrollbar column).
    // Drag across rows (the buffer lines are empty, so a same-row drag wouldn't
    // move the column) to produce a non-empty selection.
    press(&mut a, 3, 3);
    drag(&mut a, 3, 6);
    assert!(
        a.buffers[0].selection.is_some(),
        "text press-drag still selects (feature 017 intact)"
    );

    // A press on the scrollbar column starts a drag, not a selection / cursor move.
    let mut b = tall_app();
    let cur = b.buffers[0].cursor;
    press(&mut b, 79, 2);
    assert!(
        b.buffers[0].selection.is_none(),
        "bar press does not select"
    );
    assert_eq!(
        b.buffers[0].cursor, cur,
        "bar press does not move the cursor"
    );
}
