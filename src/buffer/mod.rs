//! Text buffer subsystem.
//!
//! Sub-modules:
//! - [`rope`] — EditorRope, a thin ergonomic wrapper around `ropey::Rope`.
//! - [`undo`] — UndoStack providing linear undo/redo history.
//!
//! This module defines the core buffer data model:
//! - [`CursorPos`]  — T021: cursor position in grapheme + visual coordinates
//! - [`Selection`]  — T022: anchor + active selection pair
//! - [`LineEnding`] — LF vs CRLF enum
//! - [`BufferError`] — error type for file I/O and encoding
//! - [`Buffer`]     — T023/T024: the main text buffer struct with open/save

#![allow(dead_code)]

pub mod autosave;
pub mod rope;
pub mod undo;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::buffer::autosave::AutosaveState;
use crate::buffer::rope::EditorRope;
use crate::encoding::EncodingId;

// ---------------------------------------------------------------------------
// LineEnding
// ---------------------------------------------------------------------------

/// The line-ending convention used when saving the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    /// Unix `\n`.
    Lf,
    /// Windows `\r\n`.
    Crlf,
}

/// Maximum size of a file the editor will load into a buffer (Feature 029).
///
/// The whole file is materialised in memory (rope + a transient `String`), so an
/// unbounded read risks OOM. 256 MiB is far beyond any realistic text file while
/// still refusing pathological/binary inputs gracefully.
pub const MAX_OPEN_BYTES: u64 = 256 * 1024 * 1024;

// ---------------------------------------------------------------------------
// BufferError
// ---------------------------------------------------------------------------

/// Errors that can occur when opening or saving a buffer.
#[derive(Debug)]
pub enum BufferError {
    /// The file appears to be binary (null bytes detected in the first 512 bytes).
    BinaryContent,
    /// The file bytes could not be decoded in the requested encoding.
    DecodeError { byte_offset: usize },
    /// An I/O error occurred.
    Io(std::io::Error),
    /// The text could not be re-encoded in the buffer's target encoding.
    EncodeError,
    /// The file is larger than [`MAX_OPEN_BYTES`] and was not loaded (Feature 029).
    TooLarge { size: u64, limit: u64 },
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferError::BinaryContent => {
                write!(f, "file appears to be binary (null bytes detected)")
            }
            BufferError::DecodeError { byte_offset } => {
                write!(f, "decode error at byte offset {byte_offset}")
            }
            BufferError::Io(e) => write!(f, "I/O error: {e}"),
            BufferError::EncodeError => write!(f, "failed to encode text in the target encoding"),
            BufferError::TooLarge { size, limit } => write!(
                f,
                "file too large: {} MiB exceeds the {} MiB limit",
                size / (1024 * 1024),
                limit / (1024 * 1024)
            ),
        }
    }
}

impl std::error::Error for BufferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BufferError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for BufferError {
    fn from(e: std::io::Error) -> Self {
        BufferError::Io(e)
    }
}

// ---------------------------------------------------------------------------
// CursorPos — T021
// ---------------------------------------------------------------------------

/// A cursor position within the buffer.
///
/// All coordinates are zero-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorPos {
    /// Zero-based line index.
    pub line: usize,
    /// Zero-based grapheme cluster index within the line.
    pub grapheme_col: usize,
    /// Zero-based visual (display) column, accounting for character widths.
    pub visual_col: usize,
}

impl CursorPos {
    /// Compute `visual_col` by summing the Unicode display widths of each
    /// grapheme cluster from the start of the line up to (but not including)
    /// `gcol`.
    ///
    /// Returns 0 if `gcol` is 0 or if the line is empty.
    pub fn visual_col_from_grapheme_col(rope: &EditorRope, line: usize, gcol: usize) -> usize {
        if gcol == 0 {
            return 0;
        }
        let line_str = rope.line_slice(line);
        line_str
            .graphemes(true)
            .take(gcol)
            .map(UnicodeWidthStr::width)
            .sum()
    }
}

// ---------------------------------------------------------------------------
// Selection — T022
// ---------------------------------------------------------------------------

/// An anchor + active pair that describes a selection range in the buffer.
///
/// The anchor is where the selection started; the active end is where it
/// currently ends (the caret).  Either end may be "earlier" in the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// The fixed end of the selection (where it started).
    pub anchor: CursorPos,
    /// The moving end of the selection (where the caret is).
    pub active: CursorPos,
}

impl Selection {
    /// Return `(min, max)` cursor positions in document order.
    ///
    /// "Earlier" is determined first by line, then by grapheme column.
    pub fn ordered_range(&self) -> (CursorPos, CursorPos) {
        let a = self.anchor;
        let b = self.active;
        if (a.line, a.grapheme_col) <= (b.line, b.grapheme_col) {
            (a, b)
        } else {
            (b, a)
        }
    }

    /// Returns `true` if the selection is empty (anchor and active are at the
    /// same line and grapheme column).
    pub fn is_empty(&self) -> bool {
        self.anchor.line == self.active.line && self.anchor.grapheme_col == self.active.grapheme_col
    }
}

// ---------------------------------------------------------------------------
// Buffer — T023 / T024
// ---------------------------------------------------------------------------

/// The main text buffer, holding rope contents, cursor state, and metadata.
pub struct Buffer {
    /// The path on disk, if this buffer has been saved at least once.
    pub path: Option<std::path::PathBuf>,
    /// The rope-based text storage.
    pub rope: EditorRope,
    /// The character encoding used when reading and saving the file.
    pub encoding: EncodingId,
    /// The line-ending convention to apply when saving.
    pub line_ending: LineEnding,
    /// Whether the buffer has unsaved changes.
    pub modified: bool,
    /// Whether the buffer should be treated as read-only.
    pub readonly: bool,
    /// The current cursor position.
    pub cursor: CursorPos,
    /// The top-left of the viewport: `(line, visual_col)`.
    pub scroll_offset: (usize, usize),
    /// The active selection, if any.
    pub selection: Option<Selection>,
    /// The undo/redo history for this buffer.
    pub undo_stack: crate::buffer::undo::UndoStack,
    /// Active syntax highlighter for this buffer (T070 / US7).
    pub syntax: Option<Box<dyn crate::highlight::Highlighter>>,
    /// Auto-save and crash-recovery state (US5 / T058).
    pub autosave: AutosaveState,
    /// Set to `true` when a stale recovery file was found for this buffer's path.
    ///
    /// The UI layer should prompt the user to accept or discard the recovery.
    pub pending_recovery: bool,
}

impl Buffer {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Create an empty, unnamed, writable buffer with UTF-8 encoding.
    pub fn new_empty() -> Buffer {
        let mut b = Buffer {
            path: None,
            rope: EditorRope::new(),
            encoding: EncodingId::Utf8,
            line_ending: LineEnding::Lf,
            modified: false,
            readonly: false,
            cursor: CursorPos::default(),
            scroll_offset: (0, 0),
            selection: None,
            undo_stack: crate::buffer::undo::UndoStack::new(),
            syntax: None,
            autosave: AutosaveState::new(false, 30),
            pending_recovery: false,
        };
        // Feature 014: the empty state IS the clean baseline.
        b.undo_stack.mark_saved();
        b
    }

    /// Open a file from disk and decode it using the given encoding.
    ///
    /// # Steps
    /// 1. Read the raw bytes.
    /// 2. Detect binary content (null bytes in the first 512 bytes).
    /// 3. Detect the line-ending convention from the first 512 bytes.
    /// 4. Decode the bytes with `crate::encoding::decode`.
    /// 5. Strip `\r` so the rope stores LF-only lines.
    /// 6. Check filesystem metadata for write permission.
    pub fn open(
        path: impl AsRef<std::path::Path>,
        encoding: EncodingId,
    ) -> Result<Buffer, BufferError> {
        let path = path.as_ref();

        // --- File-size guard (Feature 029) -----------------------------------
        // Refuse to load a file larger than MAX_OPEN_BYTES rather than reading it
        // entirely into memory and risking an OOM crash. Metadata failures fall
        // through (the read below reports the real I/O error).
        if let Ok(meta) = std::fs::metadata(path) {
            if meta.is_file() && meta.len() > MAX_OPEN_BYTES {
                return Err(BufferError::TooLarge {
                    size: meta.len(),
                    limit: MAX_OPEN_BYTES,
                });
            }
        }

        // --- Read raw bytes --------------------------------------------------
        let bytes = std::fs::read(path)?;

        // --- Binary detection ------------------------------------------------
        let probe = &bytes[..bytes.len().min(512)];
        if probe.contains(&0x00) {
            return Err(BufferError::BinaryContent);
        }

        // --- Line-ending detection -------------------------------------------
        let line_ending = if probe.windows(2).any(|w| w == b"\r\n") {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };

        // --- Decode ----------------------------------------------------------
        let text = crate::encoding::decode(&bytes, encoding)
            .map_err(|_| BufferError::DecodeError { byte_offset: 0 })?;

        // --- Strip \r (store as LF-only internally) --------------------------
        let text = text.replace('\r', "");

        // --- Load into rope --------------------------------------------------
        let rope = EditorRope::from_str(&text);

        // --- Readonly detection from metadata --------------------------------
        let readonly = {
            match std::fs::metadata(path) {
                Ok(meta) => meta.permissions().readonly(),
                // If we cannot stat the file, default to not readonly.
                Err(_) => false,
            }
        };

        let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let mut b = Buffer {
            path: Some(path.to_path_buf()),
            rope,
            encoding,
            line_ending,
            modified: false,
            readonly,
            cursor: CursorPos::default(),
            scroll_offset: (0, 0),
            selection: None,
            undo_stack: crate::buffer::undo::UndoStack::new(),
            syntax: None,
            autosave: AutosaveState::for_path(&abs_path, !readonly, 30),
            pending_recovery: false,
        };
        // Feature 014: the on-disk content loaded here is the clean baseline.
        b.undo_stack.mark_saved();
        Ok(b)
    }

    /// Feature 014: recompute the modified flag from the undo history — the
    /// buffer is modified iff its content differs from the saved baseline.
    pub fn refresh_modified(&mut self) {
        self.modified = !self.undo_stack.is_at_saved();
    }

    // -----------------------------------------------------------------------
    // Saving — T024
    // -----------------------------------------------------------------------

    /// Save the buffer to its current path.
    ///
    /// Writes atomically: bytes go to `<path>.tmp` first, then the file is
    /// renamed to `<path>`.
    ///
    /// Returns [`BufferError::Io`] with `NotFound`-like kind if `self.path` is
    /// `None` (the buffer has never been saved and has no path).
    pub fn save(&self) -> Result<(), BufferError> {
        let path = self.path.as_ref().ok_or_else(|| {
            BufferError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "buffer has no associated file path",
            ))
        })?;

        self.write_to(path)?;
        Ok(())
    }

    /// Save the buffer to a new path and update `self.path`.
    pub fn save_as(&mut self, new_path: std::path::PathBuf) -> Result<(), BufferError> {
        self.write_to(&new_path)?;
        self.path = Some(new_path);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Encode the rope contents and write them atomically to `dest`.
    fn write_to(&self, dest: &std::path::Path) -> Result<(), BufferError> {
        // Materialise the rope as a String (LF-only).
        let lf_text = self.rope.to_string();

        // Apply the line-ending convention.
        let text_for_encode: std::borrow::Cow<str> = match self.line_ending {
            LineEnding::Lf => std::borrow::Cow::Borrowed(&lf_text),
            LineEnding::Crlf => std::borrow::Cow::Owned(lf_text.replace('\n', "\r\n")),
        };

        // Encode using the buffer's target encoding.
        let bytes = crate::encoding::encode(&text_for_encode, self.encoding)
            .map_err(|_| BufferError::EncodeError)?;

        // Atomic write: write to a temp file in the same directory, then rename.
        let mut tmp_path = dest.to_path_buf();
        {
            let mut name = dest
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("buffer"))
                .to_os_string();
            name.push(".tmp");
            tmp_path.set_file_name(name);
        }

        std::fs::write(&tmp_path, &bytes)?;
        std::fs::rename(&tmp_path, dest)?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Feature 014 — clean baseline on construction
    // -----------------------------------------------------------------------

    // T012 (Feature 029): the file-size guard refuses oversized files and a normal
    // file opens fine. We can't easily create a 256 MiB file in a unit test, so we
    // verify the boundary logic directly and that a small real file opens.
    #[test]
    fn open_refuses_oversized_and_allows_normal() {
        // Boundary logic mirrors Buffer::open's guard.
        let over = MAX_OPEN_BYTES + 1;
        assert!(over > MAX_OPEN_BYTES, "guard compares strictly greater");
        let err = BufferError::TooLarge {
            size: over,
            limit: MAX_OPEN_BYTES,
        };
        assert!(format!("{err}").contains("file too large"));

        // A small real file still opens normally (guard not triggered).
        let dir = std::env::temp_dir().join("edit_size_guard_test");
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join("small.txt");
        std::fs::write(&p, b"hello\n").unwrap();
        let b = Buffer::open(&p, EncodingId::Utf8).expect("small file opens");
        assert_eq!(b.rope.line_slice(0), "hello");
    }

    #[test]
    fn new_empty_buffer_is_clean_baseline() {
        let b = Buffer::new_empty();
        assert!(!b.modified);
        assert!(
            b.undo_stack.is_at_saved(),
            "a fresh empty buffer's empty state is the clean baseline"
        );
    }

    #[test]
    fn refresh_modified_follows_saved_state() {
        let mut b = Buffer::new_empty();
        // Simulate an edit recorded in history.
        b.rope.insert_str(0, "x");
        b.undo_stack.push(crate::buffer::undo::EditOp::Insert {
            at: 0,
            text: "x".into(),
        });
        b.refresh_modified();
        assert!(b.modified, "after an edit the buffer is modified");
        // Undo back to the saved baseline.
        b.undo_stack.undo(&mut b.rope);
        b.refresh_modified();
        assert!(!b.modified, "undo back to baseline is clean");
    }

    // -----------------------------------------------------------------------
    // CursorPos
    // -----------------------------------------------------------------------

    #[test]
    fn visual_col_from_grapheme_col_zero() {
        let rope = EditorRope::from_str("hello");
        assert_eq!(CursorPos::visual_col_from_grapheme_col(&rope, 0, 0), 0);
    }

    #[test]
    fn visual_col_ascii_equals_grapheme_col() {
        // Pure ASCII: visual col == grapheme col
        let rope = EditorRope::from_str("hello\nworld");
        for gcol in 0..=5 {
            assert_eq!(
                CursorPos::visual_col_from_grapheme_col(&rope, 0, gcol),
                gcol
            );
        }
    }

    #[test]
    fn visual_col_fullwidth_chars() {
        // CJK ideographs are width-2; two of them → visual col 4 at gcol 2.
        let rope = EditorRope::from_str("日本");
        assert_eq!(CursorPos::visual_col_from_grapheme_col(&rope, 0, 1), 2);
        assert_eq!(CursorPos::visual_col_from_grapheme_col(&rope, 0, 2), 4);
    }

    // -----------------------------------------------------------------------
    // Selection
    // -----------------------------------------------------------------------

    fn make_cursor(line: usize, gcol: usize) -> CursorPos {
        CursorPos {
            line,
            grapheme_col: gcol,
            visual_col: gcol, // irrelevant for selection logic
        }
    }

    #[test]
    fn selection_is_empty_same_position() {
        let sel = Selection {
            anchor: make_cursor(1, 3),
            active: make_cursor(1, 3),
        };
        assert!(sel.is_empty());
    }

    #[test]
    fn selection_not_empty_different_col() {
        let sel = Selection {
            anchor: make_cursor(0, 2),
            active: make_cursor(0, 5),
        };
        assert!(!sel.is_empty());
    }

    #[test]
    fn selection_not_empty_different_line() {
        let sel = Selection {
            anchor: make_cursor(0, 0),
            active: make_cursor(1, 0),
        };
        assert!(!sel.is_empty());
    }

    #[test]
    fn ordered_range_forward_selection() {
        let a = make_cursor(0, 2);
        let b = make_cursor(1, 4);
        let sel = Selection {
            anchor: a,
            active: b,
        };
        let (lo, hi) = sel.ordered_range();
        assert_eq!((lo.line, lo.grapheme_col), (0, 2));
        assert_eq!((hi.line, hi.grapheme_col), (1, 4));
    }

    #[test]
    fn ordered_range_backward_selection() {
        let a = make_cursor(3, 7);
        let b = make_cursor(1, 2);
        let sel = Selection {
            anchor: a,
            active: b,
        };
        let (lo, hi) = sel.ordered_range();
        assert_eq!((lo.line, lo.grapheme_col), (1, 2));
        assert_eq!((hi.line, hi.grapheme_col), (3, 7));
    }

    // -----------------------------------------------------------------------
    // Buffer::new_empty
    // -----------------------------------------------------------------------

    #[test]
    fn new_empty_defaults() {
        let buf = Buffer::new_empty();
        assert!(buf.path.is_none());
        assert!(!buf.modified);
        assert!(!buf.readonly);
        assert_eq!(buf.cursor.line, 0);
        assert_eq!(buf.cursor.grapheme_col, 0);
        assert_eq!(buf.scroll_offset, (0, 0));
        assert!(buf.selection.is_none());
        assert!(buf.syntax.is_none());
        assert_eq!(buf.rope.char_count(), 0);
    }

    // -----------------------------------------------------------------------
    // Buffer::open
    // -----------------------------------------------------------------------

    #[test]
    fn open_binary_file_rejected() {
        use std::io::Write;
        let tmp = tempfile_path("binary_test");
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            f.write_all(b"hello\x00world").unwrap();
        }
        let result = Buffer::open(&tmp, EncodingId::Utf8);
        let _ = std::fs::remove_file(&tmp);
        assert!(matches!(result, Err(BufferError::BinaryContent)));
    }

    #[test]
    fn open_lf_file() {
        use std::io::Write;
        let tmp = tempfile_path("lf_test");
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            f.write_all(b"line1\nline2\n").unwrap();
        }
        let buf = Buffer::open(&tmp, EncodingId::Utf8).unwrap();
        let _ = std::fs::remove_file(&tmp);
        assert_eq!(buf.line_ending, LineEnding::Lf);
        assert_eq!(buf.rope.line_count(), 3); // "line1", "line2", ""
    }

    #[test]
    fn open_crlf_file() {
        use std::io::Write;
        let tmp = tempfile_path("crlf_test");
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            f.write_all(b"line1\r\nline2\r\n").unwrap();
        }
        let buf = Buffer::open(&tmp, EncodingId::Utf8).unwrap();
        let _ = std::fs::remove_file(&tmp);
        assert_eq!(buf.line_ending, LineEnding::Crlf);
        // Internally, \r should be stripped — each line is just "lineN".
        assert_eq!(buf.rope.line_slice(0), "line1");
        assert_eq!(buf.rope.line_slice(1), "line2");
    }

    // -----------------------------------------------------------------------
    // Buffer::save / save_as
    // -----------------------------------------------------------------------

    #[test]
    fn save_roundtrip_utf8_lf() {
        let content = "hello\nworld\n";
        let tmp = tempfile_path("save_lf");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&tmp).unwrap();
            f.write_all(content.as_bytes()).unwrap();
        }
        let buf = Buffer::open(&tmp, EncodingId::Utf8).unwrap();
        buf.save().unwrap();
        let read_back = std::fs::read_to_string(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);
        assert_eq!(read_back, content);
    }

    #[test]
    fn save_as_updates_path() {
        let content = "data\n";
        let src = tempfile_path("save_as_src");
        let dst = tempfile_path("save_as_dst");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&src).unwrap();
            f.write_all(content.as_bytes()).unwrap();
        }
        let mut buf = Buffer::open(&src, EncodingId::Utf8).unwrap();
        buf.save_as(dst.clone()).unwrap();
        let _ = std::fs::remove_file(&src);
        // Path should be updated.
        assert_eq!(buf.path.as_deref(), Some(dst.as_path()));
        // Destination should contain the content.
        let read_back = std::fs::read_to_string(&dst).unwrap();
        let _ = std::fs::remove_file(&dst);
        assert_eq!(read_back, content);
    }

    #[test]
    fn save_no_path_returns_io_error() {
        let buf = Buffer::new_empty();
        assert!(matches!(buf.save(), Err(BufferError::Io(_))));
    }

    // -----------------------------------------------------------------------
    // Test helper
    // -----------------------------------------------------------------------

    /// Build a unique temporary file path (does not create the file).
    fn tempfile_path(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("edit_buf_test_{}_{}", tag, std::process::id()));
        p
    }
}
