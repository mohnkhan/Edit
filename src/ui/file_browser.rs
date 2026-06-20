//! Feature 012: navigable file-system browser for the Open and Save dialogs.
//!
//! A single [`FileBrowser`] model is the source of truth for both modes; the
//! [`FileBrowserWidget`] renders it and [`FileBrowser::hit_test`] maps mouse
//! clicks to the same geometry that is drawn, so clicks always land on what the
//! user sees (the pattern established by the menu bar in feature 009/011).
//!
//! The model only reads the filesystem (`read_dir`, `canonicalize`); the app
//! performs the actual buffer open/save in response to the returned [`Outcome`].

#![allow(dead_code)]

use std::path::PathBuf;

use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Widget},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::security::sanitize::validate_path;
use crate::ui::theme::Theme;

// ---------------------------------------------------------------------------
// Model types
// ---------------------------------------------------------------------------

/// Whether the browser opens a file or saves the active buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowseMode {
    Open,
    Save,
}

/// Kind of a listed entry. `Parent` is the synthetic `..` row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Parent,
    Dir,
    File,
}

/// A single row in the listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub kind: EntryKind,
}

/// Result of activating an entry / confirming, returned to the app event loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Listing changed (navigated / field populated); keep the browser open.
    Navigated,
    /// Open mode: a validated file was chosen.
    OpenFile(PathBuf),
    /// Save mode: a validated destination was chosen.
    SaveFile(PathBuf),
    /// Nothing actionable (e.g. empty filename); keep open.
    None,
}

/// Result of hit-testing a mouse click against the rendered browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserHit {
    /// Click landed on the listed entry at this index.
    Entry(usize),
    /// Click landed inside the box but not on an entry (header/footer/field).
    Inside,
    /// Click landed outside the box (cancel).
    Outside,
}

/// Transient state of an open file dialog.
pub struct FileBrowser {
    pub mode: BrowseMode,
    /// Always a canonical, absolute directory with no `..` components.
    pub cwd: PathBuf,
    pub entries: Vec<Entry>,
    pub selected: usize,
    pub scroll: usize,
    /// Save mode: the filename being typed. Open mode: optional jump path.
    pub filename: String,
    pub error: Option<String>,
}

impl FileBrowser {
    /// Create a browser rooted at `start_dir` (canonicalized; falls back to the
    /// process CWD, then `/`), in the given mode, and read the initial listing.
    pub fn open(start_dir: PathBuf, mode: BrowseMode) -> Self {
        let cwd = std::fs::canonicalize(&start_dir)
            .or_else(|_| std::env::current_dir())
            .unwrap_or_else(|_| PathBuf::from("/"));
        let mut b = FileBrowser {
            mode,
            cwd,
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
            filename: String::new(),
            error: None,
        };
        b.reload();
        b
    }

    /// True when `cwd` is the filesystem root (no parent).
    pub fn is_root(&self) -> bool {
        self.cwd.parent().is_none()
    }

    /// Re-read `cwd` and rebuild the sorted entry list. On read error, set
    /// `error` and keep the previous entries (never panics).
    pub fn reload(&mut self) {
        let read = match std::fs::read_dir(&self.cwd) {
            Ok(rd) => rd,
            Err(e) => {
                self.error = Some(format!("Cannot read directory: {e}"));
                return;
            }
        };

        let mut dirs: Vec<Entry> = Vec::new();
        let mut files: Vec<Entry> = Vec::new();
        for entry in read.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_dir = match entry.file_type() {
                Ok(ft) => ft.is_dir(),
                Err(_) => entry.path().is_dir(),
            };
            if is_dir {
                dirs.push(Entry {
                    name,
                    kind: EntryKind::Dir,
                });
            } else {
                files.push(Entry {
                    name,
                    kind: EntryKind::File,
                });
            }
        }
        let by_name = |a: &Entry, b: &Entry| a.name.to_lowercase().cmp(&b.name.to_lowercase());
        dirs.sort_by(by_name);
        files.sort_by(by_name);

        let mut entries = Vec::with_capacity(dirs.len() + files.len() + 1);
        if !self.is_root() {
            entries.push(Entry {
                name: "..".to_string(),
                kind: EntryKind::Parent,
            });
        }
        entries.extend(dirs);
        entries.extend(files);

        self.entries = entries;
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
        self.scroll = 0;
        self.error = None;
    }

    // ── Navigation ─────────────────────────────────────────────────────────

    pub fn move_up(&mut self, visible_rows: usize) {
        if self.selected > 0 {
            self.selected -= 1;
        }
        self.ensure_visible(visible_rows);
    }

    pub fn move_down(&mut self, visible_rows: usize) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
        self.ensure_visible(visible_rows);
    }

    /// Keep `selected` within the `[scroll, scroll+visible_rows)` window.
    pub fn ensure_visible(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + visible_rows {
            self.scroll = self.selected + 1 - visible_rows;
        }
    }

    /// Move to the parent directory (no-op at root).
    pub fn enter_parent(&mut self) {
        if let Some(parent) = self.cwd.parent() {
            let parent = parent.to_path_buf();
            self.cwd = parent;
            self.selected = 0;
            self.reload();
        }
    }

    /// Descend into a sub-directory named `name`.
    fn enter_dir(&mut self, name: &str) {
        let target = self.cwd.join(name);
        match std::fs::canonicalize(&target) {
            Ok(canon) => {
                self.cwd = canon;
                self.selected = 0;
                self.reload();
            }
            Err(e) => self.error = Some(format!("Cannot open directory: {e}")),
        }
    }

    // ── Activation ─────────────────────────────────────────────────────────

    /// Act on the currently-selected entry (or the typed path/filename).
    pub fn activate(&mut self) -> Outcome {
        match self.mode {
            BrowseMode::Open => self.activate_open(),
            BrowseMode::Save => self.activate_save(),
        }
    }

    /// Mouse path: select `idx` first, then activate.
    pub fn activate_index(&mut self, idx: usize) -> Outcome {
        if idx < self.entries.len() {
            self.selected = idx;
        }
        self.activate()
    }

    fn activate_open(&mut self) -> Outcome {
        // Typed absolute path takes precedence (FR-006a): jump or open it.
        let field = self.filename.trim().to_string();
        if !field.is_empty() {
            let p = PathBuf::from(&field);
            if p.is_absolute() {
                if p.is_dir() {
                    self.filename.clear();
                    self.enter_dir_abs(p);
                    return Outcome::Navigated;
                } else if p.is_file() {
                    return match validate_path(&p) {
                        Ok(valid) => Outcome::OpenFile(valid),
                        Err(e) => {
                            self.error = Some(format!("{e}"));
                            Outcome::None
                        }
                    };
                } else {
                    self.error = Some("No such path".to_string());
                    return Outcome::None;
                }
            }
        }

        let entry = match self.entries.get(self.selected).cloned() {
            Some(e) => e,
            None => return Outcome::None,
        };
        match entry.kind {
            EntryKind::Parent => {
                self.enter_parent();
                Outcome::Navigated
            }
            EntryKind::Dir => {
                self.enter_dir(&entry.name);
                Outcome::Navigated
            }
            EntryKind::File => {
                let path = self.cwd.join(&entry.name);
                match validate_path(&path) {
                    Ok(valid) => Outcome::OpenFile(valid),
                    Err(e) => {
                        self.error = Some(format!("{e}"));
                        Outcome::None
                    }
                }
            }
        }
    }

    fn activate_save(&mut self) -> Outcome {
        let entry = self.entries.get(self.selected).cloned();
        let dir_highlighted = matches!(
            entry.as_ref().map(|e| e.kind),
            Some(EntryKind::Parent) | Some(EntryKind::Dir)
        );

        // Navigate folders only while no filename has been typed; once a name is
        // present (or a file is highlighted), Enter confirms the save.
        if dir_highlighted && self.filename.trim().is_empty() {
            match entry.map(|e| e.kind) {
                Some(EntryKind::Parent) => self.enter_parent(),
                Some(EntryKind::Dir) => {
                    let name = self.entries[self.selected].name.clone();
                    self.enter_dir(&name);
                }
                _ => {}
            }
            return Outcome::Navigated;
        }

        match self.selected_save_path() {
            Ok(path) => Outcome::SaveFile(path),
            Err(msg) => {
                self.error = Some(msg);
                Outcome::None
            }
        }
    }

    fn enter_dir_abs(&mut self, dir: PathBuf) {
        match std::fs::canonicalize(&dir) {
            Ok(canon) => {
                self.cwd = canon;
                self.selected = 0;
                self.reload();
            }
            Err(e) => self.error = Some(format!("Cannot open directory: {e}")),
        }
    }

    /// Resolve the Save destination from the filename field (or the highlighted
    /// file when the field is empty). Validates the directory and rejects names
    /// that are empty or contain path separators / `..`.
    pub fn selected_save_path(&self) -> Result<PathBuf, String> {
        let name = if !self.filename.trim().is_empty() {
            self.filename.trim().to_string()
        } else {
            match self.entries.get(self.selected) {
                Some(e) if e.kind == EntryKind::File => e.name.clone(),
                _ => return Err("Enter a filename".to_string()),
            }
        };
        if name.is_empty()
            || name == "."
            || name == ".."
            || name.contains('/')
            || name.contains('\\')
        {
            return Err("Invalid filename".to_string());
        }
        let dir = validate_path(&self.cwd).map_err(|e| format!("{e}"))?;
        Ok(dir.join(name))
    }

    // ── Filename / path field editing ────────────────────────────────────────

    pub fn push_char(&mut self, c: char) {
        self.filename.push(c);
    }

    /// Backspace: delete the last char of the field, or go to parent when empty.
    pub fn backspace(&mut self) {
        if self.filename.is_empty() {
            self.enter_parent();
        } else {
            self.filename.pop();
        }
    }

    // ── Mouse hit-testing (shares geometry with the widget) ──────────────────

    /// Number of entry rows visible for a frame of size `area`.
    pub fn visible_rows(&self, area: Rect) -> usize {
        compute_layout(area, self.mode).list_rows as usize
    }

    /// Map a terminal click to an entry, an inside-but-inert region, or outside.
    pub fn hit_test(&self, area: Rect, col: u16, row: u16) -> BrowserHit {
        let l = compute_layout(area, self.mode);
        let b = l.box_rect;
        let inside_box = col >= b.x && col < b.x + b.width && row >= b.y && row < b.y + b.height;
        if !inside_box {
            return BrowserHit::Outside;
        }
        if row >= l.list_top && row < l.list_top + l.list_rows {
            let idx = self.scroll + (row - l.list_top) as usize;
            if idx < self.entries.len() {
                return BrowserHit::Entry(idx);
            }
        }
        BrowserHit::Inside
    }
}

// ---------------------------------------------------------------------------
// Layout (shared by widget + hit-test)
// ---------------------------------------------------------------------------

struct BrowserLayout {
    box_rect: Rect,
    /// First terminal row of the entry list.
    list_top: u16,
    /// Number of entry rows.
    list_rows: u16,
    /// Left column / width of the inner content area.
    inner_left: u16,
    inner_width: u16,
    /// Header (path) row and footer (hints) row.
    header_row: u16,
    footer_row: u16,
    /// Filename input row (Save mode only).
    filename_row: u16,
}

fn compute_layout(area: Rect, mode: BrowseMode) -> BrowserLayout {
    let bw = 64u16.min(area.width.max(1));
    let bh = 20u16.min(area.height.max(1));
    let bx = area.x + area.width.saturating_sub(bw) / 2;
    let by = area.y + area.height.saturating_sub(bh) / 2;
    let box_rect = Rect::new(bx, by, bw, bh);

    let inner_left = bx + 1;
    let inner_width = bw.saturating_sub(2);
    let inner_top = by + 1;
    let inner_h = bh.saturating_sub(2);

    let header_row = inner_top;
    let footer_row = inner_top + inner_h.saturating_sub(1);
    // Save reserves one extra row (above the footer) for the filename field.
    let (filename_row, reserved_below) = match mode {
        BrowseMode::Save => (footer_row.saturating_sub(1), 2u16),
        BrowseMode::Open => (footer_row, 1u16),
    };
    let list_top = header_row + 1;
    // Rows between header and the reserved-below region.
    let list_rows = inner_h.saturating_sub(1 + reserved_below);

    BrowserLayout {
        box_rect,
        list_top,
        list_rows,
        inner_left,
        inner_width,
        header_row,
        footer_row,
        filename_row,
    }
}

// ---------------------------------------------------------------------------
// UTF-8 / wide-character-safe truncation
// ---------------------------------------------------------------------------

/// Approximate display width of a grapheme's leading scalar (1 narrow, 2 wide).
fn grapheme_width(g: &str) -> u16 {
    let cp = g.chars().next().map(|c| c as u32).unwrap_or(0);
    let wide = (0x1100..=0x115F).contains(&cp)
        || (0x2E80..=0x303E).contains(&cp)
        || (0x3041..=0x33BF).contains(&cp)
        || (0x4E00..=0x9FFF).contains(&cp)
        || (0xAC00..=0xD7AF).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0xFF01..=0xFF60).contains(&cp)
        || (0x1F300..=0x1F9FF).contains(&cp)
        || (0x20000..=0x2A6DF).contains(&cp);
    if wide {
        2
    } else {
        1
    }
}

/// Truncate `s` to at most `max_cols` display columns, appending `…` when cut.
/// Never splits a grapheme cluster (so never corrupts a multi-byte character).
pub fn truncate_to_width(s: &str, max_cols: u16) -> String {
    let total: u16 = s.graphemes(true).map(grapheme_width).sum();
    if total <= max_cols {
        return s.to_string();
    }
    if max_cols == 0 {
        return String::new();
    }
    let budget = max_cols.saturating_sub(1); // room for the ellipsis
    let mut out = String::new();
    let mut used = 0u16;
    for g in s.graphemes(true) {
        let w = grapheme_width(g);
        if used + w > budget {
            break;
        }
        out.push_str(g);
        used += w;
    }
    out.push('…');
    out
}

// ---------------------------------------------------------------------------
// Widget
// ---------------------------------------------------------------------------

/// Renders an open [`FileBrowser`] as a centered bordered modal.
pub struct FileBrowserWidget<'a> {
    pub browser: &'a FileBrowser,
    pub theme: &'static Theme,
}

impl<'a> Widget for FileBrowserWidget<'a> {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        let b = self.browser;
        let l = compute_layout(area, b.mode);
        let box_rect = l.box_rect;
        if box_rect.width < 4 || box_rect.height < 4 {
            return;
        }

        let base = Style::default()
            .fg(self.theme.menubar_fg)
            .bg(self.theme.menubar_bg);
        let selected = Style::default()
            .fg(self.theme.menubar_bg)
            .bg(self.theme.menu_selected_bg);

        Clear.render(box_rect, buf);
        let title = match b.mode {
            BrowseMode::Open => " Open File ",
            BrowseMode::Save => " Save As ",
        };
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(base)
            .render(box_rect, buf);

        let iw = l.inner_width;
        let put = |buf: &mut TuiBuffer, x: u16, y: u16, text: &str, style: Style| {
            let mut cx = x;
            let max_x = l.inner_left + iw;
            for g in text.graphemes(true) {
                if cx >= max_x {
                    break;
                }
                buf.get_mut(cx, y).set_symbol(g).set_style(style);
                cx += grapheme_width(g);
            }
        };

        // Header: current directory (or error notice).
        let header = match &b.error {
            Some(e) => truncate_to_width(e, iw),
            None => truncate_to_width(&b.cwd.to_string_lossy(), iw),
        };
        let header_style = if b.error.is_some() {
            base.add_modifier(Modifier::BOLD)
        } else {
            base
        };
        put(buf, l.inner_left, l.header_row, &header, header_style);

        // Entry list.
        let name_budget = iw.saturating_sub(2); // 1 for marker col + space
        for vis in 0..l.list_rows {
            let idx = b.scroll + vis as usize;
            if idx >= b.entries.len() {
                break;
            }
            let entry = &b.entries[idx];
            let y = l.list_top + vis;
            let style = if idx == b.selected { selected } else { base };
            // Fill the row so the selection highlight spans the width.
            for cx in l.inner_left..l.inner_left + iw {
                buf.get_mut(cx, y).set_symbol(" ").set_style(style);
            }
            let (marker, display): (&str, String) = match entry.kind {
                EntryKind::Parent => ("/", "..".to_string()),
                EntryKind::Dir => ("/", format!("{}/", entry.name)),
                EntryKind::File => (" ", entry.name.clone()),
            };
            put(buf, l.inner_left, y, marker, style);
            put(
                buf,
                l.inner_left + 1,
                y,
                &truncate_to_width(&display, name_budget),
                style,
            );
        }

        // Filename field (Save mode).
        if b.mode == BrowseMode::Save {
            let label = format!("Name: {}", b.filename);
            put(
                buf,
                l.inner_left,
                l.filename_row,
                &truncate_to_width(&label, iw),
                base.add_modifier(Modifier::BOLD),
            );
        }

        // Footer hints.
        let hints = match b.mode {
            BrowseMode::Open => "↑↓ move  Enter open  ← parent  Esc cancel",
            BrowseMode::Save => "↑↓ move  type name  Enter save  ← parent  Esc cancel",
        };
        put(
            buf,
            l.inner_left,
            l.footer_row,
            &truncate_to_width(hints, iw),
            base,
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_tree(tag: &str) -> PathBuf {
        // Per-test directory (tests run in parallel) — no Date/random needed.
        let base = std::env::temp_dir().join(format!("edit_fb_test_{tag}"));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("zsub")).unwrap();
        fs::create_dir_all(base.join("Asub")).unwrap();
        fs::write(base.join("bravo.txt"), b"bravo\n").unwrap();
        fs::write(base.join("alpha.txt"), b"alpha\n").unwrap();
        fs::write(base.join(".hidden"), b"h\n").unwrap();
        base
    }

    #[test]
    fn lists_sorted_dirs_then_files_with_dotfiles_and_parent() {
        let base = temp_tree("list");
        let b = FileBrowser::open(base.clone(), BrowseMode::Open);
        let names: Vec<&str> = b.entries.iter().map(|e| e.name.as_str()).collect();
        // ".." first, then dirs (case-insensitive alpha), then files incl dotfile.
        assert_eq!(names[0], "..");
        let asub = names.iter().position(|n| *n == "Asub").unwrap();
        let zsub = names.iter().position(|n| *n == "zsub").unwrap();
        let alpha = names.iter().position(|n| *n == "alpha.txt").unwrap();
        assert!(asub < zsub, "dirs sorted case-insensitively");
        assert!(zsub < alpha, "dirs come before files");
        assert!(names.contains(&".hidden"), "dotfiles shown");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn move_clamps_and_scrolls() {
        let base = temp_tree("move");
        let mut b = FileBrowser::open(base.clone(), BrowseMode::Open);
        b.move_up(3); // already at 0 → stays
        assert_eq!(b.selected, 0);
        for _ in 0..50 {
            b.move_down(3);
        }
        assert_eq!(b.selected, b.entries.len() - 1);
        assert!(b.selected < b.scroll + 3 + 1, "selection stays visible");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn enter_dir_and_parent() {
        let base = temp_tree("enter");
        let mut b = FileBrowser::open(base.clone(), BrowseMode::Open);
        let zsub = b.entries.iter().position(|e| e.name == "zsub").unwrap();
        b.selected = zsub;
        assert_eq!(b.activate(), Outcome::Navigated);
        assert!(b.cwd.ends_with("zsub"));
        // ".." back up.
        b.selected = 0;
        assert_eq!(b.entries[0].kind, EntryKind::Parent);
        b.activate();
        assert!(b.cwd.ends_with("edit_fb_test_enter"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn open_file_returns_validated_path() {
        let base = temp_tree("open");
        let mut b = FileBrowser::open(base.clone(), BrowseMode::Open);
        let idx = b
            .entries
            .iter()
            .position(|e| e.name == "alpha.txt")
            .unwrap();
        b.selected = idx;
        match b.activate() {
            Outcome::OpenFile(p) => assert!(p.ends_with("alpha.txt")),
            other => panic!("expected OpenFile, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn save_path_validation() {
        let base = temp_tree("save");
        let mut b = FileBrowser::open(base.clone(), BrowseMode::Save);
        // Empty filename, no file highlighted (selected is ".." or a dir) → err.
        b.filename.clear();
        b.selected = 0;
        assert!(b.selected_save_path().is_err());
        // Bad names.
        b.filename = "a/b.txt".to_string();
        assert!(b.selected_save_path().is_err());
        b.filename = "..".to_string();
        assert!(b.selected_save_path().is_err());
        // Good name → cwd.join(name).
        b.filename = "new.txt".to_string();
        let p = b.selected_save_path().expect("valid");
        assert!(p.ends_with("new.txt"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn reload_error_keeps_state_and_sets_error() {
        // FR-013 / SC-005: a non-existent directory must not panic.
        let mut b = FileBrowser::open(std::env::temp_dir(), BrowseMode::Open);
        let before = b.entries.clone();
        b.cwd = PathBuf::from("/this/does/not/exist/at/all");
        b.reload();
        assert!(b.error.is_some());
        assert_eq!(b.entries, before, "entries preserved on error");
    }

    #[test]
    fn truncate_never_splits_multibyte() {
        let s = "日本語ファイル.txt"; // wide chars
        let t = truncate_to_width(s, 6);
        assert!(t.ends_with('…'));
        // Result must be valid UTF-8 graphemes (no panic on chars()).
        assert!(t.chars().count() >= 1);
        // Width within budget.
        let w: u16 = t.graphemes(true).map(grapheme_width).sum();
        assert!(w <= 6, "width {w} exceeds budget");
    }

    #[test]
    fn hit_test_maps_rows_and_outside() {
        let base = temp_tree("hit");
        let b = FileBrowser::open(base.clone(), BrowseMode::Open);
        let area = Rect::new(0, 0, 80, 24);
        let l = compute_layout(area, BrowseMode::Open);
        // First list row → entry index scroll+0 = 0.
        assert_eq!(
            b.hit_test(area, l.inner_left, l.list_top),
            BrowserHit::Entry(0)
        );
        // Far outside the box → Outside.
        assert_eq!(b.hit_test(area, 0, 0), BrowserHit::Outside);
        let _ = fs::remove_dir_all(&base);
    }
}
