//! Integration tests for Feature 015: interactive Find and Replace dialogs.
//!
//! Drives the `edit` library `App` (`handle_action`) end-to-end through the
//! dialog intercept to verify find / next / prev / replace / replace-all and the
//! option toggles.

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;

fn app_with_text(text: &str) -> App {
    let mut app = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    for c in text.chars() {
        if c == '\n' {
            app.handle_action(Action::InsertNewline).unwrap();
        } else {
            app.handle_action(Action::InsertChar(c)).unwrap();
        }
    }
    app
}

fn type_into_dialog(app: &mut App, s: &str) {
    for c in s.chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
}

// ── US1: Find ─────────────────────────────────────────────────────────────────

#[test]
fn find_opens_dialog_and_finds_matches() {
    let mut app = app_with_text("foo bar foo baz foo");
    app.handle_action(Action::Find).unwrap();
    assert!(app.find_replace().is_some(), "Find opens the dialog");

    type_into_dialog(&mut app, "foo");
    app.handle_action(Action::InsertNewline).unwrap(); // run search

    assert_eq!(app.search_state.matches.len(), 3, "three occurrences");
    assert!(app.search_state.active_match.is_some(), "current match set");
    assert!(app.find_replace().is_some(), "dialog stays open");
}

#[test]
fn find_not_found_leaves_no_matches() {
    let mut app = app_with_text("hello world");
    app.handle_action(Action::Find).unwrap();
    type_into_dialog(&mut app, "zzz");
    app.handle_action(Action::InsertNewline).unwrap();
    assert!(app.search_state.matches.is_empty());
    assert_eq!(app.search_state.active_match, None);
}

#[test]
fn esc_closes_dialog_and_clears_highlights() {
    let mut app = app_with_text("aaa aaa");
    app.handle_action(Action::Find).unwrap();
    type_into_dialog(&mut app, "aaa");
    app.handle_action(Action::InsertNewline).unwrap();
    assert!(!app.search_state.matches.is_empty());
    app.handle_action(Action::MenuClose).unwrap(); // Esc
    assert!(app.find_replace().is_none());
    assert!(
        app.search_state.matches.is_empty(),
        "highlights cleared on close"
    );
}

#[test]
fn dialog_input_does_not_edit_buffer() {
    let mut app = app_with_text("content");
    let before = app.active_buffer().rope.to_string();
    app.handle_action(Action::Find).unwrap();
    type_into_dialog(&mut app, "abc");
    assert_eq!(
        app.active_buffer().rope.to_string(),
        before,
        "typing in the dialog must not change the buffer"
    );
}

// ── US2: Next / Prev with wrap ────────────────────────────────────────────────

#[test]
fn find_next_and_prev_cycle_with_wrap() {
    let mut app = app_with_text("x x x"); // three "x" at positions 0,2,4
    app.handle_action(Action::Find).unwrap();
    type_into_dialog(&mut app, "x");
    app.handle_action(Action::InsertNewline).unwrap();
    let total = app.search_state.matches.len();
    assert_eq!(total, 3);
    let first = app.search_state.active_match.unwrap();

    app.handle_action(Action::FindNext).unwrap();
    let second = app.search_state.active_match.unwrap();
    assert_eq!(second, (first + 1) % total);

    // Advance to the end then wrap.
    app.handle_action(Action::FindNext).unwrap();
    app.handle_action(Action::FindNext).unwrap();
    let wrapped = app.search_state.active_match.unwrap();
    assert_eq!(wrapped, first, "wraps back to the first match");

    // Prev wraps backward.
    app.handle_action(Action::FindPrev).unwrap();
    assert_eq!(
        app.search_state.active_match.unwrap(),
        (first + total - 1) % total
    );
}

// ── US3: Replace ──────────────────────────────────────────────────────────────

#[test]
fn replace_all_replaces_every_occurrence_and_is_undoable() {
    let mut app = app_with_text("cat cat cat");
    app.handle_action(Action::FindReplace).unwrap();
    type_into_dialog(&mut app, "cat"); // query field
    app.handle_action(Action::FocusNextField).unwrap(); // Tab → replacement
    type_into_dialog(&mut app, "dog");
    app.handle_action(Action::SelectAll).unwrap(); // Ctrl+A → Replace All

    assert_eq!(app.active_buffer().rope.to_string(), "dog dog dog");
    // Close the dialog, then Undo restores the pre-replace document in one step.
    app.handle_action(Action::MenuClose).unwrap();
    app.handle_action(Action::Undo).unwrap();
    assert_eq!(app.active_buffer().rope.to_string(), "cat cat cat");
}

#[test]
fn replace_current_replaces_one_and_advances() {
    let mut app = app_with_text("a a a");
    app.handle_action(Action::FindReplace).unwrap();
    type_into_dialog(&mut app, "a");
    app.handle_action(Action::FocusNextField).unwrap();
    type_into_dialog(&mut app, "Z");
    app.handle_action(Action::InsertNewline).unwrap(); // Replace current

    let text = app.active_buffer().rope.to_string();
    assert_eq!(text.matches('Z').count(), 1, "one replacement");
    assert_eq!(text.matches('a').count(), 2, "two remaining");
}

#[test]
fn replace_all_no_match_changes_nothing() {
    let mut app = app_with_text("hello");
    app.handle_action(Action::FindReplace).unwrap();
    type_into_dialog(&mut app, "zzz");
    app.handle_action(Action::FocusNextField).unwrap();
    type_into_dialog(&mut app, "q");
    app.handle_action(Action::SelectAll).unwrap();
    assert_eq!(app.active_buffer().rope.to_string(), "hello");
}

// ── US4: Options ──────────────────────────────────────────────────────────────

#[test]
fn case_sensitive_toggle_changes_matches() {
    let mut app = app_with_text("The the THE");
    app.handle_action(Action::Find).unwrap();
    type_into_dialog(&mut app, "the");
    app.handle_action(Action::InsertNewline).unwrap();
    assert_eq!(app.search_state.matches.len(), 3, "case-insensitive: all 3");

    app.handle_action(Action::ToggleSearchCase).unwrap(); // re-runs
    assert_eq!(
        app.search_state.matches.len(),
        1,
        "case-sensitive: only 'the'"
    );
}

#[test]
fn whole_word_toggle_excludes_substrings() {
    let mut app = app_with_text("cat category cat");
    app.handle_action(Action::Find).unwrap();
    type_into_dialog(&mut app, "cat");
    app.handle_action(Action::InsertNewline).unwrap();
    assert_eq!(
        app.search_state.matches.len(),
        3,
        "substring matches included"
    );

    app.handle_action(Action::ToggleSearchWholeWord).unwrap(); // re-runs
    assert_eq!(
        app.search_state.matches.len(),
        2,
        "whole-word excludes 'category'"
    );
}

#[test]
fn ctrl_a_is_select_all_when_no_dialog_open() {
    let mut app = app_with_text("abc");
    app.handle_action(Action::SelectAll).unwrap();
    assert!(
        app.active_buffer().selection.is_some(),
        "Ctrl+A selects all when no Find/Replace dialog is open"
    );
}
