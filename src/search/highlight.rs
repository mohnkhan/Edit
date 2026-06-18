//! Task T053: Match-span highlighting for the search subsystem.
//!
//! Converts a slice of [`CharRange`] matches into `(CharRange, Style)` pairs
//! ready for the renderer to paint over the editor area.

#![allow(dead_code, unused_variables, unused_imports)]

use ratatui::style::{Color, Modifier, Style};

use super::CharRange;

// ---------------------------------------------------------------------------
// collect_match_spans
// ---------------------------------------------------------------------------

/// Build a list of `(range, style)` pairs from a slice of search matches.
///
/// - Non-active matches receive a dim yellow-background highlight.
/// - The active match (if any) receives a bright yellow background with a
///   black foreground and bold modifier for maximum visibility.
///
/// The returned vec is in the same order as the input `matches` slice.
pub fn collect_match_spans(
    matches: &[CharRange],
    active_match: Option<usize>,
) -> Vec<(CharRange, Style)> {
    matches
        .iter()
        .enumerate()
        .map(|(i, range)| {
            let style = if active_match == Some(i) {
                // Active match: bright yellow background, black foreground, bold.
                Style::default()
                    .bg(Color::LightYellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                // Non-active match: dim yellow background.
                Style::default().bg(Color::Yellow).fg(Color::DarkGray)
            };
            (range.clone(), style)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::CharRange;

    fn range(start: usize, end: usize) -> CharRange {
        CharRange { start, end }
    }

    #[test]
    fn empty_matches_returns_empty() {
        let spans = collect_match_spans(&[], None);
        assert!(spans.is_empty());
    }

    #[test]
    fn single_match_no_active() {
        let matches = vec![range(0, 3)];
        let spans = collect_match_spans(&matches, None);
        assert_eq!(spans.len(), 1);
        // Should be the dim style (yellow bg, dark gray fg).
        assert_eq!(spans[0].1.bg, Some(Color::Yellow));
        assert_eq!(spans[0].1.fg, Some(Color::DarkGray));
    }

    #[test]
    fn single_match_active() {
        let matches = vec![range(0, 3)];
        let spans = collect_match_spans(&matches, Some(0));
        assert_eq!(spans.len(), 1);
        // Active match: bright yellow bg, black fg.
        assert_eq!(spans[0].1.bg, Some(Color::LightYellow));
        assert_eq!(spans[0].1.fg, Some(Color::Black));
    }

    #[test]
    fn multiple_matches_only_active_is_bright() {
        let matches = vec![range(0, 3), range(8, 11), range(15, 18)];
        let spans = collect_match_spans(&matches, Some(1));
        // Index 0: non-active
        assert_eq!(spans[0].1.bg, Some(Color::Yellow));
        // Index 1: active
        assert_eq!(spans[1].1.bg, Some(Color::LightYellow));
        assert_eq!(spans[1].1.fg, Some(Color::Black));
        // Index 2: non-active
        assert_eq!(spans[2].1.bg, Some(Color::Yellow));
    }

    #[test]
    fn ranges_are_passed_through_unmodified() {
        let matches = vec![range(4, 7), range(10, 13)];
        let spans = collect_match_spans(&matches, None);
        assert_eq!(spans[0].0.start, 4);
        assert_eq!(spans[0].0.end, 7);
        assert_eq!(spans[1].0.start, 10);
        assert_eq!(spans[1].0.end, 13);
    }
}
