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
        _ => None,
    }
}
