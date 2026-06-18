//! Input handling: event dispatch, keybindings, and mouse events.
//!
//! The main entry point is [`dispatch_event`], which maps crossterm events to
//! [`Action`] values consumed by the application state machine.

#![allow(dead_code)]

pub mod keymap;
pub mod mouse;

pub use keymap::{Action, KeybindingMap};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Dispatch a crossterm [`Event`] into an optional [`Action`].
///
/// Returns `None` for events that should be silently ignored (e.g. key-up on
/// platforms that report them, unsupported mouse events).
pub fn dispatch_event(event: Event, keymap: &KeybindingMap) -> Option<Action> {
    match event {
        Event::Key(key_event) => dispatch_key(key_event, keymap),
        Event::Mouse(mouse_event) => {
            let normalized = mouse::normalize_mouse(mouse_event)?;
            mouse::handle_mouse(normalized, 0)
        }
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
