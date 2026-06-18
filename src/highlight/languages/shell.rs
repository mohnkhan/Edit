//! Shell / Bash syntax highlighter — T073.

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
            r"\b(case|do|done|elif|else|esac|fi|for|function|if|in|return|select|then|time|until|while)\b"
        ).unwrap()
    })
}

fn re_double_string() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#""[^"\\]*(?:\\.[^"\\]*)*""#).unwrap())
}

fn re_single_string() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"'[^']*'").unwrap())
}

fn re_comment() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"#.*").unwrap())
}

fn re_variable() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\$\{?[A-Za-z_][A-Za-z0-9_]*\}?|\$[0-9#@*?$!-]").unwrap())
}

fn re_builtin() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b(echo|export|source|cd|pwd|ls|cat|grep|sed|awk|read|printf|test|true|false|exit|shift|set|unset|local|declare|eval|exec|trap)\b").unwrap()
    })
}

// ---------------------------------------------------------------------------
// ShellHighlighter
// ---------------------------------------------------------------------------

/// Syntax highlighter for shell scripts (`.sh`, `.bash`).
pub struct ShellHighlighter;

impl Highlighter for ShellHighlighter {
    fn name(&self) -> &'static str {
        "Shell"
    }

    fn highlight(&self, line: &str) -> Vec<Span> {
        let keyword_style = Style::default().fg(CLASSIC.highlight_keyword);
        let string_style  = Style::default().fg(CLASSIC.highlight_string);
        let comment_style = Style::default().fg(CLASSIC.highlight_comment);
        // Reuse number color for variables (as specified in the task).
        let variable_style = Style::default().fg(CLASSIC.highlight_number);

        let mut candidates: Vec<(usize, usize, Style)> = Vec::new();

        for m in re_comment().find_iter(line) {
            candidates.push((m.start(), m.end(), comment_style));
        }
        for m in re_double_string().find_iter(line) {
            candidates.push((m.start(), m.end(), string_style));
        }
        for m in re_single_string().find_iter(line) {
            candidates.push((m.start(), m.end(), string_style));
        }
        for m in re_variable().find_iter(line) {
            candidates.push((m.start(), m.end(), variable_style));
        }
        for m in re_builtin().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword_style));
        }
        for m in re_keyword().find_iter(line) {
            candidates.push((m.start(), m.end(), keyword_style));
        }

        // Sort by start, resolve overlaps greedily (first sorted wins).
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
    fn if_keyword_highlighted() {
        let h = ShellHighlighter;
        let line = "if [ -f file ]; then";
        let spans = h.highlight(line);
        let kw_style = Style::default().fg(CLASSIC.highlight_keyword);
        assert!(spans.iter().any(|s| &line[s.start..s.end] == "if" && s.style == kw_style));
    }

    #[test]
    fn variable_highlighted() {
        let h = ShellHighlighter;
        let line = "echo $HOME";
        let spans = h.highlight(line);
        let var_style = Style::default().fg(CLASSIC.highlight_number);
        assert!(spans.iter().any(|s| &line[s.start..s.end] == "$HOME" && s.style == var_style));
    }

    #[test]
    fn comment_highlighted() {
        let h = ShellHighlighter;
        let spans = h.highlight("# this is a comment");
        let cmt_style = Style::default().fg(CLASSIC.highlight_comment);
        assert!(spans.iter().any(|s| s.style == cmt_style));
    }

    #[test]
    fn string_highlighted() {
        let h = ShellHighlighter;
        let spans = h.highlight(r#"echo "hello world""#);
        let str_style = Style::default().fg(CLASSIC.highlight_string);
        assert!(spans.iter().any(|s| s.style == str_style));
    }

    #[test]
    fn spans_non_overlapping() {
        let h = ShellHighlighter;
        let spans = h.highlight(r#"if [ "$HOME" = "/root" ]; then # admin"#);
        let mut last = 0usize;
        for sp in &spans {
            assert!(sp.start >= last);
            last = sp.end;
        }
    }
}
