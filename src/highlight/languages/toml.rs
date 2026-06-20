//! TOML syntax highlighter (Feature 026).
//!
//! Line-based. Table / array-of-table headers, bare keys (before `=`), strings,
//! numbers and dates, booleans, and `#` comments. Mirrors the candidate +
//! non-overlap resolution used by the other built-in highlighters.

#![allow(dead_code, unused_variables, unused_imports)]

use std::sync::OnceLock;

use ratatui::style::Style;
use regex::Regex;

use crate::highlight::{Highlighter, Span};
use crate::ui::theme::CLASSIC;

fn re_header() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s*\[\[?[^\]]*\]\]?").unwrap())
}

fn re_key() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // A bare key at the start of the line, before `=` (captures the key text).
    RE.get_or_init(|| Regex::new(r"^\s*([A-Za-z0-9_.\-]+)\s*=").unwrap())
}

fn re_string() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#""[^"\\]*(?:\\.[^"\\]*)*"|'[^']*'"#).unwrap())
}

fn re_comment() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"#.*").unwrap())
}

fn re_date() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\d{4}-\d{2}-\d{2}([Tt ]\d{2}:\d{2}:\d{2}(\.\d+)?([Zz]|[+-]\d{2}:\d{2})?)?")
            .unwrap()
    })
}

fn re_number() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"[+-]?\b\d[\d_]*(\.\d+)?([eE][+-]?\d+)?\b|0[xX][0-9a-fA-F_]+").unwrap()
    })
}

fn re_bool() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(true|false)\b").unwrap())
}

/// Syntax highlighter for TOML files (`.toml`).
pub struct TomlHighlighter;

impl Highlighter for TomlHighlighter {
    fn name(&self) -> &'static str {
        "TOML"
    }

    fn highlight(&self, line: &str) -> Vec<Span> {
        let header = Style::default().fg(CLASSIC.highlight_type);
        let keyword = Style::default().fg(CLASSIC.highlight_keyword);
        let string = Style::default().fg(CLASSIC.highlight_string);
        let number = Style::default().fg(CLASSIC.highlight_number);
        let comment = Style::default().fg(CLASSIC.highlight_comment);

        let mut candidates: Vec<(usize, usize, Style)> = Vec::new();
        if let Some(m) = re_header().find(line) {
            candidates.push((m.start(), m.end(), header));
        }
        if let Some(caps) = re_key().captures(line) {
            if let Some(k) = caps.get(1) {
                candidates.push((k.start(), k.end(), keyword));
            }
        }
        // Strings before comments so a `#` inside a string is not a comment.
        for m in re_string().find_iter(line) {
            candidates.push((m.start(), m.end(), string));
        }
        for m in re_comment().find_iter(line) {
            candidates.push((m.start(), m.end(), comment));
        }
        for m in re_date().find_iter(line) {
            candidates.push((m.start(), m.end(), number));
        }
        for m in re_number().find_iter(line) {
            candidates.push((m.start(), m.end(), number));
        }
        for m in re_bool().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword));
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
    fn header_key_string_comment() {
        let h = TomlHighlighter;
        let line = r#"name = "edit" # the editor"#;
        let spans = h.highlight(line);
        let keyword = Style::default().fg(CLASSIC.highlight_keyword);
        let string = Style::default().fg(CLASSIC.highlight_string);
        let comment = Style::default().fg(CLASSIC.highlight_comment);
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "name" && s.style == keyword));
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "\"edit\"" && s.style == string));
        assert!(spans
            .iter()
            .any(|s| line[s.start..s.end].starts_with('#') && s.style == comment));

        let hdr = h.highlight("[package]");
        assert!(hdr
            .iter()
            .any(|s| &"[package]"[s.start..s.end] == "[package]"
                && s.style == Style::default().fg(CLASSIC.highlight_type)));
        let aot = h.highlight("[[bin]]");
        assert!(aot.iter().any(|s| &"[[bin]]"[s.start..s.end] == "[[bin]]"));
    }

    #[test]
    fn number_bool_date() {
        let h = TomlHighlighter;
        let line = "x = 42  y = true  d = 2026-06-20T13:45:00Z";
        let spans = h.highlight(line);
        let number = Style::default().fg(CLASSIC.highlight_number);
        let keyword = Style::default().fg(CLASSIC.highlight_keyword);
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "42" && s.style == number));
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "true" && s.style == keyword));
        assert!(spans
            .iter()
            .any(|s| line[s.start..s.end].starts_with("2026-06-20") && s.style == number));
    }

    #[test]
    fn spans_sorted_non_overlapping_and_no_panic() {
        let h = TomlHighlighter;
        for line in [
            r#"key = "a # not comment" # real"#,
            "[a.b.c]",
            "",
            "   ",
            r#"k = "unterminated"#,
            "café = \"ünïcode\" # δ",
            &"x".repeat(4000),
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
