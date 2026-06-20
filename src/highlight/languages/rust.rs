//! Rust syntax highlighter (Feature 026).
//!
//! Line-based, best-effort (multi-line block comments / strings are styled per
//! line, matching the other built-in highlighters). Mirrors the candidate +
//! non-overlap resolution used by [`crate::highlight::languages::c`].

#![allow(dead_code, unused_variables, unused_imports)]

use std::sync::OnceLock;

use ratatui::style::Style;
use regex::Regex;

use crate::highlight::{Highlighter, Span};
use crate::ui::theme::CLASSIC;

fn re_line_comment() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"//.*").unwrap())
}

fn re_block_comment() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"/\*.*?\*/").unwrap())
}

fn re_attribute() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"#!?\[[^\]]*\]").unwrap())
}

fn re_raw_string() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r##"b?r#*"[^"]*"#*"##).unwrap())
}

fn re_string() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#""[^"\\]*(?:\\.[^"\\]*)*""#).unwrap())
}

fn re_char() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // char or byte-char literal, e.g. 'a', '\n', b'x'. Lifetimes ('a) won't have
    // a closing quote so they are not matched.
    RE.get_or_init(|| Regex::new(r"b?'(?:\\.|[^'\\])'").unwrap())
}

fn re_macro() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*!").unwrap())
}

fn re_number() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"\b(0[xX][0-9a-fA-F_]+|0[oO][0-7_]+|0[bB][01_]+|[0-9][0-9_]*(\.[0-9_]+)?([eE][+-]?[0-9_]+)?)(([iuf])(8|16|32|64|128|size))?\b",
        )
        .unwrap()
    })
}

fn re_keyword() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"\b(as|async|await|break|const|continue|crate|dyn|else|enum|extern|false|fn|for|if|impl|in|let|loop|match|mod|move|mut|pub|ref|return|self|Self|static|struct|super|trait|true|type|union|unsafe|use|where|while)\b",
        )
        .unwrap()
    })
}

fn re_type() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Primitive types + any CamelCase identifier (best-effort type detection).
    RE.get_or_init(|| {
        Regex::new(
            r"\b(u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize|f32|f64|bool|char|str|String|Vec|Option|Result|Box|Rc|Arc|[A-Z][a-zA-Z0-9_]*)\b",
        )
        .unwrap()
    })
}

/// Syntax highlighter for Rust source files (`.rs`).
pub struct RustHighlighter;

impl Highlighter for RustHighlighter {
    fn name(&self) -> &'static str {
        "Rust"
    }

    fn highlight(&self, line: &str) -> Vec<Span> {
        let keyword = Style::default().fg(CLASSIC.highlight_keyword);
        let type_s = Style::default().fg(CLASSIC.highlight_type);
        let string = Style::default().fg(CLASSIC.highlight_string);
        let number = Style::default().fg(CLASSIC.highlight_number);
        let comment = Style::default().fg(CLASSIC.highlight_comment);
        let operator = Style::default().fg(CLASSIC.highlight_operator);

        // Priority order: comments and attributes first, then strings/chars, then
        // macros, numbers, keywords, and finally CamelCase types.
        let mut candidates: Vec<(usize, usize, Style)> = Vec::new();
        for m in re_line_comment().find_iter(line) {
            candidates.push((m.start(), m.end(), comment));
        }
        for m in re_block_comment().find_iter(line) {
            candidates.push((m.start(), m.end(), comment));
        }
        for m in re_attribute().find_iter(line) {
            candidates.push((m.start(), m.end(), operator));
        }
        for m in re_raw_string().find_iter(line) {
            candidates.push((m.start(), m.end(), string));
        }
        for m in re_string().find_iter(line) {
            candidates.push((m.start(), m.end(), string));
        }
        for m in re_char().find_iter(line) {
            candidates.push((m.start(), m.end(), string));
        }
        for m in re_macro().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword));
        }
        for m in re_number().find_iter(line) {
            candidates.push((m.start(), m.end(), number));
        }
        for m in re_keyword().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword));
        }
        for m in re_type().find_iter(line) {
            candidates.push((m.start(), m.end(), type_s));
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

    fn styled<'a>(spans: &'a [Span], line: &'a str, text: &str, style: Style) -> bool {
        spans
            .iter()
            .any(|s| &line[s.start..s.end] == text && s.style == style)
    }

    #[test]
    fn keywords_types_numbers_comment() {
        let h = RustHighlighter;
        let line = "fn main() { let x: u32 = 1; // note }";
        let spans = h.highlight(line);
        assert!(styled(
            &spans,
            line,
            "fn",
            Style::default().fg(CLASSIC.highlight_keyword)
        ));
        assert!(styled(
            &spans,
            line,
            "let",
            Style::default().fg(CLASSIC.highlight_keyword)
        ));
        assert!(styled(
            &spans,
            line,
            "u32",
            Style::default().fg(CLASSIC.highlight_type)
        ));
        assert!(styled(
            &spans,
            line,
            "1",
            Style::default().fg(CLASSIC.highlight_number)
        ));
        assert!(spans.iter().any(|s| line[s.start..s.end].starts_with("//")
            && s.style == Style::default().fg(CLASSIC.highlight_comment)));
    }

    #[test]
    fn strings_macros_attributes() {
        let h = RustHighlighter;
        let line = r#"#[derive(Debug)] let s = "hi"; println!("x");"#;
        let spans = h.highlight(line);
        let string = Style::default().fg(CLASSIC.highlight_string);
        let operator = Style::default().fg(CLASSIC.highlight_operator);
        let keyword = Style::default().fg(CLASSIC.highlight_keyword);
        assert!(
            spans.iter().any(|s| s.style == operator),
            "attribute styled"
        );
        assert!(spans
            .iter()
            .any(|s| &line[s.start..s.end] == "\"hi\"" && s.style == string));
        assert!(spans
            .iter()
            .any(|s| line[s.start..s.end].ends_with('!') && s.style == keyword));
    }

    #[test]
    fn spans_sorted_non_overlapping() {
        let h = RustHighlighter;
        let line = r#"let n: u64 = 0xFF_u64; // "x" String foo!()"#;
        let spans = h.highlight(line);
        let mut last = 0usize;
        for s in &spans {
            assert!(s.start >= last, "overlap at {}", s.start);
            assert!(s.end <= line.len());
            last = s.end;
        }
    }

    #[test]
    fn no_panic_on_edge_inputs() {
        let h = RustHighlighter;
        for line in [
            "",
            "   ",
            "let s = \"unterminated",
            "café // ünïcode",
            &"x".repeat(5000),
        ] {
            let _ = h.highlight(line);
        }
    }
}
