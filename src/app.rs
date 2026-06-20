//! Application state machine and top-level event dispatch.
//!
//! [`App`] owns all editor state and drives the main event loop.

#![allow(dead_code, unused_variables, unused_imports)]

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    buffer::undo::EditOp,
    buffer::{Buffer, CursorPos, Selection},
    config::Config,
    encoding::EncodingId,
    input::mouse::{normalize_mouse, MouseButton, NormalizedMouseKind},
    input::{dispatch_event, Action, KeybindingMap},
    search::{SearchEngine, SearchState},
    ui::dialog::{DialogField, DialogMode, FindReplaceDialog},
    ui::file_browser::{BrowseMode, BrowserHit, FileBrowser, Outcome as BrowseOutcome},
    ui::menubar::{hit_test_menu, resolve_menus, MenuBarState, MenuHit, MenuState, ResolvedMenu},
    ui::theme::{theme_by_name, Theme},
};

/// Which built-in informational overlay is open (Help menu).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpScreen {
    /// Key-binding cheat sheet (Help ▸ Help).
    Help,
    /// Program name, version, and copyright (Help ▸ About).
    About,
}

/// Confirm/dismiss dialogs that carry a boxed button bar (Feature 016).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonDialog {
    SessionRestore,
    SavePrompt,
    ExternalChange,
    RevertConfirm,
    PluginConsent,
    /// Feature 027: closing a modified buffer via its tab `[x]`. Acts on the
    /// stored buffer index (`pending_close_confirm`), not necessarily the active.
    CloseConfirm,
}

/// Interactive/list dialogs that carry a combined primary-control + boxed-button
/// focus ring (Feature 020). Unlike [`ButtonDialog`], focus stop 0 (and, for
/// Find/Replace in replace mode, stop 1) is the dialog's primary control (its
/// list or field group); the remaining stops are its buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InteractiveDialog {
    EncodingSelect,
    PluginManager,
    FindReplace,
    FileBrowser,
}

/// Feature 024: which scroll offset a scrollbar interaction drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollTarget {
    EditorV(usize),
    EditorH(usize),
    FileBrowser,
    Help,
    Encoding,
    Plugin,
}

/// Feature 024: a scrollbar's axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollAxis {
    Vertical,
    Horizontal,
}

/// Feature 024: a drawn, interactive scrollbar region for the active surface.
#[derive(Debug, Clone, Copy)]
struct ScrollbarRegion {
    rect: ratatui::layout::Rect,
    axis: ScrollAxis,
    content: usize,
    viewport: usize,
    offset: usize,
    target: ScrollTarget,
}

/// Feature 024: an in-progress thumb drag, bound to one surface/axis until release.
#[derive(Debug, Clone, Copy)]
struct ScrollbarDrag {
    target: ScrollTarget,
    axis: ScrollAxis,
    track_start: u16,
    track_len: u16,
    content: usize,
    viewport: usize,
}

/// Char index where the cursor should land after **undo**-ing `op`.
fn undo_target_idx(op: &EditOp) -> usize {
    match op {
        // Inserted text was removed → land where it had been.
        EditOp::Insert { at, .. } => *at,
        // Deleted text was reinserted → land at its end.
        EditOp::Delete { at, text } => at + text.chars().count(),
        EditOp::Composite(ops) => ops.first().map(undo_target_idx).unwrap_or(0),
    }
}

/// Char index where the cursor should land after **redo**-ing `op`.
fn redo_target_idx(op: &EditOp) -> usize {
    match op {
        // Text was (re)inserted → land at its end.
        EditOp::Insert { at, text } => at + text.chars().count(),
        // Text was (re)deleted → land where it had been.
        EditOp::Delete { at, .. } => *at,
        EditOp::Composite(ops) => ops.last().map(redo_target_idx).unwrap_or(0),
    }
}

/// Minimum terminal dimensions supported by the editor.
const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;

/// Tick interval for autosave and status-bar refresh.
const TICK_MS: u64 = 500;

/// Max gap between two clicks on the same file-browser row to count as a
/// double-click (which activates the entry). Feature 012.
const DOUBLE_CLICK_MS: u64 = 400;

/// Lines/rows/items scrolled per mouse-wheel notch (Feature 023).
const WHEEL_STEP: usize = 3;

/// Items moved per PageUp/PageDown in the small fixed-size list dialogs
/// (encoding select, plugin manager) — Feature 028. The file browser pages by its
/// actual visible-row count instead.
const DIALOG_LIST_PAGE: usize = 5;

// ── Direction enum ────────────────────────────────────────────────────────────

/// Cardinal directions for cursor movement.
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// ── Application state ────────────────────────────────────────────────────────

/// Top-level application state.
pub struct App {
    /// Loaded configuration (merged with CLI flags).
    pub config: Config,
    /// Active keybinding map.
    pub keymap: KeybindingMap,
    /// All open buffers (non-empty; at least one new empty buffer exists).
    pub buffers: Vec<Buffer>,
    /// Index of the currently active buffer.
    pub active_idx: usize,
    /// Set to false to exit the event loop.
    pub running: bool,
    /// Current terminal dimensions (columns, rows).
    pub terminal_size: (u16, u16),
    /// Active color theme.
    pub theme: &'static Theme,
    /// Split-view mode — Single (default) or Vertical (T067).
    pub split_mode: crate::ui::SplitMode,
    /// Whether the menu bar is currently active (legacy flag; superseded by menu_bar.is_active()).
    pub menu_active: bool,
    /// Pull-down menu state machine (T041).
    pub menu_bar: MenuBarState,
    /// Whether the unsaved-changes save prompt is pending.
    pub pending_save_prompt: bool,
    /// Whether the terminal is too small to render the editor.
    pub too_small: bool,
    /// Current search-and-replace session state (T055).
    pub search_state: SearchState,
    /// Transient message shown in the status bar (e.g. "Match 2/5").
    pub status_message: Option<String>,
    /// Session data pending user confirmation; `Some` = restore dialog is visible.
    pub pending_session_restore: Option<crate::session::SessionData>,
    /// Default encoding resolved from config/CLI at startup.
    pub default_encoding: EncodingId,
    /// Index into ENCODING_OPTIONS of the highlighted row; `Some` = dialog is open.
    pub pending_encoding_select: Option<usize>,
    /// The navigable file browser (Open/Save); `Some` = a file dialog is open (Feature 012).
    pub file_browser: Option<FileBrowser>,
    /// Last file-browser entry click (index + time) for double-click detection.
    /// A single click selects the row; a second click on the same row within
    /// [`DOUBLE_CLICK_MS`] activates it (enter folder / open file) — Feature 012.
    pub last_browser_click: Option<(usize, Instant)>,
    /// Feature 030: last editor left-press `(col, row, count, time)` for
    /// double/triple-click detection (count 1=single, 2=word, 3=line).
    pub last_editor_click: Option<(u16, u16, u8, Instant)>,
    /// Which Help overlay is open, if any (Feature 011).
    pub pending_help: Option<HelpScreen>,
    /// Encoding selected in the dialog, held across the filename prompt (US4).
    pub pending_save_as_encoding: Option<EncodingId>,
    /// Whether soft-wrap visual rendering is active (Feature 005).
    pub soft_wrap: bool,
    /// Computed wrap cache; `None` when soft_wrap is false.
    pub wrap_cache: Option<crate::ui::wrap::WrapCache>,
    /// Generation counter incremented on every buffer mutation for cache invalidation.
    pub wrap_text_gen: u64,

    // ── Feature 007: External File Modification Detection ─────────────────────
    /// OS-native filesystem watcher; `None` when `--no-watch` is active.
    pub file_watcher: Option<crate::watcher::FileWatcher>,
    /// Tracks when the editor last wrote each backing file path (self-write suppression).
    pub self_write_times: std::collections::HashMap<PathBuf, Instant>,
    /// Set when an external modification is detected; cleared by user's Y/N response.
    pub pending_external_change: Option<crate::watcher::ExternalChange>,
    /// One-shot status-bar notice (e.g., file-deleted notice); cleared after one render frame.
    pub watcher_notice: Option<String>,

    // ── Feature 014: Revert confirmation ──────────────────────────────────────
    /// Set to the buffer index awaiting a Revert confirmation (buffer is modified);
    /// `Some` shows a modal confirm dialog. Cleared on confirm/cancel.
    pub pending_revert_confirm: Option<usize>,

    // ── Feature 027: tab `[x]` close confirmation ─────────────────────────────
    /// Set to the buffer index awaiting a close confirmation (the clicked tab's
    /// buffer is modified). `Some` shows the [`ButtonDialog::CloseConfirm`] modal.
    pub pending_close_confirm: Option<usize>,

    /// Feature 030 (US3): the open editor right-click context menu, if any.
    pub pending_context_menu: Option<crate::ui::contextmenu::ContextMenu>,

    // ── Feature 015: interactive Find / Replace dialog ────────────────────────
    /// `Some` while an interactive Find/Replace dialog is open (modal).
    pub pending_find_replace: Option<FindReplaceDialog>,

    // ── Feature 025: Go-to-Line prompt ────────────────────────────────────────
    /// `Some(digits)` while the Go-to-Line prompt is open (modal); holds the
    /// in-progress 1-based line number being typed.
    pub pending_goto_line: Option<String>,
    /// Feature 031: caret position (index into the digit string) for the
    /// Go-to-Line input — supports mid-string edit and click-to-position.
    pub pending_goto_line_caret: usize,

    // ── Feature 016: focused dialog button ────────────────────────────────────
    /// Index of the focused button in the currently open confirm/dismiss dialog.
    /// Only one modal is open at a time, so a single field suffices. Reset to the
    /// dialog's safe default when it opens.
    pub dialog_focus: usize,
    /// Whether `dialog_focus` has been initialized for the currently open dialog
    /// (so focus defaults to the safe button once, then the user can move it).
    pub dialog_focus_init: bool,

    // ── Feature 017: mouse drag selection ─────────────────────────────────────
    /// Anchor (cursor position at the left-button press in the editor) for a
    /// drag selection; `Some` between press and the drags that follow.
    pub drag_anchor: Option<CursorPos>,

    /// Feature 024: in-progress scrollbar thumb drag; `Some` between a thumb press
    /// and the button release. While set, mouse drags scroll instead of selecting.
    scrollbar_drag: Option<ScrollbarDrag>,

    // ── Feature 018: Help scroll ──────────────────────────────────────────────
    /// First visible row of the Help cheat-sheet (scroll offset); reset on open.
    pub help_scroll: usize,

    // ── Feature 008: plugin subsystem ────────────────────────────────────────
    /// The Rhai plugin host owning the engine and registry for this session.
    pub plugin_host: crate::plugin::PluginHost,
    /// Plugins awaiting a first-run consent decision; the front item is prompted.
    pub pending_plugin_consent: Vec<crate::plugin::PluginMeta>,
    /// When true, the Options > Plugins manager overlay is open.
    pub pending_plugin_manager: bool,
    /// Cursor index within the plugin manager list.
    pub plugin_manager_cursor: usize,
}

// ── App impl ─────────────────────────────────────────────────────────────────

impl App {
    /// Create an [`App`] from a loaded [`Config`], a list of file paths, the
    /// encoding to use when opening files, an optional session to restore, and
    /// an optional corrupt-session warning to display on startup.
    pub fn new(
        config: Config,
        files: Vec<PathBuf>,
        default_encoding: EncodingId,
        session: Option<crate::session::SessionData>,
        session_warning: Option<String>,
    ) -> Self {
        let theme = theme_by_name(&config.theme);
        let readonly = config.readonly;

        // ── Feature 008: initialise the plugin host and load allowed plugins ──
        let plugin_config_dir = crate::plugin::edit_config_dir();
        let mut plugin_host = crate::plugin::PluginHost::new(config.no_plugins);
        let mut pending_plugin_consent: Vec<crate::plugin::PluginMeta> = Vec::new();
        {
            let consent_records = crate::plugin::load_consent_records(&plugin_config_dir);
            plugin_host.load_all(
                &plugin_config_dir,
                &consent_records,
                &mut pending_plugin_consent,
            );
        }

        let keymap = {
            let mut km = KeybindingMap::default_map();
            km.apply_user_overrides(&config.keybindings);
            // Plugin-provided keybindings take precedence over built-ins, except
            // safety-critical actions (Quit/Save) which cannot be overridden.
            km.apply_plugin_bindings(&plugin_host.registry.all_keybindings());
            km
        };

        // Feature 029: collect startup open failures so they can be surfaced in the
        // status bar instead of silently becoming a blank buffer.
        let mut open_errors: Vec<String> = Vec::new();
        let mut buffers: Vec<Buffer> = if files.is_empty() {
            vec![Buffer::new_empty()]
        } else {
            let mut v: Vec<Buffer> = Vec::new();
            for p in files {
                match Buffer::open(p.clone(), default_encoding) {
                    Ok(buf) => v.push(buf),
                    Err(e)
                        if matches!(&e, crate::buffer::BufferError::Io(io_err)
                            if io_err.kind() == io::ErrorKind::NotFound) =>
                    {
                        // New file (NotFound) — open an empty buffer at that path so
                        // Ctrl+S creates it.
                        log::info!("New file: {:?}", p);
                        let mut buf = Buffer::new_empty();
                        buf.path = Some(p);
                        v.push(buf);
                    }
                    Err(e) => {
                        // A real failure (permission, binary, too large, …): record it
                        // and start with a blank buffer rather than crashing.
                        log::error!("Failed to open {:?}: {}", p, e);
                        open_errors.push(format!("Open failed: {} — {}", p.display(), e));
                    }
                }
            }
            if v.is_empty() {
                v.push(Buffer::new_empty());
            }
            v
        };

        // T077 — Auto-detect syntax highlighter for each buffer on startup.
        // Feature 008: an active plugin highlighter takes precedence over the built-in
        // for the same extension; the built-in is the fallback.
        if config.highlight {
            for buf in &mut buffers {
                if let Some(ref path) = buf.path.clone() {
                    buf.syntax = plugin_host
                        .highlighter_for(path, theme)
                        .or_else(|| crate::highlight::detect_highlighter(path));
                }
            }
        }

        // T063 — On startup, check for stale recovery files and create lock files.
        if !readonly {
            let pid = std::process::id();
            for buf in &mut buffers {
                if buf.path.is_some() && buf.autosave.enabled {
                    use crate::buffer::autosave::{check_stale_lock, create_lock, LockStatus};
                    match check_stale_lock(&buf.autosave) {
                        LockStatus::StaleRecovery => {
                            buf.pending_recovery = true;
                            if let Some(p) = &buf.path {
                                log::info!("Stale recovery file found for {:?}", p);
                            }
                        }
                        LockStatus::OtherSessionActive(other_pid) => {
                            if let Some(p) = &buf.path {
                                log::warn!("Buffer {:?} is already open by pid {}", p, other_pid);
                            }
                        }
                        LockStatus::Clean => {}
                    }
                    if let Err(e) = create_lock(&buf.autosave.lock_path, pid) {
                        log::warn!("Failed to create lock file: {}", e);
                    }
                }
            }
        }

        // If a corrupt-session warning was produced but no valid session data
        // arrived, surface the warning in the status bar immediately.
        // Feature 029: a startup open failure takes precedence in the status bar;
        // otherwise fall back to the session-restore warning.
        let initial_status = if let Some(first) = open_errors.first() {
            Some(first.clone())
        } else if session_warning.is_some() && session.is_none() {
            session_warning
        } else {
            None
        };

        let soft_wrap_initial = config.soft_wrap;

        // ── Feature 007: initialise file watcher ─────────────────────────────
        let (file_watcher, initial_watch_notice) = if config.no_watch {
            (None, None)
        } else {
            match crate::watcher::FileWatcher::new() {
                Ok(mut fw) => {
                    for buf in &buffers {
                        if let Some(ref p) = buf.path {
                            if let Err(e) = fw.watch_path(p) {
                                log::warn!("FileWatcher: could not watch {:?}: {}", p, e);
                            }
                        }
                    }
                    (Some(fw), None)
                }
                Err(e) => {
                    log::warn!("FileWatcher: failed to initialise watcher: {}", e);
                    (
                        None,
                        Some(
                            "File watching unavailable — external changes won't be detected"
                                .to_owned(),
                        ),
                    )
                }
            }
        };

        // Watch-init notice is one-shot; prefer session startup warning in status_message.
        let watcher_notice = initial_watch_notice;
        let status_message = initial_status;

        Self {
            config,
            keymap,
            buffers,
            active_idx: 0,
            running: true,
            terminal_size: (80, 24),
            theme,
            split_mode: crate::ui::SplitMode::Single,
            menu_active: false,
            menu_bar: MenuBarState::new(),
            pending_save_prompt: false,
            too_small: false,
            search_state: SearchState::default(),
            status_message,
            pending_session_restore: session,
            default_encoding,
            pending_encoding_select: None,
            file_browser: None,
            last_browser_click: None,
            last_editor_click: None,
            pending_help: None,
            pending_save_as_encoding: None,
            soft_wrap: soft_wrap_initial,
            wrap_cache: None,
            wrap_text_gen: 0,
            file_watcher,
            self_write_times: std::collections::HashMap::new(),
            pending_external_change: None,
            watcher_notice,
            pending_revert_confirm: None,
            pending_close_confirm: None,
            pending_context_menu: None,
            pending_find_replace: None,
            pending_goto_line: None,
            pending_goto_line_caret: 0,
            dialog_focus: 0,
            dialog_focus_init: false,
            drag_anchor: None,
            scrollbar_drag: None,
            help_scroll: 0,
            plugin_host,
            pending_plugin_consent,
            pending_plugin_manager: false,
            plugin_manager_cursor: 0,
        }
    }

    /// Enter raw mode, start the event loop, and restore the terminal on exit.
    pub fn run(mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        // Feature 013: on terminals that support it, ask for modifier-only key
        // reports so a lone Alt can activate the menu bar. Best-effort — ignored
        // where unsupported (F10 / Alt+letter remain the entry path).
        let kbd_enhanced = matches!(
            crossterm::terminal::supports_keyboard_enhancement(),
            Ok(true)
        );
        if kbd_enhanced {
            let _ = execute!(
                stdout,
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                )
            );
        }
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);

        // T064 — Clean up lock files on exit.
        for buf in &self.buffers {
            if buf.autosave.enabled && !buf.autosave.lock_path.as_os_str().is_empty() {
                crate::buffer::autosave::release_lock(&buf.autosave.lock_path);
            }
        }

        // Feature 007: unwatch all buffer paths on exit.
        if let Some(ref mut fw) = self.file_watcher {
            for buf in &self.buffers {
                if let Some(ref p) = buf.path {
                    let _ = fw.unwatch_path(p);
                }
            }
        }

        disable_raw_mode()?;
        if kbd_enhanced {
            let _ = execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags);
        }
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    /// Return a reference to the currently active buffer.
    pub fn active_buffer(&self) -> &Buffer {
        &self.buffers[self.active_idx]
    }

    /// Return a mutable reference to the currently active buffer.
    pub fn active_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffers[self.active_idx]
    }

    // ── T011 — Encoding dialog helpers ───────────────────────────────────────

    /// Return the index in [`ENCODING_OPTIONS`] that matches `enc`, or 0 if not found.
    fn encoding_to_idx(enc: EncodingId) -> usize {
        crate::ui::dialog::ENCODING_OPTIONS
            .iter()
            .position(|(e, _)| *e == enc)
            .unwrap_or(0)
    }

    /// Return the display label for `enc` from [`ENCODING_OPTIONS`], or `"unknown"`.
    fn label_for_encoding(enc: EncodingId) -> &'static str {
        crate::ui::dialog::ENCODING_OPTIONS
            .iter()
            .find(|(e, _)| *e == enc)
            .map(|(_, label)| *label)
            .unwrap_or("unknown")
    }

    /// Feature 027: whether the buffer tab bar is shown (only with 2+ buffers).
    pub fn tab_bar_visible(&self) -> bool {
        self.buffers.len() > 1
    }

    /// Feature 027: first terminal row of the editor area — below the menu bar
    /// (row 0) and, when shown, the tab bar (row 1). Single source of truth so
    /// the render, scroll math, and mouse mapping all agree.
    pub fn editor_top(&self) -> u16 {
        1 + if self.tab_bar_visible() { 1 } else { 0 }
    }

    /// Viewport height in lines (terminal rows minus menubar, statusbar, the
    /// Feature-027 tab-bar row when shown, and — Feature 021 — the editor's bottom
    /// horizontal-scrollbar row in non-wrap mode). Single source of truth shared
    /// with the editor render and mouse mapping so scrolling/paging/cursor-
    /// visibility match what is drawn.
    fn viewport_height(&self) -> usize {
        let hbar = if self.soft_wrap { 0 } else { 1 };
        // editor_top accounts for the menu bar (+ tab bar); -1 for the status bar.
        (self.terminal_size.1 as usize).saturating_sub(self.editor_top() as usize + 1 + hbar)
    }

    /// Feature 028: `(max_scroll, page)` for the Help/About overlay, mirroring the
    /// renderer's geometry (`render_help_overlay`): box height `min(20, term_h)`,
    /// minus borders, a footer-hint row, and the reserved Close-button rows. Used to
    /// clamp keyboard scrolling so Home/End and PageUp/Down stay in range.
    fn help_view_metrics(&self, screen: HelpScreen) -> (usize, usize) {
        let dh = 20usize.min(self.terminal_size.1 as usize);
        let inner_h = dh.saturating_sub(2); // borders
        let body_rows = inner_h.saturating_sub(1 + 4); // footer hint + boxed Close button
        let total = crate::ui::help_total_lines(screen);
        let max_scroll = total.saturating_sub(body_rows);
        (max_scroll, body_rows.max(1))
    }

    /// Feature 023: scroll the editor pane `buf_idx` by `step` rows (viewport
    /// only — the cursor is not moved), clamped to `[0, content_rows-1]`. Content
    /// rows are visual rows in soft-wrap, else logical lines.
    fn wheel_scroll_editor(&mut self, buf_idx: usize, down: bool, step: usize) {
        let content_rows = if self.soft_wrap {
            self.wrap_cache
                .as_ref()
                .map(|c| c.total_visual_rows())
                .unwrap_or_else(|| self.buffers[buf_idx].rope.line_count())
        } else {
            self.buffers[buf_idx].rope.line_count()
        };
        let max = content_rows.saturating_sub(1);
        let off = self.buffers[buf_idx].scroll_offset.0;
        self.buffers[buf_idx].scroll_offset.0 = if down {
            (off + step).min(max)
        } else {
            off.saturating_sub(step)
        };
    }

    /// Feature 024: the interactive scrollbar regions for the currently-active
    /// surface (modal wins; else the editor pane under cursor column `col`). Only
    /// includes a bar when it is actually drawn (content overflows), so the
    /// interactive region equals the drawn one.
    fn scrollbar_regions(&self, col: u16, _row: u16) -> Vec<ScrollbarRegion> {
        use ratatui::layout::Rect;
        let (w, h) = self.terminal_size;
        let full = Rect::new(0, 0, w, h);
        let mut out = Vec::new();
        if let Some(screen) = self.pending_help {
            let dw = 64u16.min(w.max(1));
            let dh = 20u16.min(h.max(1));
            let dx = w.saturating_sub(dw) / 2;
            let dy = h.saturating_sub(dh) / 2;
            let body_rows = (dh as usize).saturating_sub(2 + 1 + 4); // borders + footer + button
            let content = crate::ui::help_total_lines(screen);
            if body_rows > 0 && content > body_rows && dw >= 2 {
                out.push(ScrollbarRegion {
                    rect: Rect::new(dx + dw - 2, dy + 1, 1, body_rows as u16),
                    axis: ScrollAxis::Vertical,
                    content,
                    viewport: body_rows,
                    offset: self.help_scroll.min(content),
                    target: ScrollTarget::Help,
                });
            }
        } else if let Some(idx) = self.pending_encoding_select {
            let rect = crate::ui::dialog::encoding_dialog_rect(full);
            let body_rows = (rect.height as usize).saturating_sub(2 + 4);
            let content = crate::ui::dialog::ENCODING_OPTIONS.len();
            if body_rows > 0 && content > body_rows && rect.width >= 2 {
                out.push(ScrollbarRegion {
                    rect: Rect::new(rect.x + rect.width - 2, rect.y + 1, 1, body_rows as u16),
                    axis: ScrollAxis::Vertical,
                    content,
                    viewport: body_rows,
                    offset: idx,
                    target: ScrollTarget::Encoding,
                });
            }
        } else if let Some(fb) = self.file_browser.as_ref() {
            if let Some((rect, content, viewport, offset)) = fb.list_scrollbar(full) {
                out.push(ScrollbarRegion {
                    rect,
                    axis: ScrollAxis::Vertical,
                    content,
                    viewport,
                    offset,
                    target: ScrollTarget::FileBrowser,
                });
            }
        } else if self.pending_plugin_manager {
            let rect = crate::ui::plugin_manager::manager_rect(
                &self.plugin_host,
                self.plugin_manager_cursor,
                full,
            );
            let body_rows = (rect.height as usize).saturating_sub(2 + 4);
            let content = self.plugin_host.registry.instances.len();
            if body_rows > 0 && content > body_rows && rect.width >= 2 {
                out.push(ScrollbarRegion {
                    rect: Rect::new(rect.x + rect.width - 2, rect.y + 1, 1, body_rows as u16),
                    axis: ScrollAxis::Vertical,
                    content,
                    viewport: body_rows,
                    offset: self.plugin_manager_cursor,
                    target: ScrollTarget::Plugin,
                });
            }
        } else if self.pending_find_replace.is_some() || self.pending_goto_line.is_some() {
            // Find/Replace and Go-to-Line have no scrollable content.
        } else {
            // Editor — the pane under the cursor column. Feature 027: the editor
            // starts below the tab bar when shown.
            let top = self.editor_top();
            let editor_area = Rect::new(0, top, w, h.saturating_sub(top + 1));
            let (pane, buf_idx) = if matches!(self.split_mode, crate::ui::SplitMode::Vertical) {
                let half = editor_area.width / 2;
                if col >= editor_area.x + half {
                    let idx = if self.buffers.len() > 1 {
                        self.active_idx.max(1)
                    } else {
                        0
                    };
                    (
                        Rect::new(
                            editor_area.x + half,
                            editor_area.y,
                            editor_area.width - half,
                            editor_area.height,
                        ),
                        idx,
                    )
                } else {
                    (
                        Rect::new(editor_area.x, editor_area.y, half, editor_area.height),
                        0,
                    )
                }
            } else {
                (editor_area, self.active_idx)
            };
            let (text, vbar, hbar) = crate::ui::editor_panes(pane, self.soft_wrap);
            let content_v = if self.soft_wrap {
                self.wrap_cache
                    .as_ref()
                    .map(|c| c.total_visual_rows())
                    .unwrap_or_else(|| self.buffers[buf_idx].rope.line_count())
            } else {
                self.buffers[buf_idx].rope.line_count()
            };
            let viewport_v = text.height as usize;
            if vbar.width > 0 && content_v > viewport_v {
                out.push(ScrollbarRegion {
                    rect: vbar,
                    axis: ScrollAxis::Vertical,
                    content: content_v,
                    viewport: viewport_v,
                    offset: self.buffers[buf_idx].scroll_offset.0,
                    target: ScrollTarget::EditorV(buf_idx),
                });
            }
            if let Some(hbar) = hbar {
                let gutter: u16 = if self.config.line_numbers { 4 } else { 0 };
                let viewport_h = text.width.saturating_sub(gutter) as usize;
                let content_h = crate::ui::max_visible_line_width(&self.buffers[buf_idx], text);
                if content_h > viewport_h {
                    out.push(ScrollbarRegion {
                        rect: hbar,
                        axis: ScrollAxis::Horizontal,
                        content: content_h,
                        viewport: viewport_h,
                        offset: self.buffers[buf_idx].scroll_offset.1,
                        target: ScrollTarget::EditorH(buf_idx),
                    });
                }
            }
        }
        out
    }

    /// Feature 024: write a new scroll `offset` for `target` (already bounded by
    /// the caller). Editor targets adjust the viewport only (cursor untouched).
    fn apply_scroll_target(&mut self, target: ScrollTarget, offset: usize, viewport: usize) {
        match target {
            ScrollTarget::EditorV(i) => {
                let content = if self.soft_wrap {
                    self.wrap_cache
                        .as_ref()
                        .map(|c| c.total_visual_rows())
                        .unwrap_or_else(|| self.buffers[i].rope.line_count())
                } else {
                    self.buffers[i].rope.line_count()
                };
                self.buffers[i].scroll_offset.0 = offset.min(content.saturating_sub(1));
            }
            ScrollTarget::EditorH(i) => {
                self.buffers[i].scroll_offset.1 = offset;
            }
            ScrollTarget::FileBrowser => {
                if let Some(fb) = self.file_browser.as_mut() {
                    fb.set_scroll(offset, viewport);
                }
            }
            ScrollTarget::Help => {
                self.help_scroll = offset;
            }
            ScrollTarget::Encoding => {
                let n = crate::ui::dialog::ENCODING_OPTIONS.len();
                self.pending_encoding_select = Some(offset.min(n.saturating_sub(1)));
            }
            ScrollTarget::Plugin => {
                let n = self.plugin_host.registry.instances.len();
                self.plugin_manager_cursor = offset.min(n.saturating_sub(1));
            }
        }
    }

    // ── Event loop ───────────────────────────────────────────────────────────

    fn event_loop<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> io::Result<()> {
        let mut last_tick = Instant::now();

        while self.running {
            // Ensure wrap cache is current before rendering (Feature 005).
            if self.soft_wrap {
                let w = self.content_width();
                if self
                    .wrap_cache
                    .as_ref()
                    .map(|c| c.is_stale(w, self.wrap_text_gen))
                    .unwrap_or(true)
                {
                    let rope = &self.buffers[self.active_idx].rope;
                    self.wrap_cache = Some(crate::ui::wrap::WrapCache::compute(
                        rope,
                        w,
                        self.wrap_text_gen,
                    ));
                }
            }

            terminal.draw(|frame| self.render(frame))?;
            // Feature 007: watcher_notice is one-shot — clear after one rendered frame.
            self.watcher_notice = None;

            let timeout = TICK_MS.saturating_sub(last_tick.elapsed().as_millis() as u64);

            if event::poll(Duration::from_millis(timeout))? {
                match event::read()? {
                    // Mouse events need the cursor coordinates AND live menu
                    // state to hit-test dropdown items, so they are handled in
                    // the app rather than flattened to an Action (Feature 011).
                    crossterm::event::Event::Mouse(me) => self.handle_mouse_event(me)?,
                    other => {
                        if let Some(action) = dispatch_event(other, &self.keymap) {
                            self.handle_action(action)?;
                        }
                    }
                }
            }

            if last_tick.elapsed() >= Duration::from_millis(TICK_MS) {
                self.handle_tick();
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    // ── Rendering ────────────────────────────────────────────────────────────

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let size = frame.size();
        // Keep terminal_size in sync with the actual frame so mouse hit-testing
        // (file browser, menus, cursor) uses the same geometry that is drawn.
        // Previously this was only updated on a Resize event, so on any terminal
        // that was not exactly 80x24 at startup, clicks were mapped against stale
        // geometry — e.g. a click inside the visible file-browser box read as
        // "outside" and closed the dialog (Feature 012 follow-up).
        self.terminal_size = (size.width, size.height);
        self.ensure_dialog_focus();

        // Enforce minimum terminal size
        if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
            let msg = ratatui::widgets::Paragraph::new(format!(
                "Terminal too small (min {}x{})",
                MIN_WIDTH, MIN_HEIGHT
            ))
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red));
            frame.render_widget(msg, size);
            return;
        }

        crate::ui::Ui::render(frame, self);
    }

    // ── Action dispatch ──────────────────────────────────────────────────────

    /// Build the resolved composite menu list (built-in + active plugin menus).
    /// Recomputed on demand so mid-session plugin enable/disable is reflected.
    fn resolved_menus(&self) -> Vec<ResolvedMenu> {
        resolve_menus(&self.plugin_host.registry.menu_items())
    }

    /// Open the dropdown for top-level menu `idx`, clamped against the resolved
    /// menu count.
    fn open_menu_idx(&mut self, idx: usize) {
        let menus = self.resolved_menus();
        self.menu_bar.open_menu(idx, &menus);
    }

    pub fn handle_action(&mut self, action: Action) -> io::Result<()> {
        self.ensure_dialog_focus();

        // Feature 030 (US3): the editor context menu is modal while open — Up/Down
        // move focus, Enter/Space activate the focused item (routing to the real
        // action), Esc dismisses; all other keys are consumed.
        if let Some(mut menu) = self.pending_context_menu {
            match &action {
                Action::MoveDown => {
                    menu.focus_next();
                    self.pending_context_menu = Some(menu);
                }
                Action::MoveUp => {
                    menu.focus_prev();
                    self.pending_context_menu = Some(menu);
                }
                Action::InsertNewline | Action::InsertChar(' ') => {
                    let act = crate::ui::contextmenu::ITEMS[menu.focus].1.clone();
                    self.pending_context_menu = None;
                    return self.handle_action(act);
                }
                Action::MenuClose | Action::Quit => {
                    self.pending_context_menu = None;
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 016: button focus + activation for confirm/dismiss dialogs.
        // Tab/Shift+Tab move focus; Enter/Space activate the focused button. All
        // other keys (letter shortcuts, Esc) fall through to the per-dialog guards
        // below, which still handle them.
        {
            let n = self.dialog_button_labels().len();
            if n > 0 {
                match &action {
                    // Feature 028: arrow keys move between buttons too (the button
                    // row is horizontal, so Right/Down = next, Left/Up = prev),
                    // consistent with Tab/Shift+Tab.
                    Action::FocusNextField | Action::MoveRight | Action::MoveDown => {
                        self.dialog_focus = crate::ui::buttons::next(self.dialog_focus, n);
                        return Ok(());
                    }
                    Action::FocusPrevField | Action::MoveLeft | Action::MoveUp => {
                        self.dialog_focus = crate::ui::buttons::prev(self.dialog_focus, n);
                        return Ok(());
                    }
                    Action::InsertNewline | Action::InsertChar(' ') => {
                        let idx = self.dialog_focus.min(n - 1);
                        self.activate_dialog_button(idx);
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        // Feature 020: focus-ring movement for the interactive/list dialogs.
        // Tab/Shift+Tab cycle the whole ring (primary control + buttons). All
        // other keys fall through to the per-dialog guards below, which consult
        // `dialog_focus` to decide between primary-control behavior and button
        // activation.
        {
            let ring = self.interactive_ring_len();
            if ring > 1 {
                // Feature 028: when a BUTTON is focused, arrow keys also cycle the
                // ring (consistent with Tab). When the primary control is focused,
                // arrows fall through to the list/field below (unchanged behavior).
                let on_button = self.interactive_focus_is_button().is_some();
                match &action {
                    Action::FocusNextField => {
                        self.dialog_focus = crate::ui::buttons::next(self.dialog_focus, ring);
                        self.sync_find_replace_focus();
                        return Ok(());
                    }
                    Action::FocusPrevField => {
                        self.dialog_focus = crate::ui::buttons::prev(self.dialog_focus, ring);
                        self.sync_find_replace_focus();
                        return Ok(());
                    }
                    Action::MoveRight | Action::MoveDown if on_button => {
                        self.dialog_focus = crate::ui::buttons::next(self.dialog_focus, ring);
                        self.sync_find_replace_focus();
                        return Ok(());
                    }
                    Action::MoveLeft | Action::MoveUp if on_button => {
                        self.dialog_focus = crate::ui::buttons::prev(self.dialog_focus, ring);
                        self.sync_find_replace_focus();
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        // When the session restore dialog is active, only Y/y/Enter (confirm)
        // and N/n/Escape/Quit (decline) are forwarded; everything else is
        // dropped silently so the dialog stays visible.
        if self.pending_session_restore.is_some() {
            match &action {
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'Y') => {
                    self.do_restore_session();
                    self.pending_session_restore = None;
                }
                Action::InsertNewline => {
                    self.do_restore_session();
                    self.pending_session_restore = None;
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'N') => {
                    self.pending_session_restore = None;
                }
                Action::Quit | Action::MenuClose => {
                    self.pending_session_restore = None;
                }
                _ => {}
            }
            return Ok(());
        }

        // When the save-before-quit prompt is active, only S / D / C are valid.
        // All other actions are silently dropped so the prompt stays visible.
        if self.pending_save_prompt {
            match &action {
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'S') => {
                    self.prompt_save_and_quit();
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'D') => {
                    self.prompt_discard_and_quit();
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'C') => {
                    self.prompt_cancel_quit();
                }
                // Feature 029: Esc cancels, matching the "Cancel (Esc)" label and
                // every other confirm dialog (was previously ignored).
                Action::MenuClose | Action::Quit => {
                    self.prompt_cancel_quit();
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 007 — External-change dialog intercept: only Y/Enter (reload)
        // and N/Esc (keep) are forwarded while the dialog is active.
        if self.pending_external_change.is_some() {
            match &action {
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'Y') => {
                    let ec = self.pending_external_change.take().unwrap();
                    self.reload_from_disk(ec.buf_idx);
                }
                Action::InsertNewline => {
                    let ec = self.pending_external_change.take().unwrap();
                    self.reload_from_disk(ec.buf_idx);
                }
                Action::ReloadFile => {
                    let ec = self.pending_external_change.take().unwrap();
                    self.reload_from_disk(ec.buf_idx);
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'N') => {
                    if let Some(ec) = self.pending_external_change.take() {
                        self.buffers[ec.buf_idx].modified = true;
                    }
                }
                Action::MenuClose | Action::DismissExternalChange => {
                    if let Some(ec) = self.pending_external_change.take() {
                        self.buffers[ec.buf_idx].modified = true;
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 014 — Revert confirmation intercept: Y/Enter discards changes
        // and reloads from disk; N/Esc cancels (buffer untouched).
        if let Some(buf_idx) = self.pending_revert_confirm {
            match &action {
                Action::InsertNewline => {
                    self.pending_revert_confirm = None;
                    self.reload_from_disk(buf_idx);
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'Y') => {
                    self.pending_revert_confirm = None;
                    self.reload_from_disk(buf_idx);
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'N') => {
                    self.pending_revert_confirm = None;
                }
                Action::MenuClose | Action::Quit => {
                    self.pending_revert_confirm = None;
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 027 — tab `[x]` close confirmation intercept: only S (save +
        // close), D (discard + close), C/Esc (cancel) are valid; all other input
        // is dropped so the modal stays visible. Mirrors the save-before-quit
        // prompt so the `(S)`/`(D)` label hints are accurate.
        if self.pending_close_confirm.is_some() {
            match &action {
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'S') => {
                    self.activate_dialog_button(0);
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'D') => {
                    self.activate_dialog_button(1);
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'C') => {
                    self.activate_dialog_button(2);
                }
                Action::MenuClose | Action::Quit => {
                    self.activate_dialog_button(2);
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 015 — Find/Replace dialog intercept. While open, keystrokes
        // edit the dialog fields and drive the search; the buffer is only touched
        // by an explicit Replace/Replace-All. All input is consumed.
        if self.pending_find_replace.is_some() {
            let is_replace =
                self.pending_find_replace.as_ref().unwrap().mode == DialogMode::Replace;
            // Dialog-global keys (work regardless of which stop is focused):
            // close, option toggles, and match navigation. Feature 020 keeps
            // these unchanged from feature 015.
            match &action {
                Action::MenuClose | Action::Quit => {
                    self.close_find_replace();
                    return Ok(());
                }
                Action::ToggleSearchCase => {
                    let d = self.pending_find_replace.as_mut().unwrap();
                    d.case_sensitive = !d.case_sensitive;
                    self.run_find_from_dialog();
                    return Ok(());
                }
                Action::ToggleSearchWrap => {
                    self.pending_find_replace.as_mut().unwrap().wrap ^= true;
                    return Ok(());
                }
                Action::ToggleSearchRegex => {
                    let d = self.pending_find_replace.as_mut().unwrap();
                    d.regex = !d.regex;
                    self.run_find_from_dialog();
                    return Ok(());
                }
                Action::ToggleSearchWholeWord => {
                    let d = self.pending_find_replace.as_mut().unwrap();
                    d.whole_word = !d.whole_word;
                    self.run_find_from_dialog();
                    return Ok(());
                }
                Action::FindNext => {
                    self.find_next();
                    return Ok(());
                }
                Action::FindPrev => {
                    self.find_prev();
                    return Ok(());
                }
                // Ctrl+A → Replace All (only while the Replace dialog is open).
                Action::SelectAll if is_replace => {
                    self.replace_all_from_dialog();
                    return Ok(());
                }
                _ => {}
            }
            // Button focused: Enter/Space activate it; text-editing keys are
            // ignored (they belong to the fields, which are not focused).
            if let Some(btn) = self.interactive_focus_is_button() {
                if matches!(&action, Action::InsertNewline | Action::InsertChar(' ')) {
                    self.activate_interactive_button(btn);
                }
                return Ok(());
            }
            // A field stop is focused: edit the field / run the per-mode action.
            match &action {
                Action::InsertChar(c) => {
                    self.pending_find_replace.as_mut().unwrap().insert_char(*c)
                }
                Action::Backspace => self.pending_find_replace.as_mut().unwrap().backspace(),
                Action::MoveLeft => self.pending_find_replace.as_mut().unwrap().move_left(),
                Action::MoveRight => self.pending_find_replace.as_mut().unwrap().move_right(),
                Action::InsertNewline => {
                    if is_replace {
                        self.replace_current_from_dialog();
                    } else {
                        self.run_find_from_dialog();
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 025 — Go-to-Line prompt intercept: digits edit the number,
        // Enter jumps (clamped) to the line start, Esc cancels; everything else is
        // consumed so the buffer is never modified while the prompt is open.
        if self.pending_goto_line.is_some() {
            // Feature 031: caret-aware digit editing. The value is ASCII digits, so
            // the caret index equals a byte offset.
            match &action {
                Action::InsertChar(c) if c.is_ascii_digit() => {
                    let entry = self.pending_goto_line.as_mut().unwrap();
                    let caret = self.pending_goto_line_caret.min(entry.len());
                    entry.insert(caret, *c);
                    self.pending_goto_line_caret = caret + 1;
                }
                Action::Backspace => {
                    let entry = self.pending_goto_line.as_mut().unwrap();
                    let caret = self.pending_goto_line_caret.min(entry.len());
                    if caret > 0 {
                        entry.remove(caret - 1);
                        self.pending_goto_line_caret = caret - 1;
                    }
                }
                Action::MoveLeft => {
                    self.pending_goto_line_caret = self.pending_goto_line_caret.saturating_sub(1);
                }
                Action::MoveRight => {
                    let len = self.pending_goto_line.as_ref().unwrap().len();
                    self.pending_goto_line_caret = (self.pending_goto_line_caret + 1).min(len);
                }
                Action::MoveLineStart => self.pending_goto_line_caret = 0,
                Action::MoveLineEnd => {
                    self.pending_goto_line_caret = self.pending_goto_line.as_ref().unwrap().len();
                }
                Action::InsertNewline => {
                    let entry = self.pending_goto_line.take().unwrap_or_default();
                    if let Ok(n) = entry.parse::<usize>() {
                        let count = self.buffers[self.active_idx].rope.line_count();
                        let line1 = n.clamp(1, count.max(1));
                        self.set_cursor_lc(line1 - 1, 0);
                    }
                    // Empty / non-numeric → closed with no movement.
                }
                Action::MenuClose | Action::Quit => {
                    self.pending_goto_line = None;
                }
                _ => {}
            }
            return Ok(());
        }

        // T012 — Encoding-dialog intercept: when the dialog is open, only
        // Up/Down (navigate), Enter (confirm), and Esc/MenuClose (cancel) are
        // processed; all other actions are silently consumed.
        if let Some(idx) = self.pending_encoding_select {
            let n = crate::ui::dialog::ENCODING_OPTIONS.len();
            // Esc always cancels, from any focus stop.
            if matches!(&action, Action::MenuClose) {
                self.pending_encoding_select = None;
                return Ok(());
            }
            // Button focused (OK/Cancel): Enter/Space activate; arrows no-op.
            if let Some(btn) = self.interactive_focus_is_button() {
                if matches!(&action, Action::InsertNewline | Action::InsertChar(' ')) {
                    self.activate_interactive_button(btn);
                }
                return Ok(());
            }
            // List focused: existing navigation/confirm behavior (feature 004).
            // Feature 028: PageUp/PageDown jump by a page, clamped (no wrap).
            match &action {
                Action::MoveUp => {
                    self.pending_encoding_select = Some((idx + n - 1) % n);
                }
                Action::MoveDown => {
                    self.pending_encoding_select = Some((idx + 1) % n);
                }
                Action::MovePageDown => {
                    self.pending_encoding_select = Some((idx + DIALOG_LIST_PAGE).min(n - 1));
                }
                Action::MovePageUp => {
                    self.pending_encoding_select = Some(idx.saturating_sub(DIALOG_LIST_PAGE));
                }
                Action::InsertNewline => {
                    let enc = crate::ui::dialog::ENCODING_OPTIONS[idx].0;
                    self.pending_encoding_select = None;
                    self.do_save_as_encoding(enc);
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 012 — File browser intercept (Open/Save). Arrow keys move,
        // Enter/Right activate, Left/Backspace go to parent, printable chars edit
        // the filename/path field, Esc cancels. All other actions are consumed so
        // the browser stays modal over the buffer.
        if self.file_browser.is_some() {
            // Esc always cancels, from any focus stop.
            if matches!(&action, Action::MenuClose) {
                self.file_browser = None;
                return Ok(());
            }
            // Button focused (Open|Save / Cancel): Enter/Space activate; other
            // navigation/edit keys no-op so they don't mutate the listing.
            if let Some(btn) = self.interactive_focus_is_button() {
                if matches!(&action, Action::InsertNewline | Action::InsertChar(' ')) {
                    self.activate_interactive_button(btn);
                }
                return Ok(());
            }
            // Browser focused: existing navigation/edit behavior (feature 012).
            let vis = {
                let (w, h) = self.terminal_size;
                self.file_browser
                    .as_ref()
                    .unwrap()
                    .visible_rows(ratatui::layout::Rect::new(0, 0, w, h))
            };
            let mut outcome: Option<BrowseOutcome> = None;
            {
                let fb = self.file_browser.as_mut().unwrap();
                // Feature 031: while a filename is being typed, Left/Right/Home/End
                // edit the field caret; with an empty field they keep the list
                // navigation semantics (← parent, → activate).
                let editing = !fb.filename.is_empty();
                match &action {
                    Action::MoveUp => fb.move_up(vis),
                    Action::MoveDown => fb.move_down(vis),
                    Action::MovePageUp => fb.page_up(vis),
                    Action::MovePageDown => fb.page_down(vis),
                    Action::MoveLeft if editing => fb.caret_left(),
                    Action::MoveRight if editing => fb.caret_right(),
                    Action::MoveLineStart if editing => fb.caret_home(),
                    Action::MoveLineEnd if editing => fb.caret_end(),
                    Action::MoveLeft => fb.enter_parent(),
                    Action::MoveRight | Action::InsertNewline => outcome = Some(fb.activate()),
                    Action::Backspace => fb.backspace(),
                    Action::InsertChar(c) => fb.push_char(*c),
                    _ => {}
                }
            }
            if let Some(outcome) = outcome {
                self.apply_browse_outcome(outcome);
            }
            return Ok(());
        }

        // Feature 011/018 — Help / About overlay: arrows/PageUp-Down scroll the
        // cheat sheet; Esc/Enter/Quit close; other input is consumed (modal).
        // Feature 028: Home/End jump to top/bottom and every scroll is clamped to
        // the content so keyboard scrolling stays in range.
        if let Some(screen) = self.pending_help {
            let (max_scroll, page) = self.help_view_metrics(screen);
            match &action {
                Action::MoveDown => {
                    self.help_scroll = (self.help_scroll + 1).min(max_scroll);
                }
                Action::MoveUp => self.help_scroll = self.help_scroll.saturating_sub(1),
                Action::MovePageDown => {
                    self.help_scroll = (self.help_scroll + page).min(max_scroll);
                }
                Action::MovePageUp => self.help_scroll = self.help_scroll.saturating_sub(page),
                Action::MoveLineStart => self.help_scroll = 0,
                Action::MoveLineEnd => self.help_scroll = max_scroll,
                Action::MenuClose | Action::InsertNewline | Action::Quit => {
                    self.pending_help = None;
                }
                // A printable key also dismisses (legacy behavior), except none
                // that we use for scrolling above.
                Action::InsertChar(_) => self.pending_help = None,
                _ => {}
            }
            return Ok(());
        }

        // Feature 008 — Plugin consent dialog intercept: Enter/Y = allow, Esc/N = deny.
        if !self.pending_plugin_consent.is_empty() {
            match &action {
                Action::InsertNewline => self.consent_decide(true),
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'Y') => {
                    self.consent_decide(true)
                }
                Action::InsertChar(c) if matches!(c.to_ascii_uppercase(), 'N') => {
                    self.consent_decide(false)
                }
                Action::MenuClose | Action::Quit => self.consent_decide(false),
                _ => {}
            }
            return Ok(());
        }

        // Feature 008 — Plugin manager dialog intercept: Up/Down navigate,
        // Space/Enter toggle enabled, Esc closes.
        if self.pending_plugin_manager {
            let n = self.plugin_host.registry.instances.len();
            // Esc/Quit always close, from any focus stop.
            if matches!(&action, Action::MenuClose | Action::Quit) {
                self.pending_plugin_manager = false;
                return Ok(());
            }
            // Button focused (Close): Enter/Space activate; arrows no-op.
            if let Some(btn) = self.interactive_focus_is_button() {
                if matches!(&action, Action::InsertNewline | Action::InsertChar(' ')) {
                    self.activate_interactive_button(btn);
                }
                return Ok(());
            }
            // List focused: existing navigation/toggle behavior (feature 008).
            // Feature 028: PageUp/PageDown jump by a page, clamped (no wrap).
            match &action {
                Action::MoveUp if n > 0 => {
                    self.plugin_manager_cursor = (self.plugin_manager_cursor + n - 1) % n;
                }
                Action::MoveDown if n > 0 => {
                    self.plugin_manager_cursor = (self.plugin_manager_cursor + 1) % n;
                }
                Action::MovePageDown if n > 0 => {
                    self.plugin_manager_cursor =
                        (self.plugin_manager_cursor + DIALOG_LIST_PAGE).min(n - 1);
                }
                Action::MovePageUp if n > 0 => {
                    self.plugin_manager_cursor =
                        self.plugin_manager_cursor.saturating_sub(DIALOG_LIST_PAGE);
                }
                Action::InsertChar(' ') | Action::InsertNewline => {
                    self.plugin_manager_toggle_current();
                }
                _ => {}
            }
            return Ok(());
        }

        // Feature 009 — Menu-bar navigation intercept. Placed AFTER all modal
        // dialog guards (modals win — FR-012) and BEFORE the normal action match.
        // Routes navigation/selection keys to the menu state machine over the
        // resolved (built-in + plugin) menu list, and consumes all other actions
        // so navigation never mutates the buffer (FR-006).
        if self.menu_bar.is_active() {
            // Ctrl+Q must still quit while a menu is open: close the menu and
            // fall through to the normal quit handling in the main match below.
            if matches!(action, Action::Quit) {
                self.menu_bar.close_menu();
            } else {
                let menus = self.resolved_menus();
                match action {
                    Action::MoveUp => self.menu_bar.navigate_up(&menus),
                    Action::MoveDown => self.menu_bar.navigate_down(&menus),
                    Action::MoveLeft => self.menu_bar.navigate_left(&menus),
                    Action::MoveRight => self.menu_bar.navigate_right(&menus),
                    Action::MenuClose => self.menu_bar.close_menu(),
                    Action::InsertNewline => {
                        if let Some(selected) = self.menu_bar.select_item(&menus) {
                            return self.handle_action(selected);
                        }
                    }
                    // Switching/opening menus while the bar is active.
                    Action::MenuFile => self.menu_bar.open_menu(0, &menus),
                    Action::MenuEdit => self.menu_bar.open_menu(1, &menus),
                    Action::MenuSearch => self.menu_bar.open_menu(2, &menus),
                    Action::MenuView => self.menu_bar.open_menu(3, &menus),
                    Action::MenuOptions => self.menu_bar.open_menu(4, &menus),
                    Action::MenuHelp => self.menu_bar.open_menu(5, &menus),
                    Action::MenuOpen(idx) => self.menu_bar.open_menu(idx, &menus),
                    Action::Menu => self.menu_bar.activate_bar(),
                    // Feature 013: mnemonic accelerator typed while the bar is active.
                    // In a dropdown, a matching letter activates the item (like Enter)
                    // and a non-match is an inert no-op (does NOT jump to another menu).
                    // At the top level, a matching letter opens that menu. Letters never
                    // edit the buffer while the bar is active (FR-004/FR-007).
                    Action::InsertChar(c) => match self.menu_bar.state {
                        MenuState::DropDown { .. } => {
                            if let Some(selected) = self.menu_bar.select_item_by_mnemonic(&menus, c)
                            {
                                return self.handle_action(selected);
                            }
                        }
                        MenuState::TopActive(_) => {
                            self.menu_bar.open_menu_by_mnemonic(&menus, c);
                        }
                        MenuState::Inactive => {}
                    },
                    // Everything else is consumed (no buffer mutation) while open.
                    _ => {}
                }
                return Ok(());
            }
        }

        match action {
            Action::Quit => self.handle_quit(),
            Action::Resize(w, h) => self.handle_resize(w, h),
            Action::Tick => self.handle_tick(),

            // Cursor movement — T025
            Action::MoveUp => self.move_cursor(Direction::Up),
            Action::MoveDown => self.move_cursor(Direction::Down),
            Action::MoveLeft => self.move_cursor(Direction::Left),
            Action::MoveRight => self.move_cursor(Direction::Right),
            Action::MoveLineStart => self.move_line_start(),
            Action::MoveLineEnd => self.move_line_end(),
            Action::MovePageUp => self.move_page_up(),
            Action::MovePageDown => self.move_page_down(),
            Action::MoveDocStart => self.move_doc_start(),
            Action::MoveDocEnd => self.move_doc_end(),

            // Feature 017: Shift+navigation — extend the selection while moving.
            Action::SelectLeft => self.move_cursor_selecting(Direction::Left),
            Action::SelectRight => self.move_cursor_selecting(Direction::Right),
            Action::SelectUp => self.move_cursor_selecting(Direction::Up),
            Action::SelectDown => self.move_cursor_selecting(Direction::Down),
            Action::SelectLineStart => self.select_line_start(),
            Action::SelectLineEnd => self.select_line_end(),

            // Text insertion — T026
            Action::InsertChar(c) => self.insert_char(c),
            Action::InsertNewline => self.insert_newline(),

            // Deletion — T027
            Action::Backspace => self.delete_backward(),
            Action::Delete => self.delete_forward(),

            // File browser (Feature 012). Open/Save As show the navigable browser.
            Action::Open => {
                self.file_browser = Some(FileBrowser::open(
                    self.browser_start_dir(),
                    BrowseMode::Open,
                ));
            }
            Action::SaveAs => {
                self.file_browser = Some(FileBrowser::open(
                    self.browser_start_dir(),
                    BrowseMode::Save,
                ));
            }

            // File operations (Feature 011).
            Action::New => self.new_buffer(),
            Action::Close => self.close_active_buffer(),
            Action::Revert => self.handle_revert(),

            // Edit operations (Feature 011) — previously unhandled, so both the
            // menu items and the Ctrl+Z/Y/X/C/V/A shortcuts were dead.
            Action::Undo => self.handle_undo(),
            Action::Redo => self.handle_redo(),
            Action::Cut => self.cut_selection(),
            Action::Copy => self.copy_selection(),
            Action::Paste => self.paste_clipboard(),
            Action::SelectAll => self.select_all(),

            // View toggle (Feature 011).
            Action::ToggleLineNumbers => {
                self.config.line_numbers = !self.config.line_numbers;
            }

            // Help menu (Feature 011).
            Action::Help => {
                self.help_scroll = 0;
                self.pending_help = Some(HelpScreen::Help);
            }
            Action::About => {
                self.help_scroll = 0;
                self.pending_help = Some(HelpScreen::About);
            }

            // Save prompt responses (T033)
            Action::Save => self.handle_save_action(),

            // T013 — Save As Encoding dialog trigger
            Action::SaveAsEncoding => {
                if !self.buffers.is_empty() {
                    let idx = Self::encoding_to_idx(self.buffers[self.active_idx].encoding);
                    self.pending_encoding_select = Some(idx);
                }
            }

            // Search and replace (Feature 015 — interactive dialogs)
            Action::Find => self.open_find_dialog(),
            Action::FindNext => self.find_next(),
            Action::FindPrev => self.find_prev(),
            Action::FindReplace => self.open_replace_dialog(),
            // Feature 025: open the Go-to-Line prompt (only when no other modal is
            // already open; the intercept above handles it once open).
            Action::GoToLine => {
                if self.open_button_dialog().is_none()
                    && self.interactive_dialog().is_none()
                    && self.pending_help.is_none()
                    && self.pending_goto_line.is_none()
                    && !self.menu_bar.is_active()
                // Feature 029: don't open over a menu
                {
                    self.pending_goto_line = Some(String::new());
                    self.pending_goto_line_caret = 0; // Feature 031
                }
            }
            // Search-option toggles and Tab focus are only meaningful inside an
            // open Find/Replace dialog (handled by the intercept above); inert here.
            Action::ToggleSearchCase
            | Action::ToggleSearchWrap
            | Action::ToggleSearchRegex
            | Action::ToggleSearchWholeWord
            | Action::FocusNextField
            | Action::FocusPrevField => {}

            // Menu navigation (T048 / Feature 009). Alt+<letter> opens a
            // dropdown directly; F10 (`Menu`) enters the top-level highlight
            // (no dropdown) — the DOS-faithful entry path (FR-015).
            Action::MenuFile => self.open_menu_idx(0),
            Action::MenuEdit => self.open_menu_idx(1),
            Action::MenuSearch => self.open_menu_idx(2),
            Action::MenuView => self.open_menu_idx(3),
            Action::MenuOptions => self.open_menu_idx(4),
            Action::MenuHelp => self.open_menu_idx(5),
            Action::MenuClose => self.menu_bar.close_menu(),
            Action::Menu => self.menu_bar.activate_bar(),
            Action::MenuOpen(idx) => self.open_menu_idx(idx),

            // Multi-buffer navigation (T066)
            Action::NextBuffer => self.next_buffer(),
            Action::PrevBuffer => self.prev_buffer(),

            // Split-view toggle (T067)
            Action::SplitView => {
                self.split_mode = match self.split_mode {
                    crate::ui::SplitMode::Single => crate::ui::SplitMode::Vertical,
                    crate::ui::SplitMode::Vertical => crate::ui::SplitMode::Single,
                };
            }

            // Syntax-highlight toggle (T077)
            Action::ToggleHighlight => self.toggle_highlight(),

            // Theme cycle (T081)
            Action::ToggleTheme => {
                let new_name = match self.theme.name {
                    "classic" => "high-contrast",
                    "high-contrast" => "plain",
                    _ => "classic",
                };
                self.set_theme(new_name);
            }

            // Soft-wrap toggle (Feature 005)
            Action::ToggleSoftWrap => self.handle_toggle_soft_wrap()?,

            // Feature 008 — Plugin API
            Action::OpenPluginManager => {
                self.pending_plugin_manager = true;
                self.plugin_manager_cursor = 0;
            }
            Action::PluginMenuActivated(plugin_id, item_id) => {
                let content = self.active_buffer().rope.to_string();
                if let Some(msg) = self
                    .plugin_host
                    .dispatch_menu_action(&plugin_id, &item_id, &content)
                {
                    self.status_message = Some(msg);
                }
            }

            _ => {
                log::debug!("Unhandled action: {:?}", action);
            }
        }
        Ok(())
    }

    // ── T033 — Quit flow ─────────────────────────────────────────────────────

    fn handle_quit(&mut self) {
        if self.active_buffer().modified {
            self.pending_save_prompt = true;
            // The actual dialog rendering is handled by the UI layer; here we
            // just gate the quit so the render loop can show the prompt.
            log::debug!("Buffer modified — showing save prompt before quit");
        } else {
            if let Some(data) = self.build_session_data() {
                if let Err(e) = crate::session::save_session(&data) {
                    log::warn!("session save failed: {}", e);
                }
            }
            self.running = false;
        }
    }

    /// Called when the user chooses [S]ave in the save-before-quit prompt.
    pub fn prompt_save_and_quit(&mut self) {
        match self.active_buffer().save() {
            Ok(()) => {
                // Feature 007: record write time for self-write suppression (FR-007).
                if let Some(path) = self.buffers[self.active_idx].path.clone() {
                    self.self_write_times.insert(path, Instant::now());
                }
                self.pending_save_prompt = false;
                if let Some(data) = self.build_session_data() {
                    if let Err(e) = crate::session::save_session(&data) {
                        log::warn!("session save failed: {}", e);
                    }
                }
                self.running = false;
            }
            Err(e) => {
                log::error!("Save failed: {}", e);
                // Keep the prompt open so the user can decide what to do.
            }
        }
    }

    /// Called when the user chooses [D]iscard in the save-before-quit prompt.
    pub fn prompt_discard_and_quit(&mut self) {
        self.pending_save_prompt = false;
        if let Some(data) = self.build_session_data() {
            if let Err(e) = crate::session::save_session(&data) {
                log::warn!("session save failed: {}", e);
            }
        }
        self.running = false;
    }

    /// Called when the user chooses [C]ancel in the save-before-quit prompt.
    pub fn prompt_cancel_quit(&mut self) {
        self.pending_save_prompt = false;
    }

    // ── Session save/restore ─────────────────────────────────────────────────

    /// Snapshot the current editor state into a [`SessionData`] for saving.
    ///
    /// Returns `None` when there are no saveable buffers (i.e. every open
    /// buffer is an untitled new-file stub with no path on disk).
    pub fn build_session_data(&self) -> Option<crate::session::SessionData> {
        use crate::session::{BufferEntry, SessionData, SplitLayoutKind};

        let buffers: Vec<BufferEntry> = self
            .buffers
            .iter()
            .filter_map(|buf| {
                let path = buf.path.as_ref()?;
                if !path.exists() {
                    return None;
                }
                Some(BufferEntry {
                    path: path.to_string_lossy().into_owned(),
                    cursor_line: (buf.cursor.line + 1) as u32,
                    cursor_col: (buf.cursor.grapheme_col + 1) as u32,
                })
            })
            .collect();

        if buffers.is_empty() {
            return None;
        }

        let split_layout = match self.split_mode {
            crate::ui::SplitMode::Single => SplitLayoutKind::None,
            crate::ui::SplitMode::Vertical => SplitLayoutKind::Vertical,
        };

        // active_pane: 0 for single or left pane, 1 for right pane in a split.
        let active_pane = match self.split_mode {
            crate::ui::SplitMode::Single => 0,
            crate::ui::SplitMode::Vertical => {
                if self.active_idx > 0 {
                    1
                } else {
                    0
                }
            }
        };

        Some(SessionData {
            version: 1,
            active_buffer: self.active_idx,
            split_layout,
            active_pane,
            buffers,
        })
    }

    /// Restore a previously saved session: open each recorded buffer, seek
    /// cursors, and apply the saved split layout.
    ///
    /// Called after the user confirms the session restore dialog. T020–T022
    /// (path validation, missing-file handling) are layered on top here.
    pub fn do_restore_session(&mut self) {
        use crate::security::sanitize::{validate_path, PathError};
        use crate::session::SplitLayoutKind;

        let session = match self.pending_session_restore.take() {
            Some(s) => s,
            None => return,
        };

        let mut new_buffers: Vec<Buffer> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        for entry in &session.buffers {
            let raw_path = std::path::Path::new(&entry.path);

            // T020: path traversal check.
            let open_path = match validate_path(raw_path) {
                Ok(canonical) => canonical,
                Err(PathError::Traversal) => {
                    log::warn!("session: path traversal rejected: {:?}", raw_path);
                    warnings.push(format!("session: path rejected: {}", raw_path.display()));
                    continue;
                }
                Err(PathError::Io(_)) => {
                    // Non-existent or unreadable — fall through to Buffer::open
                    // which will produce an appropriate error (T021).
                    raw_path.to_path_buf()
                }
            };

            // T021: attempt to open the buffer.
            match Buffer::open(open_path.clone(), self.default_encoding) {
                Ok(mut buf) => {
                    // Seek cursor to saved position (convert 1-based → 0-based).
                    let target_line = (entry.cursor_line as usize).saturating_sub(1);
                    let target_gcol = (entry.cursor_col as usize).saturating_sub(1);
                    let line_count = buf.rope.line_count();
                    let clamped_line = target_line.min(line_count.saturating_sub(1));
                    let max_gcol = buf.rope.grapheme_count_on_line(clamped_line);
                    let clamped_gcol = target_gcol.min(max_gcol);
                    let vcol = crate::buffer::CursorPos::visual_col_from_grapheme_col(
                        &buf.rope,
                        clamped_line,
                        clamped_gcol,
                    );
                    buf.cursor = crate::buffer::CursorPos {
                        line: clamped_line,
                        grapheme_col: clamped_gcol,
                        visual_col: vcol,
                    };
                    // Apply syntax highlighting if configured (plugin highlighter wins).
                    if self.config.highlight {
                        if let Some(ref path) = buf.path.clone() {
                            buf.syntax = self
                                .plugin_host
                                .highlighter_for(path, self.theme)
                                .or_else(|| crate::highlight::detect_highlighter(path));
                        }
                    }
                    new_buffers.push(buf);
                }
                Err(_) => {
                    let display = open_path.display().to_string();
                    log::warn!("session: {} not found or unreadable", display);
                    warnings.push(format!("session: {} not found", display));
                }
            }
        }

        // T022: handle all-failed case.
        if new_buffers.is_empty() {
            // Keep the existing blank buffer; show an error message.
            self.status_message = Some("session: no files could be restored".to_string());
            return;
        }

        // Replace buffers with restored set. Feature 028: the restored content has
        // nothing to do with the old buffer the wrap cache was built for, so it MUST
        // be invalidated here or the soft-wrap renderer slices stale offsets → panic.
        self.buffers = new_buffers;
        self.invalidate_wrap_cache();

        // Feature 007: register watches for newly-restored buffer paths.
        if let Some(ref mut fw) = self.file_watcher {
            for buf in &self.buffers {
                if let Some(ref p) = buf.path {
                    if let Err(e) = fw.watch_path(p) {
                        log::warn!("FileWatcher: could not watch {:?}: {}", p, e);
                    }
                }
            }
        }

        // Restore split layout.
        self.split_mode = match session.split_layout {
            SplitLayoutKind::None => crate::ui::SplitMode::Single,
            SplitLayoutKind::Vertical | SplitLayoutKind::Horizontal => {
                crate::ui::SplitMode::Vertical
            }
        };

        // Clamp active_idx to avoid out-of-bounds (I1).
        self.active_idx = session
            .active_buffer
            .min(self.buffers.len().saturating_sub(1));

        // Show first warning in the status bar; log the rest.
        if let Some(first) = warnings.first() {
            self.status_message = Some(first.clone());
        }
        for w in warnings.iter().skip(1) {
            log::warn!("{}", w);
        }
    }

    /// Handle an explicit Save action (Ctrl+S / F5).
    pub fn handle_save_action(&mut self) {
        // Feature 012: an unnamed buffer has no path to save to — open the Save
        // browser so the user can choose a destination.
        if self.active_buffer().path.is_none() {
            self.file_browser = Some(FileBrowser::open(
                self.browser_start_dir(),
                BrowseMode::Save,
            ));
            return;
        }
        match self.active_buffer().save() {
            Ok(()) => {
                self.active_buffer_mut().modified = false;
                // Feature 014: the just-written content is the new clean baseline.
                self.active_buffer_mut().undo_stack.mark_saved();
                // Feature 007: record write time for self-write suppression (FR-007).
                if let Some(path) = self.buffers[self.active_idx].path.clone() {
                    self.self_write_times.insert(path, Instant::now());
                }
                // Feature 029: confirm the save (was silent; Save-As already does).
                let name = self.buffers[self.active_idx]
                    .path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "file".to_string());
                self.status_message = Some(format!("Saved {name}"));
                log::info!("Buffer saved");
            }
            Err(e) => {
                // Feature 029: surface the failure (was silent — a failed save
                // looked identical to a successful one). Buffer stays modified.
                log::error!("Save failed: {}", e);
                self.status_message = Some(format!("Save failed: {e}"));
            }
        }
    }

    // ── T014 — do_save_as_encoding ────────────────────────────────────────────

    /// Write the active buffer to disk in `enc`.
    ///
    /// Case A (named buffer): sets `buf.encoding = enc`, calls `buf.save()`;
    /// on success updates the status bar; on failure reverts `buf.encoding`.
    ///
    /// Case B (unnamed buffer): handled in T020.
    pub fn do_save_as_encoding(&mut self, enc: EncodingId) {
        if self.buffers[self.active_idx].path.is_some() {
            // Case A: named buffer — encode + atomic write.
            let old_enc = self.buffers[self.active_idx].encoding;
            self.buffers[self.active_idx].encoding = enc;
            match self.buffers[self.active_idx].save() {
                Ok(()) => {
                    self.buffers[self.active_idx].modified = false;
                    self.buffers[self.active_idx].undo_stack.mark_saved(); // Feature 014
                                                                           // Feature 007: record write time for self-write suppression.
                    if let Some(path) = self.buffers[self.active_idx].path.clone() {
                        self.self_write_times.insert(path, Instant::now());
                    }
                    let label = Self::label_for_encoding(enc);
                    self.status_message = Some(format!("Saved as {}", label));
                }
                Err(e) => {
                    self.buffers[self.active_idx].encoding = old_enc;
                    self.status_message = Some(format!("Save failed: {}", e));
                }
            }
        } else {
            // Case B — unnamed buffer: store the chosen encoding so that the
            // next handle_save_as call (once the user provides a filename) will
            // write the file in the selected encoding.
            self.pending_save_as_encoding = Some(enc);
        }
    }

    fn handle_resize(&mut self, w: u16, h: u16) {
        self.terminal_size = (w, h);

        // T105 — detect too-small terminal
        self.too_small = w < MIN_WIDTH || h < MIN_HEIGHT;

        // Rebuild wrap cache for new terminal width (Feature 005, T022).
        if self.soft_wrap {
            let content_w = self.content_width();
            let rope = &self.buffers[self.active_idx].rope;
            self.wrap_cache = Some(crate::ui::wrap::WrapCache::compute(
                rope,
                content_w,
                self.wrap_text_gen,
            ));
            if let Some(ref cache) = self.wrap_cache {
                let total_vr = cache.total_visual_rows();
                let buf = &mut self.buffers[self.active_idx];
                if buf.scroll_offset.0 >= total_vr {
                    buf.scroll_offset.0 = total_vr.saturating_sub(1);
                }
            }
        }

        // Re-clamp scroll offset so cursor stays visible after resize.
        self.clamp_scroll();
    }

    fn handle_tick(&mut self) {
        // US5 — Autosave
        let interval = std::env::var("EDIT_AUTOSAVE_INTERVAL")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(self.config.autosave_interval)
            .clamp(1, 300);

        if !self.config.no_autosave {
            let buf = &self.buffers[self.active_idx];
            if buf.autosave.enabled && buf.modified {
                let elapsed = buf.autosave.last_save_at.elapsed().as_secs() as u32;
                if elapsed >= interval {
                    let ok = crate::buffer::autosave::write_recovery_for_buffer(
                        &mut self.buffers[self.active_idx],
                        interval,
                    );
                    // Feature 029: surface a recovery-write failure instead of
                    // losing crash protection silently.
                    if !ok {
                        self.status_message =
                            Some("Autosave failed — crash recovery may be unavailable".to_string());
                    }
                }
            }
        }

        // Feature 007 — drain the file-watcher event queue (non-blocking).
        // Only drain when no dialog is already pending (one prompt at a time).
        if self.pending_external_change.is_none() {
            if let Some(ref mut fw) = self.file_watcher {
                let watched_paths: Vec<PathBuf> =
                    self.buffers.iter().filter_map(|b| b.path.clone()).collect();
                if let Some(event) = fw.try_recv_event(&self.self_write_times, &watched_paths) {
                    match event.kind {
                        crate::watcher::WatchEventKind::Modified => {
                            // Find the buffer index whose path matches.
                            if let Some(buf_idx) = self
                                .buffers
                                .iter()
                                .position(|b| b.path.as_deref() == Some(event.path.as_path()))
                            {
                                self.pending_external_change =
                                    Some(crate::watcher::ExternalChange {
                                        buf_idx,
                                        path: event.path.clone(),
                                        kind: crate::watcher::WatchEventKind::Modified,
                                    });
                            }
                        }
                        crate::watcher::WatchEventKind::Deleted => {
                            let name = event
                                .path
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| event.path.display().to_string());
                            self.watcher_notice = Some(format!(
                                "[{}] File deleted on disk \u{2014} buffer kept in memory",
                                name
                            ));
                        }
                    }
                }
            }
        }
    }

    // ── Feature 007 — Reload from disk ───────────────────────────────────────

    /// Replace buffer at `buf_idx` with the current on-disk content.
    ///
    /// Uses `Buffer::open()` which runs the full encoding detection pipeline
    /// (FR-004 compliance: no raw-byte bypass).  Clears undo history.
    pub fn reload_from_disk(&mut self, buf_idx: usize) {
        let path = match self.buffers.get(buf_idx).and_then(|b| b.path.clone()) {
            Some(p) => p,
            None => return,
        };
        let enc = self.buffers[buf_idx].encoding;
        match Buffer::open(path.clone(), enc) {
            Ok(new_buf) => {
                self.buffers[buf_idx] = new_buf;
                log::info!("Buffer {} reloaded from {:?}", buf_idx, path);
            }
            Err(e) => {
                log::warn!("reload_from_disk failed for {:?}: {}", path, e);
                self.watcher_notice = Some(format!("Reload failed: {}", e));
            }
        }
    }

    // ── T025 — Cursor movement ────────────────────────────────────────────────

    /// Move the cursor one step in `dir`, clamping to valid positions and
    /// updating `scroll_offset` as necessary.
    /// Compute the cursor position one step in `dir` (no mutation, no selection).
    fn next_cursor_pos(&self, dir: Direction) -> (usize, usize) {
        let buf = &self.buffers[self.active_idx];
        let line_count = buf.rope.line_count();
        let cur = buf.cursor;
        match dir {
            Direction::Up => {
                if cur.line == 0 {
                    (0, cur.grapheme_col)
                } else {
                    let target_line = cur.line - 1;
                    let max_gcol = buf.rope.grapheme_count_on_line(target_line);
                    (target_line, cur.grapheme_col.min(max_gcol))
                }
            }
            Direction::Down => {
                if cur.line + 1 >= line_count {
                    (cur.line, cur.grapheme_col)
                } else {
                    let target_line = cur.line + 1;
                    let max_gcol = buf.rope.grapheme_count_on_line(target_line);
                    (target_line, cur.grapheme_col.min(max_gcol))
                }
            }
            Direction::Left => {
                if cur.grapheme_col > 0 {
                    (cur.line, cur.grapheme_col - 1)
                } else if cur.line > 0 {
                    // Wrap to end of previous line
                    let prev = cur.line - 1;
                    let prev_len = buf.rope.grapheme_count_on_line(prev);
                    (prev, prev_len)
                } else {
                    (0, 0)
                }
            }
            Direction::Right => {
                let line_len = buf.rope.grapheme_count_on_line(cur.line);
                if cur.grapheme_col < line_len {
                    (cur.line, cur.grapheme_col + 1)
                } else if cur.line + 1 < line_count {
                    // Wrap to start of next line
                    (cur.line + 1, 0)
                } else {
                    (cur.line, cur.grapheme_col)
                }
            }
        }
    }

    /// Set the cursor to `(line, gcol)` (computing its visual column) and clamp
    /// the viewport. Does not touch the selection.
    fn set_cursor_lc(&mut self, line: usize, gcol: usize) {
        let vcol = CursorPos::visual_col_from_grapheme_col(
            &self.buffers[self.active_idx].rope,
            line,
            gcol,
        );
        self.buffers[self.active_idx].cursor = CursorPos {
            line,
            grapheme_col: gcol,
            visual_col: vcol,
        };
        self.clamp_scroll();
    }

    pub fn move_cursor(&mut self, dir: Direction) {
        let (new_line, new_gcol) = self.next_cursor_pos(dir);
        // Feature 017: a plain (non-shift) move clears any selection.
        self.buffers[self.active_idx].selection = None;
        self.set_cursor_lc(new_line, new_gcol);
    }

    /// Feature 017: move the cursor one step in `dir` while extending the
    /// selection from its anchor (Shift+Arrow).
    pub fn move_cursor_selecting(&mut self, dir: Direction) {
        let anchor = self.selection_anchor_or_cursor();
        let (new_line, new_gcol) = self.next_cursor_pos(dir);
        self.set_cursor_lc(new_line, new_gcol);
        self.update_selection_to_cursor(anchor);
    }

    /// The current selection's anchor, or the cursor position if there is none.
    fn selection_anchor_or_cursor(&self) -> CursorPos {
        let buf = &self.buffers[self.active_idx];
        buf.selection.map(|s| s.anchor).unwrap_or(buf.cursor)
    }

    /// Set `selection` to span `anchor`→cursor, or `None` if empty (Feature 017).
    fn update_selection_to_cursor(&mut self, anchor: CursorPos) {
        let buf = &mut self.buffers[self.active_idx];
        let active = buf.cursor;
        buf.selection = if active.line == anchor.line && active.grapheme_col == anchor.grapheme_col
        {
            None
        } else {
            Some(Selection { anchor, active })
        };
    }

    /// Move the cursor to column 0 of the current line.
    pub fn move_line_start(&mut self) {
        self.buffers[self.active_idx].selection = None; // Feature 017: plain move clears
        let buf = &mut self.buffers[self.active_idx];
        buf.cursor.grapheme_col = 0;
        buf.cursor.visual_col = 0;
        self.clamp_scroll();
    }

    /// Move the cursor to the last grapheme of the current line.
    pub fn move_line_end(&mut self) {
        self.buffers[self.active_idx].selection = None; // Feature 017: plain move clears
        self.cursor_to_line_end();
    }

    /// Place the cursor at the end of its line (no selection change).
    fn cursor_to_line_end(&mut self) {
        let line = self.buffers[self.active_idx].cursor.line;
        let gcol = self.buffers[self.active_idx]
            .rope
            .grapheme_count_on_line(line);
        self.set_cursor_lc(line, gcol);
    }

    /// Feature 017: extend the selection to the start of the current line.
    pub fn select_line_start(&mut self) {
        let anchor = self.selection_anchor_or_cursor();
        self.set_cursor_lc(self.buffers[self.active_idx].cursor.line, 0);
        self.update_selection_to_cursor(anchor);
    }

    /// Feature 017: extend the selection to the end of the current line.
    pub fn select_line_end(&mut self) {
        let anchor = self.selection_anchor_or_cursor();
        self.cursor_to_line_end();
        self.update_selection_to_cursor(anchor);
    }

    /// Move the cursor up by one viewport page.
    pub fn move_page_up(&mut self) {
        let vh = self.viewport_height();
        let buf = &mut self.buffers[self.active_idx];
        let target_line = buf.cursor.line.saturating_sub(vh);
        let max_gcol = buf.rope.grapheme_count_on_line(target_line);
        let new_gcol = buf.cursor.grapheme_col.min(max_gcol);
        let new_vcol = CursorPos::visual_col_from_grapheme_col(&buf.rope, target_line, new_gcol);
        buf.cursor = CursorPos {
            line: target_line,
            grapheme_col: new_gcol,
            visual_col: new_vcol,
        };
        // Scroll up by the same amount
        buf.scroll_offset.0 = buf.scroll_offset.0.saturating_sub(vh);
        self.clamp_scroll();
    }

    /// Move the cursor down by one viewport page.
    pub fn move_page_down(&mut self) {
        let vh = self.viewport_height();
        let buf = &mut self.buffers[self.active_idx];
        let line_count = buf.rope.line_count();
        let target_line = (buf.cursor.line + vh).min(line_count.saturating_sub(1));
        let max_gcol = buf.rope.grapheme_count_on_line(target_line);
        let new_gcol = buf.cursor.grapheme_col.min(max_gcol);
        let new_vcol = CursorPos::visual_col_from_grapheme_col(&buf.rope, target_line, new_gcol);
        buf.cursor = CursorPos {
            line: target_line,
            grapheme_col: new_gcol,
            visual_col: new_vcol,
        };
        self.clamp_scroll();
    }

    /// Move cursor to the very first character of the document.
    pub fn move_doc_start(&mut self) {
        let buf = &mut self.buffers[self.active_idx];
        buf.cursor = CursorPos::default();
        buf.scroll_offset = (0, 0);
    }

    /// Move cursor to the very last line of the document.
    pub fn move_doc_end(&mut self) {
        let buf = &mut self.buffers[self.active_idx];
        let last_line = buf.rope.line_count().saturating_sub(1);
        let gcol = buf.rope.grapheme_count_on_line(last_line);
        let vcol = CursorPos::visual_col_from_grapheme_col(&buf.rope, last_line, gcol);
        buf.cursor = CursorPos {
            line: last_line,
            grapheme_col: gcol,
            visual_col: vcol,
        };
        self.clamp_scroll();
    }

    // ── Feature 030 — multi-click selection (US2 / #54) ───────────────────────

    /// Classify the current editor left-press as single (1), double (2), or triple
    /// (3) based on the previous press's time and cell. A press within
    /// [`DOUBLE_CLICK_MS`] of the previous one on the same cell increments the
    /// count (wrapping 3 → 1); otherwise it resets to 1.
    fn next_editor_click_count(&mut self, col: u16, row: u16) -> u8 {
        let now = Instant::now();
        let count = match self.last_editor_click {
            Some((pc, pr, n, t))
                if pc == col
                    && pr == row
                    && now.duration_since(t) <= Duration::from_millis(DOUBLE_CLICK_MS) =>
            {
                if n >= 3 {
                    1
                } else {
                    n + 1
                }
            }
            _ => 1,
        };
        self.last_editor_click = Some((col, row, count, now));
        count
    }

    /// Classify a grapheme for word-selection: word characters (alphanumeric or
    /// `_`), whitespace, or other (punctuation/symbols). Double-click selects a
    /// maximal run of the same class.
    fn grapheme_class(g: &str) -> u8 {
        match g.chars().next() {
            Some(c) if c.is_alphanumeric() || c == '_' => 0, // word
            Some(c) if c.is_whitespace() => 1,               // space
            _ => 2,                                          // other
        }
    }

    /// Select the word (run of same-class graphemes) under the cursor (US2).
    fn select_word_at_cursor(&mut self) {
        let buf = &self.buffers[self.active_idx];
        let line = buf.cursor.line;
        let graphemes: Vec<String> = buf
            .rope
            .line_slice(line)
            .graphemes(true)
            .map(|g| g.to_string())
            .collect();
        let len = graphemes.len();
        if len == 0 {
            self.buffers[self.active_idx].selection = None; // empty line — clear
            return;
        }
        // Clamp the index (a click at end-of-line lands on `len`).
        let idx = buf.cursor.grapheme_col.min(len - 1);
        let class = Self::grapheme_class(&graphemes[idx]);
        let mut start = idx;
        while start > 0 && Self::grapheme_class(&graphemes[start - 1]) == class {
            start -= 1;
        }
        let mut end = idx + 1;
        while end < len && Self::grapheme_class(&graphemes[end]) == class {
            end += 1;
        }
        self.set_selection_on_line(line, start, end);
    }

    /// Select the whole logical line under the cursor (US2).
    fn select_line_at_cursor(&mut self) {
        let line = self.buffers[self.active_idx].cursor.line;
        let len = self.buffers[self.active_idx]
            .rope
            .grapheme_count_on_line(line);
        self.set_selection_on_line(line, 0, len);
    }

    /// Set the active selection to `[start, end)` grapheme columns on `line`, with
    /// the cursor at `end`. A degenerate range clears the selection.
    fn set_selection_on_line(&mut self, line: usize, start: usize, end: usize) {
        let buf = &mut self.buffers[self.active_idx];
        if start >= end {
            buf.selection = None;
            return;
        }
        let vcol = |g| CursorPos::visual_col_from_grapheme_col(&buf.rope, line, g);
        let anchor = CursorPos {
            line,
            grapheme_col: start,
            visual_col: vcol(start),
        };
        let active = CursorPos {
            line,
            grapheme_col: end,
            visual_col: vcol(end),
        };
        buf.selection = Some(Selection { anchor, active });
        buf.cursor = active;
    }

    /// Adjust `scroll_offset` so that `cursor` is within the visible viewport.
    fn clamp_scroll(&mut self) {
        // Clamp to at least 1 row: a tiny/zero terminal frame (possible now that
        // terminal_size follows the real frame, feature 012 follow-up) would make
        // `vh - 1` underflow below. Guarding here keeps editing crash-free on any
        // frame size.
        let vh = self.viewport_height().max(1);

        if self.soft_wrap && self.wrap_cache.is_some() {
            let cursor_vr = self.cursor_visual_row();
            let buf = &mut self.buffers[self.active_idx];
            if cursor_vr < buf.scroll_offset.0 {
                buf.scroll_offset.0 = cursor_vr;
            } else if cursor_vr >= buf.scroll_offset.0 + vh {
                buf.scroll_offset.0 = cursor_vr.saturating_sub(vh - 1);
            }
        } else {
            let cur_line = self.buffers[self.active_idx].cursor.line;
            let buf = &mut self.buffers[self.active_idx];
            if cur_line < buf.scroll_offset.0 {
                buf.scroll_offset.0 = cur_line;
            } else if cur_line >= buf.scroll_offset.0 + vh {
                buf.scroll_offset.0 = cur_line.saturating_sub(vh - 1);
            }
        }
    }

    // ── Char-index helpers ────────────────────────────────────────────────────

    /// Convert the current cursor position to a rope char index.
    fn cursor_char_idx(&self) -> usize {
        let buf = &self.buffers[self.active_idx];
        self.char_idx_for(buf.cursor.line, buf.cursor.grapheme_col)
    }

    /// Return the rope char index for a given (line, grapheme_col) position.
    fn char_idx_for(&self, line: usize, gcol: usize) -> usize {
        let buf = &self.buffers[self.active_idx];
        // Start of line in chars
        let line_start: usize = (0..line)
            .map(|l| {
                // Each line in the rope includes the trailing \n except possibly the last
                let s = buf.rope.line_slice(l);
                s.chars().count() + 1 // +1 for the \n
            })
            .sum();

        // Walk grapheme clusters to find the char offset within the line
        let line_str = buf.rope.line_slice(line);
        let char_offset: usize = line_str
            .graphemes(true)
            .take(gcol)
            .map(|g| g.chars().count())
            .sum();

        line_start + char_offset
    }

    /// Feature 029: if the active buffer is read-only, set a status message and
    /// return `true` so the caller aborts the edit (previously a silent no-op).
    fn deny_if_readonly(&mut self) -> bool {
        if self.buffers[self.active_idx].readonly {
            self.status_message = Some("Buffer is read-only".to_string());
            true
        } else {
            false
        }
    }

    // ── T026 — Text insertion ─────────────────────────────────────────────────

    /// Insert a single character at the cursor. No-op when buffer is read-only.
    pub fn insert_char(&mut self, c: char) {
        if self.deny_if_readonly() {
            return;
        }
        // Feature 017: typing replaces the current selection.
        if self.buffers[self.active_idx].selection.is_some() {
            self.delete_selection();
        }

        let char_idx = self.cursor_char_idx();
        let s = c.to_string();

        {
            let buf = &mut self.buffers[self.active_idx];
            buf.rope.insert_str(char_idx, &s);
            buf.undo_stack.push(EditOp::Insert {
                at: char_idx,
                text: s,
            });
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);

        // Advance cursor right by one grapheme
        self.move_cursor(Direction::Right);
    }

    /// Insert a newline at the cursor, placing the cursor at column 0 of the
    /// new line.
    pub fn insert_newline(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        if self.buffers[self.active_idx].selection.is_some() {
            self.delete_selection(); // Feature 017: Enter replaces a selection
        }

        let char_idx = self.cursor_char_idx();

        {
            let buf = &mut self.buffers[self.active_idx];
            buf.rope.insert_str(char_idx, "\n");
            buf.undo_stack.push(EditOp::Insert {
                at: char_idx,
                text: "\n".to_string(),
            });
            buf.modified = true;

            // Move cursor: next line, column 0
            buf.cursor.line += 1;
            buf.cursor.grapheme_col = 0;
            buf.cursor.visual_col = 0;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);

        self.clamp_scroll();
    }

    // ── T027 — Backspace and Delete ───────────────────────────────────────────

    /// Delete the grapheme cluster immediately before the cursor.
    /// No-op at the start of the buffer or when read-only.
    pub fn delete_backward(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        // Feature 017: Backspace with a selection deletes the selection.
        if self.buffers[self.active_idx].selection.is_some() {
            self.delete_selection();
            return;
        }

        let cur = self.buffers[self.active_idx].cursor;

        // No-op at absolute beginning of the buffer
        if cur.line == 0 && cur.grapheme_col == 0 {
            return;
        }

        // Find the grapheme to remove (the one immediately before cursor)
        let (del_line, del_gcol) = if cur.grapheme_col > 0 {
            (cur.line, cur.grapheme_col - 1)
        } else {
            // At the start of a line — deleting the newline of the previous line
            let prev_line = cur.line - 1;
            let prev_len = self.buffers[self.active_idx]
                .rope
                .grapheme_count_on_line(prev_line);
            (prev_line, prev_len)
        };

        let del_char_idx = self.char_idx_for(del_line, del_gcol);

        // Collect the grapheme text (may be multi-char for combining sequences)
        let deleted_text: String = {
            let buf = &self.buffers[self.active_idx];
            let line_str = buf.rope.line_slice(del_line);
            line_str
                .graphemes(true)
                .nth(del_gcol)
                .unwrap_or("\n") // at line boundary we delete the \n
                .to_string()
        };

        let del_char_len = deleted_text.chars().count();

        {
            let buf = &mut self.buffers[self.active_idx];
            buf.rope
                .delete_range(del_char_idx..del_char_idx + del_char_len);
            buf.undo_stack.push(EditOp::Delete {
                at: del_char_idx,
                text: deleted_text,
            });
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);

        // Move cursor to the deleted position
        let new_vcol = CursorPos::visual_col_from_grapheme_col(
            &self.buffers[self.active_idx].rope,
            del_line,
            del_gcol,
        );
        let buf = &mut self.buffers[self.active_idx];
        buf.cursor = CursorPos {
            line: del_line,
            grapheme_col: del_gcol,
            visual_col: new_vcol,
        };

        self.clamp_scroll();
    }

    /// Delete the grapheme cluster at the cursor.
    /// No-op at the end of the buffer or when read-only.
    pub fn delete_forward(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        // Feature 017: Delete with a selection deletes the selection.
        if self.buffers[self.active_idx].selection.is_some() {
            self.delete_selection();
            return;
        }

        let cur = self.buffers[self.active_idx].cursor;
        let line_count = self.buffers[self.active_idx].rope.line_count();
        let gcol_count = self.buffers[self.active_idx]
            .rope
            .grapheme_count_on_line(cur.line);

        // Determine whether we're at the last possible position
        let is_last_line = cur.line + 1 >= line_count;
        let is_last_col = cur.grapheme_col >= gcol_count;

        if is_last_line && is_last_col {
            return; // At end of buffer
        }

        let del_char_idx = self.cursor_char_idx();

        // Determine the text being deleted
        let deleted_text: String = if cur.grapheme_col < gcol_count {
            // Delete grapheme at current column
            let buf = &self.buffers[self.active_idx];
            let line_str = buf.rope.line_slice(cur.line);
            line_str
                .graphemes(true)
                .nth(cur.grapheme_col)
                .unwrap_or("\n")
                .to_string()
        } else {
            // At end of line — delete the newline character
            "\n".to_string()
        };

        let del_char_len = deleted_text.chars().count();

        {
            let buf = &mut self.buffers[self.active_idx];
            buf.rope
                .delete_range(del_char_idx..del_char_idx + del_char_len);
            buf.undo_stack.push(EditOp::Delete {
                at: del_char_idx,
                text: deleted_text,
            });
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);
        // Cursor position stays the same after forward-delete
    }

    // ── T102 — Clipboard cut / copy / paste ──────────────────────────────────

    /// Copy selected text to the system clipboard.
    /// The active buffer's selected text, or `None` when there is no selection.
    ///
    /// Feature 028: `char_idx_for` returns CHAR indices, so this extracts by chars
    /// (byte-slicing a `String` panics on multibyte boundaries) and clamps a
    /// reversed/degenerate range to empty rather than panicking — defense-in-depth
    /// for copy/cut.
    pub fn selection_text(&self) -> Option<String> {
        let buf = &self.buffers[self.active_idx];
        let sel = buf.selection.as_ref()?;
        let (start, end) = sel.ordered_range();
        let s_idx = self.char_idx_for(start.line, start.grapheme_col);
        let e_idx = self.char_idx_for(end.line, end.grapheme_col);
        let full = buf.rope.to_string();
        let total = full.chars().count();
        let lo = s_idx.min(e_idx).min(total);
        let hi = s_idx.max(e_idx).min(total);
        Some(full.chars().skip(lo).take(hi - lo).collect::<String>())
    }

    pub fn copy_selection(&mut self) {
        let text = match self.selection_text() {
            Some(t) => t,
            None => return,
        };
        // Feature 029: give feedback (was silent on both success and failure).
        match arboard::Clipboard::new() {
            Ok(mut cb) => match cb.set_text(text) {
                Ok(()) => self.status_message = Some("Copied".to_string()),
                Err(e) => {
                    log::warn!("Clipboard write failed: {}", e);
                    self.status_message = Some("Clipboard unavailable".to_string());
                }
            },
            Err(e) => {
                log::warn!("Clipboard unavailable: {}", e);
                self.status_message = Some("Clipboard unavailable".to_string());
            }
        }
    }

    /// Cut selected text to the clipboard (copy + delete selection).
    pub fn cut_selection(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        if self.buffers[self.active_idx].selection.is_none() {
            return;
        }
        self.copy_selection();
        self.delete_selection();
        // Override the "Copied" set by copy_selection (unless the clipboard failed).
        if self.status_message.as_deref() == Some("Copied") {
            self.status_message = Some("Cut".to_string());
        }
    }

    /// Paste text from the system clipboard at the cursor.
    pub fn paste_clipboard(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        // Feature 017: paste replaces the current selection.
        if self.buffers[self.active_idx].selection.is_some() {
            self.delete_selection();
        }
        let text = match arboard::Clipboard::new() {
            Ok(mut cb) => match cb.get_text() {
                Ok(t) => t,
                Err(e) => {
                    // Feature 029: empty clipboard / read failure is no longer silent.
                    log::warn!("Clipboard read failed: {}", e);
                    self.status_message = Some("Nothing to paste".to_string());
                    return;
                }
            },
            Err(e) => {
                log::warn!("Clipboard unavailable: {}", e);
                self.status_message = Some("Clipboard unavailable".to_string());
                return;
            }
        };
        if text.is_empty() {
            self.status_message = Some("Nothing to paste".to_string());
            return;
        }
        let char_idx = self.cursor_char_idx();
        let char_count = text.chars().count();
        {
            let buf = &mut self.buffers[self.active_idx];
            buf.rope.insert_str(char_idx, &text);
            buf.undo_stack.push(EditOp::Insert {
                at: char_idx,
                text: text.clone(),
            });
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);
        // Advance cursor by pasted char count
        for _ in 0..char_count {
            self.move_cursor(Direction::Right);
        }
        self.status_message = Some("Pasted".to_string()); // Feature 029: feedback
    }

    /// Delete the current selection from the buffer.
    fn delete_selection(&mut self) {
        let sel = match self.buffers[self.active_idx].selection {
            Some(s) => s,
            None => return,
        };
        let (start, end) = sel.ordered_range();
        let s_idx = self.char_idx_for(start.line, start.grapheme_col);
        let e_idx = self.char_idx_for(end.line, end.grapheme_col);
        if s_idx >= e_idx {
            return;
        }
        // Feature 029: `char_idx_for` returns CHAR indices; extract the deleted
        // text by chars (byte-slicing a String with char indices panics on
        // multibyte content) — same hazard fixed in `copy_selection`/`selection_text`.
        let deleted: String = {
            let buf = &self.buffers[self.active_idx];
            let full = buf.rope.to_string();
            let total = full.chars().count();
            let lo = s_idx.min(e_idx).min(total);
            let hi = s_idx.max(e_idx).min(total);
            full.chars().skip(lo).take(hi - lo).collect()
        };
        {
            let buf = &mut self.buffers[self.active_idx];
            buf.rope.delete_range(s_idx..e_idx);
            buf.undo_stack.push(EditOp::Delete {
                at: s_idx,
                text: deleted,
            });
            buf.modified = true;
            buf.selection = None;
            buf.cursor = start;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);
    }

    // ── T055 — Find next / find prev ─────────────────────────────────────────

    // ── Feature 015 — interactive Find / Replace dialog ────────────────────────

    /// Open the Find dialog (Ctrl+F / Search ▸ Find), seeded with the last query.
    pub fn open_find_dialog(&mut self) {
        let seed = self.search_state.query.clone();
        self.pending_find_replace = Some(FindReplaceDialog::new(DialogMode::Find, seed));
    }

    /// Open the Replace dialog (Ctrl+H / Search ▸ Find Replace).
    pub fn open_replace_dialog(&mut self) {
        let seed = self.search_state.query.clone();
        let mut d = FindReplaceDialog::new(DialogMode::Replace, seed);
        if let Some(r) = &self.search_state.replacement {
            d.replacement = r.clone();
        }
        self.pending_find_replace = Some(d);
    }

    /// Close the Find/Replace dialog and clear the active match highlights.
    pub fn close_find_replace(&mut self) {
        self.pending_find_replace = None;
        self.search_state.matches.clear();
        self.search_state.active_match = None;
    }

    /// Char index of the cursor in the active buffer (for "first match at/after
    /// the cursor").
    fn cursor_char_index(&self) -> usize {
        let buf = &self.buffers[self.active_idx];
        let text = buf.rope.to_string();
        let mut char_count = 0usize;
        for (line_idx, line_text) in text.split('\n').enumerate() {
            if line_idx == buf.cursor.line {
                let gcols = unicode_segmentation::UnicodeSegmentation::graphemes(line_text, true)
                    .take(buf.cursor.grapheme_col)
                    .map(|g| g.chars().count())
                    .sum::<usize>();
                return char_count + gcols;
            }
            char_count += line_text.chars().count() + 1; // + newline
        }
        char_count
    }

    /// Copy the dialog's query/options into `search_state`, run the search,
    /// highlight matches, and jump to the first match at/after the cursor.
    fn run_find_from_dialog(&mut self) {
        let (query, case, regex, whole, wrap, replacement) = {
            let d = match self.pending_find_replace.as_ref() {
                Some(d) => d,
                None => return,
            };
            (
                d.query.clone(),
                d.case_sensitive,
                d.regex,
                d.whole_word,
                d.wrap,
                d.replacement.clone(),
            )
        };
        self.search_state.query = query.clone();
        self.search_state.case_sensitive = case;
        self.search_state.regex_mode = regex;
        self.search_state.whole_word = whole;
        self.search_state.wrap = wrap;
        self.search_state.replacement = Some(replacement);

        if query.is_empty() {
            self.search_state.matches.clear();
            self.search_state.active_match = None;
            return;
        }

        let matches = {
            let rope = &self.buffers[self.active_idx].rope;
            SearchEngine::find_all(rope, &query, regex, case, whole)
        };
        let total = matches.len();
        self.search_state.matches = matches;
        if total == 0 {
            self.search_state.active_match = None;
            self.status_message = Some("Not found".to_string());
            return;
        }
        let cursor_char = self.cursor_char_index();
        let idx = self
            .search_state
            .matches
            .iter()
            .position(|m| m.start >= cursor_char)
            .unwrap_or(0);
        self.search_state.active_match = Some(idx);
        self.status_message = Some(format!("Match {}/{}", idx + 1, total));
        self.scroll_to_match(idx);
    }

    /// Replace the current match with the dialog's replacement, then recompute
    /// matches and advance to the next (Enter in Replace mode).
    fn replace_current_from_dialog(&mut self) {
        if self.buffers[self.active_idx].readonly {
            self.status_message = Some("Buffer is read-only".to_string());
            return;
        }
        // Ensure search state reflects the dialog and we have matches.
        if self.search_state.matches.is_empty() || self.search_state.active_match.is_none() {
            self.run_find_from_dialog();
        }
        let replacement = self
            .pending_find_replace
            .as_ref()
            .map(|d| d.replacement.clone())
            .unwrap_or_default();
        let idx = match self.search_state.active_match {
            Some(i) => i,
            None => return,
        };
        let range = match self.search_state.matches.get(idx).cloned() {
            Some(r) => r,
            None => return,
        };

        // Capture the deleted text for undo.
        let deleted: String = {
            let full = self.buffers[self.active_idx].rope.to_string();
            let bs = full
                .char_indices()
                .nth(range.start)
                .map(|(b, _)| b)
                .unwrap_or(full.len());
            let be = full
                .char_indices()
                .nth(range.end)
                .map(|(b, _)| b)
                .unwrap_or(full.len());
            full[bs..be].to_string()
        };
        {
            let buf = &mut self.buffers[self.active_idx];
            buf.rope.delete_range(range.start..range.end);
            buf.rope.insert_str(range.start, &replacement);
            buf.undo_stack.push(EditOp::Composite(vec![
                EditOp::Delete {
                    at: range.start,
                    text: deleted,
                },
                EditOp::Insert {
                    at: range.start,
                    text: replacement.clone(),
                },
            ]));
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);

        // Recompute matches against the edited document (no stale offsets).
        let (query, case, regex, whole) = (
            self.search_state.query.clone(),
            self.search_state.case_sensitive,
            self.search_state.regex_mode,
            self.search_state.whole_word,
        );
        let matches = {
            let rope = &self.buffers[self.active_idx].rope;
            SearchEngine::find_all(rope, &query, regex, case, whole)
        };
        let total = matches.len();
        self.search_state.matches = matches;
        if total == 0 {
            self.search_state.active_match = None;
            self.status_message = Some("Replaced 1 — no more matches".to_string());
            return;
        }
        let next = idx.min(total - 1);
        self.search_state.active_match = Some(next);
        self.status_message = Some(format!("Replaced 1 — Match {}/{}", next + 1, total));
        self.scroll_to_match(next);
    }

    /// Replace all occurrences (Ctrl+A in Replace mode), reporting the count.
    fn replace_all_from_dialog(&mut self) {
        // Sync the dialog query/replacement/options into search_state first.
        if let Some(d) = self.pending_find_replace.as_ref() {
            self.search_state.query = d.query.clone();
            self.search_state.replacement = Some(d.replacement.clone());
            self.search_state.case_sensitive = d.case_sensitive;
            self.search_state.regex_mode = d.regex;
            self.search_state.whole_word = d.whole_word;
            self.search_state.wrap = d.wrap;
        }
        self.replace_all();
    }

    /// Jump to the next search match, wrapping at end-of-document when
    /// `search_state.wrap` is `true`.
    ///
    /// If `search_state.matches` is empty the engine is re-run first.  Sets
    /// `status_message` to reflect the result ("Match N/M", "Not found", or
    /// "Search wrapped").
    pub fn find_next(&mut self) {
        // Re-run the search if there are no cached results.
        if self.search_state.matches.is_empty() {
            let query = self.search_state.query.clone();
            let regex_mode = self.search_state.regex_mode;
            let case_sensitive = self.search_state.case_sensitive;
            let whole_word = self.search_state.whole_word;
            let rope = &self.buffers[self.active_idx].rope;
            self.search_state.matches =
                SearchEngine::find_all(rope, &query, regex_mode, case_sensitive, whole_word);
        }

        let total = self.search_state.matches.len();
        if total == 0 {
            self.status_message = Some("Not found".to_string());
            self.search_state.active_match = None;
            return;
        }

        let next_idx = match self.search_state.active_match {
            None => 0,
            Some(cur) => {
                if cur + 1 < total {
                    cur + 1
                } else if self.search_state.wrap {
                    // Set a "wrapped" message when going past the last match.
                    self.status_message = Some(format!("Search wrapped — Match 1/{}", total));
                    self.search_state.active_match = Some(0);
                    self.scroll_to_match(0);
                    return;
                } else {
                    self.status_message = Some(format!("Match {}/{} (end)", total, total));
                    return;
                }
            }
        };

        self.search_state.active_match = Some(next_idx);
        self.status_message = Some(format!("Match {}/{}", next_idx + 1, total));
        self.scroll_to_match(next_idx);
    }

    /// Jump to the previous search match, wrapping at start-of-document when
    /// `search_state.wrap` is `true`.
    pub fn find_prev(&mut self) {
        // Re-run the search if there are no cached results.
        if self.search_state.matches.is_empty() {
            let query = self.search_state.query.clone();
            let regex_mode = self.search_state.regex_mode;
            let case_sensitive = self.search_state.case_sensitive;
            let whole_word = self.search_state.whole_word;
            let rope = &self.buffers[self.active_idx].rope;
            self.search_state.matches =
                SearchEngine::find_all(rope, &query, regex_mode, case_sensitive, whole_word);
        }

        let total = self.search_state.matches.len();
        if total == 0 {
            self.status_message = Some("Not found".to_string());
            self.search_state.active_match = None;
            return;
        }

        let prev_idx = match self.search_state.active_match {
            None => total - 1,
            Some(0) => {
                if self.search_state.wrap {
                    self.status_message =
                        Some(format!("Search wrapped — Match {}/{}", total, total));
                    self.search_state.active_match = Some(total - 1);
                    self.scroll_to_match(total - 1);
                    return;
                } else {
                    self.status_message = Some(format!("Match 1/{} (start)", total));
                    return;
                }
            }
            Some(cur) => cur - 1,
        };

        self.search_state.active_match = Some(prev_idx);
        self.status_message = Some(format!("Match {}/{}", prev_idx + 1, total));
        self.scroll_to_match(prev_idx);
    }

    /// Scroll the viewport and reposition the cursor to the match at `idx`.
    fn scroll_to_match(&mut self, idx: usize) {
        let char_start = match self.search_state.matches.get(idx) {
            Some(r) => r.start,
            None => return,
        };

        // Convert char index → (line, grapheme_col) by walking the rope.
        let text = self.buffers[self.active_idx].rope.to_string();
        let mut char_count = 0usize;
        let mut target_line = 0usize;
        let mut target_gcol = 0usize;

        'outer: for (line_idx, line_text) in text.split('\n').enumerate() {
            let graphemes: Vec<&str> =
                unicode_segmentation::UnicodeSegmentation::graphemes(line_text, true).collect();
            for (gcol, g) in graphemes.iter().enumerate() {
                if char_count == char_start {
                    target_line = line_idx;
                    target_gcol = gcol;
                    break 'outer;
                }
                char_count += g.chars().count();
            }
            // Account for the '\n' character between lines.
            if char_count == char_start {
                target_line = line_idx;
                target_gcol = graphemes.len();
                break 'outer;
            }
            char_count += 1; // for '\n'
        }

        let new_vcol = CursorPos::visual_col_from_grapheme_col(
            &self.buffers[self.active_idx].rope,
            target_line,
            target_gcol,
        );

        let buf = &mut self.buffers[self.active_idx];
        buf.cursor = CursorPos {
            line: target_line,
            grapheme_col: target_gcol,
            visual_col: new_vcol,
        };
        self.clamp_scroll();
    }

    // ── T057 — Replace All ────────────────────────────────────────────────────

    /// Replace every occurrence of the current search query with the
    /// replacement string.
    ///
    /// Replacements are applied in **reverse** document order so that earlier
    /// char indices remain valid while later ones are being modified.  All
    /// individual delete+insert pairs are wrapped in a single
    /// `EditOp::Composite` so the entire replace-all is one undo step.
    ///
    /// Sets `status_message` to "Replaced N occurrences" (or "No matches").
    pub fn replace_all(&mut self) {
        if self.buffers[self.active_idx].readonly {
            self.status_message = Some("Buffer is read-only".to_string());
            return;
        }

        let replacement = match &self.search_state.replacement {
            Some(r) => r.clone(),
            None => {
                self.status_message = Some("No replacement string set".to_string());
                return;
            }
        };

        // Re-run find to get fresh positions.
        {
            let query = self.search_state.query.clone();
            let regex_mode = self.search_state.regex_mode;
            let case_sensitive = self.search_state.case_sensitive;
            let whole_word = self.search_state.whole_word;
            let rope = &self.buffers[self.active_idx].rope;
            self.search_state.matches =
                SearchEngine::find_all(rope, &query, regex_mode, case_sensitive, whole_word);
        }

        let matches = self.search_state.matches.clone();
        if matches.is_empty() {
            self.status_message = Some("No matches".to_string());
            return;
        }

        let count = matches.len();
        let mut ops: Vec<EditOp> = Vec::with_capacity(count * 2);

        // Apply in reverse order (last match first) to keep earlier indices stable.
        for m in matches.iter().rev() {
            let del_len = m.end - m.start;
            // Capture the text that will be deleted for undo.
            let deleted_text: String = {
                let buf = &self.buffers[self.active_idx];
                let full = buf.rope.to_string();
                // Convert char indices to byte indices for slicing.
                let byte_start = full
                    .char_indices()
                    .nth(m.start)
                    .map(|(b, _)| b)
                    .unwrap_or(full.len());
                let byte_end = full
                    .char_indices()
                    .nth(m.end)
                    .map(|(b, _)| b)
                    .unwrap_or(full.len());
                full[byte_start..byte_end].to_string()
            };

            // Apply the deletion.
            {
                let buf = &mut self.buffers[self.active_idx];
                buf.rope.delete_range(m.start..m.end);
            }
            ops.push(EditOp::Delete {
                at: m.start,
                text: deleted_text,
            });

            // Apply the insertion.
            {
                let buf = &mut self.buffers[self.active_idx];
                buf.rope.insert_str(m.start, &replacement);
            }
            ops.push(EditOp::Insert {
                at: m.start,
                text: replacement.clone(),
            });
        }

        // Push all ops as one composite undo entry.
        {
            let buf = &mut self.buffers[self.active_idx];
            buf.undo_stack.push(EditOp::Composite(ops));
            buf.modified = true;
        }
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);

        // Invalidate the cached match positions — they are no longer valid.
        self.search_state.matches.clear();
        self.search_state.active_match = None;

        self.status_message = Some(format!(
            "Replaced {} occurrence{}",
            count,
            if count == 1 { "" } else { "s" }
        ));
    }

    // ── T103 — SaveAs action ─────────────────────────────────────────────────

    /// Save the active buffer to a new path and update buffer.path.
    ///
    /// If `pending_save_as_encoding` is set (meaning the user selected an
    /// encoding via the encoding dialog before typing a filename), that
    /// encoding is applied to the buffer before writing and then cleared.
    pub fn handle_save_as(
        &mut self,
        new_path: std::path::PathBuf,
    ) -> Result<(), crate::buffer::BufferError> {
        if let Some(enc) = self.pending_save_as_encoding.take() {
            self.buffers[self.active_idx].encoding = enc;
        }
        self.active_buffer_mut().save_as(new_path)
    }

    /// Discard a pending encoding selection (called when the user cancels
    /// the filename-input dialog that follows encoding selection).
    pub fn cancel_pending_save_as_encoding(&mut self) {
        self.pending_save_as_encoding = None;
    }

    // ── T066 — Next / previous buffer ────────────────────────────────────────

    /// Cycle forward to the next open buffer, wrapping around.
    /// Feature 028: invalidate the soft-wrap cache so the next frame rebuilds it
    /// for the now-active buffer. The cache stores per-line visual byte offsets for
    /// ONE buffer's content at a given `wrap_text_gen`; whenever the active buffer's
    /// content identity changes (switch / open / close / session restore) the cache
    /// must be considered stale, or the renderer would slice the new buffer's lines
    /// with the old buffer's offsets (the session-restore crash). Bumping the
    /// generation makes `WrapCache::is_stale` true on the next render-loop check.
    fn invalidate_wrap_cache(&mut self) {
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);
    }

    pub fn next_buffer(&mut self) {
        if self.buffers.len() <= 1 {
            return;
        }
        self.active_idx = (self.active_idx + 1) % self.buffers.len();
        self.invalidate_wrap_cache();
        self.clamp_scroll();
    }

    /// Cycle backward to the previous open buffer, wrapping around.
    pub fn prev_buffer(&mut self) {
        if self.buffers.len() <= 1 {
            return;
        }
        self.active_idx = if self.active_idx == 0 {
            self.buffers.len() - 1
        } else {
            self.active_idx - 1
        };
        self.invalidate_wrap_cache();
        self.clamp_scroll();
    }

    // ── T069 — Open file into new buffer ─────────────────────────────────────

    /// Open `path` as a new buffer and make it the active buffer.
    ///
    /// Uses [`crate::security::sanitize::validate_path`] to guard against
    /// path-traversal attacks.  On success the new buffer is appended to
    /// `self.buffers` and `active_idx` is updated to point at it.
    pub fn handle_open_file(&mut self, path: PathBuf) {
        use crate::security::sanitize::validate_path;

        let safe_path = match validate_path(&path) {
            Ok(p) => p,
            Err(e) => {
                // Feature 029: surface rejected paths instead of failing silently.
                log::warn!("handle_open_file: path rejected ({:?}): {}", path, e);
                self.status_message = Some(format!("Open failed: {}", path.display()));
                return;
            }
        };

        let default_encoding = crate::encoding::encoding_from_str(&self.config.default_encoding);

        match Buffer::open(safe_path.clone(), default_encoding) {
            Ok(buf) => {
                // Feature 007: watch the newly-opened file.
                if let Some(ref mut fw) = self.file_watcher {
                    if let Err(e) = fw.watch_path(&safe_path) {
                        log::warn!("FileWatcher: could not watch {:?}: {}", safe_path, e);
                    }
                }
                self.buffers.push(buf);
                self.active_idx = self.buffers.len() - 1;
                self.invalidate_wrap_cache();
                let name = safe_path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| safe_path.display().to_string());
                self.status_message = Some(format!("Opened {name}"));
                log::info!("Opened {:?} as buffer {}", safe_path, self.active_idx);
            }
            Err(e) => {
                // Feature 029: surface the failure (path + reason) instead of a
                // silent no-op that leaves the user wondering why nothing opened.
                log::error!("handle_open_file: failed to open {:?}: {}", safe_path, e);
                self.status_message = Some(format!("Open failed: {} — {}", safe_path.display(), e));
            }
        }
    }

    // ── Feature 012 — File browser ─────────────────────────────────────────────

    /// Directory the file browser should open at: the active buffer's parent
    /// directory if it has a path, else the process current directory.
    pub fn browser_start_dir(&self) -> PathBuf {
        if let Some(parent) = self
            .buffers
            .get(self.active_idx)
            .and_then(|b| b.path.as_ref())
            .and_then(|p| p.parent())
        {
            return parent.to_path_buf();
        }
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    /// Apply the result of a browser activation: open/save the chosen file and
    /// close the browser, or leave it open to keep navigating.
    pub fn apply_browse_outcome(&mut self, outcome: BrowseOutcome) {
        match outcome {
            BrowseOutcome::Navigated | BrowseOutcome::None => {}
            BrowseOutcome::OpenFile(path) => {
                self.file_browser = None;
                self.handle_open_file(path);
            }
            BrowseOutcome::SaveFile(path) => {
                self.file_browser = None;
                self.do_save_as(path);
            }
        }
    }

    // ── Feature 011 — File / Edit menu actions ────────────────────────────────

    /// Open a fresh empty buffer and make it active (File ▸ New / Ctrl+N).
    pub fn new_buffer(&mut self) {
        self.buffers.push(Buffer::new_empty());
        self.active_idx = self.buffers.len() - 1;
    }

    /// Close the active buffer (File ▸ Close). The sole buffer is replaced by a
    /// fresh empty one so the editor always has something to edit.
    pub fn close_active_buffer(&mut self) {
        self.close_buffer_at(self.active_idx);
    }

    /// Close the buffer at `idx`, adjusting `active_idx` so the same logical
    /// buffer stays active where possible (Feature 027). Closing the last
    /// remaining buffer replaces it with an empty scratch buffer. No prompt — the
    /// caller is responsible for any unsaved-changes confirmation.
    pub fn close_buffer_at(&mut self, idx: usize) {
        if idx >= self.buffers.len() {
            return;
        }
        if self.buffers.len() <= 1 {
            self.buffers[0] = Buffer::new_empty();
            self.active_idx = 0;
            self.invalidate_wrap_cache();
            self.clamp_scroll();
            return;
        }
        self.buffers.remove(idx);
        if self.active_idx > idx {
            self.active_idx -= 1;
        } else if self.active_idx >= self.buffers.len() {
            self.active_idx = self.buffers.len() - 1;
        }
        self.invalidate_wrap_cache();
        self.clamp_scroll();
    }

    /// Feature 027: a tab's `[x]` was clicked. A clean buffer closes immediately;
    /// a modified one opens the [`ButtonDialog::CloseConfirm`] modal (no silent
    /// data loss).
    pub fn tab_close_clicked(&mut self, idx: usize) {
        if idx >= self.buffers.len() {
            return;
        }
        if self.buffers[idx].modified {
            self.pending_close_confirm = Some(idx);
        } else {
            self.close_buffer_at(idx);
        }
    }

    // ── Feature 016 — dialog buttons (confirm/dismiss dialogs) ────────────────

    /// Initialize the focused dialog control once per dialog opening; reset when no
    /// dialog is open. Called from render and before key/mouse handling so focus is
    /// correct even without a prior frame.
    ///
    /// - Confirm/dismiss dialogs (Feature 016): focus the safe default button.
    /// - Interactive dialogs (Feature 020): focus the **primary control** (stop 0 —
    ///   the field/list), NOT a button. Feature 028 fix: without this, `dialog_focus`
    ///   carried over from a previous dialog could land on a button when an
    ///   interactive dialog opened, so typed characters were swallowed and the caret
    ///   hidden (the Save-As "can't type / can't see what I type" bug).
    fn ensure_dialog_focus(&mut self) {
        if self.open_button_dialog().is_some() {
            if !self.dialog_focus_init {
                self.dialog_focus = self.dialog_default_focus();
                self.dialog_focus_init = true;
            }
        } else if self.interactive_dialog().is_some() {
            if !self.dialog_focus_init {
                self.dialog_focus = 0; // primary control (field/list)
                self.dialog_focus_init = true;
            }
        } else {
            self.dialog_focus_init = false;
        }
    }

    /// The currently-open confirm/dismiss dialog that has a button bar, if any.
    /// Order matches modal precedence.
    fn open_button_dialog(&self) -> Option<ButtonDialog> {
        if self.pending_session_restore.is_some() {
            Some(ButtonDialog::SessionRestore)
        } else if self.pending_save_prompt {
            Some(ButtonDialog::SavePrompt)
        } else if self.pending_external_change.is_some() {
            Some(ButtonDialog::ExternalChange)
        } else if self.pending_revert_confirm.is_some() {
            Some(ButtonDialog::RevertConfirm)
        } else if self.pending_close_confirm.is_some() {
            Some(ButtonDialog::CloseConfirm)
        } else if !self.pending_plugin_consent.is_empty() {
            Some(ButtonDialog::PluginConsent)
        } else {
            None
        }
    }

    /// Ordered button labels for the open confirm/dismiss dialog (tab order).
    pub fn dialog_button_labels(&self) -> Vec<&'static str> {
        // Feature 021: each label carries its activating key. Dispatch
        // (`activate_dialog_button`) keys on the button index, never this text.
        match self.open_button_dialog() {
            Some(ButtonDialog::SessionRestore) => vec!["Restore (Enter)", "Decline (Esc)"],
            Some(ButtonDialog::SavePrompt) => vec!["Save (S)", "Discard (D)", "Cancel (Esc)"],
            Some(ButtonDialog::ExternalChange) => vec!["Reload (Enter)", "Keep (Esc)"],
            Some(ButtonDialog::RevertConfirm) => vec!["Revert (Enter)", "Cancel (Esc)"],
            Some(ButtonDialog::PluginConsent) => vec!["Allow (Enter)", "Deny (Esc)"],
            Some(ButtonDialog::CloseConfirm) => vec!["Save (S)", "Discard (D)", "Cancel (Esc)"],
            None => vec![],
        }
    }

    /// Safe default-focused button index for the open dialog (R6).
    pub fn dialog_default_focus(&self) -> usize {
        match self.open_button_dialog() {
            Some(ButtonDialog::SavePrompt) => 2,     // Cancel
            Some(ButtonDialog::ExternalChange) => 1, // Keep
            Some(ButtonDialog::RevertConfirm) => 1,  // Cancel
            Some(ButtonDialog::PluginConsent) => 1,  // Deny
            Some(ButtonDialog::CloseConfirm) => 2,   // Cancel
            _ => 0,
        }
    }

    /// Whether clicking outside the dialog box cancels it (all current ones have a
    /// safe cancel).
    pub fn dialog_supports_outside_cancel(&self) -> bool {
        self.open_button_dialog().is_some()
    }

    /// Button index treated as "cancel/no/keep" for an outside click.
    fn dialog_cancel_index(&self) -> Option<usize> {
        match self.open_button_dialog()? {
            ButtonDialog::SessionRestore => Some(1), // Decline
            ButtonDialog::SavePrompt => Some(2),     // Cancel
            ButtonDialog::ExternalChange => Some(1), // Keep
            ButtonDialog::RevertConfirm => Some(1),  // Cancel
            ButtonDialog::PluginConsent => Some(1),  // Deny
            ButtonDialog::CloseConfirm => Some(2),   // Cancel
        }
    }

    /// Run the choice for button `idx` of the open confirm/dismiss dialog, reusing
    /// the existing handlers so a button == the corresponding key shortcut.
    pub fn activate_dialog_button(&mut self, idx: usize) {
        match self.open_button_dialog() {
            Some(ButtonDialog::SessionRestore) => {
                if idx == 0 {
                    self.do_restore_session();
                }
                self.pending_session_restore = None;
            }
            Some(ButtonDialog::SavePrompt) => match idx {
                0 => self.prompt_save_and_quit(),
                1 => self.prompt_discard_and_quit(),
                _ => self.prompt_cancel_quit(),
            },
            Some(ButtonDialog::ExternalChange) => {
                if let Some(ec) = self.pending_external_change.take() {
                    if idx == 0 {
                        self.reload_from_disk(ec.buf_idx);
                    } else {
                        self.buffers[ec.buf_idx].modified = true;
                    }
                }
            }
            Some(ButtonDialog::RevertConfirm) => {
                if idx == 0 {
                    if let Some(b) = self.pending_revert_confirm.take() {
                        self.reload_from_disk(b);
                    }
                } else {
                    self.pending_revert_confirm = None;
                }
            }
            Some(ButtonDialog::PluginConsent) => self.consent_decide(idx == 0),
            Some(ButtonDialog::CloseConfirm) => {
                // Feature 027: operate on the stored (clicked) index, not the
                // active buffer (M1). Save → save then close; Discard → close;
                // Cancel → dismiss, nothing closes. A failed save keeps the
                // dialog open so no changes are silently lost (Principle VII).
                if let Some(bidx) = self.pending_close_confirm.take() {
                    match idx {
                        0 => match self.buffers.get(bidx).map(|b| b.save()) {
                            Some(Ok(())) => {
                                if let Some(path) = self.buffers[bidx].path.clone() {
                                    self.self_write_times.insert(path, Instant::now());
                                }
                                self.close_buffer_at(bidx);
                            }
                            Some(Err(e)) => {
                                log::error!("Save failed on tab close: {}", e);
                                self.pending_close_confirm = Some(bidx); // keep open
                            }
                            None => {}
                        },
                        1 => self.close_buffer_at(bidx),
                        _ => {} // Cancel — already cleared by take()
                    }
                }
            }
            None => {}
        }
    }

    /// Title + body lines for the open confirm dialog (Feature 016). Centralized
    /// here so the overlay render and mouse hit-test share identical geometry.
    fn dialog_view_text(&self) -> Option<(&'static str, Vec<String>)> {
        let kind = self.open_button_dialog()?;
        let active_name = || {
            self.buffers
                .get(self.active_idx)
                .and_then(|b| b.path.as_ref())
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "[No Name]".to_string())
        };
        let v = match kind {
            ButtonDialog::SessionRestore => (
                "Restore Session",
                vec!["Restore previous session?".to_string()],
            ),
            ButtonDialog::SavePrompt => (
                "Unsaved Changes",
                vec![format!("Save changes to {}?", active_name())],
            ),
            ButtonDialog::ExternalChange => {
                let mut lines = vec!["File changed on disk.".to_string()];
                if let Some(ec) = &self.pending_external_change {
                    if self.buffers.get(ec.buf_idx).map(|b| b.modified) == Some(true) {
                        lines.push("WARNING: unsaved changes will be lost.".to_string());
                    }
                }
                ("External Change", lines)
            }
            ButtonDialog::RevertConfirm => (
                "Revert",
                vec![
                    format!("Revert {} to last saved version?", active_name()),
                    "Unsaved changes will be lost.".to_string(),
                ],
            ),
            ButtonDialog::PluginConsent => {
                let name = self
                    .pending_plugin_consent
                    .first()
                    .map(|m| m.id.clone())
                    .unwrap_or_default();
                (
                    "Plugin Consent",
                    vec![format!("Allow plugin '{}' to run?", name)],
                )
            }
            ButtonDialog::CloseConfirm => {
                // Name the buffer being closed (the stored index, not active).
                let name = self
                    .pending_close_confirm
                    .and_then(|i| self.buffers.get(i))
                    .and_then(|b| b.path.as_ref())
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "[No Name]".to_string());
                (
                    "Unsaved Changes",
                    vec![format!("Save changes to {} before closing?", name)],
                )
            }
        };
        Some(v)
    }

    /// Overlay rect for the open confirm dialog, sized to fit its body + button
    /// row. Shared by the renderer and mouse hit-testing so clicks land on the
    /// drawn buttons (Feature 016).
    pub fn button_dialog_rect(&self) -> Option<ratatui::layout::Rect> {
        let (_title, body) = self.dialog_view_text()?;
        let labels = self.dialog_button_labels();
        let (tw, th) = self.terminal_size;
        let body_w = body
            .iter()
            .map(|l| unicode_width::UnicodeWidthStr::width(l.as_str()) as u16)
            .max()
            .unwrap_or(0);
        // Each button: width(label)+4, plus 1-col gaps.
        let buttons_w: u16 = labels
            .iter()
            .map(|l| unicode_width::UnicodeWidthStr::width(*l) as u16 + 4)
            .sum::<u16>()
            + labels.len().saturating_sub(1) as u16;
        let inner = body_w.max(buttons_w);
        let dw = (inner + 4).clamp(24, tw.max(24)).min(tw.max(1));
        // borders(2) + body lines + gap(1) + button row(3)
        let dh = (body.len() as u16 + 6).min(th.max(1));
        let dx = tw.saturating_sub(dw) / 2;
        let dy = th.saturating_sub(dh) / 2;
        Some(ratatui::layout::Rect::new(dx, dy, dw, dh))
    }

    /// Everything the renderer needs to draw the open confirm dialog:
    /// `(rect, title, body lines, button labels, focused index)`. `None` when no
    /// button-dialog is open (Feature 016).
    #[allow(clippy::type_complexity)]
    pub fn button_dialog_render(
        &self,
    ) -> Option<(
        ratatui::layout::Rect,
        &'static str,
        Vec<String>,
        Vec<&'static str>,
        usize,
    )> {
        let rect = self.button_dialog_rect()?;
        let (title, body) = self.dialog_view_text()?;
        let labels = self.dialog_button_labels();
        let focus = self.dialog_focus.min(labels.len().saturating_sub(1));
        Some((rect, title, body, labels, focus))
    }

    // ── Feature 020 — interactive/list dialog focus ring ──────────────────────
    //
    // The four interactive dialogs (encoding select, plugin manager, Find/Replace,
    // file browser) reuse `dialog_focus` as a ring index: stop 0 (and stop 1 for
    // Find/Replace in replace mode) is the primary control; later stops are boxed
    // buttons. `Tab`/`Shift+Tab` move the index; a button is activated by
    // Enter/Space or a click. While the primary control is focused, the dialog's
    // existing keys behave exactly as before.

    /// The currently-open interactive/list dialog, if any. These are mutually
    /// exclusive in practice; the order is a defensive precedence.
    fn interactive_dialog(&self) -> Option<InteractiveDialog> {
        if self.pending_find_replace.is_some() {
            Some(InteractiveDialog::FindReplace)
        } else if self.pending_encoding_select.is_some() {
            Some(InteractiveDialog::EncodingSelect)
        } else if self.file_browser.is_some() {
            Some(InteractiveDialog::FileBrowser)
        } else if self.pending_plugin_manager {
            Some(InteractiveDialog::PluginManager)
        } else {
            None
        }
    }

    /// Number of primary-control focus stops that precede the buttons in the ring
    /// (1 for the list/browser dialogs; 1 in Find mode and 2 in Replace mode).
    fn interactive_field_stops(&self) -> usize {
        match self.interactive_dialog() {
            Some(InteractiveDialog::FindReplace) => {
                match self.pending_find_replace.as_ref().map(|d| d.mode) {
                    Some(DialogMode::Replace) => 2,
                    _ => 1,
                }
            }
            Some(_) => 1,
            None => 0,
        }
    }

    /// Ordered boxed-button labels for the open interactive dialog (tab order
    /// after the primary control). Mode-aware for Find/Replace and the file
    /// browser.
    pub fn interactive_button_labels(&self) -> Vec<&'static str> {
        // Feature 021: labels carry their activating key; dispatch
        // (`activate_interactive_button`) keys on index + mode, not this text.
        match self.interactive_dialog() {
            Some(InteractiveDialog::EncodingSelect) => vec!["OK (Enter)", "Cancel (Esc)"],
            Some(InteractiveDialog::PluginManager) => vec!["Close (Esc)"],
            Some(InteractiveDialog::FileBrowser) => {
                let save = matches!(
                    self.file_browser.as_ref().map(|b| b.mode),
                    Some(crate::ui::file_browser::BrowseMode::Save)
                );
                if save {
                    vec!["Save (Enter)", "Cancel (Esc)"]
                } else {
                    vec!["Open (Enter)", "Cancel (Esc)"]
                }
            }
            Some(InteractiveDialog::FindReplace) => {
                let replace = matches!(
                    self.pending_find_replace.as_ref().map(|d| d.mode),
                    Some(DialogMode::Replace)
                );
                if replace {
                    vec![
                        "Find (Enter)",
                        "Replace",
                        "Replace All (Ctrl+A)",
                        "Close (Esc)",
                    ]
                } else {
                    vec!["Find (Enter)", "Close (Esc)"]
                }
            }
            None => vec![],
        }
    }

    /// Total focus stops in the ring (primary-control stops + buttons).
    fn interactive_ring_len(&self) -> usize {
        self.interactive_field_stops() + self.interactive_button_labels().len()
    }

    /// `Some(button_index)` when `dialog_focus` is on a button rather than the
    /// primary control; `None` when the primary control is focused (or no
    /// interactive dialog is open).
    pub fn interactive_focus_is_button(&self) -> Option<usize> {
        self.interactive_dialog()?;
        let fs = self.interactive_field_stops();
        if self.dialog_focus >= fs {
            Some(self.dialog_focus - fs)
        } else {
            None
        }
    }

    /// Keep `FindReplaceDialog.focus` in sync with the ring's field stops so the
    /// edited/rendered field matches the focused stop (stop 0 → Query, stop 1 →
    /// Replacement). No-op when a button stop is focused.
    fn sync_find_replace_focus(&mut self) {
        let f = self.dialog_focus;
        if let Some(d) = self.pending_find_replace.as_mut() {
            let field = match f {
                0 => DialogField::Query,
                1 if d.mode == DialogMode::Replace => DialogField::Replacement,
                _ => return,
            };
            d.set_focus(field);
        }
    }

    /// Outer overlay `Rect` for the open interactive dialog — the single geometry
    /// source shared by the renderer and mouse hit-testing so a click always
    /// lands on the button that was drawn.
    pub fn interactive_dialog_rect(&self) -> Option<ratatui::layout::Rect> {
        let (tw, th) = self.terminal_size;
        let area = ratatui::layout::Rect::new(0, 0, tw, th);
        match self.interactive_dialog()? {
            InteractiveDialog::EncodingSelect => {
                Some(crate::ui::dialog::encoding_dialog_rect(area))
            }
            InteractiveDialog::PluginManager => Some(crate::ui::plugin_manager::manager_rect(
                &self.plugin_host,
                self.plugin_manager_cursor,
                area,
            )),
            InteractiveDialog::FileBrowser => {
                Some(self.file_browser.as_ref().unwrap().box_rect(area))
            }
            InteractiveDialog::FindReplace => Some(crate::ui::find_replace_rect(
                self.pending_find_replace.as_ref().unwrap(),
                area,
            )),
        }
    }

    /// Run the action bound to button `idx` of the open interactive dialog. Each
    /// maps onto an action the dialog already performs (no new actions).
    pub fn activate_interactive_button(&mut self, idx: usize) {
        match self.interactive_dialog() {
            Some(InteractiveDialog::EncodingSelect) => {
                if idx == 0 {
                    if let Some(sel) = self.pending_encoding_select {
                        let enc = crate::ui::dialog::ENCODING_OPTIONS[sel].0;
                        self.pending_encoding_select = None;
                        self.do_save_as_encoding(enc);
                    }
                } else {
                    self.pending_encoding_select = None;
                }
            }
            Some(InteractiveDialog::PluginManager) => {
                // Sole button is Close.
                self.pending_plugin_manager = false;
            }
            Some(InteractiveDialog::FileBrowser) => {
                if idx == 0 {
                    let outcome = self.file_browser.as_mut().unwrap().activate();
                    self.apply_browse_outcome(outcome);
                } else {
                    self.file_browser = None;
                }
            }
            Some(InteractiveDialog::FindReplace) => {
                // Dispatch on (mode, index), not label text (labels carry key hints).
                // Find mode ring buttons: [Find, Close]; Replace mode:
                // [Find, Replace, Replace All, Close].
                let replace = matches!(
                    self.pending_find_replace.as_ref().map(|d| d.mode),
                    Some(DialogMode::Replace)
                );
                if replace {
                    match idx {
                        0 => self.run_find_from_dialog(),
                        1 => self.replace_current_from_dialog(),
                        2 => self.replace_all_from_dialog(),
                        _ => self.close_find_replace(),
                    }
                } else {
                    match idx {
                        0 => self.run_find_from_dialog(),
                        _ => self.close_find_replace(),
                    }
                }
            }
            None => {}
        }
    }

    /// File ▸ Revert (Feature 014): reload the active buffer from its last saved
    /// version on disk, discarding in-editor changes. No-op with a notice when the
    /// buffer was never saved; asks for confirmation when there are unsaved changes.
    pub fn handle_revert(&mut self) {
        let idx = self.active_idx;
        if self.buffers[idx].path.is_none() {
            self.status_message = Some("Nothing to revert (never saved)".to_string());
            return;
        }
        if self.buffers[idx].modified {
            // Confirm before discarding unsaved changes.
            self.pending_revert_confirm = Some(idx);
        } else {
            // Clean buffer — reload directly (harmless re-read).
            self.reload_from_disk(idx);
        }
    }

    /// Write the active buffer to `path` (File ▸ Save As).
    pub fn do_save_as(&mut self, path: PathBuf) {
        // Feature 029: honor an encoding chosen before the destination was picked
        // (the Save-As-Encoding → file-browser flow). Previously this path ignored
        // `pending_save_as_encoding`, silently writing the file in the old encoding.
        if let Some(enc) = self.pending_save_as_encoding.take() {
            self.buffers[self.active_idx].encoding = enc;
        }
        match self.buffers[self.active_idx].save_as(path.clone()) {
            Ok(()) => {
                self.buffers[self.active_idx].modified = false;
                self.buffers[self.active_idx].undo_stack.mark_saved(); // Feature 014
                                                                       // Feature 007: suppress the watcher event from our own write.
                self.self_write_times.insert(path.clone(), Instant::now());
                self.status_message = Some(format!("Saved as {}", path.display()));
            }
            Err(e) => {
                log::error!("save_as failed for {:?}: {}", path, e);
                self.status_message = Some(format!("Save As failed: {e}"));
            }
        }
    }

    /// Select the entire buffer (Edit ▸ Select All / Ctrl+A).
    pub fn select_all(&mut self) {
        let buf = &mut self.buffers[self.active_idx];
        let line_count = buf.rope.line_count();
        if line_count == 0 {
            return;
        }
        let last_line = line_count - 1;
        let last_gcol = buf.rope.grapheme_count_on_line(last_line);
        let anchor = CursorPos {
            line: 0,
            grapheme_col: 0,
            visual_col: 0,
        };
        let active_vcol = CursorPos::visual_col_from_grapheme_col(&buf.rope, last_line, last_gcol);
        let active = CursorPos {
            line: last_line,
            grapheme_col: last_gcol,
            visual_col: active_vcol,
        };
        buf.selection = Some(Selection { anchor, active });
        buf.cursor = active;
    }

    /// Undo the most recent edit (Edit ▸ Undo / Ctrl+Z).
    pub fn handle_undo(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        let op = {
            let buf = &mut self.buffers[self.active_idx];
            buf.undo_stack.undo(&mut buf.rope)
        };
        match op {
            Some(op) => {
                self.apply_history_cursor(undo_target_idx(&op));
                self.status_message = Some("Undo".to_string());
            }
            None => self.status_message = Some("Nothing to undo".to_string()),
        }
    }

    /// Redo the most recently undone edit (Edit ▸ Redo / Ctrl+Y).
    pub fn handle_redo(&mut self) {
        if self.deny_if_readonly() {
            return;
        }
        let op = {
            let buf = &mut self.buffers[self.active_idx];
            buf.undo_stack.redo(&mut buf.rope)
        };
        match op {
            Some(op) => {
                self.apply_history_cursor(redo_target_idx(&op));
                self.status_message = Some("Redo".to_string());
            }
            None => self.status_message = Some("Nothing to redo".to_string()),
        }
    }

    /// Shared post-undo/redo bookkeeping: mark dirty, drop selection, invalidate
    /// the wrap cache, and move the cursor to `char_idx` (clamped to the buffer).
    fn apply_history_cursor(&mut self, char_idx: usize) {
        let (line, gcol) = self.line_col_for_char_idx(char_idx);
        let buf = &mut self.buffers[self.active_idx];
        // Feature 014: undo/redo may return the content to the saved baseline —
        // derive Modified from the undo history instead of forcing it true.
        buf.refresh_modified();
        buf.selection = None;
        let vcol = CursorPos::visual_col_from_grapheme_col(&buf.rope, line, gcol);
        buf.cursor = CursorPos {
            line,
            grapheme_col: gcol,
            visual_col: vcol,
        };
        self.wrap_text_gen = self.wrap_text_gen.wrapping_add(1);
        self.clamp_scroll();
    }

    /// Convert a rope char index into a `(line, grapheme_col)` position — the
    /// inverse of [`Self::char_idx_for`]. Clamps past-end indices to the end.
    fn line_col_for_char_idx(&self, char_idx: usize) -> (usize, usize) {
        let buf = &self.buffers[self.active_idx];
        let line_count = buf.rope.line_count();
        let mut remaining = char_idx;
        for line in 0..line_count {
            let line_str = buf.rope.line_slice(line);
            let line_chars = line_str.chars().count();
            if remaining <= line_chars {
                let mut acc = 0usize;
                let mut gcol = 0usize;
                for g in line_str.graphemes(true) {
                    if acc >= remaining {
                        break;
                    }
                    acc += g.chars().count();
                    gcol += 1;
                }
                return (line, gcol);
            }
            remaining -= line_chars + 1; // +1 for the line's trailing newline
        }
        let last = line_count.saturating_sub(1);
        (last, buf.rope.grapheme_count_on_line(last))
    }

    // ── Feature 011 — Mouse menu interaction ──────────────────────────────────

    /// Handle a raw mouse event: open top-level menus, activate dropdown items,
    /// close menus on an outside click, or reposition the editor cursor.
    ///
    /// Uses [`hit_test_menu`] with the same geometry the menu bar renders with,
    /// so clicks land on exactly what is drawn — for both built-in and plugin
    /// menus, and for dropdown items (which previously could not be clicked).
    pub fn handle_mouse_event(&mut self, me: crossterm::event::MouseEvent) -> io::Result<()> {
        let Some(ev) = normalize_mouse(me) else {
            return Ok(());
        };

        // Feature 030 (US3): while the context menu is open it is modal — a press
        // on an item activates it, a press elsewhere dismisses. (Wheel/drag ignored.)
        if let Some(menu) = self.pending_context_menu {
            if ev.kind == NormalizedMouseKind::Press {
                let (w, h) = self.terminal_size;
                let rect = crate::ui::contextmenu::menu_rect(
                    &menu,
                    ratatui::layout::Rect::new(0, 0, w, h),
                );
                if ev.button == MouseButton::Left {
                    if let Some(idx) = crate::ui::contextmenu::hit_test(rect, ev.col, ev.row) {
                        let act = crate::ui::contextmenu::ITEMS[idx].1.clone();
                        self.pending_context_menu = None;
                        return self.handle_action(act);
                    }
                }
                // Press outside the menu (or non-left) dismisses it.
                self.pending_context_menu = None;
            }
            return Ok(());
        }

        // Feature 030 (US3): a right-click in the editor opens the context menu —
        // but only when no other modal/menu is active (modal precedence, FR-010).
        if ev.kind == NormalizedMouseKind::Press && ev.button == MouseButton::Right {
            let (_, term_rows) = self.terminal_size;
            let in_editor = ev.row >= self.editor_top() && ev.row + 1 < term_rows;
            let any_modal = self.open_button_dialog().is_some()
                || self.interactive_dialog().is_some()
                || self.pending_help.is_some()
                || self.pending_goto_line.is_some()
                || self.menu_bar.is_active();
            if in_editor && !any_modal {
                self.pending_context_menu =
                    Some(crate::ui::contextmenu::ContextMenu::new(ev.col, ev.row));
            }
            return Ok(());
        }

        // Feature 024: while a scrollbar thumb drag is active, mouse drags scroll
        // (proportional) instead of selecting text. Released below.
        if ev.kind == NormalizedMouseKind::Drag && self.scrollbar_drag.is_some() {
            let d = self.scrollbar_drag.unwrap();
            let click = match d.axis {
                ScrollAxis::Vertical => ev.row.saturating_sub(d.track_start),
                ScrollAxis::Horizontal => ev.col.saturating_sub(d.track_start),
            } as usize;
            let off = crate::ui::scrollbar::pos_to_offset(
                d.track_len as usize,
                d.content,
                d.viewport,
                click,
            );
            self.apply_scroll_target(d.target, off, d.viewport);
            return Ok(());
        }

        // Feature 024: a button release ends any scrollbar drag.
        if ev.kind == NormalizedMouseKind::Release {
            self.scrollbar_drag = None;
            return Ok(());
        }

        // Feature 017: a left-drag in the editor extends the selection from the
        // anchor set on the preceding press (only when no modal/menu is active).
        if ev.kind == NormalizedMouseKind::Drag && ev.button == MouseButton::Left {
            if let Some(anchor) = self.drag_anchor {
                if !self.menu_bar.is_active() {
                    self.handle_mouse_click(ev.col, ev.row);
                    self.update_selection_to_cursor(anchor);
                }
            }
            return Ok(());
        }

        // Feature 023: mouse-wheel scrolling. Routed to the open modal/overlay
        // (modal wins), else the editor pane under the cursor. Placed BEFORE the
        // Press/Left guard so wheel events are not dropped, and returns so the
        // click/cursor paths below are never entered for a wheel event.
        if matches!(
            ev.kind,
            NormalizedMouseKind::ScrollUp | NormalizedMouseKind::ScrollDown
        ) {
            let down = ev.kind == NormalizedMouseKind::ScrollDown;
            let step = WHEEL_STEP;
            if self.pending_help.is_some() {
                self.help_scroll = if down {
                    self.help_scroll.saturating_add(step)
                } else {
                    self.help_scroll.saturating_sub(step)
                };
            } else if let Some(idx) = self.pending_encoding_select {
                let n = crate::ui::dialog::ENCODING_OPTIONS.len();
                self.pending_encoding_select = Some(if down {
                    (idx + step).min(n - 1)
                } else {
                    idx.saturating_sub(step)
                });
            } else if self.file_browser.is_some() {
                let (w, h) = self.terminal_size;
                let vis = ratatui::layout::Rect::new(0, 0, w, h);
                if let Some(fb) = self.file_browser.as_mut() {
                    let rows = fb.visible_rows(vis);
                    for _ in 0..step {
                        if down {
                            fb.move_down(rows);
                        } else {
                            fb.move_up(rows);
                        }
                    }
                }
            } else if self.pending_plugin_manager {
                let n = self.plugin_host.registry.instances.len();
                if n > 0 {
                    self.plugin_manager_cursor = if down {
                        (self.plugin_manager_cursor + step).min(n - 1)
                    } else {
                        self.plugin_manager_cursor.saturating_sub(step)
                    };
                }
            } else if self.pending_find_replace.is_some() || self.pending_goto_line.is_some() {
                // Find/Replace and Go-to-Line have no scrollable content — ignore.
            } else {
                // Editor: ignore the menu/tab rows (above editor_top) and the
                // status-bar row (last).
                let (w, term_rows) = self.terminal_size;
                if ev.row >= self.editor_top() && ev.row + 1 < term_rows {
                    let buf_idx = if matches!(self.split_mode, crate::ui::SplitMode::Vertical) {
                        if ev.col >= w / 2 && self.buffers.len() > 1 {
                            self.active_idx.max(1)
                        } else {
                            0
                        }
                    } else {
                        self.active_idx
                    };
                    self.wheel_scroll_editor(buf_idx, down, step);
                }
            }
            return Ok(());
        }

        // Only left-button presses drive the menu / cursor for now.
        if ev.kind != NormalizedMouseKind::Press || ev.button != MouseButton::Left {
            return Ok(());
        }

        // Feature 024: a press on a scrollbar — page on the track, drag on the
        // thumb. Checked BEFORE the editor click / feature-017 drag-anchor and the
        // modal entry/button handlers, so a bar press never selects or places the
        // cursor. Bars occupy reserved cells that don't overlap those targets.
        for r in self.scrollbar_regions(ev.col, ev.row) {
            let inside = ev.col >= r.rect.x
                && ev.col < r.rect.x + r.rect.width
                && ev.row >= r.rect.y
                && ev.row < r.rect.y + r.rect.height;
            if !inside {
                continue;
            }
            let (track_start, click, track_len) = match r.axis {
                ScrollAxis::Vertical => (r.rect.y, ev.row.saturating_sub(r.rect.y), r.rect.height),
                ScrollAxis::Horizontal => (r.rect.x, ev.col.saturating_sub(r.rect.x), r.rect.width),
            };
            let zone = crate::ui::scrollbar::hit_zone(
                track_len as usize,
                r.content,
                r.viewport,
                r.offset,
                click as usize,
            );
            let max_off = r.content.saturating_sub(r.viewport);
            match zone {
                crate::ui::scrollbar::HitZone::Above => {
                    let off = r.offset.saturating_sub(r.viewport);
                    self.apply_scroll_target(r.target, off, r.viewport);
                }
                crate::ui::scrollbar::HitZone::Below => {
                    let off = (r.offset + r.viewport).min(max_off);
                    self.apply_scroll_target(r.target, off, r.viewport);
                }
                crate::ui::scrollbar::HitZone::Thumb => {
                    self.scrollbar_drag = Some(ScrollbarDrag {
                        target: r.target,
                        axis: r.axis,
                        track_start,
                        track_len,
                        content: r.content,
                        viewport: r.viewport,
                    });
                }
            }
            return Ok(());
        }

        // Feature 021 — Help/About overlay: a click on the boxed Close button
        // dismisses it (same effect as Esc). Other clicks are inert (modal).
        if self.pending_help.is_some() {
            let (w, h) = self.terminal_size;
            let rects = crate::ui::help_close_button_rects(ratatui::layout::Rect::new(0, 0, w, h));
            if crate::ui::buttons::hit_test_buttons(&rects, ev.col, ev.row).is_some() {
                self.pending_help = None;
            }
            return Ok(());
        }

        // Feature 031 (#58) — Go-to-Line: a click in the digit field positions the
        // caret. Geometry mirrors the render: a centered box of width
        // `(19 + digits.len()).clamp(20, w)`; the digits start after the border +
        // the "Go to line: " (12-col) prefix.
        if let Some(entry) = self.pending_goto_line.clone() {
            let (w, h) = self.terminal_size;
            let dw = ((19 + entry.len()) as u16).clamp(20, w.max(1));
            let dh = 3u16.min(h.max(1));
            let dx = w.saturating_sub(dw) / 2;
            let dy = h.saturating_sub(dh) / 2;
            let value_x = dx + 1 + "Go to line: ".len() as u16;
            let field_w = dw.saturating_sub(2 + 12);
            if ev.row == dy + 1 && ev.col >= value_x && ev.col < value_x + field_w {
                self.pending_goto_line_caret =
                    crate::ui::width::field_caret_at(&entry, field_w, ev.col - value_x);
            }
            return Ok(());
        }

        // Feature 020 — interactive/list dialog buttons: a click on a boxed
        // button activates it directly (buttons win over the list/entry hit-test
        // that follows). Uses the same geometry the renderer drew with.
        self.ensure_dialog_focus();
        if self.interactive_dialog().is_some() {
            if let Some(rect) = self.interactive_dialog_rect() {
                let labels = self.interactive_button_labels();
                let rects = crate::ui::buttons::button_rects(rect, &labels);
                if let Some(i) = crate::ui::buttons::hit_test_buttons(&rects, ev.col, ev.row) {
                    self.activate_interactive_button(i);
                    return Ok(());
                }
                // Feature 030 (#53): a click on a list row selects that row and
                // focuses the list (primary control). Buttons were checked first.
                match self.interactive_dialog() {
                    Some(InteractiveDialog::EncodingSelect) => {
                        if let Some(idx) = crate::ui::dialog::encoding_row_hit(rect, ev.col, ev.row)
                        {
                            self.pending_encoding_select = Some(idx);
                            self.dialog_focus = 0;
                            return Ok(());
                        }
                    }
                    Some(InteractiveDialog::PluginManager) => {
                        if let Some(idx) = crate::ui::plugin_manager::manager_row_hit(
                            &self.plugin_host,
                            rect,
                            ev.col,
                            ev.row,
                        ) {
                            self.plugin_manager_cursor = idx;
                            self.dialog_focus = 0;
                            return Ok(());
                        }
                    }
                    Some(InteractiveDialog::FindReplace) => {
                        // Feature 031 (#58): a click in a field's text box moves the
                        // caret to the clicked grapheme and focuses that field.
                        let (w, h) = self.terminal_size;
                        let full = ratatui::layout::Rect::new(0, 0, w, h);
                        if let Some(d) = self.pending_find_replace.as_ref() {
                            let fields = crate::ui::find_replace_field_rects(d, full);
                            for (field, fr) in fields {
                                if ev.row == fr.y && ev.col >= fr.x && ev.col < fr.x + fr.width {
                                    let value = match field {
                                        crate::ui::dialog::DialogField::Query => &d.query,
                                        crate::ui::dialog::DialogField::Replacement => {
                                            &d.replacement
                                        }
                                    };
                                    let caret = crate::ui::width::field_caret_at(
                                        value,
                                        fr.width,
                                        ev.col - fr.x,
                                    );
                                    let d = self.pending_find_replace.as_mut().unwrap();
                                    d.set_focus(field);
                                    d.caret = caret;
                                    self.dialog_focus = match field {
                                        crate::ui::dialog::DialogField::Query => 0,
                                        crate::ui::dialog::DialogField::Replacement => 1,
                                    };
                                    return Ok(());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            // Not on a button or row: fall through to dialog-specific handling (file
            // browser entry clicks below; the other dialogs ignore the click).
        }

        // Feature 012 — file browser: a single click selects the row; a second
        // click on the same row (within DOUBLE_CLICK_MS) activates it (enter
        // folder / open file). This matches file-dialog convention and avoids a
        // double-click on a folder navigating in and then immediately opening
        // whatever file lands under the cursor in the new listing. A click
        // outside the box cancels.
        if self.file_browser.is_some() {
            let (w, h) = self.terminal_size;
            let area = ratatui::layout::Rect::new(0, 0, w, h);
            // Feature 031 (#58): a click inside the Name/path field box positions
            // the caret there (checked before the list/outside hit-test).
            {
                let fb = self.file_browser.as_ref().unwrap();
                let fr = fb.field_text_rect(area);
                if ev.row == fr.y && ev.col >= fr.x && ev.col < fr.x + fr.width {
                    self.file_browser.as_mut().unwrap().caret_click(fr, ev.col);
                    return Ok(());
                }
            }
            let hit = self
                .file_browser
                .as_ref()
                .unwrap()
                .hit_test(area, ev.col, ev.row);
            match hit {
                BrowserHit::Entry(idx) => {
                    let now = Instant::now();
                    let double = self.last_browser_click.is_some_and(|(prev, t)| {
                        prev == idx
                            && now.duration_since(t) <= Duration::from_millis(DOUBLE_CLICK_MS)
                    });
                    if double {
                        self.last_browser_click = None;
                        let outcome = self.file_browser.as_mut().unwrap().activate_index(idx);
                        self.apply_browse_outcome(outcome);
                    } else {
                        // First click: just move the highlight to the row.
                        self.last_browser_click = Some((idx, now));
                        self.file_browser.as_mut().unwrap().selected = idx;
                    }
                }
                BrowserHit::Outside => {
                    self.last_browser_click = None;
                    self.file_browser = None;
                }
                BrowserHit::Inside => self.last_browser_click = None,
            }
            return Ok(());
        }

        // Feature 016 — confirm/dismiss dialogs: a click on a boxed button
        // activates it; a click outside the dialog cancels (where safe). Uses the
        // same geometry the renderer drew with.
        self.ensure_dialog_focus();
        if self.open_button_dialog().is_some() {
            if let Some(rect) = self.button_dialog_rect() {
                let labels = self.dialog_button_labels();
                let rects = crate::ui::buttons::button_rects(rect, &labels);
                if let Some(i) = crate::ui::buttons::hit_test_buttons(&rects, ev.col, ev.row) {
                    self.activate_dialog_button(i);
                } else {
                    let inside = ev.col >= rect.x
                        && ev.col < rect.x + rect.width
                        && ev.row >= rect.y
                        && ev.row < rect.y + rect.height;
                    if !inside {
                        if let Some(c) = self.dialog_cancel_index() {
                            self.activate_dialog_button(c);
                        }
                    }
                }
            }
            return Ok(());
        }

        // Modal dialogs win: ignore menu/editor mouse while one is open.
        if self.pending_save_prompt
            || self.pending_session_restore.is_some()
            || self.pending_encoding_select.is_some()
            || self.pending_help.is_some()
            || self.pending_external_change.is_some()
            || !self.pending_plugin_consent.is_empty()
            || self.pending_plugin_manager
            || self.pending_revert_confirm.is_some()
            || self.pending_find_replace.is_some()
            || self.pending_goto_line.is_some()
            || self.pending_close_confirm.is_some()
        {
            return Ok(());
        }

        // Feature 027 — tab bar: a click on the tab row switches buffers (label)
        // or closes one (`[x]`), and never reaches the editor (FR-008). Uses the
        // same geometry as the renderer. A click on the row outside any tab is a
        // no-op. Reached only when no modal is open (guarded above).
        if self.tab_bar_visible() && ev.row + 1 == self.editor_top() {
            let area = ratatui::layout::Rect::new(0, ev.row, self.terminal_size.0, 1);
            for r in crate::ui::tabbar::tab_hit_regions(area, &self.buffers, self.active_idx) {
                if ev.col == r.close_rect.x {
                    self.tab_close_clicked(r.idx);
                    return Ok(());
                }
                if ev.col >= r.label_rect.x && ev.col < r.label_rect.x + r.label_rect.width {
                    self.active_idx = r.idx;
                    self.clamp_scroll();
                    return Ok(());
                }
            }
            return Ok(());
        }

        let menus = self.resolved_menus();
        let toggle_states = [(Action::ToggleSoftWrap, self.soft_wrap)];
        let term_width = self.terminal_size.0;

        match hit_test_menu(
            &menus,
            &self.menu_bar.state,
            &toggle_states,
            term_width,
            ev.col,
            ev.row,
        ) {
            MenuHit::TopLevel(idx) => {
                // Clicking the title of the already-open menu closes it (toggle).
                if let MenuState::DropDown { top_idx, .. } = self.menu_bar.state {
                    if top_idx == idx {
                        self.menu_bar.close_menu();
                        return Ok(());
                    }
                }
                self.menu_bar.open_menu(idx, &menus);
            }
            MenuHit::Item { top_idx, item_idx } => {
                self.menu_bar.state = MenuState::DropDown { top_idx, item_idx };
                if let Some(action) = self.menu_bar.select_item(&menus) {
                    return self.handle_action(action);
                }
            }
            MenuHit::Outside => {
                if self.menu_bar.is_active() {
                    self.menu_bar.close_menu();
                } else {
                    // Editor click: position the cursor first (maps the click to a
                    // line/grapheme col).
                    self.handle_mouse_click(ev.col, ev.row);
                    // Feature 030: classify single/double/triple click.
                    match self.next_editor_click_count(ev.col, ev.row) {
                        2 => self.select_word_at_cursor(),
                        3 => self.select_line_at_cursor(),
                        _ => {
                            // Single click: clear selection, set the drag anchor so
                            // a following drag selects (Feature 017).
                            self.buffers[self.active_idx].selection = None;
                            self.drag_anchor = Some(self.buffers[self.active_idx].cursor);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // ── T111 — Mouse click cursor repositioning ───────────────────────────────

    /// Reposition the cursor when the user clicks inside the editor area.
    ///
    /// `col` and `row` are 0-based terminal coordinates.  Row 0 is the menu bar
    /// and the last row is the status bar, so editor rows are `1..terminal_rows-1`.
    pub fn handle_mouse_click(&mut self, col: u16, row: u16) {
        let (term_cols, term_rows) = self.terminal_size;

        // Feature 027: the editor starts at `editor_top()` (below the menu bar and,
        // when shown, the tab bar). Rows above it and the status row are not editor.
        let top = self.editor_top();
        if row < top || row >= term_rows.saturating_sub(1) {
            return;
        }
        // Feature 021: the editor reserves its rightmost column for the vertical
        // scrollbar and (non-wrap) its bottom row for the horizontal scrollbar;
        // clicks on those reserved cells must not move the cursor.
        if col >= term_cols.saturating_sub(1) {
            return;
        }
        if !self.soft_wrap && row == term_rows.saturating_sub(2) {
            return;
        }

        let clicked_row = (row - top) as usize; // 0-based editor row

        // Feature 029: the line-number gutter occupies `gutter` columns on the
        // left; the text area starts after it. Map the raw terminal column into the
        // text area (a click on the gutter clamps to column 0 via saturating_sub).
        let gutter: u16 = if self.config.line_numbers { 4 } else { 0 };
        let col = col.saturating_sub(gutter);

        // Soft-wrap mode: map (visual_row, visual_col) → (logical_line, grapheme_col).
        if self.soft_wrap {
            if let Some(ref cache) = self.wrap_cache {
                let scroll_vr = self.buffers[self.active_idx].scroll_offset.0;
                let visual_row = scroll_vr + clicked_row;
                if let Some((logical_line, start_byte_u32)) = cache.visual_to_logical(visual_row) {
                    let start_byte = start_byte_u32 as usize;
                    let line_str = self.buffers[self.active_idx].rope.line_slice(logical_line);
                    // Compute which segment end byte is.
                    let seg_end = {
                        let starts = &cache.visual_starts[logical_line];
                        let seg_idx_opt = starts.iter().position(|&b| b as usize == start_byte);
                        seg_idx_opt
                            .and_then(|si| starts.get(si + 1))
                            .map(|&b| b as usize)
                            .unwrap_or(line_str.len())
                    };
                    // Effective text column accounting for '»' marker on continuation rows.
                    let is_continuation = start_byte > 0;
                    let text_col_start: usize = if is_continuation { 1 } else { 0 };
                    let target_vcol = col as usize;

                    let mut vis_col = text_col_start;
                    let mut found_gcol: usize = {
                        // Default: walk logical line to find byte=start_byte → grapheme col.
                        line_str
                            .graphemes(true)
                            .scan(0usize, |b, g| {
                                let c = *b;
                                *b += g.len();
                                Some((c, g))
                            })
                            .take_while(|(b, _)| *b < start_byte)
                            .count()
                    };
                    let mut cur_byte = start_byte;

                    for grapheme in line_str[start_byte..seg_end].graphemes(true) {
                        let gw = unicode_segmentation_width(grapheme) as usize;
                        if vis_col + gw > target_vcol {
                            break;
                        }
                        vis_col += gw;
                        cur_byte += grapheme.len();
                        found_gcol += 1;
                    }

                    let _ = cur_byte; // used for iteration side effects

                    let new_vcol = CursorPos::visual_col_from_grapheme_col(
                        &self.buffers[self.active_idx].rope,
                        logical_line,
                        found_gcol,
                    );
                    let buf = &mut self.buffers[self.active_idx];
                    buf.cursor = CursorPos {
                        line: logical_line,
                        grapheme_col: found_gcol,
                        visual_col: new_vcol,
                    };
                    self.clamp_scroll();
                    return;
                }
            }
        }

        // Normal mode (non-wrap): existing logic.
        let buf = &self.buffers[self.active_idx];
        let scroll_line = buf.scroll_offset.0;
        let target_line = scroll_line + clicked_row;
        let line_count = buf.rope.line_count();
        if target_line >= line_count {
            return;
        }

        let line_str = buf.rope.line_slice(target_line);
        // Feature 029: account for the horizontal scroll offset — the first visible
        // text column is `scroll_offset.1`, so the absolute clicked column is
        // `scroll_offset.1 + col` (col already has the gutter removed).
        let target_x: u16 = (buf.scroll_offset.1 as u16).saturating_add(col);
        let mut visual_x: u16 = 0;
        let mut found_gcol: usize = 0;

        for (gcol, grapheme) in line_str.graphemes(true).enumerate() {
            let w = unicode_segmentation_width(grapheme);
            if visual_x + w > target_x {
                found_gcol = gcol;
                break;
            }
            visual_x += w;
            found_gcol = gcol + 1;
        }

        let new_vcol = CursorPos::visual_col_from_grapheme_col(
            &self.buffers[self.active_idx].rope,
            target_line,
            found_gcol,
        );

        let buf = &mut self.buffers[self.active_idx];
        buf.cursor = CursorPos {
            line: target_line,
            grapheme_col: found_gcol,
            visual_col: new_vcol,
        };
        self.clamp_scroll();
    }

    // ── T081 — Theme switching ────────────────────────────────────────────────

    /// Switch the active color theme by name.
    ///
    /// Valid built-in names: `"classic"`, `"high-contrast"`, `"plain"`.
    /// Unknown names silently fall back to `"classic"`.
    pub fn set_theme(&mut self, name: &str) {
        self.theme = crate::ui::theme::theme_by_name(name);
        log::debug!("Theme set to: {}", self.theme.name);
    }

    // ── Feature 008 — Plugin consent & manager ───────────────────────────────

    /// Record the user's consent decision for the front pending plugin, persist it,
    /// and (if allowed) load the plugin immediately.
    fn consent_decide(&mut self, allow: bool) {
        if self.pending_plugin_consent.is_empty() {
            return;
        }
        let plugin = self.pending_plugin_consent.remove(0);
        let dir = crate::plugin::edit_config_dir();
        let rec = crate::plugin::consent::ConsentRecord {
            allowed: allow,
            consented_at: crate::plugin::utc_now_rfc3339(),
            version_consented: plugin.version.to_string(),
        };
        if let Err(e) = crate::plugin::save_consent_record(&dir, &plugin.id, &rec) {
            log::warn!("failed to persist consent for {}: {e}", plugin.id);
        }
        if allow {
            let id = plugin.id.clone();
            match self.plugin_host.load_plugin_now(plugin) {
                Ok(()) => {
                    self.status_message = Some(format!("Plugin '{id}' enabled"));
                    self.reapply_plugin_highlighters();
                }
                Err(e) => {
                    log::warn!("failed to load consented plugin {id}: {e}");
                    self.status_message = Some(format!("Plugin '{id}' failed to load"));
                }
            }
        } else {
            self.status_message = Some(format!("Plugin '{}' disabled", plugin.id));
        }
    }

    /// Toggle the enabled state of the plugin under the manager cursor and persist it.
    fn plugin_manager_toggle_current(&mut self) {
        let idx = self.plugin_manager_cursor;
        let Some((id, new_enabled, version)) =
            self.plugin_host.registry.instances.get(idx).map(|i| {
                (
                    i.plugin.id.clone(),
                    !i.enabled,
                    i.plugin.version.to_string(),
                )
            })
        else {
            return;
        };
        self.plugin_host.registry.set_enabled(&id, new_enabled);
        let dir = crate::plugin::edit_config_dir();
        let rec = crate::plugin::consent::ConsentRecord {
            allowed: new_enabled,
            consented_at: crate::plugin::utc_now_rfc3339(),
            version_consented: version,
        };
        if let Err(e) = crate::plugin::save_consent_record(&dir, &id, &rec) {
            log::warn!("failed to persist plugin toggle for {id}: {e}");
        }
        self.status_message = Some(format!(
            "Plugin '{id}' {}",
            if new_enabled { "enabled" } else { "disabled" }
        ));
        self.reapply_plugin_highlighters();
    }

    /// Re-attach plugin highlighters to open buffers (e.g. after enabling a plugin).
    fn reapply_plugin_highlighters(&mut self) {
        if !self.config.highlight {
            return;
        }
        let theme = self.theme;
        for i in 0..self.buffers.len() {
            if let Some(path) = self.buffers[i].path.clone() {
                if let Some(hl) = self.plugin_host.highlighter_for(&path, theme) {
                    self.buffers[i].syntax = Some(hl);
                }
            }
        }
    }

    // ── T077 — Syntax-highlight toggle ───────────────────────────────────────

    /// Toggle syntax highlighting on the active buffer.
    ///
    /// If highlighting is currently active it is disabled.  If it is off and
    /// the buffer has a known path, the correct highlighter is re-detected and
    /// assigned.  Buffers with no path stay un-highlighted.
    pub fn toggle_highlight(&mut self) {
        if self.active_buffer().syntax.is_some() {
            self.active_buffer_mut().syntax = None;
            log::debug!("Syntax highlighting disabled");
        } else if let Some(path) = self.active_buffer().path.clone() {
            // A plugin highlighter takes precedence over the built-in (Feature 008).
            let hl = self
                .plugin_host
                .highlighter_for(&path, self.theme)
                .or_else(|| crate::highlight::detect_highlighter(&path));
            let name = hl.as_ref().map(|h| h.name());
            self.active_buffer_mut().syntax = hl;
            log::debug!("Syntax highlighting enabled: {:?}", name);
        }
    }

    // ── Feature 005 — Soft-wrap helpers ──────────────────────────────────────

    /// Viewport content width: terminal columns minus the gutter (if line numbers
    /// on) and minus the editor's rightmost vertical-scrollbar column (Feature 021).
    fn content_width(&self) -> u16 {
        let gutter: u16 = if self.config.line_numbers { 4 } else { 0 };
        self.terminal_size
            .0
            .saturating_sub(gutter)
            .saturating_sub(1)
    }

    /// Compute the global visual row index for the cursor position (using wrap cache).
    fn cursor_visual_row(&self) -> usize {
        let cache = match self.wrap_cache.as_ref() {
            Some(c) => c,
            None => return self.buffers[self.active_idx].cursor.line,
        };
        let cursor = self.buffers[self.active_idx].cursor;
        let line_str = self.buffers[self.active_idx].rope.line_slice(cursor.line);
        let cursor_byte: usize = line_str
            .graphemes(true)
            .take(cursor.grapheme_col)
            .map(|g| g.len())
            .sum();
        let starts = match cache.visual_starts.get(cursor.line) {
            Some(s) => s,
            None => return 0,
        };
        let seg_idx = starts
            .partition_point(|&s| (s as usize) <= cursor_byte)
            .saturating_sub(1);
        let rows_before: usize = (0..cursor.line)
            .map(|l| cache.visual_starts.get(l).map(|v| v.len()).unwrap_or(1))
            .sum();
        rows_before + seg_idx
    }

    /// Persist `self.config` to `$XDG_CONFIG_HOME/edit/config.toml` using
    /// an atomic tmp-rename. On failure: log warn, set status message, do not revert.
    fn save_config_to_disk(&mut self) {
        let config_path = crate::config::config_path();
        let tmp_path = config_path.with_extension("toml.tmp");

        let toml_str = match toml::to_string(&self.config) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Config serialize failed: {}", e);
                self.status_message = Some(format!("Config save failed: {}", e));
                return;
            }
        };

        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Err(e) = std::fs::write(&tmp_path, &toml_str) {
            log::warn!("Config write failed: {}", e);
            self.status_message = Some(format!("Config save failed: {}", e));
            return;
        }

        if let Err(e) = std::fs::rename(&tmp_path, &config_path) {
            log::warn!("Config rename failed: {}", e);
            self.status_message = Some(format!("Config save failed: {}", e));
        }
    }

    /// Toggle soft-wrap on or off. Handles the width guard, cache rebuild/drop,
    /// horizontal-scroll reset, and config persistence.
    fn handle_toggle_soft_wrap(&mut self) -> io::Result<()> {
        // Width guard (only applied when turning ON).
        let content_w = self.content_width();
        if !self.soft_wrap && content_w < 10 {
            self.status_message =
                Some("Terminal too narrow for soft wrap (min 10 columns)".to_string());
            return Ok(());
        }

        self.soft_wrap = !self.soft_wrap;
        self.config.soft_wrap = self.soft_wrap;

        if self.soft_wrap {
            // Build cache; reset horizontal scroll on all buffers.
            let rope = &self.buffers[self.active_idx].rope;
            self.wrap_cache = Some(crate::ui::wrap::WrapCache::compute(
                rope,
                content_w,
                self.wrap_text_gen,
            ));
            for buf in &mut self.buffers {
                buf.scroll_offset.1 = 0;
            }
        } else {
            // Drop cache; reset horizontal scroll for all buffers.
            self.wrap_cache = None;
            for buf in &mut self.buffers {
                buf.scroll_offset.1 = 0;
            }
        }

        self.save_config_to_disk();
        Ok(())
    }
}

/// Display width of a grapheme cluster.
///
/// Feature 029: delegates to the single shared width helper
/// ([`crate::ui::width::display_width`]) — `unicode-width`-based, so combining
/// marks are 0, East-Asian wide and emoji are 2. Replaces the old first-scalar
/// heuristic that mis-measured combining marks and emoji.
fn unicode_segmentation_width(grapheme: &str) -> u16 {
    crate::ui::width::display_width(grapheme)
}

// ---------------------------------------------------------------------------
// Tests — T016 / T017 / T018 / T019 / T022
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::encoding::EncodingId;

    fn make_app() -> App {
        App::new(Config::default(), vec![], EncodingId::Utf8, None, None)
    }

    // ── Feature 020 — interactive/list dialog focus ring ──────────────────────

    // T009: ring length and field-stop counts per dialog/mode.
    #[test]
    fn interactive_ring_math_per_dialog() {
        use crate::ui::dialog::{DialogMode, FindReplaceDialog};
        let mut a = make_app();
        a.terminal_size = (80, 24);
        assert_eq!(a.interactive_ring_len(), 0, "no dialog open");

        a.pending_encoding_select = Some(0);
        assert_eq!(a.interactive_field_stops(), 1);
        assert_eq!(a.interactive_ring_len(), 3); // List + OK + Cancel
        a.pending_encoding_select = None;

        a.pending_plugin_manager = true;
        assert_eq!(a.interactive_ring_len(), 2); // List + Close
        a.pending_plugin_manager = false;

        a.pending_find_replace = Some(FindReplaceDialog::new(DialogMode::Find, String::new()));
        assert_eq!(a.interactive_field_stops(), 1);
        assert_eq!(a.interactive_ring_len(), 3); // Query + Find + Close
        a.pending_find_replace = Some(FindReplaceDialog::new(DialogMode::Replace, String::new()));
        assert_eq!(a.interactive_field_stops(), 2);
        assert_eq!(a.interactive_ring_len(), 6); // Query+Replacement + 4 buttons
    }

    // T009: dialog_focus → primary-control vs button-index boundary.
    #[test]
    fn interactive_focus_is_button_boundary() {
        let mut a = make_app();
        a.pending_encoding_select = Some(0); // field_stops 1, ring 3
        a.dialog_focus = 0;
        assert_eq!(a.interactive_focus_is_button(), None);
        a.dialog_focus = 1;
        assert_eq!(a.interactive_focus_is_button(), Some(0));
        a.dialog_focus = 2;
        assert_eq!(a.interactive_focus_is_button(), Some(1));
    }

    // T040b: dialog rect + button layout recompute without panic across a range
    // of terminal sizes; at a normal size the rect stays within bounds.
    #[test]
    fn interactive_geometry_across_sizes_no_panic() {
        use crate::ui::dialog::{DialogMode, FindReplaceDialog};
        let sizes = [(80u16, 24u16), (20, 8), (200, 60), (4, 3), (40, 15)];
        for (w, h) in sizes {
            let mut a = make_app();
            a.terminal_size = (w, h);
            for setup in 0..3 {
                a.pending_encoding_select = None;
                a.pending_plugin_manager = false;
                a.pending_find_replace = None;
                match setup {
                    0 => a.pending_encoding_select = Some(3),
                    1 => a.pending_plugin_manager = true,
                    _ => {
                        a.pending_find_replace =
                            Some(FindReplaceDialog::new(DialogMode::Replace, "abc".into()))
                    }
                }
                if let Some(r) = a.interactive_dialog_rect() {
                    let labels = a.interactive_button_labels();
                    // Must not panic on any size (overflow buttons are dropped).
                    let rects = crate::ui::buttons::button_rects(r, &labels);
                    // Horizontal bound always holds (centered_rect clamps width).
                    assert!(r.x + r.width <= w.max(1), "rect within width");
                    if w >= 80 && h >= 24 {
                        assert!(!rects.is_empty(), "buttons fit at a normal size");
                    }
                }
            }
        }
    }

    // T040b: a wide/CJK button label is width-measured (no panic, fits its box).
    #[test]
    fn wide_label_button_rects_are_width_correct() {
        // "あ" is double-width; a 2-grapheme label → width 4 → box width 4+4=8.
        let area = ratatui::layout::Rect::new(0, 0, 60, 10);
        let rects = crate::ui::buttons::button_rects(area, &["ああ", "OK"]);
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].width, 8, "double-width label measured correctly");
    }

    // T040: each interactive dialog renders a boxed button row with exactly one
    // focused control (the focused button shows the `▶` marker exactly once).
    #[test]
    fn interactive_dialogs_render_one_focused_button() {
        use crate::ui::dialog::{DialogMode, FindReplaceDialog};
        use ratatui::{backend::TestBackend, Terminal};
        let render_marker_count = |app: &mut App| -> usize {
            let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
            t.draw(|f| app.render(f)).unwrap();
            t.backend()
                .buffer()
                .content()
                .iter()
                .filter(|c| c.symbol() == "▶")
                .count()
        };
        for setup in 0..3 {
            let mut a = make_app();
            a.terminal_size = (80, 24);
            match setup {
                0 => a.pending_encoding_select = Some(0),
                1 => a.pending_plugin_manager = true,
                _ => {
                    a.pending_find_replace =
                        Some(FindReplaceDialog::new(DialogMode::Replace, "x".into()))
                }
            }
            // Focus the first button (stop = field_stops) and keep it across the
            // render (ensure_dialog_focus would otherwise reset focus to 0).
            a.dialog_focus_init = true;
            a.dialog_focus = a.interactive_field_stops();
            assert_eq!(
                render_marker_count(&mut a),
                1,
                "exactly one focused button rendered (setup {setup})"
            );
        }
    }

    // ── Feature 027 — tab-bar-aware editor geometry ──────────────────────────

    // T003: editor_top()/viewport_height reflect the tab-bar row only with 2+ buffers.
    #[test]
    fn editor_top_and_viewport_height_track_tab_bar() {
        let mut a = make_app();
        a.terminal_size = (80, 24);
        a.soft_wrap = false;
        // One buffer → no tab bar; editor at row 1; height = 24-2-hbar(1) = 21.
        assert!(!a.tab_bar_visible());
        assert_eq!(a.editor_top(), 1);
        assert_eq!(a.viewport_height(), 21);
        // Two buffers → tab bar row; editor at row 2; height drops by 1 → 20.
        a.buffers.push(crate::buffer::Buffer::new_empty());
        assert!(a.tab_bar_visible());
        assert_eq!(a.editor_top(), 2);
        assert_eq!(a.viewport_height(), 20);
        // Soft-wrap (no hbar): one buffer 22, two buffers 21.
        a.soft_wrap = true;
        assert_eq!(a.viewport_height(), 21);
    }

    // T027 (Feature 029): a file-open failure surfaces an "Open failed" status
    // rather than silently doing nothing.
    #[test]
    fn open_failure_surfaces_status() {
        let mut a = make_app();
        let before = a.buffers.len();
        a.handle_open_file(std::path::PathBuf::from(
            "/nonexistent_edit_dir_xyz/nope.txt",
        ));
        assert!(
            a.status_message
                .as_deref()
                .unwrap_or("")
                .contains("Open failed"),
            "open failure is surfaced, got {:?}",
            a.status_message
        );
        assert_eq!(a.buffers.len(), before, "no buffer added on failure");
    }

    // T025 (Feature 029): editing a read-only buffer surfaces a message instead of
    // a silent no-op; copy with a selection reports "Copied".
    #[test]
    fn readonly_edit_and_copy_give_feedback() {
        let mut a = make_app();
        a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("abc\n");
        a.buffers[0].readonly = true;
        a.handle_action(Action::InsertChar('x')).unwrap();
        assert_eq!(a.status_message.as_deref(), Some("Buffer is read-only"));
        assert_eq!(a.buffers[0].rope.line_slice(0), "abc", "no edit applied");

        // Copy with a selection reports feedback (clipboard may be unavailable in
        // the test env — accept either the success or the unavailable message).
        a.buffers[0].readonly = false;
        a.status_message = None;
        a.select_all();
        a.copy_selection();
        let msg = a.status_message.as_deref().unwrap_or("");
        assert!(
            msg == "Copied" || msg == "Clipboard unavailable",
            "copy gives feedback, got {msg:?}"
        );
    }

    // T023 (Feature 029): with line numbers on, a click maps past the gutter; a
    // click within the gutter clamps to column 0; horizontal scroll is added.
    #[test]
    fn click_accounts_for_gutter_and_hscroll() {
        let mut a = make_app();
        a.terminal_size = (80, 24);
        a.config.line_numbers = true;
        a.soft_wrap = false;
        a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("abcdefghij\n");
        a.active_idx = 0;
        // Gutter is 4 cols; editor_top is row 1 (single buffer). Click at terminal
        // col 4+3=7 → text column 3.
        a.handle_mouse_click(7, 1);
        assert_eq!(a.buffers[0].cursor.grapheme_col, 3);
        // Click within the gutter (col 2) clamps to column 0.
        a.handle_mouse_click(2, 1);
        assert_eq!(a.buffers[0].cursor.grapheme_col, 0);
        // With a horizontal scroll of 2, a click at col 4+1=5 → text column 2+1=3.
        a.buffers[0].scroll_offset.1 = 2;
        a.handle_mouse_click(5, 1);
        assert_eq!(a.buffers[0].cursor.grapheme_col, 3);
    }

    // T013 (Feature 031): Go-to-Line is a caret-aware digit input.
    #[test]
    fn goto_line_caret_editing() {
        let mut a = make_app();
        a.pending_goto_line = Some(String::new());
        a.pending_goto_line_caret = 0;
        for c in ['1', '2', '3'] {
            a.handle_action(Action::InsertChar(c)).unwrap();
        }
        assert_eq!(a.pending_goto_line.as_deref(), Some("123"));
        assert_eq!(a.pending_goto_line_caret, 3);
        // Home, then insert mid-string.
        a.handle_action(Action::MoveLineStart).unwrap();
        assert_eq!(a.pending_goto_line_caret, 0);
        a.handle_action(Action::InsertChar('9')).unwrap();
        assert_eq!(a.pending_goto_line.as_deref(), Some("9123"));
        assert_eq!(a.pending_goto_line_caret, 1);
        // Non-digit rejected; caret unchanged.
        a.handle_action(Action::InsertChar('x')).unwrap();
        assert_eq!(a.pending_goto_line.as_deref(), Some("9123"));
        // Right then Backspace removes the grapheme before the caret.
        a.handle_action(Action::MoveRight).unwrap(); // caret 2
        a.handle_action(Action::Backspace).unwrap(); // removes '1' → "923"
        assert_eq!(a.pending_goto_line.as_deref(), Some("923"));
        assert_eq!(a.pending_goto_line_caret, 1);
        // End clamps; Left clamps at 0.
        a.handle_action(Action::MoveLineEnd).unwrap();
        assert_eq!(a.pending_goto_line_caret, 3);
        a.handle_action(Action::MoveLineStart).unwrap();
        a.handle_action(Action::MoveLeft).unwrap();
        assert_eq!(a.pending_goto_line_caret, 0);
    }

    // T017 (Feature 029): the save-before-quit prompt cancels on Esc.
    #[test]
    fn save_prompt_cancels_on_esc() {
        let mut a = make_app();
        a.pending_save_prompt = true;
        a.handle_action(Action::MenuClose).unwrap();
        assert!(!a.pending_save_prompt, "Esc cancels the save prompt");
        assert!(a.running, "cancel does not quit");
    }

    // T019 (Feature 029): Go-to-Line does not open while a menu is active.
    #[test]
    fn goto_line_does_not_open_over_menu() {
        let mut a = make_app();
        let menus = a.resolved_menus();
        a.menu_bar.open_menu(0, &menus);
        assert!(a.menu_bar.is_active());
        a.handle_action(Action::GoToLine).unwrap();
        assert!(
            a.pending_goto_line.is_none(),
            "Go-to-Line must not open over an active menu"
        );
    }

    // T021 (Feature 029): completing Save-As applies a pending encoding selection.
    #[test]
    fn do_save_as_applies_pending_encoding() {
        let mut a = make_app();
        let dir = std::env::temp_dir().join("edit_saveas_enc_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("enc.txt");
        a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("hi\n");
        a.pending_save_as_encoding = Some(EncodingId::Utf16Le);
        a.do_save_as(path);
        assert_eq!(
            a.buffers[0].encoding,
            EncodingId::Utf16Le,
            "encoding applied"
        );
        assert!(
            a.pending_save_as_encoding.is_none(),
            "pending encoding cleared"
        );
    }

    // T014 (Feature 029): plain save reports success; a failed save reports the
    // error and keeps the buffer modified (no silent success-looking failure).
    #[test]
    fn save_reports_success_and_failure() {
        let mut a = make_app();
        // Success: a real writable temp path.
        let dir = std::env::temp_dir().join("edit_save_fb_test");
        let _ = std::fs::create_dir_all(&dir);
        let ok_path = dir.join("ok.txt");
        a.buffers[0].path = Some(ok_path.clone());
        a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("hi\n");
        a.buffers[0].modified = true;
        a.handle_save_action();
        assert!(
            a.status_message
                .as_deref()
                .unwrap_or("")
                .starts_with("Saved"),
            "success shows a Saved message, got {:?}",
            a.status_message
        );
        assert!(!a.buffers[0].modified, "clean after a successful save");

        // Failure: a path whose parent directory does not exist → save errors.
        a.status_message = None;
        a.buffers[0].path = Some(std::path::PathBuf::from(
            "/nonexistent_edit_dir_xyz/cannot/write.txt",
        ));
        a.buffers[0].modified = true;
        a.handle_save_action();
        assert!(
            a.status_message
                .as_deref()
                .unwrap_or("")
                .contains("Save failed"),
            "failure is surfaced, got {:?}",
            a.status_message
        );
        assert!(a.buffers[0].modified, "stays modified after a failed save");
    }

    // T005 (Feature 030): double-click selects the word under the cursor; triple
    // selects the line; works over multibyte; degenerate cases don't panic.
    #[test]
    fn word_and_line_selection() {
        let mut a = make_app();
        a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("foo bar_baz, café\n");
        a.active_idx = 0;
        let put = |a: &mut App, g: usize| {
            a.buffers[0].cursor = crate::buffer::CursorPos {
                line: 0,
                grapheme_col: g,
                visual_col: g,
            };
        };
        // Cursor in "bar_baz" (underscore is a word char) → whole token.
        put(&mut a, 5);
        a.select_word_at_cursor();
        assert_eq!(a.selection_text().as_deref(), Some("bar_baz"));
        // Cursor in the multibyte word "café".
        put(&mut a, 13);
        a.select_word_at_cursor();
        assert_eq!(a.selection_text().as_deref(), Some("café"));
        // Triple-click selects the whole line content.
        a.select_line_at_cursor();
        assert_eq!(a.selection_text().as_deref(), Some("foo bar_baz, café"));
        // Empty line → no panic, no selection.
        a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("\n");
        put(&mut a, 0);
        a.select_word_at_cursor();
        assert!(a.buffers[0].selection.is_none());
    }

    // T005 (Feature 030): click-count classification (single/double/triple) within
    // the time+cell window, wrapping after 3.
    #[test]
    fn editor_click_count_classification() {
        let mut a = make_app();
        assert_eq!(a.next_editor_click_count(5, 5), 1);
        assert_eq!(a.next_editor_click_count(5, 5), 2);
        assert_eq!(a.next_editor_click_count(5, 5), 3);
        assert_eq!(a.next_editor_click_count(5, 5), 1, "wraps after triple");
        // A different cell resets to single.
        assert_eq!(a.next_editor_click_count(9, 9), 1);
    }

    // T006 (Feature 029): delete_selection over multibyte text removes the right
    // characters, records the correct undo text, and never panics.
    #[test]
    fn delete_selection_is_char_safe_multibyte() {
        use crate::buffer::{CursorPos, Selection};
        let mut a = make_app();
        a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("éàûü\n");
        a.active_idx = 0;
        let cur = |g: usize| CursorPos {
            line: 0,
            grapheme_col: g,
            visual_col: g,
        };
        // Select the first two graphemes "éà" and delete.
        a.buffers[0].selection = Some(Selection {
            anchor: cur(0),
            active: cur(2),
        });
        a.delete_selection();
        assert_eq!(a.buffers[0].rope.line_slice(0), "ûü");
        assert!(a.buffers[0].selection.is_none());
        // Undo restores the deleted "éà".
        a.handle_action(Action::Undo).unwrap();
        assert_eq!(a.buffers[0].rope.line_slice(0), "éàûü");
    }

    // T022 (Feature 028): selection_text is char-safe (multibyte) and never panics
    // on a degenerate/reversed range.
    #[test]
    fn selection_text_is_char_safe_and_panic_free() {
        use crate::buffer::{CursorPos, Selection};
        let mut a = make_app();
        // Multibyte content: each "é" is 2 bytes; byte-slicing would risk a panic.
        a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("éàûü\n");
        a.active_idx = 0;
        let cur = |g: usize| CursorPos {
            line: 0,
            grapheme_col: g,
            visual_col: g,
        };
        // Forward selection of the first two graphemes.
        a.buffers[0].selection = Some(Selection {
            anchor: cur(0),
            active: cur(2),
        });
        assert_eq!(a.selection_text().as_deref(), Some("éà"));
        // Reversed selection yields the same text (ordered internally), no panic.
        a.buffers[0].selection = Some(Selection {
            anchor: cur(4),
            active: cur(2),
        });
        assert_eq!(a.selection_text().as_deref(), Some("ûü"));
        // Degenerate (empty) selection → empty string, no panic.
        a.buffers[0].selection = Some(Selection {
            anchor: cur(1),
            active: cur(1),
        });
        assert_eq!(a.selection_text().as_deref(), Some(""));
        // No selection → None.
        a.buffers[0].selection = None;
        assert_eq!(a.selection_text(), None);
    }

    // T021b (Feature 028): PageUp/PageDown page the encoding-select and plugin-
    // manager lists, clamped to range (no wrap).
    #[test]
    fn page_keys_clamp_encoding_select_list() {
        let mut a = make_app();
        let n = crate::ui::dialog::ENCODING_OPTIONS.len();
        a.pending_encoding_select = Some(0);
        a.handle_action(Action::MovePageDown).unwrap();
        assert_eq!(a.pending_encoding_select, Some(DIALOG_LIST_PAGE.min(n - 1)));
        // Repeated page-downs clamp to the last item.
        for _ in 0..5 {
            a.handle_action(Action::MovePageDown).unwrap();
        }
        assert_eq!(a.pending_encoding_select, Some(n - 1));
        for _ in 0..5 {
            a.handle_action(Action::MovePageUp).unwrap();
        }
        assert_eq!(a.pending_encoding_select, Some(0));
    }

    #[test]
    fn page_keys_clamp_plugin_manager_list() {
        let mut a = make_app();
        // With no plugins installed the list is empty — paging must be a safe no-op.
        a.pending_plugin_manager = true;
        a.plugin_manager_cursor = 0;
        a.handle_action(Action::MovePageDown).unwrap();
        a.handle_action(Action::MovePageUp).unwrap();
        assert_eq!(a.plugin_manager_cursor, 0);
        assert!(
            a.pending_plugin_manager,
            "list paging never closes the dialog"
        );
    }

    // T017 (Feature 028): Help scrolls from the keyboard with Home/End/Page keys,
    // clamped to the content.
    #[test]
    fn help_keyboard_scroll_clamps() {
        let mut a = make_app();
        a.terminal_size = (80, 24);
        a.pending_help = Some(HelpScreen::Help);
        let (max_scroll, _page) = a.help_view_metrics(HelpScreen::Help);
        assert!(max_scroll > 0, "Help overflows a 24-row terminal");
        // End → bottom; Home → top.
        a.handle_action(Action::MoveLineEnd).unwrap();
        assert_eq!(a.help_scroll, max_scroll);
        a.handle_action(Action::MoveLineStart).unwrap();
        assert_eq!(a.help_scroll, 0);
        // PageDown clamps to max even when pressed many times.
        for _ in 0..50 {
            a.handle_action(Action::MovePageDown).unwrap();
        }
        assert_eq!(a.help_scroll, max_scroll);
        // Down never exceeds max; Up returns toward 0.
        a.handle_action(Action::MoveDown).unwrap();
        assert_eq!(a.help_scroll, max_scroll);
        for _ in 0..200 {
            a.handle_action(Action::MoveUp).unwrap();
        }
        assert_eq!(a.help_scroll, 0);
        // Help is still open (scroll keys don't dismiss it).
        assert_eq!(a.pending_help, Some(HelpScreen::Help));
    }

    // T019 (Feature 028): Home/End move the editor cursor to line start/end.
    #[test]
    fn home_end_move_cursor_to_line_bounds() {
        let mut a = make_app();
        a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("hello world\n");
        a.active_idx = 0;
        a.handle_action(Action::MoveLineEnd).unwrap();
        assert_eq!(
            a.buffers[0].cursor.grapheme_col,
            "hello world".chars().count()
        );
        a.handle_action(Action::MoveLineStart).unwrap();
        assert_eq!(a.buffers[0].cursor.grapheme_col, 0);
    }

    // T014 (Feature 028): arrow keys move focus between buttons in a confirm dialog
    // (016 ring), consistent with Tab, with wrap-around.
    #[test]
    fn arrow_keys_move_confirm_dialog_buttons() {
        let mut a = make_app();
        // SavePrompt has 3 buttons (Save/Discard/Cancel); default focus = 2 (Cancel).
        a.pending_save_prompt = true;
        a.handle_action(Action::MoveRight).unwrap(); // ensure sets 2, then next → 0
        assert_eq!(a.dialog_focus, 0);
        a.handle_action(Action::MoveRight).unwrap(); // → 1
        assert_eq!(a.dialog_focus, 1);
        a.handle_action(Action::MoveLeft).unwrap(); // → 0
        assert_eq!(a.dialog_focus, 0);
        a.handle_action(Action::MoveLeft).unwrap(); // wrap → 2
        assert_eq!(a.dialog_focus, 2);
        // Down/Up behave like Right/Left on the single-row button bar.
        a.handle_action(Action::MoveDown).unwrap(); // wrap → 0
        assert_eq!(a.dialog_focus, 0);
        a.handle_action(Action::MoveUp).unwrap(); // wrap → 2
        assert_eq!(a.dialog_focus, 2);
    }

    // T014 (Feature 028): in an interactive dialog with a button focused, arrows
    // cycle the ring; with the primary control focused, arrows are NOT consumed by
    // the button ring (they drive the list/field).
    #[test]
    fn arrow_keys_cycle_interactive_buttons_when_button_focused() {
        use crate::ui::file_browser::{BrowseMode, FileBrowser};
        let mut a = make_app();
        a.file_browser = Some(FileBrowser::open(
            std::path::PathBuf::from("."),
            BrowseMode::Save,
        ));
        let ring = a.interactive_ring_len();
        assert!(ring >= 2, "file browser has a primary control + button(s)");
        // Focus the first button (stop 1); keep init so ensure won't reset to 0.
        a.dialog_focus = 1;
        a.dialog_focus_init = true;
        a.handle_action(Action::MoveRight).unwrap();
        assert_eq!(a.dialog_focus, crate::ui::buttons::next(1, ring));
        a.handle_action(Action::MoveLeft).unwrap();
        assert_eq!(a.dialog_focus, 1);
    }

    // T011 (Feature 028): opening an interactive dialog resets focus to the primary
    // control (stop 0), even if a previous dialog left dialog_focus on a button —
    // so typing reaches the field (the Save-As typing bug).
    #[test]
    fn interactive_dialog_opens_focused_on_primary_control() {
        use crate::ui::file_browser::{BrowseMode, FileBrowser};
        let mut a = make_app();
        // Simulate stale focus left on a button by a prior (now-closed) dialog.
        a.dialog_focus = 2;
        a.dialog_focus_init = false;
        // Open the Save browser.
        a.file_browser = Some(FileBrowser::open(
            std::path::PathBuf::from("."),
            BrowseMode::Save,
        ));
        a.ensure_dialog_focus();
        assert_eq!(a.dialog_focus, 0, "focus resets to the primary field");
        assert!(
            a.interactive_focus_is_button().is_none(),
            "primary control focused, not a button"
        );
    }

    // T007 (Feature 028): end-to-end render after a soft-wrap buffer switch with a
    // stale wrap cache must not panic — the session-restore crash exercised through
    // the real render path. The render reads `wrap_cache` as-is (the run loop's
    // rebuild has not happened yet), so the renderer's own clamp must protect it.
    #[test]
    fn render_after_softwrap_buffer_switch_with_stale_cache_no_panic() {
        use ratatui::{backend::TestBackend, Terminal};
        let mut a = make_app();
        a.terminal_size = (80, 24);
        a.soft_wrap = true;
        // Buffer 0: long content; buffer 1: short + empty lines.
        let mut b0 = crate::buffer::Buffer::new_empty();
        b0.rope = crate::buffer::rope::EditorRope::from_str(
            "this is a fairly long line that will wrap several times in a narrow pane\n",
        );
        let mut b1 = crate::buffer::Buffer::new_empty();
        b1.rope = crate::buffer::rope::EditorRope::from_str("ab\n\n");
        a.buffers = vec![b0, b1];
        a.active_idx = 0;
        // Build the wrap cache for buffer 0, then switch to buffer 1 WITHOUT a loop
        // rebuild — the cache now describes the wrong (longer) content.
        a.wrap_cache = Some(crate::ui::wrap::WrapCache::compute(
            &a.buffers[0].rope,
            20,
            a.wrap_text_gen,
        ));
        a.active_idx = 1;
        let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
        t.draw(|f| a.render(f)).unwrap(); // must not panic
    }

    // T005 (Feature 028): switching/closing the active buffer invalidates the
    // soft-wrap cache by bumping wrap_text_gen, so the renderer never reuses stale
    // per-line offsets against the new content.
    #[test]
    fn buffer_changes_invalidate_wrap_cache() {
        let mut a = make_app();
        a.buffers = vec![
            crate::buffer::Buffer::new_empty(),
            crate::buffer::Buffer::new_empty(),
        ];
        a.active_idx = 0;

        let g0 = a.wrap_text_gen;
        a.invalidate_wrap_cache();
        assert_ne!(a.wrap_text_gen, g0, "invalidate bumps the generation");

        let g1 = a.wrap_text_gen;
        a.next_buffer();
        assert_ne!(a.wrap_text_gen, g1, "next_buffer invalidates");

        let g2 = a.wrap_text_gen;
        a.prev_buffer();
        assert_ne!(a.wrap_text_gen, g2, "prev_buffer invalidates");

        let g3 = a.wrap_text_gen;
        a.close_buffer_at(1);
        assert_ne!(a.wrap_text_gen, g3, "close_buffer_at invalidates");
    }

    // T010: close_buffer_at removes the buffer and keeps the right buffer active.
    #[test]
    fn close_buffer_at_adjusts_active_index() {
        let mut a = make_app();
        // Four buffers A,B,C,D; active = C (idx 2).
        a.buffers = vec![
            crate::buffer::Buffer::new_empty(),
            crate::buffer::Buffer::new_empty(),
            crate::buffer::Buffer::new_empty(),
            crate::buffer::Buffer::new_empty(),
        ];
        for (i, b) in a.buffers.iter_mut().enumerate() {
            b.path = Some(std::path::PathBuf::from(format!("f{i}.txt")));
        }
        a.active_idx = 2;
        // Close before active → active shifts down to stay on the same buffer.
        a.close_buffer_at(0); // [f1,f2,f3], active was f2 → idx 1
        assert_eq!(a.buffers.len(), 3);
        assert_eq!(a.active_idx, 1);
        assert_eq!(
            a.buffers[a.active_idx].path.as_ref().unwrap().to_str(),
            Some("f2.txt")
        );
        // Close after active → active index unchanged.
        a.close_buffer_at(2); // remove f3 → [f1,f2], active still f2 (idx 1)
        assert_eq!(a.active_idx, 1);
        assert_eq!(
            a.buffers[a.active_idx].path.as_ref().unwrap().to_str(),
            Some("f2.txt")
        );
        // Close the active (last) → previous becomes active.
        a.close_buffer_at(1); // remove f2 → [f1], active clamps to 0
        assert_eq!(a.buffers.len(), 1);
        assert_eq!(a.active_idx, 0);
        // Closing the final buffer replaces it with an empty scratch buffer.
        a.close_buffer_at(0);
        assert_eq!(a.buffers.len(), 1);
        assert_eq!(a.active_idx, 0);
        assert!(a.buffers[0].path.is_none());
    }

    // T010: tab_close_clicked prompts for a modified buffer, closes a clean one.
    #[test]
    fn tab_close_clicked_prompts_only_when_modified() {
        let mut a = make_app();
        a.buffers = vec![
            crate::buffer::Buffer::new_empty(),
            crate::buffer::Buffer::new_empty(),
        ];
        a.buffers[1].modified = true;
        a.active_idx = 0;
        // Clean buffer (idx 0) closes immediately, no prompt.
        a.tab_close_clicked(0);
        assert_eq!(a.buffers.len(), 1);
        assert!(a.pending_close_confirm.is_none());
        // Re-create a modified second buffer; its [x] opens the confirm.
        a.buffers.push(crate::buffer::Buffer::new_empty());
        a.buffers[1].modified = true;
        a.tab_close_clicked(1);
        assert_eq!(a.buffers.len(), 2, "nothing closed yet");
        assert_eq!(a.pending_close_confirm, Some(1));
        // Discard (button 1) closes it.
        a.activate_dialog_button(1);
        assert_eq!(a.buffers.len(), 1);
        assert!(a.pending_close_confirm.is_none());
    }

    // ── Feature 025 — Go-to-Line prompt render ────────────────────────────────

    // T008/L1: the Go-to-Line overlay renders at a normal and a tiny terminal
    // without panicking, and shows its title at a normal size.
    #[test]
    fn goto_line_overlay_renders_without_panic() {
        use ratatui::{backend::TestBackend, Terminal};
        let render = |w: u16, h: u16| -> String {
            let mut a = make_app();
            a.terminal_size = (w, h);
            a.pending_goto_line = Some("42".to_string());
            let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
            t.draw(|f| a.render(f)).unwrap();
            t.backend()
                .buffer()
                .content()
                .iter()
                .map(|c| c.symbol().to_string())
                .collect()
        };
        let big = render(80, 24);
        assert!(big.contains("Go to Line"), "title shown at a normal size");
        // Tiny terminal must not panic.
        let _ = render(10, 3);
        let _ = render(4, 2);
    }

    // ── Feature 023 — mouse-wheel editor scroll ──────────────────────────────

    // T003: wheel_scroll_editor moves the viewport by the step, clamps at top and
    // bottom, and never changes the cursor.
    #[test]
    fn wheel_scroll_editor_clamps_and_keeps_cursor() {
        let mut a = make_app();
        a.terminal_size = (80, 24);
        for _ in 0..50 {
            a.handle_action(Action::InsertNewline).unwrap();
        }
        a.buffers[0].scroll_offset.0 = 0;
        a.buffers[0].cursor.line = 5;
        a.buffers[0].cursor.grapheme_col = 0;
        let cur = a.buffers[0].cursor;

        a.wheel_scroll_editor(0, true, 3);
        assert_eq!(a.buffers[0].scroll_offset.0, 3, "scrolled down by step");
        assert_eq!(a.buffers[0].cursor, cur, "cursor unchanged by wheel scroll");

        a.wheel_scroll_editor(0, false, 3);
        assert_eq!(a.buffers[0].scroll_offset.0, 0, "scrolled back up");
        a.wheel_scroll_editor(0, false, 3);
        assert_eq!(a.buffers[0].scroll_offset.0, 0, "clamped at the top");

        // Drive to the bottom and confirm the clamp.
        for _ in 0..100 {
            a.wheel_scroll_editor(0, true, 3);
        }
        let max = a.buffers[0].rope.line_count().saturating_sub(1);
        assert_eq!(a.buffers[0].scroll_offset.0, max, "clamped at the bottom");
    }

    // ── Feature 021 — editor scrollbar geometry ──────────────────────────────

    // T019: the editor renders its scrollbars (overflowing buffer) with line
    // numbers on, in split view, and across a range of sizes without panicking,
    // and a scrollbar thumb/track glyph is present.
    #[test]
    fn editor_scrollbars_render_with_gutter_split_and_resize() {
        use ratatui::{backend::TestBackend, Terminal};
        let bar_glyphs = ['█', '░', '▲', '▼', '◄', '►'];
        let render_has_bar = |app: &mut App, w: u16, h: u16| -> bool {
            let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
            t.draw(|f| app.render(f)).unwrap();
            t.backend().buffer().content().iter().any(|c| {
                c.symbol()
                    .chars()
                    .next()
                    .is_some_and(|g| bar_glyphs.contains(&g))
            })
        };
        for (w, h) in [(80u16, 24u16), (100, 40), (80, 24)] {
            let mut a = make_app();
            a.terminal_size = (w, h);
            a.config.line_numbers = true;
            // A buffer taller than the viewport → vertical scrollbar.
            for _ in 0..(h as usize + 20) {
                a.handle_action(Action::InsertNewline).unwrap();
            }
            assert!(render_has_bar(&mut a, w, h), "single view: scrollbar drawn");
            // Split view renders bars in each pane without panic.
            a.split_mode = crate::ui::SplitMode::Vertical;
            assert!(render_has_bar(&mut a, w, h), "split view: scrollbar drawn");
        }
    }

    // T007: viewport_height accounts for the reserved horizontal-scrollbar row in
    // non-wrap mode, but not in soft-wrap (no horizontal bar there).
    #[test]
    fn viewport_height_reserves_hbar_row_in_nonwrap_only() {
        let mut a = make_app();
        a.terminal_size = (80, 24);
        a.soft_wrap = false;
        assert_eq!(a.viewport_height(), 21, "24 - menu - status - hbar row");
        a.soft_wrap = true;
        assert_eq!(a.viewport_height(), 22, "soft-wrap: no horizontal bar row");
    }

    // T007: content_width reserves the rightmost vertical-scrollbar column.
    #[test]
    fn content_width_reserves_vbar_column() {
        let mut a = make_app();
        a.terminal_size = (80, 24);
        a.config.line_numbers = false;
        assert_eq!(a.content_width(), 79, "80 - vbar column");
        a.config.line_numbers = true;
        assert_eq!(a.content_width(), 75, "80 - gutter(4) - vbar column");
    }

    // T007: a click on the reserved scrollbar cells does not move the cursor.
    #[test]
    fn click_on_reserved_scrollbar_cells_is_inert() {
        let mut a = make_app();
        a.terminal_size = (80, 24);
        a.soft_wrap = false;
        for _ in 0..3 {
            a.handle_action(Action::InsertNewline).unwrap();
        }
        a.buffers[0].cursor.line = 1;
        a.buffers[0].cursor.grapheme_col = 0;
        let before = a.buffers[0].cursor;
        // Rightmost column = vertical scrollbar.
        a.handle_mouse_click(79, 3);
        assert_eq!(a.buffers[0].cursor, before, "vbar-column click ignored");
        // Bottom editor row (row 22 = terminal rows - 2) = horizontal scrollbar.
        a.handle_mouse_click(5, 22);
        assert_eq!(a.buffers[0].cursor, before, "hbar-row click ignored");
    }

    // Feature 018: Help renders a grouped Key|Action table and scrolls.
    #[test]
    fn help_renders_table_and_scrolls() {
        use ratatui::{backend::TestBackend, Terminal};
        let render = |app: &mut App| -> String {
            let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
            t.draw(|f| app.render(f)).unwrap();
            t.backend()
                .buffer()
                .content()
                .iter()
                .map(|c| c.symbol())
                .collect()
        };
        let mut app = make_app();
        app.handle_action(Action::Help).unwrap();
        let top = render(&mut app);
        assert!(top.contains("File"), "section heading shown");
        assert!(top.contains("Ctrl+S"), "a key row shown");
        assert!(
            top.contains("scroll"),
            "scroll hint shown when content overflows"
        );
        assert!(
            !top.contains("Dialogs"),
            "later section not visible before scrolling"
        );

        // Scroll down a lot; the last section becomes visible.
        for _ in 0..40 {
            app.handle_action(Action::MoveDown).unwrap();
        }
        let bottom = render(&mut app);
        assert!(
            bottom.contains("Dialogs"),
            "scrolling reveals later sections"
        );
    }

    // Feature 019: the Find dialog renders its query in a labeled, bordered
    // input box with a caret (matching the file-browser box from feature 018).
    #[test]
    fn find_dialog_renders_bordered_box_with_caret() {
        use ratatui::{backend::TestBackend, Terminal};
        let mut app = make_app();
        app.handle_action(Action::Find).unwrap();
        for c in "needle".chars() {
            app.handle_action(Action::InsertChar(c)).unwrap();
        }
        let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
        t.draw(|f| app.render(f)).unwrap();
        let s: String = t
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(
            s.contains('┌') && s.contains('└') && s.contains('│'),
            "field box borders drawn"
        );
        assert!(s.contains("Find what:"), "field label shown");
        assert!(s.contains('▏'), "caret glyph shown in the focused field");
        assert!(s.contains("needle"), "typed query shown in the box");
        for label in ["Case", "Wrap", "Regex", "Word"] {
            assert!(s.contains(label), "option {label} still shown");
        }
        assert!(s.contains("Esc close"), "hint row still shown");
    }

    // Feature 019: the Replace dialog renders BOTH fields as bordered boxes; the
    // caret appears only in the focused field (FR-005 / contract C-2).
    #[test]
    fn replace_dialog_renders_two_boxes_and_focused_caret() {
        use ratatui::{backend::TestBackend, Terminal};
        let render = |app: &mut App| -> String {
            let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
            t.draw(|f| app.render(f)).unwrap();
            t.backend()
                .buffer()
                .content()
                .iter()
                .map(|c| c.symbol())
                .collect()
        };
        let mut app = make_app();
        app.handle_action(Action::FindReplace).unwrap();
        for c in "foo".chars() {
            app.handle_action(Action::InsertChar(c)).unwrap();
        }
        let s = render(&mut app);
        assert!(
            s.contains("Find what:") && s.contains("Replace with:"),
            "both field labels shown"
        );
        // Two bordered boxes => at least two top-left corners.
        assert!(s.matches('┌').count() >= 2, "two field boxes drawn");
        assert_eq!(
            s.matches('▏').count(),
            1,
            "exactly one caret (focused field only)"
        );

        // Switching focus to the replacement field moves the (single) caret there.
        app.handle_action(Action::FocusNextField).unwrap();
        for c in "bar".chars() {
            app.handle_action(Action::InsertChar(c)).unwrap();
        }
        let s2 = render(&mut app);
        assert_eq!(
            s2.matches('▏').count(),
            1,
            "still exactly one caret after Tab"
        );
        assert!(s2.contains("bar"), "replacement text rendered in its box");
    }

    // Feature 019: the taller boxed Replace dialog must render fully within the
    // frame at the minimum supported terminal size without panic (FR-009 /
    // contract C-5). Below the minimum the app shows its "too small" guard
    // instead, so the boundary case is exactly MIN_WIDTH x MIN_HEIGHT.
    #[test]
    fn replace_dialog_renders_at_minimum_terminal() {
        use ratatui::{backend::TestBackend, Terminal};
        let mut app = make_app();
        app.handle_action(Action::FindReplace).unwrap();
        app.handle_action(Action::InsertChar('x')).unwrap();
        let mut t = Terminal::new(TestBackend::new(MIN_WIDTH, MIN_HEIGHT)).unwrap();
        t.draw(|f| app.render(f)).unwrap();
        let s: String = t
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        // Both boxes plus the hint fit at the minimum size.
        assert!(
            s.matches('┌').count() >= 2,
            "both field boxes drawn at min size"
        );
        assert!(
            s.contains("Replace with:"),
            "second field present at min size"
        );
        assert!(s.contains("Esc close"), "hint row present at min size");
    }

    // Feature 017: Select All renders the selected text with reverse-video.
    #[test]
    fn select_all_renders_reverse_highlight() {
        use ratatui::{backend::TestBackend, Terminal};
        let mut app = make_app();
        for c in "hello".chars() {
            app.handle_action(Action::InsertChar(c)).unwrap();
        }
        app.handle_action(Action::SelectAll).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| app.render(f)).unwrap();
        let buf = terminal.backend().buffer();
        // Editor content starts at row 1 (row 0 is the menu bar); no gutter.
        let cell = buf.get(0, 1);
        assert_eq!(cell.symbol(), "h");
        assert!(
            cell.style()
                .add_modifier
                .contains(ratatui::style::Modifier::REVERSED),
            "selected cell rendered with reverse video"
        );
        // A clean buffer (no selection) has no reversed content cell.
        let mut app2 = make_app();
        app2.handle_action(Action::InsertChar('h')).unwrap();
        let mut t2 = Terminal::new(TestBackend::new(80, 24)).unwrap();
        t2.draw(|f| app2.render(f)).unwrap();
        assert!(
            !t2.backend()
                .buffer()
                .get(0, 1)
                .style()
                .add_modifier
                .contains(ratatui::style::Modifier::REVERSED),
            "no selection → no reverse highlight"
        );
    }

    // Regression (Feature 012 follow-up): render must sync `terminal_size` to the
    // real frame so mouse hit-testing uses the same geometry that is drawn. When
    // it was stale, clicks inside the visible file-browser box on a non-80x24
    // terminal mapped to "outside" and closed the dialog.
    #[test]
    fn render_syncs_terminal_size_to_frame() {
        use ratatui::{backend::TestBackend, Terminal};
        let mut app = make_app();
        app.terminal_size = (80, 24); // stale default
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        terminal.draw(|f| app.render(f)).unwrap();
        assert_eq!(
            app.terminal_size,
            (120, 40),
            "terminal_size must follow the actual frame size"
        );
    }

    // Feature 016: the save prompt renders boxed, focusable buttons.
    #[test]
    fn save_prompt_renders_boxed_buttons() {
        use ratatui::{backend::TestBackend, Terminal};
        let mut app = make_app();
        app.handle_action(Action::InsertChar('x')).unwrap();
        app.handle_action(Action::Quit).unwrap();
        assert!(app.pending_save_prompt);
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| app.render(f)).unwrap();
        let content: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(content.contains("Save"), "Save button label drawn");
        assert!(content.contains("Discard"), "Discard button label drawn");
        assert!(content.contains("Cancel"), "Cancel button label drawn");
        assert!(content.contains('▶'), "focused-button marker drawn");
        assert!(
            content.contains('┌') && content.contains('│'),
            "boxed button borders drawn"
        );
    }

    fn make_app_with_encoding(enc: EncodingId) -> App {
        let mut app = make_app();
        app.buffers[0].encoding = enc;
        app
    }

    // ── T016 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_save_as_encoding_action_opens_dialog() {
        let mut app = make_app(); // UTF-8 buffer (index 0 in ENCODING_OPTIONS)
        app.handle_action(Action::SaveAsEncoding).unwrap();
        assert_eq!(app.pending_encoding_select, Some(0));
    }

    #[test]
    fn test_dialog_preselects_current_encoding() {
        let mut app = make_app_with_encoding(EncodingId::Utf16Le); // index 1
        app.handle_action(Action::SaveAsEncoding).unwrap();
        assert_eq!(app.pending_encoding_select, Some(1));
    }

    #[test]
    fn test_dialog_move_down_increments_idx() {
        let mut app = make_app();
        app.pending_encoding_select = Some(1);
        app.handle_action(Action::MoveDown).unwrap();
        assert_eq!(app.pending_encoding_select, Some(2));
    }

    #[test]
    fn test_dialog_move_down_wraps_at_end() {
        let mut app = make_app();
        app.pending_encoding_select = Some(6); // last item
        app.handle_action(Action::MoveDown).unwrap();
        assert_eq!(app.pending_encoding_select, Some(0));
    }

    #[test]
    fn test_dialog_move_up_wraps_at_start() {
        let mut app = make_app();
        app.pending_encoding_select = Some(0);
        app.handle_action(Action::MoveUp).unwrap();
        assert_eq!(app.pending_encoding_select, Some(6));
    }

    #[test]
    fn test_dialog_escape_closes() {
        let mut app = make_app();
        app.pending_encoding_select = Some(3);
        app.handle_action(Action::MenuClose).unwrap();
        assert_eq!(app.pending_encoding_select, None);
    }

    #[test]
    fn test_dialog_other_action_consumed() {
        let mut app = make_app();
        app.pending_encoding_select = Some(2);
        let gcol_before = app.buffers[0].cursor.grapheme_col;
        app.handle_action(Action::MoveLeft).unwrap();
        // Dialog state must be preserved (action consumed, not passed to editor).
        assert_eq!(app.pending_encoding_select, Some(2));
        // Cursor must not have moved.
        assert_eq!(app.buffers[0].cursor.grapheme_col, gcol_before);
    }

    // ── Feature 012 — File browser (Open) ───────────────────────────────────

    #[test]
    fn test_open_action_opens_browser() {
        let mut app = make_app();
        assert!(app.file_browser.is_none());
        app.handle_action(Action::Open).unwrap();
        let fb = app.file_browser.as_ref().expect("browser open");
        assert_eq!(fb.mode, BrowseMode::Open);
    }

    #[test]
    fn test_browser_typing_edits_field() {
        let mut app = make_app();
        app.handle_action(Action::Open).unwrap();
        for c in "ab".chars() {
            app.handle_action(Action::InsertChar(c)).unwrap();
        }
        assert_eq!(app.file_browser.as_ref().unwrap().filename, "ab");
        app.handle_action(Action::Backspace).unwrap();
        assert_eq!(app.file_browser.as_ref().unwrap().filename, "a");
    }

    #[test]
    fn test_browser_escape_cancels_without_opening() {
        let mut app = make_app();
        let n_before = app.buffers.len();
        app.handle_action(Action::Open).unwrap();
        app.handle_action(Action::MenuClose).unwrap();
        assert!(app.file_browser.is_none());
        assert_eq!(app.buffers.len(), n_before, "cancel must not open a buffer");
    }

    #[test]
    fn test_browser_inert_action_keeps_open() {
        let mut app = make_app();
        app.handle_action(Action::Open).unwrap();
        let gcol_before = app.buffers[0].cursor.grapheme_col;
        // ToggleHighlight is consumed by the browser intercept (no effect).
        app.handle_action(Action::ToggleHighlight).unwrap();
        assert!(app.file_browser.is_some());
        assert_eq!(app.buffers[0].cursor.grapheme_col, gcol_before);
    }

    #[test]
    fn test_browser_typed_path_opens_file() {
        // Open mode: typing an absolute file path + Enter loads it (FR-006a).
        let mut path = std::env::temp_dir();
        path.push("edit_browser_open_test_012.txt");
        std::fs::write(&path, "hello from disk\n").expect("write temp file");

        let mut app = make_app();
        let n_before = app.buffers.len();
        app.handle_action(Action::Open).unwrap();
        for c in path.to_string_lossy().chars() {
            app.handle_action(Action::InsertChar(c)).unwrap();
        }
        app.handle_action(Action::InsertNewline).unwrap();

        assert!(app.file_browser.is_none(), "browser closes after opening");
        assert_eq!(app.buffers.len(), n_before + 1, "a new buffer is added");
        assert!(app
            .active_buffer()
            .rope
            .to_string()
            .contains("hello from disk"));

        let _ = std::fs::remove_file(&path);
    }

    // ── Feature 011 — wired menu actions ─────────────────────────────────────

    #[test]
    fn test_undo_redo_round_trip() {
        let mut app = make_app();
        app.insert_char('a');
        app.insert_char('b');
        assert_eq!(app.active_buffer().rope.to_string(), "ab");
        app.handle_action(Action::Undo).unwrap();
        assert_eq!(app.active_buffer().rope.to_string(), "a");
        app.handle_action(Action::Redo).unwrap();
        assert_eq!(app.active_buffer().rope.to_string(), "ab");
    }

    #[test]
    fn test_undo_empty_reports_nothing() {
        let mut app = make_app();
        app.handle_action(Action::Undo).unwrap();
        assert_eq!(app.status_message.as_deref(), Some("Nothing to undo"));
    }

    #[test]
    fn test_select_all_spans_buffer() {
        let mut app = make_app();
        app.insert_char('x');
        app.insert_char('y');
        app.handle_action(Action::SelectAll).unwrap();
        let sel = app.active_buffer().selection.expect("selection set");
        assert_eq!(sel.anchor.line, 0);
        assert_eq!(sel.anchor.grapheme_col, 0);
        assert_eq!(sel.active.grapheme_col, 2);
    }

    #[test]
    fn test_cut_deletes_selection_without_clipboard() {
        // cut_selection copies (may no-op headless) then deletes — the delete
        // must happen regardless of clipboard availability.
        let mut app = make_app();
        app.insert_char('x');
        app.insert_char('y');
        app.handle_action(Action::SelectAll).unwrap();
        app.handle_action(Action::Cut).unwrap();
        assert_eq!(app.active_buffer().rope.to_string(), "");
    }

    #[test]
    fn test_new_buffer_action_adds_buffer() {
        let mut app = make_app();
        let n = app.buffers.len();
        app.handle_action(Action::New).unwrap();
        assert_eq!(app.buffers.len(), n + 1);
        assert_eq!(app.active_idx, app.buffers.len() - 1);
    }

    #[test]
    fn test_toggle_line_numbers_flips_config() {
        let mut app = make_app();
        let before = app.config.line_numbers;
        app.handle_action(Action::ToggleLineNumbers).unwrap();
        assert_eq!(app.config.line_numbers, !before);
    }

    #[test]
    fn test_about_action_opens_and_closes() {
        let mut app = make_app();
        app.handle_action(Action::About).unwrap();
        assert_eq!(app.pending_help, Some(HelpScreen::About));
        app.handle_action(Action::MenuClose).unwrap();
        assert_eq!(app.pending_help, None);
    }

    #[test]
    fn test_save_browser_writes_file() {
        let dir = std::env::temp_dir().join("edit_saveas_test_012");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("saved.txt");
        let _ = std::fs::remove_file(&path);

        let mut app = make_app();
        app.insert_char('h');
        app.insert_char('i');
        app.handle_action(Action::SaveAs).unwrap();
        assert_eq!(app.file_browser.as_ref().unwrap().mode, BrowseMode::Save);
        // Point the browser at the temp dir, type a filename, confirm.
        app.file_browser = Some(FileBrowser::open(dir.clone(), BrowseMode::Save));
        for c in "saved.txt".chars() {
            app.handle_action(Action::InsertChar(c)).unwrap();
        }
        app.handle_action(Action::InsertNewline).unwrap();

        assert!(app.file_browser.is_none(), "browser closes after save");
        let written = std::fs::read_to_string(&path).expect("file written");
        assert!(written.contains("hi"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_unnamed_buffer_opens_save_browser() {
        let mut app = make_app(); // make_app starts with an unnamed buffer
        assert!(app.active_buffer().path.is_none());
        app.handle_action(Action::Save).unwrap();
        let fb = app.file_browser.as_ref().expect("save browser opened");
        assert_eq!(fb.mode, BrowseMode::Save);
    }

    // ── Feature 011 — mouse menu interaction ─────────────────────────────────

    fn mouse_press(col: u16, row: u16) -> crossterm::event::MouseEvent {
        crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }
    }

    #[test]
    fn test_mouse_click_opens_top_level_menu() {
        let mut app = make_app();
        // Click "Edit" (col 7, row 0).
        app.handle_mouse_event(mouse_press(7, 0)).unwrap();
        assert!(matches!(
            app.menu_bar.state,
            MenuState::DropDown { top_idx: 1, .. }
        ));
    }

    #[test]
    fn test_mouse_click_activates_dropdown_item() {
        let mut app = make_app();
        // Open File menu, then click the "Open" item (row 2).
        app.handle_mouse_event(mouse_press(1, 0)).unwrap();
        app.handle_mouse_event(mouse_press(3, 2)).unwrap();
        // "Open" → Action::Open → opens the file browser and closes the menu.
        assert!(app.file_browser.is_some());
        assert!(!app.menu_bar.is_active());
    }

    #[test]
    fn test_mouse_click_outside_closes_menu() {
        let mut app = make_app();
        app.handle_mouse_event(mouse_press(1, 0)).unwrap(); // open File
        assert!(app.menu_bar.is_active());
        // Click far down in the editor area.
        app.handle_mouse_event(mouse_press(40, 12)).unwrap();
        assert!(!app.menu_bar.is_active());
    }

    // ── T017 — Cancel contract ──────────────────────────────────────────────

    #[test]
    fn test_cancel_does_not_write_and_leaves_encoding_unchanged() {
        let mut app = make_app();
        // Start with UTF-8 encoding.
        assert_eq!(app.buffers[0].encoding, EncodingId::Utf8);
        app.pending_encoding_select = Some(3); // e.g. CP437 selected
                                               // Cancel via MenuClose.
        app.handle_action(Action::MenuClose).unwrap();
        // Dialog closed.
        assert_eq!(app.pending_encoding_select, None);
        // Encoding unchanged.
        assert_eq!(app.buffers[0].encoding, EncodingId::Utf8);
        // No status message about encoding change.
        assert!(
            app.status_message
                .as_deref()
                .is_none_or(|m| !m.starts_with("Saved as")),
            "cancel must not produce a 'Saved as' message"
        );
    }

    // ── T018 — Encoding persistence ─────────────────────────────────────────

    #[test]
    fn test_encoding_persists_on_regular_save() {
        let path = std::env::temp_dir().join("edit_test_persist.txt");
        std::fs::write(&path, b"Hello").unwrap();

        let mut app = App::new(
            Config::default(),
            vec![path.clone()],
            EncodingId::Utf8,
            None,
            None,
        );
        // Save as UTF-16 LE via do_save_as_encoding (Case A).
        app.do_save_as_encoding(EncodingId::Utf16Le);
        // Subsequent regular save must use the new encoding.
        app.buffers[0].save().unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            bytes[0..2],
            [0xFF, 0xFE],
            "file must start with UTF-16 LE BOM"
        );
    }

    // ── T019 — Dialog reopens with updated preselect ─────────────────────────

    #[test]
    fn test_dialog_reopens_with_updated_preselect() {
        let path = std::env::temp_dir().join("edit_test_preselect.txt");
        std::fs::write(&path, b"Hello").unwrap();

        let mut app = App::new(
            Config::default(),
            vec![path.clone()],
            EncodingId::Utf8,
            None,
            None,
        );
        app.do_save_as_encoding(EncodingId::Utf16Be);
        let _ = std::fs::remove_file(&path);
        // Re-open dialog — must pre-select UTF-16 BE (index 2).
        app.handle_action(Action::SaveAsEncoding).unwrap();
        assert_eq!(app.pending_encoding_select, Some(2));
    }

    // ── T022 — Pending encoding cleared on filename-prompt cancel ────────────

    #[test]
    fn test_unnamed_buf_encoding_cleared_on_filename_cancel() {
        let mut app = make_app(); // unnamed buffer
        app.pending_save_as_encoding = Some(EncodingId::Utf16Le);
        app.cancel_pending_save_as_encoding();
        assert_eq!(app.pending_save_as_encoding, None);
    }

    #[test]
    fn test_unnamed_buf_encoding_applied_after_filename_confirm() {
        let path = std::env::temp_dir().join("edit_test_t022_confirm.txt");
        let mut app = make_app(); // unnamed buffer

        // Simulate: user selected UTF-16 LE via encoding dialog for unnamed buf.
        app.pending_save_as_encoding = Some(EncodingId::Utf16Le);

        // Simulate: user typed a filename and confirmed → handle_save_as called.
        let result = app.handle_save_as(path.clone());
        // The write may fail (no actual FS write in make_app), but the
        // encoding assignment happens before the write. We care that
        // pending_save_as_encoding was consumed and the buffer encoding set.
        assert_eq!(
            app.pending_save_as_encoding, None,
            "pending must be cleared"
        );
        assert_eq!(
            app.active_buffer().encoding,
            EncodingId::Utf16Le,
            "buffer encoding must be updated even if write fails"
        );
        let _ = std::fs::remove_file(&path);
        let _ = result; // allow write failure (unnamed buf has no content path)
    }

    // ── Feature 005 — Soft-wrap tests (T024, T025) ────────────────────────────

    fn make_app_with_long_line() -> App {
        let mut app = make_app();
        // Insert a 60-grapheme line to test soft-wrap
        let long = "A".repeat(60);
        let char_idx = 0;
        app.buffers[0].rope.insert_str(char_idx, &long);
        app.buffers[0].modified = true;
        app.wrap_text_gen = app.wrap_text_gen.wrapping_add(1);
        app
    }

    #[test]
    fn toggle_soft_wrap_on_builds_cache() {
        let mut app = make_app();
        app.terminal_size = (80, 24);
        // Default: soft_wrap is false, no cache.
        assert!(!app.soft_wrap);
        assert!(app.wrap_cache.is_none());
        // Toggle on.
        app.handle_action(Action::ToggleSoftWrap).unwrap();
        assert!(app.soft_wrap, "soft_wrap must be true after toggle");
        assert!(
            app.wrap_cache.is_some(),
            "wrap_cache must be Some after enabling"
        );
    }

    #[test]
    fn toggle_soft_wrap_off_drops_cache_and_resets_hscroll() {
        let mut app = make_app();
        app.terminal_size = (80, 24);
        // Enable then disable.
        app.handle_action(Action::ToggleSoftWrap).unwrap();
        app.buffers[0].scroll_offset.1 = 10; // simulate horizontal scroll while on
        app.handle_action(Action::ToggleSoftWrap).unwrap();
        assert!(
            !app.soft_wrap,
            "soft_wrap must be false after second toggle"
        );
        assert!(
            app.wrap_cache.is_none(),
            "wrap_cache must be None after disabling"
        );
        assert_eq!(
            app.buffers[0].scroll_offset.1, 0,
            "h-scroll must be reset on disable"
        );
    }

    #[test]
    fn soft_wrap_toggle_cycle_cursor_unchanged() {
        let mut app = make_app_with_long_line();
        app.terminal_size = (40, 24);
        // Move cursor to col 5.
        for _ in 0..5 {
            app.move_cursor(Direction::Right);
        }
        let cursor_before = app.buffers[0].cursor;
        // Enable wrap.
        app.handle_action(Action::ToggleSoftWrap).unwrap();
        // Disable wrap.
        app.handle_action(Action::ToggleSoftWrap).unwrap();
        let cursor_after = app.buffers[0].cursor;
        assert_eq!(
            cursor_before.line, cursor_after.line,
            "line must be unchanged"
        );
        assert_eq!(
            cursor_before.grapheme_col, cursor_after.grapheme_col,
            "gcol must be unchanged"
        );
    }

    #[test]
    fn home_on_wrapped_line_goes_to_logical_col_zero() {
        let mut app = make_app();
        app.terminal_size = (20, 24);
        // Insert 50 chars so line wraps multiple times at width 20.
        let long = "ABCDEFGHIJ".repeat(5); // 50 chars
        app.buffers[0].rope.insert_str(0, &long);
        app.buffers[0].modified = true;
        app.wrap_text_gen += 1;
        // Move cursor to middle.
        for _ in 0..25 {
            app.move_cursor(Direction::Right);
        }
        // Enable wrap.
        app.handle_action(Action::ToggleSoftWrap).unwrap();
        // Home should go to grapheme_col 0 of the logical line.
        app.move_line_start();
        assert_eq!(
            app.buffers[0].cursor.grapheme_col, 0,
            "Home must go to col 0 of logical line"
        );
        assert_eq!(app.buffers[0].cursor.line, 0, "line must remain 0");
    }

    #[test]
    fn end_on_wrapped_line_goes_to_logical_line_end() {
        let mut app = make_app();
        app.terminal_size = (20, 24);
        let long = "ABCDEFGHIJ".repeat(5); // 50 chars
        app.buffers[0].rope.insert_str(0, &long);
        app.buffers[0].modified = true;
        app.wrap_text_gen += 1;
        app.handle_action(Action::ToggleSoftWrap).unwrap();
        app.move_line_end();
        assert_eq!(app.buffers[0].cursor.line, 0, "line must remain 0");
        assert_eq!(
            app.buffers[0].cursor.grapheme_col, 50,
            "End must go to col 50"
        );
    }

    #[test]
    fn up_down_move_between_logical_lines_in_wrap_mode() {
        let mut app = make_app();
        app.terminal_size = (20, 24);
        // Line 0: 50 chars (wraps), Line 1: "Second"
        let long = "A".repeat(50);
        app.buffers[0].rope.insert_str(0, &(long + "\nSecond"));
        app.buffers[0].modified = true;
        app.wrap_text_gen += 1;
        app.handle_action(Action::ToggleSoftWrap).unwrap();
        // Cursor on line 0, col 0.
        assert_eq!(app.buffers[0].cursor.line, 0);
        // Down should go to line 1 (the logical next line).
        app.move_cursor(Direction::Down);
        assert_eq!(
            app.buffers[0].cursor.line, 1,
            "Down must go to logical line 1"
        );
    }

    #[test]
    fn save_while_soft_wrap_active_no_extra_newlines() {
        let dir = std::env::temp_dir();
        let path = dir.join("edit_soft_wrap_save_test.txt");
        let content = "A".repeat(200);
        std::fs::write(&path, &content).unwrap();

        let mut app = App::new(
            Config::default(),
            vec![path.clone()],
            EncodingId::Utf8,
            None,
            None,
        );
        app.terminal_size = (40, 24);
        app.handle_action(Action::ToggleSoftWrap).unwrap();
        assert!(app.soft_wrap, "soft_wrap must be enabled");

        // Save.
        app.handle_save_action();

        let saved = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            saved, content,
            "saved bytes must be identical to original content"
        );
    }
}
