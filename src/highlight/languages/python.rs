//! Python syntax highlighter — T072.

#![allow(dead_code, unused_variables, unused_imports)]

use std::sync::OnceLock;

use ratatui::style::{Color, Style};
use regex::Regex;

use crate::highlight::{Highlighter, Span};
use crate::ui::theme::CLASSIC;

// ---------------------------------------------------------------------------
// Lazy-compiled regex patterns
// ---------------------------------------------------------------------------

fn re_keyword() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"\b(False|None|True|and|as|assert|async|await|break|class|continue|def|del|elif|else|except|finally|for|from|global|if|import|in|is|lambda|nonlocal|not|or|pass|raise|return|try|while|with|yield)\b"
        ).unwrap()
    })
}

fn re_double_string() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#""[^"\\]*(?:\\.[^"\\]*)*""#).unwrap())
}

fn re_single_string() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"'[^'\\]*(?:\\.[^'\\]*)*'").unwrap())
}

fn re_comment() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"#.*").unwrap())
}

fn re_number() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b(0[xX][0-9a-fA-F]+|0[oO][0-7]+|0[bB][01]+|[0-9]+(\.[0-9]*)?([eE][+-]?[0-9]+)?[jJ]?)\b").unwrap()
    })
}

fn re_decorator() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s*@\w+").unwrap())
}

// ---------------------------------------------------------------------------
// PythonHighlighter
// ---------------------------------------------------------------------------

/// Syntax highlighter for Python source files (`.py`).
pub struct PythonHighlighter;

impl Highlighter for PythonHighlighter {
    fn name(&self) -> &'static str {
        "Python"
    }

    fn highlight(&self, line: &str) -> Vec<Span> {
        let keyword_style = Style::default().fg(CLASSIC.highlight_keyword);
        let string_style = Style::default().fg(CLASSIC.highlight_string);
        let comment_style = Style::default().fg(CLASSIC.highlight_comment);
        let number_style = Style::default().fg(CLASSIC.highlight_number);

        let mut candidates: Vec<(usize, usize, Style)> = Vec::new();

        // Comment wins over everything else on the same column.
        for m in re_comment().find_iter(line) {
            candidates.push((m.start(), m.end(), comment_style));
        }
        for m in re_double_string().find_iter(line) {
            candidates.push((m.start(), m.end(), string_style));
        }
        for m in re_single_string().find_iter(line) {
            candidates.push((m.start(), m.end(), string_style));
        }
        for m in re_number().find_iter(line) {
            candidates.push((m.start(), m.end(), number_style));
        }
        for m in re_decorator().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword_style));
        }
        for m in re_keyword().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword_style));
        }

        // Sort by start; resolve overlaps greedily.
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
    fn def_keyword_highlighted() {
        let h = PythonHighlighter;
        let line = "def foo(x):";
        let spans = h.highlight(line);
        let kw_style = Style::default().fg(CLASSIC.highlight_keyword);
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "def" && s.style == kw_style));
    }

    #[test]
    fn comment_highlighted() {
        let h = PythonHighlighter;
        let spans = h.highlight("x = 1  # comment");
        let cmt_style = Style::default().fg(CLASSIC.highlight_comment);
        assert!(spans.iter().any(|s| s.style == cmt_style));
    }

    #[test]
    fn string_highlighted() {
        let h = PythonHighlighter;
        let spans = h.highlight(r#"x = "hello""#);
        let str_style = Style::default().fg(CLASSIC.highlight_string);
        assert!(spans.iter().any(|s| s.style == str_style));
    }

    #[test]
    fn none_highlighted_as_keyword() {
        let h = PythonHighlighter;
        let line = "x = None";
        let spans = h.highlight(line);
        let kw_style = Style::default().fg(CLASSIC.highlight_keyword);
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "None" && s.style == kw_style));
    }

    #[test]
    fn spans_non_overlapping() {
        let h = PythonHighlighter;
        let spans = h.highlight(r#"def foo(x): # comment with "string""#);
        let mut last = 0usize;
        for sp in &spans {
            assert!(sp.start >= last);
            last = sp.end;
        }
    }
}
