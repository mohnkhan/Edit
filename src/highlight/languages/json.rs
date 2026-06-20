//! JSON syntax highlighter (Feature 026).
//!
//! Line-based. A `"…"` immediately followed by `:` is styled as a key (type
//! class); other strings are values. Mirrors the candidate + non-overlap
//! resolution used by the other built-in highlighters.

#![allow(dead_code, unused_variables, unused_imports)]

use std::sync::OnceLock;

use ratatui::style::Style;
use regex::Regex;

use crate::highlight::{Highlighter, Span};
use crate::ui::theme::CLASSIC;

fn re_string() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#""[^"\\]*(?:\\.[^"\\]*)*""#).unwrap())
}

fn re_number() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?").unwrap())
}

fn re_literal() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(true|false|null)\b").unwrap())
}

fn re_punct() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[\{\}\[\]:,]").unwrap())
}

/// Syntax highlighter for JSON files (`.json`).
pub struct JsonHighlighter;

impl Highlighter for JsonHighlighter {
    fn name(&self) -> &'static str {
        "JSON"
    }

    fn highlight(&self, line: &str) -> Vec<Span> {
        let key = Style::default().fg(CLASSIC.highlight_type);
        let string = Style::default().fg(CLASSIC.highlight_string);
        let number = Style::default().fg(CLASSIC.highlight_number);
        let keyword = Style::default().fg(CLASSIC.highlight_keyword);
        let operator = Style::default().fg(CLASSIC.highlight_operator);

        let mut candidates: Vec<(usize, usize, Style)> = Vec::new();
        // Strings first (so a ':' inside a string isn't read as punctuation). A
        // string followed (after optional spaces) by ':' is a key.
        for m in re_string().find_iter(line) {
            let after = line[m.end()..].trim_start();
            let style = if after.starts_with(':') { key } else { string };
            candidates.push((m.start(), m.end(), style));
        }
        for m in re_literal().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword));
        }
        for m in re_number().find_iter(line) {
            candidates.push((m.start(), m.end(), number));
        }
        for m in re_punct().find_iter(line) {
            candidates.push((m.start(), m.end(), operator));
        }

        candidates.sort_by_key(|&(start, end, _)| (start, usize::MAX - end));
        let mut spans = Vec::new();
        let mut last_end = 0usize;
        for (start, end, style) in candidates {
            if start >= last_end {
                spans.push(Span { start, end, style });
                last_end = end;
            }
        }
        spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_value_number_literal() {
        let h = JsonHighlighter;
        let line = r#"  "name": "edit", "n": 42, "ok": true, "x": null"#;
        let spans = h.highlight(line);
        let key = Style::default().fg(CLASSIC.highlight_type);
        let string = Style::default().fg(CLASSIC.highlight_string);
        let number = Style::default().fg(CLASSIC.highlight_number);
        let keyword = Style::default().fg(CLASSIC.highlight_keyword);
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "\"name\"" && s.style == key));
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "\"edit\"" && s.style == string));
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "42" && s.style == number));
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "true" && s.style == keyword));
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "null" && s.style == keyword));
    }

    #[test]
    fn spans_sorted_non_overlapping_and_no_panic() {
        let h = JsonHighlighter;
        for line in [
            r#"{"a": [1, 2, "b: c"], "d": -3.5e2}"#,
            "",
            "   ",
            r#""unterminated"#,
            "café: 1 — ünïcode",
            &"1".repeat(4000),
        ] {
            let spans = h.highlight(line);
            let mut last = 0usize;
            for s in &spans {
                assert!(s.start >= last, "overlap at {} in {line:?}", s.start);
                assert!(s.end <= line.len());
                last = s.end;
            }
        }
    }
}
