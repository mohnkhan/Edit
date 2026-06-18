//! Markdown syntax highlighter — T075.

#![allow(dead_code, unused_variables, unused_imports)]

use std::sync::OnceLock;

use ratatui::style::{Color, Modifier, Style};
use regex::Regex;

use crate::highlight::{Highlighter, Span};
use crate::ui::theme::CLASSIC;

// ---------------------------------------------------------------------------
// Lazy-compiled regex patterns
// ---------------------------------------------------------------------------

/// ATX headings: `# Title` through `###### Title`.
fn re_heading() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^#{1,6}(\s|$)").unwrap())
}

/// Fenced code block delimiters: lines starting with ` ``` ` or `~~~`.
fn re_fenced_code() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(`{3,}|~{3,})").unwrap())
}

/// Bold text: `**…**` or `__…__`.
fn re_bold() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\*\*[^*]+\*\*|__[^_]+__").unwrap())
}

/// Italic text: `*…*` or `_…_` (single markers).
fn re_italic() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\*[^*\s][^*]*\*|_[^_\s][^_]*_").unwrap())
}

/// Inline code spans: `` `…` ``.
fn re_code_span() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"`[^`]+`").unwrap())
}

/// Link text: `[label](url)`.
fn re_link() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[[^\]]*\]\([^)]*\)").unwrap())
}

/// Block-quote prefix: lines starting with `>`.
fn re_blockquote() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^>+\s?").unwrap())
}

// ---------------------------------------------------------------------------
// MarkdownHighlighter
// ---------------------------------------------------------------------------

/// Syntax highlighter for Markdown files (`.md`).
pub struct MarkdownHighlighter;

impl Highlighter for MarkdownHighlighter {
    fn name(&self) -> &'static str {
        "Markdown"
    }

    fn highlight(&self, line: &str) -> Vec<Span> {
        // keyword color  → headings, fenced-code delimiters, block-quotes
        let heading_style = Style::default().fg(CLASSIC.highlight_keyword);
        // string color   → bold text, links
        let bold_style = Style::default().fg(CLASSIC.highlight_string);
        // number color   → italic text (dim)
        let italic_style = Style::default().fg(CLASSIC.highlight_number);
        // comment color  → inline code spans
        let code_span_style = Style::default().fg(CLASSIC.highlight_comment);

        let mut candidates: Vec<(usize, usize, Style)> = Vec::new();

        // Fenced code block delimiters (whole line).
        for m in re_fenced_code().find_iter(line) {
            candidates.push((m.start(), line.len(), heading_style));
        }

        // Heading prefix (whole line gets heading color).
        for m in re_heading().find_iter(line) {
            candidates.push((0, line.len(), heading_style));
        }

        // Block-quote prefix.
        for m in re_blockquote().find_iter(line) {
            candidates.push((m.start(), m.end(), heading_style));
        }

        // Inline code — higher priority than bold/italic.
        for m in re_code_span().find_iter(line) {
            candidates.push((m.start(), m.end(), code_span_style));
        }

        // Links.
        for m in re_link().find_iter(line) {
            candidates.push((m.start(), m.end(), bold_style));
        }

        // Bold before italic so `**bold**` is caught first.
        for m in re_bold().find_iter(line) {
            candidates.push((m.start(), m.end(), bold_style));
        }

        // Italic.
        for m in re_italic().find_iter(line) {
            candidates.push((m.start(), m.end(), italic_style));
        }

        // Sort by start; longest span wins ties.
        candidates.sort_by_key(|&(start, end, _)| (start, usize::MAX - end));

        let mut spans: Vec<Span> = Vec::new();
        let mut last_end = 0usize;
        for (start, end, style) in candidates {
            if start >= last_end && start < end {
                let clamped_end = end.min(line.len());
                spans.push(Span {
                    start,
                    end: clamped_end,
                    style,
                });
                last_end = clamped_end;
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
    fn heading_highlighted() {
        let h = MarkdownHighlighter;
        let spans = h.highlight("## Hello World");
        let kw_style = Style::default().fg(CLASSIC.highlight_keyword);
        assert!(spans.iter().any(|s| s.style == kw_style));
    }

    #[test]
    fn code_span_highlighted() {
        let h = MarkdownHighlighter;
        let line = "Use `cargo build` to compile.";
        let spans = h.highlight(line);
        let cmt_style = Style::default().fg(CLASSIC.highlight_comment);
        assert!(spans.iter().any(|s| s.style == cmt_style));
    }

    #[test]
    fn bold_highlighted() {
        let h = MarkdownHighlighter;
        let spans = h.highlight("This is **bold** text.");
        let str_style = Style::default().fg(CLASSIC.highlight_string);
        assert!(spans.iter().any(|s| s.style == str_style));
    }

    #[test]
    fn italic_highlighted() {
        let h = MarkdownHighlighter;
        let spans = h.highlight("This is *italic* text.");
        let num_style = Style::default().fg(CLASSIC.highlight_number);
        assert!(spans.iter().any(|s| s.style == num_style));
    }

    #[test]
    fn spans_non_overlapping() {
        let h = MarkdownHighlighter;
        let spans = h.highlight("## Heading with **bold** and `code`");
        let mut last = 0usize;
        for sp in &spans {
            assert!(sp.start >= last, "overlap at {}: {:?}", sp.start, sp);
            last = sp.end;
        }
    }

    #[test]
    fn fenced_code_highlighted() {
        let h = MarkdownHighlighter;
        let spans = h.highlight("```rust");
        let kw_style = Style::default().fg(CLASSIC.highlight_keyword);
        assert!(spans.iter().any(|s| s.style == kw_style));
    }
}
