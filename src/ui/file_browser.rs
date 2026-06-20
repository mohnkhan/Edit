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
    /// File byte size; `None` for directories / `..` / unreadable metadata (Feature 022).
    pub size: Option<u64>,
    /// Modified time as Unix epoch seconds; `None` if unreadable (Feature 022).
    pub mtime: Option<u64>,
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
    /// Full sorted listing (source of truth) — Feature 022.
    pub all_entries: Vec<Entry>,
    /// The currently displayed (filtered) listing.
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
            all_entries: Vec::new(),
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
            // Feature 022: best-effort size + modified time (epoch secs).
            let meta = entry.metadata().ok();
            let is_dir = match entry.file_type() {
                Ok(ft) => ft.is_dir(),
                Err(_) => entry.path().is_dir(),
            };
            let mtime = meta.as_ref().and_then(|m| m.modified().ok()).and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
            });
            if is_dir {
                dirs.push(Entry {
                    name,
                    kind: EntryKind::Dir,
                    size: None,
                    mtime,
                });
            } else {
                files.push(Entry {
                    name,
                    kind: EntryKind::File,
                    size: meta.as_ref().map(|m| m.len()),
                    mtime,
                });
            }
        }
        let by_name = |a: &Entry, b: &Entry| a.name.to_lowercase().cmp(&b.name.to_lowercase());
        dirs.sort_by(by_name);
        files.sort_by(by_name);

        let mut all = Vec::with_capacity(dirs.len() + files.len() + 1);
        if !self.is_root() {
            all.push(Entry {
                name: "..".to_string(),
                kind: EntryKind::Parent,
                size: None,
                mtime: None,
            });
        }
        all.extend(dirs);
        all.extend(files);

        self.all_entries = all;
        self.error = None;
        self.scroll = 0;
        // Feature 022: derive the displayed (filtered) listing.
        self.apply_filter();
    }

    /// Feature 022: derive `entries` (displayed) from `all_entries` by applying the
    /// field text as a filter. Empty or an absolute path → no filtering; a pattern
    /// with `*`/`?` → glob; otherwise case-insensitive substring. Directories and
    /// `..` are always kept so navigation is never blocked. Re-clamps the selection.
    pub fn apply_filter(&mut self) {
        let pat = self.filename.trim();
        let no_filter = pat.is_empty() || pat.starts_with('/');
        self.entries = if no_filter {
            self.all_entries.clone()
        } else if is_glob(pat) {
            self.all_entries
                .iter()
                .filter(|e| !matches!(e.kind, EntryKind::File) || glob_match(pat, &e.name))
                .cloned()
                .collect()
        } else {
            let needle = pat.to_lowercase();
            self.all_entries
                .iter()
                .filter(|e| {
                    !matches!(e.kind, EntryKind::File) || e.name.to_lowercase().contains(&needle)
                })
                .cloned()
                .collect()
        };
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
        if self.scroll >= self.entries.len() {
            self.scroll = 0;
        }
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

    /// Feature 028: move the selection up by ~one page, clamped to the top.
    pub fn page_up(&mut self, visible_rows: usize) {
        let step = visible_rows.max(1);
        self.selected = self.selected.saturating_sub(step);
        self.ensure_visible(visible_rows);
    }

    /// Feature 028: move the selection down by ~one page, clamped to the bottom.
    pub fn page_down(&mut self, visible_rows: usize) {
        let step = visible_rows.max(1);
        let last = self.entries.len().saturating_sub(1);
        self.selected = (self.selected + step).min(last);
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
            // Feature 028: saturating to avoid any underflow if the guard changes.
            self.scroll = (self.selected + 1).saturating_sub(visible_rows);
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
        self.apply_filter(); // Feature 022: live filtering
    }

    /// Backspace: delete the last char of the field, or go to parent when empty.
    pub fn backspace(&mut self) {
        if self.filename.is_empty() {
            self.enter_parent();
        } else {
            self.filename.pop();
            self.apply_filter(); // Feature 022: live filtering
        }
    }

    // ── Mouse hit-testing (shares geometry with the widget) ──────────────────

    /// Number of entry rows visible for a frame of size `area`.
    pub fn visible_rows(&self, area: Rect) -> usize {
        compute_layout(area, self.mode).list_rows as usize
    }

    /// Feature 020: outer box rect of the browser, for sharing button geometry
    /// between the renderer and the app's mouse hit-testing.
    pub fn box_rect(&self, area: Rect) -> Rect {
        compute_layout(area, self.mode).box_rect
    }

    /// Feature 024: the interactive vertical-scrollbar region for the listing, or
    /// `None` when the list fits. Returns `(bar_rect, content, viewport, offset)`
    /// where `bar_rect` is the single drawn column (matching the renderer).
    pub fn list_scrollbar(&self, area: Rect) -> Option<(Rect, usize, usize, usize)> {
        let l = compute_layout(area, self.mode);
        let content = self.entries.len();
        let viewport = l.list_rows as usize;
        if content <= viewport || l.list_rows == 0 || l.inner_width == 0 {
            return None;
        }
        let col = l.inner_left + l.inner_width - 1;
        Some((
            Rect::new(col, l.list_top, 1, l.list_rows),
            content,
            viewport,
            self.scroll,
        ))
    }

    /// Feature 024: set the listing scroll offset directly (clamped), keeping the
    /// selection within the visible window. Used by scrollbar drag/click.
    pub fn set_scroll(&mut self, offset: usize, viewport: usize) {
        let max = self.entries.len().saturating_sub(viewport);
        self.scroll = offset.min(max);
        // Keep the highlighted row within the visible window.
        if self.selected < self.scroll {
            self.selected = self.scroll;
        } else if viewport > 0 && self.selected >= self.scroll + viewport {
            self.selected = self.scroll + viewport - 1;
        }
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
    /// Feature 018: row of the input-field label, and the bordered field box.
    field_label_row: u16,
    field_box: Rect,
}

fn compute_layout(area: Rect, mode: BrowseMode) -> BrowserLayout {
    let bw = 64u16.min(area.width.max(1));
    // Feature 020: +4 rows over the original 20 reserve a 1-row gap and a 3-row
    // boxed-button box (Open|Save / Cancel) along the bottom interior.
    let bh = 24u16.min(area.height.max(1));
    let bx = area.x + area.width.saturating_sub(bw) / 2;
    let by = area.y + area.height.saturating_sub(bh) / 2;
    let box_rect = Rect::new(bx, by, bw, bh);

    let inner_left = bx + 1;
    let inner_width = bw.saturating_sub(2);
    let inner_top = by + 1;
    // Reserve the bottom 4 interior rows for the button area (gap + 3-row box).
    let inner_h = bh.saturating_sub(2).saturating_sub(4);

    let header_row = inner_top;
    let footer_row = inner_top + inner_h.saturating_sub(1);
    // Feature 018: both modes reserve a labeled, bordered input box above the
    // footer — a label row + a 3-row box. The list shrinks to fit.
    let _ = mode; // both modes use the same field region now
    let field_box_y = footer_row.saturating_sub(3);
    let field_box = Rect::new(inner_left, field_box_y, inner_width, 3);
    let field_label_row = field_box_y.saturating_sub(1);
    let list_top = header_row + 1;
    let list_rows = field_label_row.saturating_sub(list_top);

    BrowserLayout {
        box_rect,
        list_top,
        list_rows,
        inner_left,
        inner_width,
        header_row,
        footer_row,
        field_label_row,
        field_box,
    }
}

// ---------------------------------------------------------------------------
// UTF-8 / wide-character-safe truncation
// ---------------------------------------------------------------------------

/// Approximate display width of a grapheme's leading scalar (1 narrow, 2 wide).
/// Display width of a grapheme cluster.
///
/// Feature 029: delegates to the single shared width helper
/// ([`crate::ui::width::display_width`]) so combining marks (0), wide CJK (2), and
/// emoji are measured consistently everywhere. Kept as a thin alias because many
/// call sites in this module (and `tabbar`) import it.
pub fn grapheme_width(g: &str) -> u16 {
    crate::ui::width::display_width(g)
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
// Feature 022: glob matching + size/date formatting (std-only, no new crates)
// ---------------------------------------------------------------------------

/// Case-insensitive wildcard match anchored to the whole `name`. Supports `*`
/// (any run, incl. empty) and `?` (exactly one char). No character classes.
/// Classic linear two-pointer backtracking.
pub fn glob_match(pattern: &str, name: &str) -> bool {
    let p: Vec<char> = pattern.to_lowercase().chars().collect();
    let s: Vec<char> = name.to_lowercase().chars().collect();
    let (mut pi, mut si) = (0usize, 0usize);
    let (mut star, mut mark): (Option<usize>, usize) = (None, 0);
    while si < s.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == s[si]) {
            pi += 1;
            si += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = si;
            pi += 1;
        } else if let Some(st) = star {
            pi = st + 1;
            mark += 1;
            si = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// `true` when `pattern` should be treated as a glob (has `*` or `?`).
pub fn is_glob(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

/// Human-readable byte size: `0B`, `1023B`, `1.0K`, `15K`, `3.4M`, `2.0G`.
/// Sub-10 magnitudes get one decimal; larger values are rounded to integer.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "K", "M", "G"];
    if bytes < 1024 {
        return format!("{bytes}B");
    }
    let mut val = bytes as f64;
    let mut unit = 0usize;
    while val >= 1024.0 && unit < UNITS.len() - 1 {
        val /= 1024.0;
        unit += 1;
    }
    if val < 10.0 {
        format!("{:.1}{}", val, UNITS[unit])
    } else {
        format!("{:.0}{}", val, UNITS[unit])
    }
}

/// Format a Unix epoch-seconds timestamp as `YYYY-MM-DD HH:MM` in UTC, using the
/// days-from-civil algorithm (no `chrono`/`time` dependency).
pub fn format_mtime(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (hh, mm) = ((rem / 3600) as u32, ((rem % 3600) / 60) as u32);
    // days since 1970-01-01 → civil (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}")
}

// ---------------------------------------------------------------------------
// Widget
// ---------------------------------------------------------------------------

/// Renders an open [`FileBrowser`] as a centered bordered modal.
pub struct FileBrowserWidget<'a> {
    pub browser: &'a FileBrowser,
    pub theme: &'static Theme,
    /// Feature 020: focused boxed button (`Some(i)`), or `None` when the browser
    /// list/field is focused. Buttons are `Open`/`Save` (0) / `Cancel` (1).
    pub button_focus: Option<usize>,
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

        // Entry list. Feature 021: when the listing overflows the visible rows,
        // reserve the rightmost interior column for a vertical scrollbar so entry
        // names are never drawn under the bar.
        let bar_w: u16 = if crate::ui::scrollbar::is_needed(b.entries.len(), l.list_rows as usize) {
            1
        } else {
            0
        };
        let row_w = iw.saturating_sub(bar_w);
        // Feature 022: detail columns — size (right-aligned) + modified date.
        // Shown only when the row is wide enough; otherwise name-only (degrade).
        const SIZE_W: u16 = 6; // "1023B" / "1.0K" / "<DIR>"
        const DATE_W: u16 = 16; // "YYYY-MM-DD HH:MM"
        let detail_w = SIZE_W + 1 + DATE_W; // size + gap + date
        let show_detail = row_w >= 1 + 8 + 1 + detail_w; // marker + min name + gap + detail
        let name_budget = if show_detail {
            row_w.saturating_sub(1 + 1 + detail_w) // marker + gap-before-detail + detail
        } else {
            row_w.saturating_sub(2) // marker + space (legacy)
        };
        for vis in 0..l.list_rows {
            let idx = b.scroll + vis as usize;
            if idx >= b.entries.len() {
                break;
            }
            let entry = &b.entries[idx];
            let y = l.list_top + vis;
            let style = if idx == b.selected { selected } else { base };
            // Fill the row so the selection highlight spans the width (minus the
            // reserved scrollbar column).
            for cx in l.inner_left..l.inner_left + row_w {
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
            if show_detail {
                // Size column (right-aligned): "<DIR>" for dirs/parent, human size
                // for files, blank when metadata is unreadable.
                let size_str = match entry.kind {
                    EntryKind::File => entry.size.map(human_size).unwrap_or_default(),
                    _ => "<DIR>".to_string(),
                };
                let date_str = entry.mtime.map(format_mtime).unwrap_or_default();
                let detail_start = l.inner_left + row_w - detail_w;
                let size_w = size_str
                    .graphemes(true)
                    .map(grapheme_width)
                    .sum::<u16>()
                    .min(SIZE_W);
                put(
                    buf,
                    detail_start + SIZE_W - size_w,
                    y,
                    &truncate_to_width(&size_str, SIZE_W),
                    style,
                );
                put(
                    buf,
                    detail_start + SIZE_W + 1,
                    y,
                    &truncate_to_width(&date_str, DATE_W),
                    style,
                );
            }
        }

        // Feature 021: vertical scrollbar over the list area's right column when
        // the listing overflows (the column was reserved above).
        if bar_w > 0 {
            crate::ui::scrollbar::render_vertical(
                buf,
                Rect::new(l.inner_left, l.list_top, iw, l.list_rows),
                b.entries.len(),
                l.list_rows as usize,
                b.scroll,
                self.theme,
            );
        }

        // Feature 018: labeled, bordered input box (both modes) so it's clear the
        // field is typeable. Save = filename; Open = jump-to path.
        let field_label = match b.mode {
            BrowseMode::Save => "Name:",
            BrowseMode::Open => "Go to path:",
        };
        put(
            buf,
            l.inner_left,
            l.field_label_row,
            field_label,
            base.add_modifier(Modifier::BOLD),
        );
        Block::default()
            .borders(Borders::ALL)
            .style(base)
            .render(l.field_box, buf);
        // Field text with an always-visible caret, right-anchored so the caret
        // (end of text) stays visible when the value is long.
        let box_inner_w = l.field_box.width.saturating_sub(2);
        let text_with_caret = format!("{}▏", b.filename);
        let shown = {
            let total: u16 = text_with_caret.graphemes(true).map(grapheme_width).sum();
            if total <= box_inner_w {
                text_with_caret
            } else {
                // Keep the tail (caret + latest chars) visible.
                let mut acc = 0u16;
                let mut tail = String::new();
                for g in text_with_caret.graphemes(true).rev() {
                    let w = grapheme_width(g);
                    if acc + w > box_inner_w {
                        break;
                    }
                    acc += w;
                    tail.insert_str(0, g);
                }
                tail
            }
        };
        put(buf, l.field_box.x + 1, l.field_box.y + 1, &shown, base);

        // Footer hints.
        let hints = match b.mode {
            BrowseMode::Open => "↑↓ move  type path  Enter open  ← parent  Esc cancel",
            BrowseMode::Save => "↑↓ move  type name  Enter save  ← parent  Esc cancel",
        };
        put(
            buf,
            l.inner_left,
            l.footer_row,
            &truncate_to_width(hints, iw),
            base,
        );

        // Feature 020: boxed confirm/cancel buttons in the bottom interior rows.
        // `usize::MAX` highlights none (used when the list/field is focused).
        let confirm = match b.mode {
            BrowseMode::Open => "Open",
            BrowseMode::Save => "Save",
        };
        let labels = [confirm, "Cancel"];
        let rects = crate::ui::buttons::button_rects(box_rect, &labels);
        crate::ui::buttons::render_buttons(
            buf,
            &rects,
            &labels,
            self.button_focus.unwrap_or(usize::MAX),
            self.theme,
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

    // T019 (Feature 028): PageUp/PageDown move by a page and clamp; no underflow.
    #[test]
    fn page_down_up_move_by_page_and_clamp() {
        let base = temp_tree("paging");
        let mut b = FileBrowser::open(base, BrowseMode::Open);
        let n = b.entries.len();
        assert!(n >= 3, "fixture should yield several entries");
        b.selected = 0;
        let vis = 2;
        b.page_down(vis);
        assert_eq!(b.selected, 2.min(n - 1), "page down moves by ~one page");
        // Repeated page-downs clamp to the last entry, never overrun.
        for _ in 0..10 {
            b.page_down(vis);
        }
        assert_eq!(b.selected, n - 1);
        // Page up clamps to the top with no underflow.
        for _ in 0..10 {
            b.page_up(vis);
        }
        assert_eq!(b.selected, 0);
        assert_eq!(b.scroll, 0);
    }

    // Feature 018: the editable field renders as a labeled, bordered box with a
    // caret in BOTH modes (Open's path field was previously invisible).
    fn render_browser(b: &FileBrowser) -> String {
        use ratatui::{buffer::Buffer, layout::Rect};
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        FileBrowserWidget {
            browser: b,
            theme: crate::ui::theme::theme_by_name("classic"),
            button_focus: None,
        }
        .render(area, &mut buf);
        buf.content().iter().map(|c| c.symbol()).collect()
    }

    // ── Feature 022 — glob / size / date helpers ──────────────────────────────

    #[test]
    fn glob_match_basics() {
        assert!(glob_match("*.log", "a.log"));
        assert!(glob_match("*.LOG", "a.log"), "case-insensitive");
        assert!(!glob_match("*.log", "a.txt"));
        assert!(glob_match("te?t", "test"));
        assert!(glob_match("te?t", "text"));
        assert!(!glob_match("te?t", "tt"), "? is exactly one char");
        assert!(glob_match("*", "anything"));
        assert!(glob_match("a*z", "abcz"));
        assert!(!glob_match("a*z", "abc"), "anchored to whole name");
        assert!(glob_match("*foo*", "xxfooyy"));
    }

    #[test]
    fn human_size_boundaries() {
        assert_eq!(human_size(0), "0B");
        assert_eq!(human_size(1023), "1023B");
        assert_eq!(human_size(1024), "1.0K");
        assert_eq!(human_size(1536), "1.5K");
        assert_eq!(human_size(20 * 1024), "20K");
        assert_eq!(human_size(3 * 1024 * 1024 + 512 * 1024), "3.5M");
        assert_eq!(human_size(2 * 1024 * 1024 * 1024), "2.0G");
    }

    // Feature 022: the listing shows a <DIR> marker for directories and a size +
    // date for files; a long name truncates while detail columns remain.
    #[test]
    fn listing_shows_detail_columns_and_truncates_name() {
        let base = std::env::temp_dir().join("edit_fb_detail");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("subdir")).unwrap();
        fs::write(
            base.join("averylongfilename_that_should_be_truncated_in_the_listing.txt"),
            vec![b'x'; 2048],
        )
        .unwrap();
        let b = FileBrowser::open(base.clone(), BrowseMode::Open);
        let rendered = render_browser(&b);
        assert!(
            rendered.contains("<DIR>"),
            "directory shows a <DIR> indicator"
        );
        assert!(
            rendered.contains("2.0K"),
            "file shows a human-readable size"
        );
        assert!(
            rendered.contains('…'),
            "long name is truncated with an ellipsis"
        );
        // A date column (a 20xx year) is present.
        assert!(rendered.contains("20"), "a modified date is shown");
        let _ = fs::remove_dir_all(&base);
    }

    // Feature 022: apply_filter keeps dirs + ".." and filters files.
    #[test]
    fn apply_filter_keeps_dirs_and_filters_files() {
        let base = std::env::temp_dir().join("edit_fb_filter_keep");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("sub")).unwrap();
        fs::write(base.join("a.log"), b"x").unwrap();
        fs::write(base.join("b.txt"), b"x").unwrap();
        let mut b = FileBrowser::open(base.clone(), BrowseMode::Open);

        // Glob filter: only *.log files, but dirs + ".." remain.
        b.filename = "*.log".to_string();
        b.apply_filter();
        let names: Vec<&str> = b.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&".."), "parent kept");
        assert!(names.contains(&"sub"), "directory kept");
        assert!(names.contains(&"a.log"));
        assert!(!names.contains(&"b.txt"), "non-matching file hidden");

        // Substring filter (case-insensitive).
        b.filename = "B".to_string();
        b.apply_filter();
        let names: Vec<&str> = b.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"b.txt"));
        assert!(!names.contains(&"a.log"));
        assert!(
            names.contains(&"sub"),
            "dir still kept under substring filter"
        );

        // Clearing restores the full listing.
        b.filename.clear();
        b.apply_filter();
        assert_eq!(b.entries.len(), b.all_entries.len());

        // Absolute path is a jump target, not a filter (listing unfiltered).
        b.filename = "/etc".to_string();
        b.apply_filter();
        assert_eq!(b.entries.len(), b.all_entries.len());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn apply_filter_reclamps_selection() {
        let base = std::env::temp_dir().join("edit_fb_filter_clamp");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        for i in 0..5 {
            fs::write(base.join(format!("f{i}.txt")), b"x").unwrap();
        }
        fs::write(base.join("only.log"), b"x").unwrap();
        let mut b = FileBrowser::open(base.clone(), BrowseMode::Open);
        b.selected = b.entries.len() - 1; // last entry
        b.filename = "*.log".to_string();
        b.apply_filter();
        assert!(
            b.selected < b.entries.len(),
            "selection re-clamped into the filtered list"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn format_mtime_known_epoch() {
        // 2021-01-01 00:00:00 UTC = 1_609_459_200.
        assert_eq!(format_mtime(1_609_459_200), "2021-01-01 00:00");
        // 1970-01-01 00:00:00 UTC = 0.
        assert_eq!(format_mtime(0), "1970-01-01 00:00");
        // 1_781_876_700 = 2026-06-19 13:45 UTC.
        assert_eq!(format_mtime(1_781_876_700), "2026-06-19 13:45");
    }

    // Feature 021: a vertical scrollbar (thumb glyph) appears only when the
    // listing overflows the visible rows.
    #[test]
    fn scrollbar_shown_only_when_list_overflows() {
        let base = std::env::temp_dir().join("edit_fb_scroll_overflow");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        // Far more entries than the ~14 visible list rows of a 24-row terminal.
        for i in 0..60 {
            fs::write(base.join(format!("file{i:03}.txt")), b"x").unwrap();
        }
        let b = FileBrowser::open(base.clone(), BrowseMode::Open);
        let rendered = render_browser(&b);
        assert!(
            rendered.contains('█') || rendered.contains('▲'),
            "overflowing listing draws a scrollbar"
        );

        // A directory that fits draws no scrollbar.
        let small = temp_tree("fb_no_scroll");
        let sb = FileBrowser::open(small, BrowseMode::Open);
        let small_render = render_browser(&sb);
        assert!(
            !small_render.contains('█') && !small_render.contains('▲'),
            "a listing that fits draws no scrollbar"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn open_mode_shows_path_input_box_with_caret() {
        let base = temp_tree("open_field");
        let b = FileBrowser::open(base.clone(), BrowseMode::Open);
        let s = render_browser(&b);
        assert!(
            s.contains("Go to path:"),
            "Open mode shows a path field label"
        );
        assert!(s.contains('▏'), "field shows a caret");
        assert!(
            s.contains('┌') && s.contains('│'),
            "field is a bordered box"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn save_mode_shows_name_input_box() {
        let base = temp_tree("save_field");
        let mut b = FileBrowser::open(base.clone(), BrowseMode::Save);
        b.filename = "report.txt".to_string();
        let s = render_browser(&b);
        assert!(s.contains("Name:"), "Save mode shows a name field label");
        assert!(s.contains("report.txt"), "typed filename shown in the box");
        assert!(s.contains('┌'), "field is a bordered box");
        let _ = fs::remove_dir_all(&base);
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
