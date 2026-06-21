// Feature 042: test code may use unwrap/expect freely (re-allow the app-tree deny).
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::config::Config;
use crate::encoding::EncodingId;

fn make_app() -> App {
    App::new(Config::default(), vec![], EncodingId::Utf8, None, None)
}

// ── Feature 020 — interactive/list dialog focus ring ──────────────────────

// T009: ring length and field-stop counts per dialog/mode.
#[test]
fn interactive_ring_math_per_dialog() {
    use crate::ui::dialog::{DialogMode, FindReplaceDialog};
    let mut a = make_app();
    a.terminal_size = (80, 24);
    assert_eq!(a.interactive_ring_len(), 0, "no dialog open");

    a.set_encoding_select(0);
    assert_eq!(a.interactive_field_stops(), 1);
    assert_eq!(a.interactive_ring_len(), 3); // List + OK + Cancel
    a.close_modal();

    a.modal = Modal::PluginManager { cursor: 0 };
    assert_eq!(a.interactive_ring_len(), 2); // List + Close
    a.close_modal();

    a.modal = Modal::FindReplace(FindReplaceDialog::new(DialogMode::Find, String::new()));
    assert_eq!(a.interactive_field_stops(), 1);
    assert_eq!(a.interactive_ring_len(), 3); // Query + Find + Close
    a.modal = Modal::FindReplace(FindReplaceDialog::new(DialogMode::Replace, String::new()));
    assert_eq!(a.interactive_field_stops(), 2);
    assert_eq!(a.interactive_ring_len(), 6); // Query+Replacement + 4 buttons
}

// T009: dialog_focus → primary-control vs button-index boundary.
#[test]
fn interactive_focus_is_button_boundary() {
    let mut a = make_app();
    a.set_encoding_select(0); // field_stops 1, ring 3
    a.dialog_focus = 0;
    assert_eq!(a.interactive_focus_is_button(), None);
    a.dialog_focus = 1;
    assert_eq!(a.interactive_focus_is_button(), Some(0));
    a.dialog_focus = 2;
    assert_eq!(a.interactive_focus_is_button(), Some(1));
}

// T040b: dialog rect + button layout recompute without panic across a range
// of terminal sizes; at a normal size the rect stays within bounds.
#[test]
fn interactive_geometry_across_sizes_no_panic() {
    use crate::ui::dialog::{DialogMode, FindReplaceDialog};
    let sizes = [(80u16, 24u16), (20, 8), (200, 60), (4, 3), (40, 15)];
    for (w, h) in sizes {
        let mut a = make_app();
        a.terminal_size = (w, h);
        for setup in 0..3 {
            a.close_modal();
            a.close_modal();
            a.close_modal();
            match setup {
                0 => a.set_encoding_select(3),
                1 => a.modal = Modal::PluginManager { cursor: 0 },
                _ => {
                    a.modal = Modal::FindReplace(FindReplaceDialog::new(
                        DialogMode::Replace,
                        "abc".into(),
                    ))
                }
            }
            if let Some(r) = a.interactive_dialog_rect() {
                let labels = a.interactive_button_labels();
                // Must not panic on any size (overflow buttons are dropped).
                let rects = crate::ui::buttons::button_rects(r, &labels);
                // Horizontal bound always holds (centered_rect clamps width).
                assert!(r.x + r.width <= w.max(1), "rect within width");
                if w >= 80 && h >= 24 {
                    assert!(!rects.is_empty(), "buttons fit at a normal size");
                }
            }
        }
    }
}

// T040b: a wide/CJK button label is width-measured (no panic, fits its box).
#[test]
fn wide_label_button_rects_are_width_correct() {
    // "あ" is double-width; a 2-grapheme label → width 4 → box width 4+4=8.
    let area = ratatui::layout::Rect::new(0, 0, 60, 10);
    let rects = crate::ui::buttons::button_rects(area, &["ああ", "OK"]);
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].width, 8, "double-width label measured correctly");
}

// T040: each interactive dialog renders a boxed button row with exactly one
// focused control (the focused button shows the `▶` marker exactly once).
#[test]
fn interactive_dialogs_render_one_focused_button() {
    use crate::ui::dialog::{DialogMode, FindReplaceDialog};
    use ratatui::{backend::TestBackend, Terminal};
    let render_marker_count = |app: &mut App| -> usize {
        let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
        t.draw(|f| app.render(f)).unwrap();
        t.backend()
            .buffer()
            .content()
            .iter()
            .filter(|c| c.symbol() == "▶")
            .count()
    };
    for setup in 0..3 {
        let mut a = make_app();
        a.terminal_size = (80, 24);
        match setup {
            0 => a.set_encoding_select(0),
            1 => a.modal = Modal::PluginManager { cursor: 0 },
            _ => {
                a.modal =
                    Modal::FindReplace(FindReplaceDialog::new(DialogMode::Replace, "x".into()))
            }
        }
        // Focus the first button (stop = field_stops) and keep it across the
        // render (ensure_dialog_focus would otherwise reset focus to 0).
        a.dialog_focus_init = true;
        a.dialog_focus = a.interactive_field_stops();
        assert_eq!(
            render_marker_count(&mut a),
            1,
            "exactly one focused button rendered (setup {setup})"
        );
    }
}

// ── Feature 027 — tab-bar-aware editor geometry ──────────────────────────

// T003: editor_top()/viewport_height reflect the tab-bar row only with 2+ buffers.
#[test]
fn editor_top_and_viewport_height_track_tab_bar() {
    let mut a = make_app();
    a.terminal_size = (80, 24);
    a.soft_wrap = false;
    // One buffer → no tab bar; editor at row 1; height = 24-2-hbar(1) = 21.
    assert!(!a.tab_bar_visible());
    assert_eq!(a.editor_top(), 1);
    assert_eq!(a.viewport_height(), 21);
    // Two buffers → tab bar row; editor at row 2; height drops by 1 → 20.
    a.buffers.push(crate::buffer::Buffer::new_empty());
    assert!(a.tab_bar_visible());
    assert_eq!(a.editor_top(), 2);
    assert_eq!(a.viewport_height(), 20);
    // Soft-wrap (no hbar): one buffer 22, two buffers 21.
    a.soft_wrap = true;
    assert_eq!(a.viewport_height(), 21);
}

// T027 (Feature 029): a file-open failure surfaces an "Open failed" status
// rather than silently doing nothing.
#[test]
fn open_failure_surfaces_status() {
    let mut a = make_app();
    let before = a.buffers.len();
    a.handle_open_file(std::path::PathBuf::from(
        "/nonexistent_edit_dir_xyz/nope.txt",
    ));
    assert!(
        a.status_message
            .as_deref()
            .unwrap_or("")
            .contains("Open failed"),
        "open failure is surfaced, got {:?}",
        a.status_message
    );
    assert_eq!(a.buffers.len(), before, "no buffer added on failure");
}

// T025 (Feature 029): editing a read-only buffer surfaces a message instead of
// a silent no-op; copy with a selection reports "Copied".
#[test]
fn readonly_edit_and_copy_give_feedback() {
    let mut a = make_app();
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("abc\n");
    a.buffers[0].readonly = true;
    a.handle_action(Action::InsertChar('x')).unwrap();
    assert_eq!(a.status_message.as_deref(), Some("Buffer is read-only"));
    assert_eq!(a.buffers[0].rope.line_slice(0), "abc", "no edit applied");

    // Copy with a selection reports feedback (clipboard may be unavailable in
    // the test env — accept either the success or the unavailable message).
    a.buffers[0].readonly = false;
    a.status_message = None;
    a.select_all();
    a.copy_selection();
    let msg = a.status_message.as_deref().unwrap_or("");
    assert!(
        msg == "Copied" || msg == "Clipboard unavailable",
        "copy gives feedback, got {msg:?}"
    );
}

// T023 (Feature 029): with line numbers on, a click maps past the gutter; a
// click within the gutter clamps to column 0; horizontal scroll is added.
#[test]
fn click_accounts_for_gutter_and_hscroll() {
    let mut a = make_app();
    a.terminal_size = (80, 24);
    a.config.line_numbers = true;
    a.soft_wrap = false;
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("abcdefghij\n");
    a.active_idx = 0;
    // Gutter is 4 cols; editor_top is row 1 (single buffer). Click at terminal
    // col 4+3=7 → text column 3.
    a.handle_mouse_click(7, 1);
    assert_eq!(a.buffers[0].cursor.grapheme_col, 3);
    // Click within the gutter (col 2) clamps to column 0.
    a.handle_mouse_click(2, 1);
    assert_eq!(a.buffers[0].cursor.grapheme_col, 0);
    // With a horizontal scroll of 2, a click at col 4+1=5 → text column 2+1=3.
    a.buffers[0].scroll_offset.1 = 2;
    a.handle_mouse_click(5, 1);
    assert_eq!(a.buffers[0].cursor.grapheme_col, 3);
}

// T013 (Feature 031): Go-to-Line is a caret-aware digit input.
#[test]
fn goto_line_caret_editing() {
    let mut a = make_app();
    a.modal = Modal::GotoLine {
        digits: String::new(),
        caret: 0,
    };
    if let Modal::GotoLine { caret, .. } = &mut a.modal {
        *caret = 0;
    }
    for c in ['1', '2', '3'] {
        a.handle_action(Action::InsertChar(c)).unwrap();
    }
    assert_eq!(a.goto_line_digits(), Some("123"));
    assert_eq!(a.goto_line_caret(), 3);
    // Home, then insert mid-string.
    a.handle_action(Action::MoveLineStart).unwrap();
    assert_eq!(a.goto_line_caret(), 0);
    a.handle_action(Action::InsertChar('9')).unwrap();
    assert_eq!(a.goto_line_digits(), Some("9123"));
    assert_eq!(a.goto_line_caret(), 1);
    // Non-digit rejected; caret unchanged.
    a.handle_action(Action::InsertChar('x')).unwrap();
    assert_eq!(a.goto_line_digits(), Some("9123"));
    // Right then Backspace removes the grapheme before the caret.
    a.handle_action(Action::MoveRight).unwrap(); // caret 2
    a.handle_action(Action::Backspace).unwrap(); // removes '1' → "923"
    assert_eq!(a.goto_line_digits(), Some("923"));
    assert_eq!(a.goto_line_caret(), 1);
    // End clamps; Left clamps at 0.
    a.handle_action(Action::MoveLineEnd).unwrap();
    assert_eq!(a.goto_line_caret(), 3);
    a.handle_action(Action::MoveLineStart).unwrap();
    a.handle_action(Action::MoveLeft).unwrap();
    assert_eq!(a.goto_line_caret(), 0);
}

// T017 (Feature 029): the save-before-quit prompt cancels on Esc.
#[test]
fn save_prompt_cancels_on_esc() {
    let mut a = make_app();
    a.modal = Modal::SavePrompt;
    a.handle_action(Action::MenuClose).unwrap();
    assert!(!a.is_save_prompt_open(), "Esc cancels the save prompt");
    assert!(a.running, "cancel does not quit");
}

// T019 (Feature 029): Go-to-Line does not open while a menu is active.
#[test]
fn goto_line_does_not_open_over_menu() {
    let mut a = make_app();
    let menus = a.resolved_menus();
    a.menu_bar.open_menu(0, &menus);
    assert!(a.menu_bar.is_active());
    a.handle_action(Action::GoToLine).unwrap();
    assert!(
        a.goto_line_digits().is_none(),
        "Go-to-Line must not open over an active menu"
    );
}

// T021 (Feature 029): completing Save-As applies a pending encoding selection.
#[test]
fn do_save_as_applies_pending_encoding() {
    let mut a = make_app();
    let dir = std::env::temp_dir().join("edit_saveas_enc_test");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("enc.txt");
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("hi\n");
    a.pending_save_as_encoding = Some(EncodingId::Utf16Le);
    a.do_save_as(path);
    assert_eq!(
        a.buffers[0].encoding,
        EncodingId::Utf16Le,
        "encoding applied"
    );
    assert!(
        a.pending_save_as_encoding.is_none(),
        "pending encoding cleared"
    );
}

// T014 (Feature 029): plain save reports success; a failed save reports the
// error and keeps the buffer modified (no silent success-looking failure).
#[test]
fn save_reports_success_and_failure() {
    let mut a = make_app();
    // Success: a real writable temp path.
    let dir = std::env::temp_dir().join("edit_save_fb_test");
    let _ = std::fs::create_dir_all(&dir);
    let ok_path = dir.join("ok.txt");
    a.buffers[0].path = Some(ok_path.clone());
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("hi\n");
    a.buffers[0].modified = true;
    a.handle_save_action();
    assert!(
        a.status_message
            .as_deref()
            .unwrap_or("")
            .starts_with("Saved"),
        "success shows a Saved message, got {:?}",
        a.status_message
    );
    assert!(!a.buffers[0].modified, "clean after a successful save");

    // Failure: a path whose parent directory does not exist → save errors.
    a.status_message = None;
    a.buffers[0].path = Some(std::path::PathBuf::from(
        "/nonexistent_edit_dir_xyz/cannot/write.txt",
    ));
    a.buffers[0].modified = true;
    a.handle_save_action();
    assert!(
        a.status_message
            .as_deref()
            .unwrap_or("")
            .contains("Save failed"),
        "failure is surfaced, got {:?}",
        a.status_message
    );
    assert!(a.buffers[0].modified, "stays modified after a failed save");
}

// T003 (Feature 032): next_word_pos finds the right word boundaries, including
// across line boundaries and at buffer ends.
#[test]
fn next_word_pos_boundaries() {
    let mut a = make_app();
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("foo  bar_baz, café\nsecond\n");
    a.active_idx = 0;
    let put = |a: &mut App, l: usize, g: usize| {
        a.buffers[0].cursor = crate::buffer::CursorPos {
            line: l,
            grapheme_col: g,
            visual_col: g,
        };
    };
    // Right: start of "foo" → start of "bar_baz" (skip word + spaces).
    put(&mut a, 0, 0);
    assert_eq!(a.next_word_pos(Direction::Right), (0, 5));
    // Right: start of "bar_baz" → the comma (its own token, no spaces to skip).
    put(&mut a, 0, 5);
    assert_eq!(a.next_word_pos(Direction::Right), (0, 12));
    // Left: start of "bar_baz" (5) → start of "foo" (0).
    put(&mut a, 0, 5);
    assert_eq!(a.next_word_pos(Direction::Left), (0, 0));
    // Right at end of line 0 → start of line 1.
    put(&mut a, 0, 18);
    assert_eq!(a.next_word_pos(Direction::Right), (1, 0));
    // Left at column 0 of line 1 → end of line 0.
    put(&mut a, 1, 0);
    assert_eq!(a.next_word_pos(Direction::Left), (0, 18));
    // Buffer ends: no-op.
    put(&mut a, 0, 0);
    assert_eq!(a.next_word_pos(Direction::Left), (0, 0));
    let last = a.buffers[0].rope.line_count() - 1;
    let last_len = a.buffers[0].rope.grapheme_count_on_line(last);
    put(&mut a, last, last_len);
    assert_eq!(a.next_word_pos(Direction::Right), (last, last_len));
}

// T007/T010 (Feature 032): word move clears selection; word-select builds it.
#[test]
fn move_and_select_by_word() {
    let mut a = make_app();
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("foo bar baz\n");
    a.active_idx = 0;
    a.buffers[0].cursor = crate::buffer::CursorPos::default();
    // Two word-selects: first → start of "bar" ("foo "), second → start of
    // "baz" ("foo bar ").
    a.move_word_selecting(Direction::Right);
    assert_eq!(a.selection_text().as_deref(), Some("foo "));
    a.move_word_selecting(Direction::Right);
    assert_eq!(a.selection_text().as_deref(), Some("foo bar "));
    assert_eq!(a.buffers[0].cursor.grapheme_col, 8);
    // A plain word move clears the selection.
    a.move_word(Direction::Left);
    assert!(a.buffers[0].selection.is_none());
}

// T013 (Feature 032): word delete is one undo step; respects selection,
// buffer ends, and read-only.
#[test]
fn delete_by_word_behaviors() {
    let mut a = make_app();
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("foo bar baz\n");
    a.active_idx = 0;
    // Cursor after "foo" (gcol 3): Ctrl+Backspace deletes "foo".
    a.buffers[0].cursor = crate::buffer::CursorPos {
        line: 0,
        grapheme_col: 3,
        visual_col: 3,
    };
    a.delete_word(Direction::Left);
    assert_eq!(a.buffers[0].rope.line_slice(0), " bar baz");
    // One undo step restores it.
    a.handle_action(Action::Undo).unwrap();
    assert_eq!(a.buffers[0].rope.line_slice(0), "foo bar baz");
    // Delete forward from start removes "foo " (word + trailing whitespace run).
    a.buffers[0].cursor = crate::buffer::CursorPos::default();
    a.delete_word(Direction::Right);
    assert_eq!(a.buffers[0].rope.line_slice(0), "bar baz");
    // Read-only blocks deletion and reports the message.
    a.handle_action(Action::Undo).unwrap();
    a.buffers[0].readonly = true;
    a.buffers[0].cursor = crate::buffer::CursorPos {
        line: 0,
        grapheme_col: 3,
        visual_col: 3,
    };
    a.delete_word(Direction::Left);
    assert_eq!(
        a.buffers[0].rope.line_slice(0),
        "foo bar baz",
        "read-only: no change"
    );
    assert_eq!(a.status_message.as_deref(), Some("Buffer is read-only"));
}

// T005 (Feature 030): double-click selects the word under the cursor; triple
// selects the line; works over multibyte; degenerate cases don't panic.
#[test]
fn word_and_line_selection() {
    let mut a = make_app();
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("foo bar_baz, café\n");
    a.active_idx = 0;
    let put = |a: &mut App, g: usize| {
        a.buffers[0].cursor = crate::buffer::CursorPos {
            line: 0,
            grapheme_col: g,
            visual_col: g,
        };
    };
    // Cursor in "bar_baz" (underscore is a word char) → whole token.
    put(&mut a, 5);
    a.select_word_at_cursor();
    assert_eq!(a.selection_text().as_deref(), Some("bar_baz"));
    // Cursor in the multibyte word "café".
    put(&mut a, 13);
    a.select_word_at_cursor();
    assert_eq!(a.selection_text().as_deref(), Some("café"));
    // Triple-click selects the whole line content.
    a.select_line_at_cursor();
    assert_eq!(a.selection_text().as_deref(), Some("foo bar_baz, café"));
    // Empty line → no panic, no selection.
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("\n");
    put(&mut a, 0);
    a.select_word_at_cursor();
    assert!(a.buffers[0].selection.is_none());
}

// T005 (Feature 030): click-count classification (single/double/triple) within
// the time+cell window, wrapping after 3.
#[test]
fn editor_click_count_classification() {
    let mut a = make_app();
    assert_eq!(a.next_editor_click_count(5, 5), 1);
    assert_eq!(a.next_editor_click_count(5, 5), 2);
    assert_eq!(a.next_editor_click_count(5, 5), 3);
    assert_eq!(a.next_editor_click_count(5, 5), 1, "wraps after triple");
    // A different cell resets to single.
    assert_eq!(a.next_editor_click_count(9, 9), 1);
}

// T006 (Feature 029): delete_selection over multibyte text removes the right
// characters, records the correct undo text, and never panics.
#[test]
fn delete_selection_is_char_safe_multibyte() {
    use crate::buffer::{CursorPos, Selection};
    let mut a = make_app();
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("éàûü\n");
    a.active_idx = 0;
    let cur = |g: usize| CursorPos {
        line: 0,
        grapheme_col: g,
        visual_col: g,
    };
    // Select the first two graphemes "éà" and delete.
    a.buffers[0].selection = Some(Selection {
        anchor: cur(0),
        active: cur(2),
    });
    a.delete_selection();
    assert_eq!(a.buffers[0].rope.line_slice(0), "ûü");
    assert!(a.buffers[0].selection.is_none());
    // Undo restores the deleted "éà".
    a.handle_action(Action::Undo).unwrap();
    assert_eq!(a.buffers[0].rope.line_slice(0), "éàûü");
}

// T022 (Feature 028): selection_text is char-safe (multibyte) and never panics
// on a degenerate/reversed range.
#[test]
fn selection_text_is_char_safe_and_panic_free() {
    use crate::buffer::{CursorPos, Selection};
    let mut a = make_app();
    // Multibyte content: each "é" is 2 bytes; byte-slicing would risk a panic.
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("éàûü\n");
    a.active_idx = 0;
    let cur = |g: usize| CursorPos {
        line: 0,
        grapheme_col: g,
        visual_col: g,
    };
    // Forward selection of the first two graphemes.
    a.buffers[0].selection = Some(Selection {
        anchor: cur(0),
        active: cur(2),
    });
    assert_eq!(a.selection_text().as_deref(), Some("éà"));
    // Reversed selection yields the same text (ordered internally), no panic.
    a.buffers[0].selection = Some(Selection {
        anchor: cur(4),
        active: cur(2),
    });
    assert_eq!(a.selection_text().as_deref(), Some("ûü"));
    // Degenerate (empty) selection → empty string, no panic.
    a.buffers[0].selection = Some(Selection {
        anchor: cur(1),
        active: cur(1),
    });
    assert_eq!(a.selection_text().as_deref(), Some(""));
    // No selection → None.
    a.buffers[0].selection = None;
    assert_eq!(a.selection_text(), None);
}

// T021b (Feature 028): PageUp/PageDown page the encoding-select and plugin-
// manager lists, clamped to range (no wrap).
#[test]
fn page_keys_clamp_encoding_select_list() {
    let mut a = make_app();
    let n = crate::ui::dialog::ENCODING_OPTIONS.len();
    a.set_encoding_select(0);
    a.handle_action(Action::MovePageDown).unwrap();
    assert_eq!(a.encoding_select_row(), Some(DIALOG_LIST_PAGE.min(n - 1)));
    // Repeated page-downs clamp to the last item.
    for _ in 0..5 {
        a.handle_action(Action::MovePageDown).unwrap();
    }
    assert_eq!(a.encoding_select_row(), Some(n - 1));
    for _ in 0..5 {
        a.handle_action(Action::MovePageUp).unwrap();
    }
    assert_eq!(a.encoding_select_row(), Some(0));
}

#[test]
fn page_keys_clamp_plugin_manager_list() {
    let mut a = make_app();
    // With no plugins installed the list is empty — paging must be a safe no-op.
    a.modal = Modal::PluginManager { cursor: 0 };
    a.handle_action(Action::MovePageDown).unwrap();
    a.handle_action(Action::MovePageUp).unwrap();
    assert_eq!(a.plugin_manager_cursor(), 0);
    assert!(
        a.is_plugin_manager_open(),
        "list paging never closes the dialog"
    );
}

// T017 (Feature 028): Help scrolls from the keyboard with Home/End/Page keys,
// clamped to the content.
#[test]
fn help_keyboard_scroll_clamps() {
    let mut a = make_app();
    a.terminal_size = (80, 24);
    a.modal = Modal::Help {
        screen: HelpScreen::Help,
        scroll: 0,
    };
    let (max_scroll, _page) = a.help_view_metrics(HelpScreen::Help);
    assert!(max_scroll > 0, "Help overflows a 24-row terminal");
    // End → bottom; Home → top.
    a.handle_action(Action::MoveLineEnd).unwrap();
    assert_eq!(a.help_scroll(), max_scroll);
    a.handle_action(Action::MoveLineStart).unwrap();
    assert_eq!(a.help_scroll(), 0);
    // PageDown clamps to max even when pressed many times.
    for _ in 0..50 {
        a.handle_action(Action::MovePageDown).unwrap();
    }
    assert_eq!(a.help_scroll(), max_scroll);
    // Down never exceeds max; Up returns toward 0.
    a.handle_action(Action::MoveDown).unwrap();
    assert_eq!(a.help_scroll(), max_scroll);
    for _ in 0..200 {
        a.handle_action(Action::MoveUp).unwrap();
    }
    assert_eq!(a.help_scroll(), 0);
    // Help is still open (scroll keys don't dismiss it).
    assert_eq!(a.help_screen(), Some(HelpScreen::Help));
}

// T019 (Feature 028): Home/End move the editor cursor to line start/end.
#[test]
fn home_end_move_cursor_to_line_bounds() {
    let mut a = make_app();
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("hello world\n");
    a.active_idx = 0;
    a.handle_action(Action::MoveLineEnd).unwrap();
    assert_eq!(
        a.buffers[0].cursor.grapheme_col,
        "hello world".chars().count()
    );
    a.handle_action(Action::MoveLineStart).unwrap();
    assert_eq!(a.buffers[0].cursor.grapheme_col, 0);
}

// T014 (Feature 028): arrow keys move focus between buttons in a confirm dialog
// (016 ring), consistent with Tab, with wrap-around.
#[test]
fn arrow_keys_move_confirm_dialog_buttons() {
    let mut a = make_app();
    // SavePrompt has 3 buttons (Save/Discard/Cancel); default focus = 2 (Cancel).
    a.modal = Modal::SavePrompt;
    a.handle_action(Action::MoveRight).unwrap(); // ensure sets 2, then next → 0
    assert_eq!(a.dialog_focus, 0);
    a.handle_action(Action::MoveRight).unwrap(); // → 1
    assert_eq!(a.dialog_focus, 1);
    a.handle_action(Action::MoveLeft).unwrap(); // → 0
    assert_eq!(a.dialog_focus, 0);
    a.handle_action(Action::MoveLeft).unwrap(); // wrap → 2
    assert_eq!(a.dialog_focus, 2);
    // Down/Up behave like Right/Left on the single-row button bar.
    a.handle_action(Action::MoveDown).unwrap(); // wrap → 0
    assert_eq!(a.dialog_focus, 0);
    a.handle_action(Action::MoveUp).unwrap(); // wrap → 2
    assert_eq!(a.dialog_focus, 2);
}

// T014 (Feature 028): in an interactive dialog with a button focused, arrows
// cycle the ring; with the primary control focused, arrows are NOT consumed by
// the button ring (they drive the list/field).
#[test]
fn arrow_keys_cycle_interactive_buttons_when_button_focused() {
    use crate::ui::file_browser::{BrowseMode, FileBrowser};
    let mut a = make_app();
    a.modal = Modal::FileBrowser(FileBrowser::open(
        std::path::PathBuf::from("."),
        BrowseMode::Save,
    ));
    let ring = a.interactive_ring_len();
    assert!(ring >= 2, "file browser has a primary control + button(s)");
    // Focus the first button (stop 1); keep init so ensure won't reset to 0.
    a.dialog_focus = 1;
    a.dialog_focus_init = true;
    a.handle_action(Action::MoveRight).unwrap();
    assert_eq!(a.dialog_focus, crate::ui::buttons::next(1, ring));
    a.handle_action(Action::MoveLeft).unwrap();
    assert_eq!(a.dialog_focus, 1);
}

// T011 (Feature 028): opening an interactive dialog resets focus to the primary
// control (stop 0), even if a previous dialog left dialog_focus on a button —
// so typing reaches the field (the Save-As typing bug).
#[test]
fn interactive_dialog_opens_focused_on_primary_control() {
    use crate::ui::file_browser::{BrowseMode, FileBrowser};
    let mut a = make_app();
    // Simulate stale focus left on a button by a prior (now-closed) dialog.
    a.dialog_focus = 2;
    a.dialog_focus_init = false;
    // Open the Save browser.
    a.modal = Modal::FileBrowser(FileBrowser::open(
        std::path::PathBuf::from("."),
        BrowseMode::Save,
    ));
    a.ensure_dialog_focus();
    assert_eq!(a.dialog_focus, 0, "focus resets to the primary field");
    assert!(
        a.interactive_focus_is_button().is_none(),
        "primary control focused, not a button"
    );
}

// T007 (Feature 028): end-to-end render after a soft-wrap buffer switch with a
// stale wrap cache must not panic — the session-restore crash exercised through
// the real render path. The render reads `wrap_cache` as-is (the run loop's
// rebuild has not happened yet), so the renderer's own clamp must protect it.
#[test]
fn render_after_softwrap_buffer_switch_with_stale_cache_no_panic() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut a = make_app();
    a.terminal_size = (80, 24);
    a.soft_wrap = true;
    // Buffer 0: long content; buffer 1: short + empty lines.
    let mut b0 = crate::buffer::Buffer::new_empty();
    b0.rope = crate::buffer::rope::EditorRope::from_str(
        "this is a fairly long line that will wrap several times in a narrow pane\n",
    );
    let mut b1 = crate::buffer::Buffer::new_empty();
    b1.rope = crate::buffer::rope::EditorRope::from_str("ab\n\n");
    a.buffers = vec![b0, b1];
    a.active_idx = 0;
    // Build the wrap cache for buffer 0, then switch to buffer 1 WITHOUT a loop
    // rebuild — the cache now describes the wrong (longer) content.
    a.wrap_cache = Some(crate::ui::wrap::WrapCache::compute(
        &a.buffers[0].rope,
        20,
        a.wrap_text_gen,
    ));
    a.active_idx = 1;
    let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
    t.draw(|f| a.render(f)).unwrap(); // must not panic
}

// T005 (Feature 028): switching/closing the active buffer invalidates the
// soft-wrap cache by bumping wrap_text_gen, so the renderer never reuses stale
// per-line offsets against the new content.
#[test]
fn buffer_changes_invalidate_wrap_cache() {
    let mut a = make_app();
    a.buffers = vec![
        crate::buffer::Buffer::new_empty(),
        crate::buffer::Buffer::new_empty(),
    ];
    a.active_idx = 0;

    let g0 = a.wrap_text_gen;
    a.invalidate_wrap_cache();
    assert_ne!(a.wrap_text_gen, g0, "invalidate bumps the generation");

    let g1 = a.wrap_text_gen;
    a.next_buffer();
    assert_ne!(a.wrap_text_gen, g1, "next_buffer invalidates");

    let g2 = a.wrap_text_gen;
    a.prev_buffer();
    assert_ne!(a.wrap_text_gen, g2, "prev_buffer invalidates");

    let g3 = a.wrap_text_gen;
    a.close_buffer_at(1);
    assert_ne!(a.wrap_text_gen, g3, "close_buffer_at invalidates");
}

// T010: close_buffer_at removes the buffer and keeps the right buffer active.
#[test]
fn close_buffer_at_adjusts_active_index() {
    let mut a = make_app();
    // Four buffers A,B,C,D; active = C (idx 2).
    a.buffers = vec![
        crate::buffer::Buffer::new_empty(),
        crate::buffer::Buffer::new_empty(),
        crate::buffer::Buffer::new_empty(),
        crate::buffer::Buffer::new_empty(),
    ];
    for (i, b) in a.buffers.iter_mut().enumerate() {
        b.path = Some(std::path::PathBuf::from(format!("f{i}.txt")));
    }
    a.active_idx = 2;
    // Close before active → active shifts down to stay on the same buffer.
    a.close_buffer_at(0); // [f1,f2,f3], active was f2 → idx 1
    assert_eq!(a.buffers.len(), 3);
    assert_eq!(a.active_idx, 1);
    assert_eq!(
        a.buffers[a.active_idx].path.as_ref().unwrap().to_str(),
        Some("f2.txt")
    );
    // Close after active → active index unchanged.
    a.close_buffer_at(2); // remove f3 → [f1,f2], active still f2 (idx 1)
    assert_eq!(a.active_idx, 1);
    assert_eq!(
        a.buffers[a.active_idx].path.as_ref().unwrap().to_str(),
        Some("f2.txt")
    );
    // Close the active (last) → previous becomes active.
    a.close_buffer_at(1); // remove f2 → [f1], active clamps to 0
    assert_eq!(a.buffers.len(), 1);
    assert_eq!(a.active_idx, 0);
    // Closing the final buffer replaces it with an empty scratch buffer.
    a.close_buffer_at(0);
    assert_eq!(a.buffers.len(), 1);
    assert_eq!(a.active_idx, 0);
    assert!(a.buffers[0].path.is_none());
}

// T010: tab_close_clicked prompts for a modified buffer, closes a clean one.
#[test]
fn tab_close_clicked_prompts_only_when_modified() {
    let mut a = make_app();
    a.buffers = vec![
        crate::buffer::Buffer::new_empty(),
        crate::buffer::Buffer::new_empty(),
    ];
    a.buffers[1].modified = true;
    a.active_idx = 0;
    // Clean buffer (idx 0) closes immediately, no prompt.
    a.tab_close_clicked(0);
    assert_eq!(a.buffers.len(), 1);
    assert!(a.close_confirm_target().is_none());
    // Re-create a modified second buffer; its [x] opens the confirm.
    a.buffers.push(crate::buffer::Buffer::new_empty());
    a.buffers[1].modified = true;
    a.tab_close_clicked(1);
    assert_eq!(a.buffers.len(), 2, "nothing closed yet");
    assert_eq!(a.close_confirm_target(), Some(1));
    // Discard (button 1) closes it.
    a.activate_dialog_button(1);
    assert_eq!(a.buffers.len(), 1);
    assert!(a.close_confirm_target().is_none());
}

// ── Feature 025 — Go-to-Line prompt render ────────────────────────────────

// T008/L1: the Go-to-Line overlay renders at a normal and a tiny terminal
// without panicking, and shows its title at a normal size.
#[test]
fn goto_line_overlay_renders_without_panic() {
    use ratatui::{backend::TestBackend, Terminal};
    let render = |w: u16, h: u16| -> String {
        let mut a = make_app();
        a.terminal_size = (w, h);
        a.modal = Modal::GotoLine {
            digits: "42".to_string(),
            caret: 0,
        };
        let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
        t.draw(|f| a.render(f)).unwrap();
        t.backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect()
    };
    let big = render(80, 24);
    assert!(big.contains("Go to Line"), "title shown at a normal size");
    // Tiny terminal must not panic.
    let _ = render(10, 3);
    let _ = render(4, 2);
}

// ── Feature 023 — mouse-wheel editor scroll ──────────────────────────────

// T003: wheel_scroll_editor moves the viewport by the step, clamps at top and
// bottom, and never changes the cursor.
#[test]
fn wheel_scroll_editor_clamps_and_keeps_cursor() {
    let mut a = make_app();
    a.terminal_size = (80, 24);
    for _ in 0..50 {
        a.handle_action(Action::InsertNewline).unwrap();
    }
    a.buffers[0].scroll_offset.0 = 0;
    a.buffers[0].cursor.line = 5;
    a.buffers[0].cursor.grapheme_col = 0;
    let cur = a.buffers[0].cursor;

    a.wheel_scroll_editor(0, true, 3);
    assert_eq!(a.buffers[0].scroll_offset.0, 3, "scrolled down by step");
    assert_eq!(a.buffers[0].cursor, cur, "cursor unchanged by wheel scroll");

    a.wheel_scroll_editor(0, false, 3);
    assert_eq!(a.buffers[0].scroll_offset.0, 0, "scrolled back up");
    a.wheel_scroll_editor(0, false, 3);
    assert_eq!(a.buffers[0].scroll_offset.0, 0, "clamped at the top");

    // Drive to the bottom and confirm the clamp.
    for _ in 0..100 {
        a.wheel_scroll_editor(0, true, 3);
    }
    let max = a.buffers[0].rope.line_count().saturating_sub(1);
    assert_eq!(a.buffers[0].scroll_offset.0, max, "clamped at the bottom");
}

// ── Feature 021 — editor scrollbar geometry ──────────────────────────────

// T019: the editor renders its scrollbars (overflowing buffer) with line
// numbers on, in split view, and across a range of sizes without panicking,
// and a scrollbar thumb/track glyph is present.
#[test]
fn editor_scrollbars_render_with_gutter_split_and_resize() {
    use ratatui::{backend::TestBackend, Terminal};
    let bar_glyphs = ['█', '░', '▲', '▼', '◄', '►'];
    let render_has_bar = |app: &mut App, w: u16, h: u16| -> bool {
        let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
        t.draw(|f| app.render(f)).unwrap();
        t.backend().buffer().content().iter().any(|c| {
            c.symbol()
                .chars()
                .next()
                .is_some_and(|g| bar_glyphs.contains(&g))
        })
    };
    for (w, h) in [(80u16, 24u16), (100, 40), (80, 24)] {
        let mut a = make_app();
        a.terminal_size = (w, h);
        a.config.line_numbers = true;
        // A buffer taller than the viewport → vertical scrollbar.
        for _ in 0..(h as usize + 20) {
            a.handle_action(Action::InsertNewline).unwrap();
        }
        assert!(render_has_bar(&mut a, w, h), "single view: scrollbar drawn");
        // Split view renders bars in each pane without panic.
        a.split_mode = crate::ui::SplitMode::Vertical;
        assert!(render_has_bar(&mut a, w, h), "split view: scrollbar drawn");
    }
}

// T007: viewport_height accounts for the reserved horizontal-scrollbar row in
// non-wrap mode, but not in soft-wrap (no horizontal bar there).
#[test]
fn viewport_height_reserves_hbar_row_in_nonwrap_only() {
    let mut a = make_app();
    a.terminal_size = (80, 24);
    a.soft_wrap = false;
    assert_eq!(a.viewport_height(), 21, "24 - menu - status - hbar row");
    a.soft_wrap = true;
    assert_eq!(a.viewport_height(), 22, "soft-wrap: no horizontal bar row");
}

// T007: content_width reserves the rightmost vertical-scrollbar column.
#[test]
fn content_width_reserves_vbar_column() {
    let mut a = make_app();
    a.terminal_size = (80, 24);
    a.config.line_numbers = false;
    assert_eq!(a.content_width(), 79, "80 - vbar column");
    a.config.line_numbers = true;
    assert_eq!(a.content_width(), 75, "80 - gutter(4) - vbar column");
}

// T007: a click on the reserved scrollbar cells does not move the cursor.
#[test]
fn click_on_reserved_scrollbar_cells_is_inert() {
    let mut a = make_app();
    a.terminal_size = (80, 24);
    a.soft_wrap = false;
    for _ in 0..3 {
        a.handle_action(Action::InsertNewline).unwrap();
    }
    a.buffers[0].cursor.line = 1;
    a.buffers[0].cursor.grapheme_col = 0;
    let before = a.buffers[0].cursor;
    // Rightmost column = vertical scrollbar.
    a.handle_mouse_click(79, 3);
    assert_eq!(a.buffers[0].cursor, before, "vbar-column click ignored");
    // Bottom editor row (row 22 = terminal rows - 2) = horizontal scrollbar.
    a.handle_mouse_click(5, 22);
    assert_eq!(a.buffers[0].cursor, before, "hbar-row click ignored");
}

// Feature 018: Help renders a grouped Key|Action table and scrolls.
#[test]
fn help_renders_table_and_scrolls() {
    use ratatui::{backend::TestBackend, Terminal};
    let render = |app: &mut App| -> String {
        let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
        t.draw(|f| app.render(f)).unwrap();
        t.backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect()
    };
    let mut app = make_app();
    app.handle_action(Action::Help).unwrap();
    let top = render(&mut app);
    assert!(top.contains("File"), "section heading shown");
    assert!(top.contains("Ctrl+S"), "a key row shown");
    assert!(
        top.contains("scroll"),
        "scroll hint shown when content overflows"
    );
    assert!(
        !top.contains("Dialogs"),
        "later section not visible before scrolling"
    );

    // Scroll down a lot; the last section becomes visible.
    for _ in 0..40 {
        app.handle_action(Action::MoveDown).unwrap();
    }
    let bottom = render(&mut app);
    assert!(
        bottom.contains("Dialogs"),
        "scrolling reveals later sections"
    );
}

// Feature 019: the Find dialog renders its query in a labeled, bordered
// input box with a caret (matching the file-browser box from feature 018).
#[test]
fn find_dialog_renders_bordered_box_with_caret() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut app = make_app();
    app.handle_action(Action::Find).unwrap();
    for c in "needle".chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
    let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
    t.draw(|f| app.render(f)).unwrap();
    let s: String = t
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();
    assert!(
        s.contains('┌') && s.contains('└') && s.contains('│'),
        "field box borders drawn"
    );
    assert!(s.contains("Find what:"), "field label shown");
    assert!(s.contains('▏'), "caret glyph shown in the focused field");
    assert!(s.contains("needle"), "typed query shown in the box");
    for label in ["Case", "Wrap", "Regex", "Word"] {
        assert!(s.contains(label), "option {label} still shown");
    }
    assert!(s.contains("Esc close"), "hint row still shown");
}

// Feature 019: the Replace dialog renders BOTH fields as bordered boxes; the
// caret appears only in the focused field (FR-005 / contract C-2).
#[test]
fn replace_dialog_renders_two_boxes_and_focused_caret() {
    use ratatui::{backend::TestBackend, Terminal};
    let render = |app: &mut App| -> String {
        let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
        t.draw(|f| app.render(f)).unwrap();
        t.backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect()
    };
    let mut app = make_app();
    app.handle_action(Action::FindReplace).unwrap();
    for c in "foo".chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
    let s = render(&mut app);
    assert!(
        s.contains("Find what:") && s.contains("Replace with:"),
        "both field labels shown"
    );
    // Two bordered boxes => at least two top-left corners.
    assert!(s.matches('┌').count() >= 2, "two field boxes drawn");
    assert_eq!(
        s.matches('▏').count(),
        1,
        "exactly one caret (focused field only)"
    );

    // Switching focus to the replacement field moves the (single) caret there.
    app.handle_action(Action::FocusNextField).unwrap();
    for c in "bar".chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
    let s2 = render(&mut app);
    assert_eq!(
        s2.matches('▏').count(),
        1,
        "still exactly one caret after Tab"
    );
    assert!(s2.contains("bar"), "replacement text rendered in its box");
}

// Feature 019: the taller boxed Replace dialog must render fully within the
// frame at the minimum supported terminal size without panic (FR-009 /
// contract C-5). Below the minimum the app shows its "too small" guard
// instead, so the boundary case is exactly MIN_WIDTH x MIN_HEIGHT.
#[test]
fn replace_dialog_renders_at_minimum_terminal() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut app = make_app();
    app.handle_action(Action::FindReplace).unwrap();
    app.handle_action(Action::InsertChar('x')).unwrap();
    let mut t = Terminal::new(TestBackend::new(MIN_WIDTH, MIN_HEIGHT)).unwrap();
    t.draw(|f| app.render(f)).unwrap();
    let s: String = t
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();
    // Both boxes plus the hint fit at the minimum size.
    assert!(
        s.matches('┌').count() >= 2,
        "both field boxes drawn at min size"
    );
    assert!(
        s.contains("Replace with:"),
        "second field present at min size"
    );
    assert!(s.contains("Esc close"), "hint row present at min size");
}

// Feature 017: Select All renders the selected text with reverse-video.
#[test]
fn select_all_renders_reverse_highlight() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut app = make_app();
    for c in "hello".chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
    app.handle_action(Action::SelectAll).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|f| app.render(f)).unwrap();
    let buf = terminal.backend().buffer();
    // Editor content starts at row 1 (row 0 is the menu bar); no gutter.
    let cell = buf.get(0, 1);
    assert_eq!(cell.symbol(), "h");
    assert!(
        cell.style()
            .add_modifier
            .contains(ratatui::style::Modifier::REVERSED),
        "selected cell rendered with reverse video"
    );
    // A clean buffer (no selection) has no reversed content cell.
    let mut app2 = make_app();
    app2.handle_action(Action::InsertChar('h')).unwrap();
    let mut t2 = Terminal::new(TestBackend::new(80, 24)).unwrap();
    t2.draw(|f| app2.render(f)).unwrap();
    assert!(
        !t2.backend()
            .buffer()
            .get(0, 1)
            .style()
            .add_modifier
            .contains(ratatui::style::Modifier::REVERSED),
        "no selection → no reverse highlight"
    );
}

// Regression (Feature 012 follow-up): render must sync `terminal_size` to the
// real frame so mouse hit-testing uses the same geometry that is drawn. When
// it was stale, clicks inside the visible file-browser box on a non-80x24
// terminal mapped to "outside" and closed the dialog.
#[test]
fn render_syncs_terminal_size_to_frame() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut app = make_app();
    app.terminal_size = (80, 24); // stale default
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal.draw(|f| app.render(f)).unwrap();
    assert_eq!(
        app.terminal_size,
        (120, 40),
        "terminal_size must follow the actual frame size"
    );
}

// Feature 016: the save prompt renders boxed, focusable buttons.
#[test]
fn save_prompt_renders_boxed_buttons() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut app = make_app();
    app.handle_action(Action::InsertChar('x')).unwrap();
    app.handle_action(Action::Quit).unwrap();
    assert!(app.is_save_prompt_open());
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|f| app.render(f)).unwrap();
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();
    assert!(content.contains("Save"), "Save button label drawn");
    assert!(content.contains("Discard"), "Discard button label drawn");
    assert!(content.contains("Cancel"), "Cancel button label drawn");
    assert!(content.contains('▶'), "focused-button marker drawn");
    assert!(
        content.contains('┌') && content.contains('│'),
        "boxed button borders drawn"
    );
}

fn make_app_with_encoding(enc: EncodingId) -> App {
    let mut app = make_app();
    app.buffers[0].encoding = enc;
    app
}

// ── T016 tests ─────────────────────────────────────────────────────────

#[test]
fn test_save_as_encoding_action_opens_dialog() {
    let mut app = make_app(); // UTF-8 buffer (index 0 in ENCODING_OPTIONS)
    app.handle_action(Action::SaveAsEncoding).unwrap();
    assert_eq!(app.encoding_select_row(), Some(0));
}

#[test]
fn test_dialog_preselects_current_encoding() {
    let mut app = make_app_with_encoding(EncodingId::Utf16Le); // index 1
    app.handle_action(Action::SaveAsEncoding).unwrap();
    assert_eq!(app.encoding_select_row(), Some(1));
}

#[test]
fn test_dialog_move_down_increments_idx() {
    let mut app = make_app();
    app.set_encoding_select(1);
    app.handle_action(Action::MoveDown).unwrap();
    assert_eq!(app.encoding_select_row(), Some(2));
}

#[test]
fn test_dialog_move_down_wraps_at_end() {
    let mut app = make_app();
    app.set_encoding_select(6); // last item
    app.handle_action(Action::MoveDown).unwrap();
    assert_eq!(app.encoding_select_row(), Some(0));
}

#[test]
fn test_dialog_move_up_wraps_at_start() {
    let mut app = make_app();
    app.set_encoding_select(0);
    app.handle_action(Action::MoveUp).unwrap();
    assert_eq!(app.encoding_select_row(), Some(6));
}

#[test]
fn test_dialog_escape_closes() {
    let mut app = make_app();
    app.set_encoding_select(3);
    app.handle_action(Action::MenuClose).unwrap();
    assert_eq!(app.encoding_select_row(), None);
}

#[test]
fn test_dialog_other_action_consumed() {
    let mut app = make_app();
    app.set_encoding_select(2);
    let gcol_before = app.buffers[0].cursor.grapheme_col;
    app.handle_action(Action::MoveLeft).unwrap();
    // Dialog state must be preserved (action consumed, not passed to editor).
    assert_eq!(app.encoding_select_row(), Some(2));
    // Cursor must not have moved.
    assert_eq!(app.buffers[0].cursor.grapheme_col, gcol_before);
}

// ── Feature 012 — File browser (Open) ───────────────────────────────────

#[test]
fn test_open_action_opens_browser() {
    let mut app = make_app();
    assert!(app.file_browser().is_none());
    app.handle_action(Action::Open).unwrap();
    let fb = app.file_browser().expect("browser open");
    assert_eq!(fb.mode, BrowseMode::Open);
}

#[test]
fn test_browser_typing_edits_field() {
    let mut app = make_app();
    app.handle_action(Action::Open).unwrap();
    for c in "ab".chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
    assert_eq!(app.file_browser().unwrap().filename, "ab");
    app.handle_action(Action::Backspace).unwrap();
    assert_eq!(app.file_browser().unwrap().filename, "a");
}

#[test]
fn test_browser_escape_cancels_without_opening() {
    let mut app = make_app();
    let n_before = app.buffers.len();
    app.handle_action(Action::Open).unwrap();
    app.handle_action(Action::MenuClose).unwrap();
    assert!(app.file_browser().is_none());
    assert_eq!(app.buffers.len(), n_before, "cancel must not open a buffer");
}

#[test]
fn test_browser_inert_action_keeps_open() {
    let mut app = make_app();
    app.handle_action(Action::Open).unwrap();
    let gcol_before = app.buffers[0].cursor.grapheme_col;
    // ToggleHighlight is consumed by the browser intercept (no effect).
    app.handle_action(Action::ToggleHighlight).unwrap();
    assert!(app.file_browser().is_some());
    assert_eq!(app.buffers[0].cursor.grapheme_col, gcol_before);
}

#[test]
fn test_browser_typed_path_opens_file() {
    // Open mode: typing an absolute file path + Enter loads it (FR-006a).
    let mut path = std::env::temp_dir();
    path.push("edit_browser_open_test_012.txt");
    std::fs::write(&path, "hello from disk\n").expect("write temp file");

    let mut app = make_app();
    let n_before = app.buffers.len();
    app.handle_action(Action::Open).unwrap();
    for c in path.to_string_lossy().chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
    app.handle_action(Action::InsertNewline).unwrap();

    assert!(app.file_browser().is_none(), "browser closes after opening");
    assert_eq!(app.buffers.len(), n_before + 1, "a new buffer is added");
    assert!(app
        .active_buffer()
        .rope
        .to_string()
        .contains("hello from disk"));

    let _ = std::fs::remove_file(&path);
}

// ── Feature 011 — wired menu actions ─────────────────────────────────────

#[test]
fn test_undo_redo_round_trip() {
    let mut app = make_app();
    app.insert_char('a');
    app.insert_char('b');
    assert_eq!(app.active_buffer().rope.to_string(), "ab");
    app.handle_action(Action::Undo).unwrap();
    assert_eq!(app.active_buffer().rope.to_string(), "a");
    app.handle_action(Action::Redo).unwrap();
    assert_eq!(app.active_buffer().rope.to_string(), "ab");
}

#[test]
fn test_undo_empty_reports_nothing() {
    let mut app = make_app();
    app.handle_action(Action::Undo).unwrap();
    assert_eq!(app.status_message.as_deref(), Some("Nothing to undo"));
}

#[test]
fn test_select_all_spans_buffer() {
    let mut app = make_app();
    app.insert_char('x');
    app.insert_char('y');
    app.handle_action(Action::SelectAll).unwrap();
    let sel = app.active_buffer().selection.expect("selection set");
    assert_eq!(sel.anchor.line, 0);
    assert_eq!(sel.anchor.grapheme_col, 0);
    assert_eq!(sel.active.grapheme_col, 2);
}

#[test]
fn test_cut_deletes_selection_without_clipboard() {
    // cut_selection copies (may no-op headless) then deletes — the delete
    // must happen regardless of clipboard availability.
    let mut app = make_app();
    app.insert_char('x');
    app.insert_char('y');
    app.handle_action(Action::SelectAll).unwrap();
    app.handle_action(Action::Cut).unwrap();
    assert_eq!(app.active_buffer().rope.to_string(), "");
}

#[test]
fn test_new_buffer_action_adds_buffer() {
    let mut app = make_app();
    let n = app.buffers.len();
    app.handle_action(Action::New).unwrap();
    assert_eq!(app.buffers.len(), n + 1);
    assert_eq!(app.active_idx, app.buffers.len() - 1);
}

#[test]
fn test_toggle_line_numbers_flips_config() {
    let mut app = make_app();
    let before = app.config.line_numbers;
    app.handle_action(Action::ToggleLineNumbers).unwrap();
    assert_eq!(app.config.line_numbers, !before);
}

#[test]
fn test_about_action_opens_and_closes() {
    let mut app = make_app();
    app.handle_action(Action::About).unwrap();
    assert_eq!(app.help_screen(), Some(HelpScreen::About));
    app.handle_action(Action::MenuClose).unwrap();
    assert_eq!(app.help_screen(), None);
}

#[test]
fn test_save_browser_writes_file() {
    let dir = std::env::temp_dir().join("edit_saveas_test_012");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("saved.txt");
    let _ = std::fs::remove_file(&path);

    let mut app = make_app();
    app.insert_char('h');
    app.insert_char('i');
    app.handle_action(Action::SaveAs).unwrap();
    assert_eq!(app.file_browser().unwrap().mode, BrowseMode::Save);
    // Point the browser at the temp dir, type a filename, confirm.
    app.modal = Modal::FileBrowser(FileBrowser::open(dir.clone(), BrowseMode::Save));
    for c in "saved.txt".chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
    app.handle_action(Action::InsertNewline).unwrap();

    assert!(app.file_browser().is_none(), "browser closes after save");
    let written = std::fs::read_to_string(&path).expect("file written");
    assert!(written.contains("hi"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_save_unnamed_buffer_opens_save_browser() {
    let mut app = make_app(); // make_app starts with an unnamed buffer
    assert!(app.active_buffer().path.is_none());
    app.handle_action(Action::Save).unwrap();
    let fb = app.file_browser().expect("save browser opened");
    assert_eq!(fb.mode, BrowseMode::Save);
}

// ── Feature 011 — mouse menu interaction ─────────────────────────────────

fn mouse_press(col: u16, row: u16) -> crossterm::event::MouseEvent {
    crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: col,
        row,
        modifiers: crossterm::event::KeyModifiers::NONE,
    }
}

// Feature 034: a stale cursor (line past the buffer's content) must NOT crash
// the renderer — it is clamped into range before any line is read. This is the
// root cause of the "line index out of range" panic seen on session restore /
// buffer switch (the renderer indexes lines by cursor.line).
#[test]
fn render_with_stale_cursor_line_is_clamped_not_panicking() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut a = make_app();
    a.terminal_size = (80, 24);
    a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("hi");
    // Park the cursor far past the single line (the stale-cursor condition).
    a.buffers[0].cursor = crate::buffer::CursorPos {
        line: 9,
        grapheme_col: 9,
        visual_col: 9,
    };
    let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
    t.draw(|f| a.render(f)).unwrap(); // must not panic (even in debug)
                                      // Cursor was clamped into range.
    assert_eq!(a.buffers[0].cursor.line, 0);
    assert!(a.buffers[0].cursor.grapheme_col <= 2);
}

// Feature 034: `line_slice` is panic-safe — an out-of-range line in a release
// build returns an empty string instead of crashing (debug builds assert).
#[cfg(not(debug_assertions))]
#[test]
fn line_slice_out_of_range_is_empty_in_release() {
    let r = crate::buffer::rope::EditorRope::from_str("only one line");
    assert_eq!(r.line_slice(7), "");
}

// Feature 033: with the tab bar shown (2+ buffers), an open menu's dropdown
// must overlay the tab-bar row — the first dropdown item used to be hidden
// behind the tab bar (z-order bug).
#[test]
fn open_menu_dropdown_overlays_tab_bar() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut app = make_app();
    app.terminal_size = (80, 24);
    // Two buffers → the tab bar occupies row 1.
    app.buffers.push(crate::buffer::Buffer::new_empty());
    app.active_idx = 0;
    assert!(app.tab_bar_visible());
    // Open the File menu (its dropdown drops from row 0 into row 1+).
    app.menu_bar.state = MenuState::DropDown {
        top_idx: 0,
        item_idx: 0,
    };
    let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
    t.draw(|f| app.render(f)).unwrap();
    // Row 1 must show the first dropdown item ("New"), not be fully covered by
    // the tab bar.
    let row1: String = (0..80)
        .map(|x| t.backend().buffer().get(x, 1).symbol().to_string())
        .collect();
    assert!(
        row1.contains("New"),
        "first File-menu item should overlay the tab bar at row 1, got: {row1:?}"
    );
}

#[test]
fn test_mouse_click_opens_top_level_menu() {
    let mut app = make_app();
    // Click "Edit" (col 7, row 0).
    app.handle_mouse_event(mouse_press(7, 0)).unwrap();
    assert!(matches!(
        app.menu_bar.state,
        MenuState::DropDown { top_idx: 1, .. }
    ));
}

/// Regression: with 2+ buffers the tab bar occupies row 1, but an open menu
/// dropdown overlays it (feature 033). The *first* dropdown item — which lands
/// on row 1 — must still be clickable; previously the tab-bar click handler
/// swallowed the click, so e.g. Search ▸ Find / Help ▸ Help could not be
/// invoked by mouse while keyboard Enter still worked.
#[test]
fn first_dropdown_item_clickable_with_tab_bar_open() {
    let mut app = make_app();
    app.terminal_size = (80, 24);
    // Two buffers → tab bar on row 1.
    app.buffers.push(crate::buffer::Buffer::new_empty());
    app.active_idx = 0;
    assert!(app.tab_bar_visible());

    // Open the Search menu by clicking its title (col 13, row 0), then click
    // its first item ("Find", row 1) — the row the tab bar also occupies.
    app.handle_mouse_event(mouse_press(13, 0)).unwrap();
    assert!(
        matches!(app.menu_bar.state, MenuState::DropDown { top_idx: 2, .. }),
        "clicking Search should open its dropdown"
    );
    app.handle_mouse_event(mouse_press(13, 1)).unwrap();

    // Find ▸ first item fired: the Find dialog is open and the menu closed.
    assert!(
        app.find_replace().is_some(),
        "first Search item should invoke Find, not switch tabs"
    );
    assert!(!app.menu_bar.is_active());
    // The click must not have changed the active buffer.
    assert_eq!(app.active_idx, 0);
}

// Feature 039 (Phase 2): the tab-bar row's owner is resolved by the single
// `LAYER_PRECEDENCE` order, so paint and hit-test agree by construction. This
// generalizes the two point regressions above (`repro_menu_click_over_tabs`,
// `first_dropdown_item_clickable_with_tab_bar_open`) into the precedence
// invariant they were special-casing.
#[test]
fn top_row_owner_follows_layer_precedence() {
    let mut app = make_app();
    app.terminal_size = (80, 24);

    // Single buffer, nothing open: the tab bar is hidden, so the editor owns
    // the top region.
    assert!(!app.tab_bar_visible());
    assert_eq!(app.top_row_owner(), Layer::Editor);

    // 2+ buffers, no dropdown: the tab bar is the topmost active layer.
    app.buffers.push(crate::buffer::Buffer::new_empty());
    assert!(app.tab_bar_visible());
    assert_eq!(app.top_row_owner(), Layer::TabBar);

    // Open a dropdown: it sits ABOVE the tab bar in LAYER_PRECEDENCE and must
    // win the tab row (this is exactly the 033/038 fix, now precedence-derived).
    app.menu_bar.state = MenuState::DropDown {
        top_idx: 0,
        item_idx: 0,
    };
    assert_eq!(app.top_row_owner(), Layer::MenuDropDown);

    // A foreground modal outranks everything (it is dispatched even earlier in
    // the mouse handler, but the precedence ordering must still reflect that).
    app.menu_bar.close_menu();
    app.modal = Modal::Help {
        screen: HelpScreen::Help,
        scroll: 0,
    };
    assert_eq!(app.top_row_owner(), Layer::Modal);

    // Precedence is strictly ordered top→bottom with no duplicates.
    for win in LAYER_PRECEDENCE.windows(2) {
        assert_ne!(win[0], win[1]);
    }
}

#[test]
fn test_mouse_click_activates_dropdown_item() {
    let mut app = make_app();
    // Open File menu, then click the "Open" item (row 2).
    app.handle_mouse_event(mouse_press(1, 0)).unwrap();
    app.handle_mouse_event(mouse_press(3, 2)).unwrap();
    // "Open" → Action::Open → opens the file browser and closes the menu.
    assert!(app.file_browser().is_some());
    assert!(!app.menu_bar.is_active());
}

#[test]
fn test_mouse_click_outside_closes_menu() {
    let mut app = make_app();
    app.handle_mouse_event(mouse_press(1, 0)).unwrap(); // open File
    assert!(app.menu_bar.is_active());
    // Click far down in the editor area.
    app.handle_mouse_event(mouse_press(40, 12)).unwrap();
    assert!(!app.menu_bar.is_active());
}

// ── T017 — Cancel contract ──────────────────────────────────────────────

#[test]
fn test_cancel_does_not_write_and_leaves_encoding_unchanged() {
    let mut app = make_app();
    // Start with UTF-8 encoding.
    assert_eq!(app.buffers[0].encoding, EncodingId::Utf8);
    app.set_encoding_select(3); // e.g. CP437 selected
                                // Cancel via MenuClose.
    app.handle_action(Action::MenuClose).unwrap();
    // Dialog closed.
    assert_eq!(app.encoding_select_row(), None);
    // Encoding unchanged.
    assert_eq!(app.buffers[0].encoding, EncodingId::Utf8);
    // No status message about encoding change.
    assert!(
        app.status_message
            .as_deref()
            .is_none_or(|m| !m.starts_with("Saved as")),
        "cancel must not produce a 'Saved as' message"
    );
}

// ── T018 — Encoding persistence ─────────────────────────────────────────

#[test]
fn test_encoding_persists_on_regular_save() {
    let path = std::env::temp_dir().join("edit_test_persist.txt");
    std::fs::write(&path, b"Hello").unwrap();

    let mut app = App::new(
        Config::default(),
        vec![path.clone()],
        EncodingId::Utf8,
        None,
        None,
    );
    // Save as UTF-16 LE via do_save_as_encoding (Case A).
    app.do_save_as_encoding(EncodingId::Utf16Le);
    // Subsequent regular save must use the new encoding.
    app.buffers[0].save().unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        bytes[0..2],
        [0xFF, 0xFE],
        "file must start with UTF-16 LE BOM"
    );
}

// ── T019 — Dialog reopens with updated preselect ─────────────────────────

#[test]
fn test_dialog_reopens_with_updated_preselect() {
    let path = std::env::temp_dir().join("edit_test_preselect.txt");
    std::fs::write(&path, b"Hello").unwrap();

    let mut app = App::new(
        Config::default(),
        vec![path.clone()],
        EncodingId::Utf8,
        None,
        None,
    );
    app.do_save_as_encoding(EncodingId::Utf16Be);
    let _ = std::fs::remove_file(&path);
    // Re-open dialog — must pre-select UTF-16 BE (index 2).
    app.handle_action(Action::SaveAsEncoding).unwrap();
    assert_eq!(app.encoding_select_row(), Some(2));
}

// ── T022 — Pending encoding cleared on filename-prompt cancel ────────────

#[test]
fn test_unnamed_buf_encoding_cleared_on_filename_cancel() {
    let mut app = make_app(); // unnamed buffer
    app.pending_save_as_encoding = Some(EncodingId::Utf16Le);
    app.cancel_pending_save_as_encoding();
    assert_eq!(app.pending_save_as_encoding, None);
}

#[test]
fn test_unnamed_buf_encoding_applied_after_filename_confirm() {
    let path = std::env::temp_dir().join("edit_test_t022_confirm.txt");
    let mut app = make_app(); // unnamed buffer

    // Simulate: user selected UTF-16 LE via encoding dialog for unnamed buf.
    app.pending_save_as_encoding = Some(EncodingId::Utf16Le);

    // Simulate: user typed a filename and confirmed → handle_save_as called.
    let result = app.handle_save_as(path.clone());
    // The write may fail (no actual FS write in make_app), but the
    // encoding assignment happens before the write. We care that
    // pending_save_as_encoding was consumed and the buffer encoding set.
    assert_eq!(
        app.pending_save_as_encoding, None,
        "pending must be cleared"
    );
    assert_eq!(
        app.active_buffer().encoding,
        EncodingId::Utf16Le,
        "buffer encoding must be updated even if write fails"
    );
    let _ = std::fs::remove_file(&path);
    let _ = result; // allow write failure (unnamed buf has no content path)
}

// ── Feature 005 — Soft-wrap tests (T024, T025) ────────────────────────────

fn make_app_with_long_line() -> App {
    let mut app = make_app();
    // Insert a 60-grapheme line to test soft-wrap
    let long = "A".repeat(60);
    let char_idx = 0;
    app.buffers[0].rope.insert_str(char_idx, &long);
    app.buffers[0].modified = true;
    app.wrap_text_gen = app.wrap_text_gen.wrapping_add(1);
    app
}

#[test]
fn toggle_soft_wrap_on_builds_cache() {
    let mut app = make_app();
    app.terminal_size = (80, 24);
    // Default: soft_wrap is false, no cache.
    assert!(!app.soft_wrap);
    assert!(app.wrap_cache.is_none());
    // Toggle on.
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    assert!(app.soft_wrap, "soft_wrap must be true after toggle");
    assert!(
        app.wrap_cache.is_some(),
        "wrap_cache must be Some after enabling"
    );
}

#[test]
fn toggle_soft_wrap_off_drops_cache_and_resets_hscroll() {
    let mut app = make_app();
    app.terminal_size = (80, 24);
    // Enable then disable.
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    app.buffers[0].scroll_offset.1 = 10; // simulate horizontal scroll while on
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    assert!(
        !app.soft_wrap,
        "soft_wrap must be false after second toggle"
    );
    assert!(
        app.wrap_cache.is_none(),
        "wrap_cache must be None after disabling"
    );
    assert_eq!(
        app.buffers[0].scroll_offset.1, 0,
        "h-scroll must be reset on disable"
    );
}

#[test]
fn soft_wrap_toggle_cycle_cursor_unchanged() {
    let mut app = make_app_with_long_line();
    app.terminal_size = (40, 24);
    // Move cursor to col 5.
    for _ in 0..5 {
        app.move_cursor(Direction::Right);
    }
    let cursor_before = app.buffers[0].cursor;
    // Enable wrap.
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    // Disable wrap.
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    let cursor_after = app.buffers[0].cursor;
    assert_eq!(
        cursor_before.line, cursor_after.line,
        "line must be unchanged"
    );
    assert_eq!(
        cursor_before.grapheme_col, cursor_after.grapheme_col,
        "gcol must be unchanged"
    );
}

#[test]
fn home_on_wrapped_line_goes_to_logical_col_zero() {
    let mut app = make_app();
    app.terminal_size = (20, 24);
    // Insert 50 chars so line wraps multiple times at width 20.
    let long = "ABCDEFGHIJ".repeat(5); // 50 chars
    app.buffers[0].rope.insert_str(0, &long);
    app.buffers[0].modified = true;
    app.wrap_text_gen += 1;
    // Move cursor to middle.
    for _ in 0..25 {
        app.move_cursor(Direction::Right);
    }
    // Enable wrap.
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    // Home should go to grapheme_col 0 of the logical line.
    app.move_line_start();
    assert_eq!(
        app.buffers[0].cursor.grapheme_col, 0,
        "Home must go to col 0 of logical line"
    );
    assert_eq!(app.buffers[0].cursor.line, 0, "line must remain 0");
}

#[test]
fn end_on_wrapped_line_goes_to_logical_line_end() {
    let mut app = make_app();
    app.terminal_size = (20, 24);
    let long = "ABCDEFGHIJ".repeat(5); // 50 chars
    app.buffers[0].rope.insert_str(0, &long);
    app.buffers[0].modified = true;
    app.wrap_text_gen += 1;
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    app.move_line_end();
    assert_eq!(app.buffers[0].cursor.line, 0, "line must remain 0");
    assert_eq!(
        app.buffers[0].cursor.grapheme_col, 50,
        "End must go to col 50"
    );
}

#[test]
fn up_down_move_between_logical_lines_in_wrap_mode() {
    let mut app = make_app();
    app.terminal_size = (20, 24);
    // Line 0: 50 chars (wraps), Line 1: "Second"
    let long = "A".repeat(50);
    app.buffers[0].rope.insert_str(0, &(long + "\nSecond"));
    app.buffers[0].modified = true;
    app.wrap_text_gen += 1;
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    // Cursor on line 0, col 0.
    assert_eq!(app.buffers[0].cursor.line, 0);
    // Down should go to line 1 (the logical next line).
    app.move_cursor(Direction::Down);
    assert_eq!(
        app.buffers[0].cursor.line, 1,
        "Down must go to logical line 1"
    );
}

#[test]
fn save_while_soft_wrap_active_no_extra_newlines() {
    let dir = std::env::temp_dir();
    let path = dir.join("edit_soft_wrap_save_test.txt");
    let content = "A".repeat(200);
    std::fs::write(&path, &content).unwrap();

    let mut app = App::new(
        Config::default(),
        vec![path.clone()],
        EncodingId::Utf8,
        None,
        None,
    );
    app.terminal_size = (40, 24);
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    assert!(app.soft_wrap, "soft_wrap must be enabled");

    // Save.
    app.handle_save_action();

    let saved = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        saved, content,
        "saved bytes must be identical to original content"
    );
}
