//! C / C-header syntax highlighter — T071.

#![allow(dead_code, unused_variables, unused_imports)]

use std::sync::OnceLock;

use regex::Regex;
use ratatui::style::{Color, Style};

use crate::highlight::{Highlighter, Span};
use crate::ui::theme::CLASSIC;

// ---------------------------------------------------------------------------
// Lazy-compiled regex patterns
// ---------------------------------------------------------------------------

fn re_keyword() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"\b(auto|break|case|char|const|continue|default|do|double|else|enum|extern|float|for|goto|if|inline|int|long|register|restrict|return|short|signed|sizeof|static|struct|switch|typedef|union|unsigned|void|volatile|while)\b"
        ).unwrap()
    })
}

fn re_string() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#""[^"\\]*(?:\\.[^"\\]*)*""#).unwrap())
}

fn re_char_literal() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"'[^'\\]*(?:\\.[^'\\]*)*'").unwrap())
}

fn re_line_comment() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"//.*").unwrap())
}

fn re_block_comment() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"/\*.*?\*/").unwrap())
}

fn re_number() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b(0[xX][0-9a-fA-F]+[uUlL]*|0[0-7]+[uUlL]*|[0-9]+(\.[0-9]*)?([eE][+-]?[0-9]+)?[fFlLuU]*)\b").unwrap()
    })
}

fn re_preprocessor() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s*#\w+").unwrap())
}

// ---------------------------------------------------------------------------
// CHighlighter
// ---------------------------------------------------------------------------

/// Syntax highlighter for C source files (`.c` and `.h`).
pub struct CHighlighter;

impl Highlighter for CHighlighter {
    fn name(&self) -> &'static str {
        "C"
    }

    fn highlight(&self, line: &str) -> Vec<Span> {
        let keyword_style = Style::default().fg(CLASSIC.highlight_keyword);
        let string_style  = Style::default().fg(CLASSIC.highlight_string);
        let comment_style = Style::default().fg(CLASSIC.highlight_comment);
        let number_style  = Style::default().fg(CLASSIC.highlight_number);

        let mut spans: Vec<Span> = Vec::new();

        // Collect all candidates with priorities:
        //  1. Line comment   (highest priority — rest of line is comment)
        //  2. Block comment  (single-line portion)
        //  3. String literal
        //  4. Char literal
        //  5. Number
        //  6. Preprocessor directive
        //  7. Keyword

        let mut candidates: Vec<(usize, usize, Style)> = Vec::new();

        for m in re_line_comment().find_iter(line) {
            candidates.push((m.start(), m.end(), comment_style));
        }
        for m in re_block_comment().find_iter(line) {
            candidates.push((m.start(), m.end(), comment_style));
        }
        for m in re_string().find_iter(line) {
            candidates.push((m.start(), m.end(), string_style));
        }
        for m in re_char_literal().find_iter(line) {
            candidates.push((m.start(), m.end(), string_style));
        }
        for m in re_number().find_iter(line) {
            candidates.push((m.start(), m.end(), number_style));
        }
        for m in re_preprocessor().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword_style));
        }
        for m in re_keyword().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword_style));
        }

        // Sort by start; resolve overlaps by keeping the first-sorted span.
        candidates.sort_by_key(|&(start, end, _)| (start, usize::MAX - end));

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
    fn keywords_highlighted() {
        let h = CHighlighter;
        let spans = h.highlight("int main(void) {");
        let kw_style = Style::default().fg(CLASSIC.highlight_keyword);
        // Should contain spans for "int" and "void"
        assert!(spans.iter().any(|s| &"int main(void) {"[s.start..s.end] == "int" && s.style == kw_style));
        assert!(spans.iter().any(|s| &"int main(void) {"[s.start..s.end] == "void" && s.style == kw_style));
    }

    #[test]
    fn string_highlighted() {
        let h = CHighlighter;
        let spans = h.highlight(r#"char *s = "hello";"#);
        let str_style = Style::default().fg(CLASSIC.highlight_string);
        assert!(spans.iter().any(|s| s.style == str_style));
    }

    #[test]
    fn line_comment_highlighted() {
        let h = CHighlighter;
        let spans = h.highlight("int x; // comment");
        let cmt_style = Style::default().fg(CLASSIC.highlight_comment);
        assert!(spans.iter().any(|s| s.style == cmt_style));
    }

    #[test]
    fn number_highlighted() {
        let h = CHighlighter;
        let spans = h.highlight("int x = 42;");
        let num_style = Style::default().fg(CLASSIC.highlight_number);
        assert!(spans.iter().any(|s| s.style == num_style));
    }

    #[test]
    fn spans_non_overlapping() {
        let h = CHighlighter;
        let spans = h.highlight(r#"int x = 42; // "comment" with string"#);
        let mut last = 0usize;
        for sp in &spans {
            assert!(sp.start >= last, "overlap at {}", sp.start);
            last = sp.end;
        }
    }
}
