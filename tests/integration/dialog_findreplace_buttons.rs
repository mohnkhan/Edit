//! Feature 020 — Find/Replace dialog: combined field + button focus ring.
//!
//! Verifies the mode-dependent ring (Find: [Query, Find, Close]; Replace:
//! [Query, Replacement, Find, Replace, Replace All, Close]), field/ring sync,
//! mouse activation, and zero regression to editing/options/match-nav keys.

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;
use edit::ui::dialog::DialogField;

fn app_with_text(text: &str) -> App {
    let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    a.terminal_size = (80, 24);
    for c in text.chars() {
        if c == '\n' {
            a.handle_action(Action::InsertNewline).unwrap();
        } else {
            a.handle_action(Action::InsertChar(c)).unwrap();
        }
    }
    a
}

fn type_into(app: &mut App, s: &str) {
    for c in s.chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
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

// ── T032 [US2] ring + field sync ────────────────────────────────────────────

#[test]
fn find_mode_ring_is_query_find_close() {
    let mut a = app_with_text("foo");
    a.handle_action(Action::Find).unwrap();
    assert_eq!(a.interactive_button_labels(), vec!["Find", "Close"]);
    // Stops: 0=Query, 1=Find, 2=Close, wrap -> 0.
    a.handle_action(Action::FocusNextField).unwrap();
    assert_eq!(a.interactive_focus_is_button(), Some(0)); // Find
    a.handle_action(Action::FocusNextField).unwrap();
    assert_eq!(a.interactive_focus_is_button(), Some(1)); // Close
    a.handle_action(Action::FocusNextField).unwrap();
    assert_eq!(a.interactive_focus_is_button(), None, "wrapped to Query");
}

#[test]
fn replace_mode_ring_has_six_stops_and_syncs_field() {
    let mut a = app_with_text("foo");
    a.handle_action(Action::FindReplace).unwrap();
    assert_eq!(
        a.interactive_button_labels(),
        vec!["Find", "Replace", "Replace All", "Close"]
    );
    // Stop 0 = Query.
    assert_eq!(
        a.pending_find_replace.as_ref().unwrap().focus,
        DialogField::Query
    );
    // Stop 1 = Replacement (field focus syncs).
    a.handle_action(Action::FocusNextField).unwrap();
    assert_eq!(
        a.pending_find_replace.as_ref().unwrap().focus,
        DialogField::Replacement
    );
    assert_eq!(a.interactive_focus_is_button(), None, "still a field stop");
    // Stops 2..5 = buttons.
    for expect in 0..4 {
        a.handle_action(Action::FocusNextField).unwrap();
        assert_eq!(a.interactive_focus_is_button(), Some(expect));
    }
    // Wrap back to Query.
    a.handle_action(Action::FocusNextField).unwrap();
    assert_eq!(a.interactive_focus_is_button(), None);
    assert_eq!(
        a.pending_find_replace.as_ref().unwrap().focus,
        DialogField::Query
    );
}

// ── T033 [US1] mouse activation ─────────────────────────────────────────────

#[test]
fn click_find_button_runs_search() {
    let mut a = app_with_text("foo bar foo");
    a.handle_action(Action::Find).unwrap();
    type_into(&mut a, "foo");
    let rect = a.interactive_dialog_rect().unwrap();
    let labels = a.interactive_button_labels();
    let rects = edit::ui::buttons::button_rects(rect, &labels);
    let find = rects[0];
    click(&mut a, find.x + 1, find.y + 1);
    assert_eq!(
        a.search_state.matches.len(),
        2,
        "Find button ran the search"
    );
}

#[test]
fn click_replace_all_button_replaces_all() {
    let mut a = app_with_text("foo foo foo");
    a.handle_action(Action::FindReplace).unwrap();
    type_into(&mut a, "foo"); // into Query
    a.handle_action(Action::FocusNextField).unwrap(); // -> Replacement field
    type_into(&mut a, "bar");
    let rect = a.interactive_dialog_rect().unwrap();
    let labels = a.interactive_button_labels();
    let rects = edit::ui::buttons::button_rects(rect, &labels);
    // labels: [Find, Replace, Replace All, Close]
    let all = rects[2];
    click(&mut a, all.x + 1, all.y + 1);
    assert_eq!(a.active_buffer().rope.to_string(), "bar bar bar");
}

#[test]
fn click_close_button_closes_dialog() {
    let mut a = app_with_text("x");
    a.handle_action(Action::Find).unwrap();
    let rect = a.interactive_dialog_rect().unwrap();
    let labels = a.interactive_button_labels();
    let rects = edit::ui::buttons::button_rects(rect, &labels);
    let close = rects[1];
    click(&mut a, close.x + 1, close.y + 1);
    assert!(a.pending_find_replace.is_none(), "Close closed the dialog");
}

// ── T034 [US3] no regression ────────────────────────────────────────────────

#[test]
fn editing_options_matchnav_unchanged_on_field() {
    let mut a = app_with_text("Foo foo FOO");
    a.handle_action(Action::Find).unwrap();
    type_into(&mut a, "fox");
    a.handle_action(Action::Backspace).unwrap(); // "fo"
    type_into(&mut a, "o"); // "foo"
    assert_eq!(a.pending_find_replace.as_ref().unwrap().query, "foo");
    // Option toggles still work (regardless of focus).
    a.handle_action(Action::ToggleSearchCase).unwrap();
    assert!(a.pending_find_replace.as_ref().unwrap().case_sensitive);
    a.handle_action(Action::ToggleSearchCase).unwrap();
    assert!(!a.pending_find_replace.as_ref().unwrap().case_sensitive);
    // Run + match nav.
    a.handle_action(Action::InsertNewline).unwrap();
    assert_eq!(a.search_state.matches.len(), 3);
    let first = a.search_state.active_match;
    a.handle_action(Action::FindNext).unwrap();
    assert_ne!(a.search_state.active_match, first, "F3 advances the match");
}

#[test]
fn ctrl_a_replace_all_still_works_on_field() {
    let mut a = app_with_text("foo foo");
    a.handle_action(Action::FindReplace).unwrap();
    type_into(&mut a, "foo");
    a.handle_action(Action::FocusNextField).unwrap(); // Replacement
    type_into(&mut a, "baz");
    a.handle_action(Action::SelectAll).unwrap(); // Ctrl+A == Replace All
    assert_eq!(a.active_buffer().rope.to_string(), "baz baz");
}

#[test]
fn text_keys_ignored_while_button_focused() {
    let mut a = app_with_text("x");
    a.handle_action(Action::Find).unwrap();
    type_into(&mut a, "abc");
    a.handle_action(Action::FocusNextField).unwrap(); // focus Find button
    assert!(a.interactive_focus_is_button().is_some());
    a.handle_action(Action::InsertChar('Z')).unwrap(); // must NOT edit the field
    assert_eq!(a.pending_find_replace.as_ref().unwrap().query, "abc");
}

#[test]
fn esc_closes_from_any_stop() {
    let mut a = app_with_text("x");
    a.handle_action(Action::FindReplace).unwrap();
    a.handle_action(Action::FocusNextField).unwrap();
    a.handle_action(Action::FocusNextField).unwrap(); // some button
    a.handle_action(Action::MenuClose).unwrap();
    assert!(a.pending_find_replace.is_none(), "Esc closes from any stop");
}
