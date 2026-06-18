//! YAML syntax highlighter — T074.

#![allow(dead_code, unused_variables, unused_imports)]

use std::sync::OnceLock;

use ratatui::style::{Color, Style};
use regex::Regex;

use crate::highlight::{Highlighter, Span};
use crate::ui::theme::CLASSIC;

// ---------------------------------------------------------------------------
// Lazy-compiled regex patterns
// ---------------------------------------------------------------------------

/// YAML key: capture group 1 is leading whitespace, group 2 is the key name,
/// followed by optional whitespace and a colon.
///
/// The `regex` crate does not support lookahead, so we match the full
/// `key:` pattern and use the captured key span when emitting the highlight.
fn re_key() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(\s*)([a-zA-Z_][a-zA-Z0-9_ -]*)(\s*:)").unwrap())
}

/// Single-quoted string value following a colon.
fn re_single_string_value() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r":[^\S\n]*'[^']*'").unwrap())
}

/// Double-quoted string value following a colon.
fn re_double_string_value() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#":[^\S\n]*"[^"]*""#).unwrap())
}

/// Bare single-quoted string anywhere.
fn re_single_string() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"'[^']*'").unwrap())
}

/// Bare double-quoted string anywhere.
fn re_double_string() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#""[^"]*""#).unwrap())
}

fn re_comment() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"#.*").unwrap())
}

fn re_boolean_null() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b(true|false|null|yes|no|True|False|Null|YES|NO|TRUE|FALSE|NULL)\b").unwrap()
    })
}

fn re_number() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?\b").unwrap())
}

fn re_anchor_alias() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[&*][A-Za-z_][A-Za-z0-9_]*").unwrap())
}

fn re_document_marker() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(---|\.\.\.)\s*$").unwrap())
}

// ---------------------------------------------------------------------------
// YamlHighlighter
// ---------------------------------------------------------------------------

/// Syntax highlighter for YAML files (`.yaml`, `.yml`).
pub struct YamlHighlighter;

impl Highlighter for YamlHighlighter {
    fn name(&self) -> &'static str {
        "YAML"
    }

    fn highlight(&self, line: &str) -> Vec<Span> {
        let keyword_style = Style::default().fg(CLASSIC.highlight_keyword);
        let string_style = Style::default().fg(CLASSIC.highlight_string);
        let comment_style = Style::default().fg(CLASSIC.highlight_comment);
        let number_style = Style::default().fg(CLASSIC.highlight_number);

        let mut candidates: Vec<(usize, usize, Style)> = Vec::new();

        // Document markers (--- / ...) as keyword color.
        for m in re_document_marker().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword_style));
        }

        // Comments take priority over everything to their right.
        for m in re_comment().find_iter(line) {
            candidates.push((m.start(), m.end(), comment_style));
        }

        // String values (after colon) before bare strings.
        for m in re_single_string_value().find_iter(line) {
            // The match includes the leading `:`, find where the string itself
            // starts by looking for the `'`.
            let text = m.as_str();
            if let Some(q_off) = text.find('\'') {
                let start = m.start() + q_off;
                candidates.push((start, m.end(), string_style));
            }
        }
        for m in re_double_string_value().find_iter(line) {
            let text = m.as_str();
            if let Some(q_off) = text.find('"') {
                let start = m.start() + q_off;
                candidates.push((start, m.end(), string_style));
            }
        }
        // Bare quoted strings.
        for m in re_single_string().find_iter(line) {
            candidates.push((m.start(), m.end(), string_style));
        }
        for m in re_double_string().find_iter(line) {
            candidates.push((m.start(), m.end(), string_style));
        }

        // Booleans / null.
        for m in re_boolean_null().find_iter(line) {
            candidates.push((m.start(), m.end(), number_style));
        }

        // Numbers.
        for m in re_number().find_iter(line) {
            candidates.push((m.start(), m.end(), number_style));
        }

        // Anchors and aliases.
        for m in re_anchor_alias().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword_style));
        }

        // Keys: use capture group 2 (the identifier itself) so leading
        // whitespace and the trailing `:` are excluded from the highlight.
        for caps in re_key().captures_iter(line) {
            if let Some(key_match) = caps.get(2) {
                candidates.push((key_match.start(), key_match.end(), keyword_style));
            }
        }

        // Sort by start offset; longest span wins ties at the same start.
        candidates.sort_by_key(|&(start, end, _)| (start, usize::MAX - end));

        let mut spans: Vec<Span> = Vec::new();
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_highlighted() {
        let h = YamlHighlighter;
        let line = "name: Alice";
        let spans = h.highlight(line);
        let kw_style = Style::default().fg(CLASSIC.highlight_keyword);
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "name" && s.style == kw_style));
    }

    #[test]
    fn comment_highlighted() {
        let h = YamlHighlighter;
        let spans = h.highlight("  # this is a comment");
        let cmt_style = Style::default().fg(CLASSIC.highlight_comment);
        assert!(spans.iter().any(|s| s.style == cmt_style));
    }

    #[test]
    fn boolean_highlighted() {
        let h = YamlHighlighter;
        let line = "enabled: true";
        let spans = h.highlight(line);
        let num_style = Style::default().fg(CLASSIC.highlight_number);
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "true" && s.style == num_style));
    }

    #[test]
    fn spans_non_overlapping() {
        let h = YamlHighlighter;
        let spans = h.highlight(r#"host: "localhost" # comment"#);
        let mut last = 0usize;
        for sp in &spans {
            assert!(sp.start >= last);
            last = sp.end;
        }
    }
}
