//! Input handling: event dispatch, keybindings, and mouse events.
//!
//! The main entry point is [`dispatch_event`], which maps crossterm events to
//! [`Action`] values consumed by the application state machine.

#![allow(dead_code)]

pub mod keymap;
pub mod mouse;

pub use keymap::{Action, KeybindingMap};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, ModifierKeyCode};

/// Dispatch a crossterm [`Event`] into an optional [`Action`].
///
/// Returns `None` for events that should be silently ignored (e.g. key-up on
/// platforms that report them, unsupported mouse events).
pub fn dispatch_event(event: Event, keymap: &KeybindingMap) -> Option<Action> {
    match event {
        Event::Key(key_event) => dispatch_key(key_event, keymap),
        // Mouse events are handled directly by the app (App::handle_mouse_event),
        // which has the cursor coordinates and live menu state needed to hit-test
        // dropdown items. dispatch_event therefore ignores them.
        Event::Mouse(_) => None,
        Event::Resize(w, h) => Some(handle_resize(w, h)),
        Event::FocusGained | Event::FocusLost | Event::Paste(_) => None,
    }
}

/// Convert a key press to an [`Action`], checking the keymap first then
/// falling back to character insertion.
fn dispatch_key(key: KeyEvent, keymap: &KeybindingMap) -> Option<Action> {
    // Ignore key-release and key-repeat events on platforms that report them
    if key.kind == KeyEventKind::Release {
        return None;
    }

    // Feature 013: a lone Alt key (no other key) activates the menu bar like F10.
    // Only delivered by terminals that report modifier-only keys (keyboard
    // enhancement); a no-op everywhere else (graceful degradation).
    if let KeyCode::Modifier(ModifierKeyCode::LeftAlt | ModifierKeyCode::RightAlt) = key.code {
        return Some(Action::Menu);
    }

    // Build a canonical key string for keymap lookup
    let key_str = key_to_string(&key);
    if let Some(action) = keymap.get_action(&key_str) {
        return Some(action.clone());
    }

    // Fall back to character insertion for printable characters
    match key.code {
        KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => Some(Action::InsertChar(c)),
        KeyCode::Char(c) if key.modifiers == KeyModifiers::SHIFT => Some(Action::InsertChar(c)),
        KeyCode::Enter => Some(Action::InsertNewline),
        KeyCode::Backspace => Some(Action::Backspace),
        KeyCode::Delete => Some(Action::Delete),
        KeyCode::Left => Some(Action::MoveLeft),
        KeyCode::Right => Some(Action::MoveRight),
        KeyCode::Up => Some(Action::MoveUp),
        KeyCode::Down => Some(Action::MoveDown),
        KeyCode::Home => Some(Action::MoveLineStart),
        KeyCode::End => Some(Action::MoveLineEnd),
        KeyCode::PageUp => Some(Action::MovePageUp),
        KeyCode::PageDown => Some(Action::MovePageDown),
        _ => None,
    }
}

/// Produce a canonical string representation of a key event for keymap lookup.
///
/// Format: `[Alt+][Ctrl+][Shift+]<key>`  e.g. `"Ctrl+S"`, `"Alt+F"`, `"F1"`.
pub fn key_to_string(key: &KeyEvent) -> String {
    let mut parts = String::new();
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push_str("Alt+");
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push_str("Ctrl+");
    }
    // Only emit "Shift+" for non-character keys where Shift is meaningful
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        match key.code {
            KeyCode::Char(_) => {}
            _ => parts.push_str("Shift+"),
        }
    }
    let key_name = match key.code {
        KeyCode::Char(c) => c.to_uppercase().to_string(),
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        _ => return String::new(),
    };
    parts.push_str(&key_name);
    parts
}

/// Produce a resize action from terminal dimensions.
pub fn handle_resize(w: u16, h: u16) -> Action {
    Action::Resize(w, h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventState;

    fn key(code: KeyCode, mods: KeyModifiers, kind: KeyEventKind) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: mods,
            kind,
            state: KeyEventState::NONE,
        }
    }

    // T019 (Feature 028): Home/End map to line-start/line-end; PageUp/PageDown to
    // the page actions (regression guard for the keyboard-nav contract).
    #[test]
    fn home_end_pageup_pagedown_map_to_movement_actions() {
        let km = KeybindingMap::default_map();
        let press = |c| key(c, KeyModifiers::NONE, KeyEventKind::Press);
        assert_eq!(
            dispatch_key(press(KeyCode::Home), &km),
            Some(Action::MoveLineStart)
        );
        assert_eq!(
            dispatch_key(press(KeyCode::End), &km),
            Some(Action::MoveLineEnd)
        );
        assert_eq!(
            dispatch_key(press(KeyCode::PageUp), &km),
            Some(Action::MovePageUp)
        );
        assert_eq!(
            dispatch_key(press(KeyCode::PageDown), &km),
            Some(Action::MovePageDown)
        );
    }

    // T022 (feature 013): a lone Alt key press activates the menu bar (like F10).
    #[test]
    fn lone_alt_press_maps_to_menu() {
        let km = KeybindingMap::default_map();
        for m in [ModifierKeyCode::LeftAlt, ModifierKeyCode::RightAlt] {
            let ev = key(
                KeyCode::Modifier(m),
                KeyModifiers::NONE,
                KeyEventKind::Press,
            );
            assert_eq!(dispatch_key(ev, &km), Some(Action::Menu));
        }
    }

    #[test]
    fn lone_alt_release_is_ignored() {
        let km = KeybindingMap::default_map();
        let ev = key(
            KeyCode::Modifier(ModifierKeyCode::LeftAlt),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        );
        assert_eq!(dispatch_key(ev, &km), None);
    }

    #[test]
    fn alt_letter_still_opens_menu_no_regression() {
        let km = KeybindingMap::default_map();
        let ev = key(KeyCode::Char('f'), KeyModifiers::ALT, KeyEventKind::Press);
        assert_eq!(dispatch_key(ev, &km), Some(Action::MenuFile));
    }

    #[test]
    fn plain_letter_still_inserts() {
        let km = KeybindingMap::default_map();
        let ev = key(KeyCode::Char('x'), KeyModifiers::NONE, KeyEventKind::Press);
        assert_eq!(dispatch_key(ev, &km), Some(Action::InsertChar('x')));
    }
}
