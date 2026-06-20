//! Feature 027 — buffer tab bar.
//!
//! Drives `App::handle_mouse_event` against the tab bar to verify click-to-switch,
//! `[x]` close (clean + modified-with-confirm), and that the editor geometry below
//! the bar stays correct (click-to-place-cursor, wheel, keyboard switching).

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use edit::app::App;
use edit::buffer::rope::EditorRope;
use edit::buffer::Buffer;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::ui::tabbar::{tab_hit_regions, TabRegion};

fn app() -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    a
}

/// Replace the buffer list with named buffers (`a.txt`, `b.txt`, …), active = 0.
fn with_buffers(a: &mut App, names: &[&str]) {
    a.buffers = names
        .iter()
        .map(|n| {
            let mut b = Buffer::new_empty();
            b.path = Some(std::path::PathBuf::from(n));
            b
        })
        .collect();
    a.active_idx = 0;
}

fn click(a: &mut App, col: u16, row: u16) {
    a.handle_mouse_event(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
}

fn wheel_down(a: &mut App, col: u16, row: u16) {
    a.handle_mouse_event(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    })
    .unwrap();
}

/// The tab regions as the renderer/handler compute them (row 1, full width).
fn regions(a: &App) -> Vec<TabRegion> {
    let area = ratatui::layout::Rect::new(0, 1, a.terminal_size.0, 1);
    tab_hit_regions(area, &a.buffers, a.active_idx)
}

// ── US1: visibility + click-to-switch ──────────────────────────────────────

#[test]
fn tab_bar_hidden_with_one_buffer() {
    let a = app();
    assert_eq!(a.buffers.len(), 1);
    assert!(!a.tab_bar_visible());
    assert_eq!(a.editor_top(), 1);
}

#[test]
fn clicking_second_tab_switches_active() {
    let mut a = app();
    with_buffers(&mut a, &["a.txt", "b.txt"]);
    assert!(a.tab_bar_visible());
    assert_eq!(a.editor_top(), 2);
    let r = regions(&a);
    assert_eq!(r.len(), 2);
    // Click inside the second tab's label region (row 1 = tab row).
    let target = r[1].label_rect;
    click(&mut a, target.x + 1, 1);
    assert_eq!(a.active_idx, 1);
}

#[test]
fn click_on_tab_row_outside_any_tab_is_noop() {
    let mut a = app();
    with_buffers(&mut a, &["a.txt", "b.txt"]);
    let before = a.buffers[a.active_idx].cursor;
    // Far right of the tab row, past the last tab.
    click(&mut a, 79, 1);
    assert_eq!(a.active_idx, 0);
    assert_eq!(a.buffers[a.active_idx].cursor, before);
}

// ── US2: [x] close + unsaved confirm ────────────────────────────────────────

#[test]
fn close_box_on_clean_buffer_closes_immediately() {
    let mut a = app();
    with_buffers(&mut a, &["a.txt", "b.txt"]);
    let r = regions(&a);
    let close = r[1].close_rect; // close the second (clean) buffer
    click(&mut a, close.x, 1);
    assert_eq!(a.buffers.len(), 1);
    assert!(a.pending_close_confirm.is_none());
    // With one buffer left, the tab bar is hidden again.
    assert!(!a.tab_bar_visible());
}

#[test]
fn close_box_on_modified_buffer_opens_confirm() {
    let mut a = app();
    with_buffers(&mut a, &["a.txt", "b.txt"]);
    a.buffers[1].modified = true;
    let r = regions(&a);
    let close = r[1].close_rect;
    click(&mut a, close.x, 1);
    // Nothing closed yet; the confirm targets the clicked (not active) buffer.
    assert_eq!(a.buffers.len(), 2);
    assert_eq!(a.pending_close_confirm, Some(1));
    // Cancel (button 2) keeps the buffer.
    a.activate_dialog_button(2);
    assert_eq!(a.buffers.len(), 2);
    assert!(a.pending_close_confirm.is_none());
    // Re-open and Discard (button 1) closes it.
    a.buffers[1].modified = true;
    a.tab_close_clicked(1);
    assert_eq!(a.pending_close_confirm, Some(1));
    a.activate_dialog_button(1);
    assert_eq!(a.buffers.len(), 1);
}

// ── US3: geometry no-regression beneath the bar ─────────────────────────────

#[test]
fn editor_click_accounts_for_tab_row() {
    let mut a = app();
    with_buffers(&mut a, &["a.txt", "b.txt"]);
    // Give the active buffer several lines so the click maps to a real line.
    a.buffers[0].rope = EditorRope::from_str("L0\nL1\nL2\nL3\nL4\nL5\n");
    a.active_idx = 0;
    // editor_top() == 2, so the first editor row is terminal row 2 → line 0.
    click(&mut a, 0, 2);
    assert_eq!(a.buffers[0].cursor.line, 0);
    // Terminal row 4 → editor row 2 → line 2.
    click(&mut a, 0, 4);
    assert_eq!(a.buffers[0].cursor.line, 2);
}

#[test]
fn wheel_scrolls_reduced_editor_without_moving_cursor() {
    let mut a = app();
    with_buffers(&mut a, &["a.txt", "b.txt"]);
    let mut text = String::new();
    for i in 0..200 {
        text.push_str(&format!("line {i}\n"));
    }
    a.buffers[0].rope = EditorRope::from_str(&text);
    a.active_idx = 0;
    let cursor_before = a.buffers[0].cursor;
    let off_before = a.buffers[0].scroll_offset.0;
    // Wheel over the editor area (row well below the tab row).
    wheel_down(&mut a, 10, 10);
    assert!(a.buffers[0].scroll_offset.0 > off_before, "editor scrolled");
    assert_eq!(
        a.buffers[0].cursor, cursor_before,
        "wheel does not move cursor"
    );
}

#[test]
fn keyboard_switching_unchanged_with_tab_bar() {
    let mut a = app();
    with_buffers(&mut a, &["a.txt", "b.txt", "c.txt"]);
    a.next_buffer();
    assert_eq!(a.active_idx, 1);
    a.next_buffer();
    assert_eq!(a.active_idx, 2);
    a.prev_buffer();
    assert_eq!(a.active_idx, 1);
}
