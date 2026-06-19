// src/input/mouse.rs
// Tasks T018, T051: Mouse event normalization and menu click hit-testing.
// Translates raw crossterm mouse events into editor-internal types and, where
// applicable, maps them to editor Actions.

#![allow(dead_code, unused_variables, unused_imports)]

use crossterm::event::{MouseButton as CtMouseButton, MouseEvent as CtMouseEvent, MouseEventKind};

use crate::input::keymap::Action;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Editor-internal mouse button identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Coarse-grained kind of mouse interaction understood by the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizedMouseKind {
    Press,
    Release,
    Drag,
    ScrollUp,
    ScrollDown,
}

/// A crossterm mouse event reduced to only the information the editor needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedMouseEvent {
    /// Terminal column (0-based) at which the event occurred.
    pub col: u16,
    /// Terminal row (0-based) at which the event occurred.
    pub row: u16,
    /// Which mouse button was involved (may be `Left` for scroll events by
    /// convention — callers should inspect `kind` first).
    pub button: MouseButton,
    /// What kind of interaction this was.
    pub kind: NormalizedMouseKind,
}

// ---------------------------------------------------------------------------
// Event normalisation
// ---------------------------------------------------------------------------

/// Convert a raw crossterm [`CtMouseEvent`] into a [`NormalizedMouseEvent`].
///
/// Returns `None` for event kinds the editor does not yet handle (e.g.
/// `MouseEventKind::Moved`).
pub fn normalize_mouse(event: CtMouseEvent) -> Option<NormalizedMouseEvent> {
    let col = event.column;
    let row = event.row;

    match event.kind {
        MouseEventKind::Down(btn) => Some(NormalizedMouseEvent {
            col,
            row,
            button: map_button(btn),
            kind: NormalizedMouseKind::Press,
        }),
        MouseEventKind::Up(btn) => Some(NormalizedMouseEvent {
            col,
            row,
            button: map_button(btn),
            kind: NormalizedMouseKind::Release,
        }),
        MouseEventKind::Drag(btn) => Some(NormalizedMouseEvent {
            col,
            row,
            button: map_button(btn),
            kind: NormalizedMouseKind::Drag,
        }),
        MouseEventKind::ScrollUp => Some(NormalizedMouseEvent {
            col,
            row,
            // Scroll events carry no meaningful button; default to Left so
            // callers always have a valid enum value — they should branch on
            // `kind` instead.
            button: MouseButton::Left,
            kind: NormalizedMouseKind::ScrollUp,
        }),
        MouseEventKind::ScrollDown => Some(NormalizedMouseEvent {
            col,
            row,
            button: MouseButton::Left,
            kind: NormalizedMouseKind::ScrollDown,
        }),
        // MouseEventKind::Moved and any future variants are not yet handled.
        _ => None,
    }
}

/// Map a crossterm button to the editor's button type.
fn map_button(btn: CtMouseButton) -> MouseButton {
    match btn {
        CtMouseButton::Left => MouseButton::Left,
        CtMouseButton::Right => MouseButton::Right,
        CtMouseButton::Middle => MouseButton::Middle,
    }
}

// Mouse → menu/cursor routing now lives in `App::handle_mouse_event`
// (src/app.rs), which has the live menu state and terminal geometry needed to
// hit-test dropdown items via `ui::menubar::hit_test_menu`. This module only
// normalises raw crossterm events.

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyModifiers, MouseEvent as CtMouseEvent, MouseEventKind};

    fn make_ct_event(kind: MouseEventKind, col: u16, row: u16) -> CtMouseEvent {
        CtMouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn normalize_down_left() {
        let ev = make_ct_event(MouseEventKind::Down(CtMouseButton::Left), 10, 5);
        let norm = normalize_mouse(ev).expect("should normalise");
        assert_eq!(norm.col, 10);
        assert_eq!(norm.row, 5);
        assert_eq!(norm.button, MouseButton::Left);
        assert_eq!(norm.kind, NormalizedMouseKind::Press);
    }

    #[test]
    fn normalize_up_right() {
        let ev = make_ct_event(MouseEventKind::Up(CtMouseButton::Right), 3, 7);
        let norm = normalize_mouse(ev).expect("should normalise");
        assert_eq!(norm.button, MouseButton::Right);
        assert_eq!(norm.kind, NormalizedMouseKind::Release);
    }

    #[test]
    fn normalize_drag_middle() {
        let ev = make_ct_event(MouseEventKind::Drag(CtMouseButton::Middle), 0, 0);
        let norm = normalize_mouse(ev).expect("should normalise");
        assert_eq!(norm.button, MouseButton::Middle);
        assert_eq!(norm.kind, NormalizedMouseKind::Drag);
    }

    #[test]
    fn normalize_scroll_up() {
        let ev = make_ct_event(MouseEventKind::ScrollUp, 1, 2);
        let norm = normalize_mouse(ev).expect("should normalise");
        assert_eq!(norm.kind, NormalizedMouseKind::ScrollUp);
        assert_eq!(norm.col, 1);
        assert_eq!(norm.row, 2);
    }

    #[test]
    fn normalize_scroll_down() {
        let ev = make_ct_event(MouseEventKind::ScrollDown, 4, 8);
        let norm = normalize_mouse(ev).expect("should normalise");
        assert_eq!(norm.kind, NormalizedMouseKind::ScrollDown);
    }

    #[test]
    fn normalize_moved_returns_none() {
        let ev = make_ct_event(MouseEventKind::Moved, 0, 0);
        assert!(normalize_mouse(ev).is_none());
    }
}
