//! Syntax highlighting subsystem — T070.
//!
//! Defines the [`Highlighter`] trait, the [`Span`] struct, and the
//! [`detect_highlighter`] factory that picks the right highlighter based on
//! a file-path extension.

#![allow(dead_code, unused_variables, unused_imports)]

pub mod languages;

use std::path::Path;

// ---------------------------------------------------------------------------
// Span
// ---------------------------------------------------------------------------

/// A styled byte range within a single line of text.
///
/// `start` and `end` are **byte** offsets into the line string.  The range
/// `[start, end)` is a half-open interval (same convention as Rust slices).
#[derive(Debug, Clone)]
pub struct Span {
    /// Start byte offset within the line (inclusive).
    pub start: usize,
    /// End byte offset within the line (exclusive).
    pub end: usize,
    /// Ratatui style to apply to this range.
    pub style: ratatui::style::Style,
}

// ---------------------------------------------------------------------------
// Highlighter trait
// ---------------------------------------------------------------------------

/// A language-specific syntax highlighter.
///
/// Implementors are required to be `Send + Sync` so they can be stored in a
/// `Buffer` and shared freely across threads if needed.
pub trait Highlighter: Send + Sync {
    /// Return a list of [`Span`]s describing how to style `line`.
    ///
    /// The spans must be non-overlapping and sorted by `start` offset.
    /// Spans that extend past the end of `line` are clamped by the renderer.
    fn highlight(&self, line: &str) -> Vec<Span>;

    /// A short, human-readable name for this highlighter (e.g. `"C"`).
    fn name(&self) -> &'static str;
}

// ---------------------------------------------------------------------------
// detect_highlighter
// ---------------------------------------------------------------------------

/// Select a [`Highlighter`] based on the file extension of `path`.
///
/// Returns `None` when the extension is unrecognised or absent.
pub fn detect_highlighter(path: &Path) -> Option<Box<dyn Highlighter>> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("c") | Some("h") => Some(Box::new(crate::highlight::languages::c::CHighlighter)),
        Some("py") => Some(Box::new(
            crate::highlight::languages::python::PythonHighlighter,
        )),
        Some("sh") | Some("bash") => Some(Box::new(
            crate::highlight::languages::shell::ShellHighlighter,
        )),
        Some("yaml") | Some("yml") => {
            Some(Box::new(crate::highlight::languages::yaml::YamlHighlighter))
        }
        Some("md") => Some(Box::new(
            crate::highlight::languages::markdown::MarkdownHighlighter,
        )),
        // Feature 026: Rust / JSON / TOML.
        Some("rs") => Some(Box::new(crate::highlight::languages::rust::RustHighlighter)),
        Some("json") => Some(Box::new(crate::highlight::languages::json::JsonHighlighter)),
        Some("toml") => Some(Box::new(crate::highlight::languages::toml::TomlHighlighter)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Feature 026: the new extensions resolve to their highlighters; unknown → None.
    #[test]
    fn detect_resolves_rust_json_toml() {
        let name = |p: &str| detect_highlighter(Path::new(p)).map(|h| h.name());
        assert_eq!(name("a.rs"), Some("Rust"));
        assert_eq!(name("Cargo.toml"), Some("TOML"));
        assert_eq!(name("config.json"), Some("JSON"));
        assert_eq!(name("a.c"), Some("C"));
        assert_eq!(name("a.unknownext"), None);
        assert_eq!(name("noext"), None);
    }

    // Highlighting representative lines of each new language yields sorted,
    // non-overlapping spans with no panic.
    #[test]
    fn new_highlighters_produce_valid_spans() {
        let samples = [
            ("x.rs", "pub fn f<T>(x: u32) -> Result<T> { /* c */ \"s\" }"),
            ("x.json", r#"{"k": [1, true, null], "s": "v"}"#),
            ("x.toml", "[pkg]\nname = \"e\" # c\nver = 1.2"),
        ];
        for (path, text) in samples {
            let h = detect_highlighter(Path::new(path)).unwrap();
            for line in text.lines() {
                let spans = h.highlight(line);
                let mut last = 0usize;
                for s in &spans {
                    assert!(s.start >= last, "overlap in {path}: {line:?}");
                    assert!(s.end <= line.len());
                    last = s.end;
                }
            }
        }
    }
}
