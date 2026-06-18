//! Application state machine and top-level event dispatch.
//!
//! [`App`] owns all editor state and drives the main event loop.

#![allow(dead_code, unused_variables, unused_imports)]

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    buffer::undo::EditOp,
    buffer::{Buffer, CursorPos},
    config::Config,
    encoding::EncodingId,
    input::{dispatch_event, Action, KeybindingMap},
    search::{SearchEngine, SearchState},
    ui::menubar::MenuBarState,
    ui::theme::{theme_by_name, Theme},
};

/// Minimum terminal dimensions supported by the editor.
const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;

/// Tick interval for autosave and status-bar refresh.
const TICK_MS: u64 = 500;

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
    /// Encoding selected in the dialog, held across the filename prompt (US4).
    pub pending_save_as_encoding: Option<EncodingId>,
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
        let keymap = {
            let mut km = KeybindingMap::default_map();
            km.apply_user_overrides(&config.keybindings);
            km
        };
        let theme = theme_by_name(&config.theme);
        let readonly = config.readonly;

        let mut buffers: Vec<Buffer> = if files.is_empty() {
            vec![Buffer::new_empty()]
        } else {
            files
                .into_iter()
                .map(|p| {
                    Buffer::open(p.clone(), default_encoding).unwrap_or_else(|e| {
                        // New file (NotFound) — open an empty buffer at that path so
                        // Ctrl+S creates it. All other errors get an untitled buffer.
                        if matches!(&e, crate::buffer::BufferError::Io(io_err)
                            if io_err.kind() == io::ErrorKind::NotFound)
                        {
                            log::info!("New file: {:?}", p);
                            let mut buf = Buffer::new_empty();
                            buf.path = Some(p);
                            buf
                        } else {
                            log::error!("Failed to open {:?}: {}", p, e);
                            Buffer::new_empty()
                        }
                    })
                })
                .collect()
        };

        // T077 — Auto-detect syntax highlighter for each buffer on startup.
        if config.highlight {
            for buf in &mut buffers {
                if let Some(ref path) = buf.path.clone() {
                    buf.syntax = crate::highlight::detect_highlighter(path);
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
        let initial_status = if session_warning.is_some() && session.is_none() {
            session_warning
        } else {
            None
        };

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
            status_message: initial_status,
            pending_session_restore: session,
            default_encoding,
            pending_encoding_select: None,
            pending_save_as_encoding: None,
        }
    }

    /// Enter raw mode, start the event loop, and restore the terminal on exit.
    pub fn run(mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);

        // T064 — Clean up lock files on exit.
        for buf in &self.buffers {
            if buf.autosave.enabled && !buf.autosave.lock_path.as_os_str().is_empty() {
                crate::buffer::autosave::release_lock(&buf.autosave.lock_path);
            }
        }

        disable_raw_mode()?;
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

    /// Viewport height in lines (terminal rows minus menubar and statusbar).
    fn viewport_height(&self) -> usize {
        (self.terminal_size.1 as usize).saturating_sub(2)
    }

    // ── Event loop ───────────────────────────────────────────────────────────

    fn event_loop<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> io::Result<()> {
        let mut last_tick = Instant::now();

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            let timeout = TICK_MS.saturating_sub(last_tick.elapsed().as_millis() as u64);

            if event::poll(Duration::from_millis(timeout))? {
                let ev = event::read()?;
                if let Some(action) = dispatch_event(ev, &self.keymap) {
                    self.handle_action(action)?;
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

    fn render(&self, frame: &mut ratatui::Frame) {
        let size = frame.size();

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

    pub fn handle_action(&mut self, action: Action) -> io::Result<()> {
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
                _ => {}
            }
            return Ok(());
        }

        // T012 — Encoding-dialog intercept: when the dialog is open, only
        // Up/Down (navigate), Enter (confirm), and Esc/MenuClose (cancel) are
        // processed; all other actions are silently consumed.
        if let Some(idx) = self.pending_encoding_select {
            let n = crate::ui::dialog::ENCODING_OPTIONS.len();
            match &action {
                Action::MoveUp => {
                    self.pending_encoding_select = Some((idx + n - 1) % n);
                }
                Action::MoveDown => {
                    self.pending_encoding_select = Some((idx + 1) % n);
                }
                Action::InsertNewline => {
                    let enc = crate::ui::dialog::ENCODING_OPTIONS[idx].0;
                    self.pending_encoding_select = None;
                    self.do_save_as_encoding(enc);
                }
                Action::MenuClose => {
                    self.pending_encoding_select = None;
                }
                _ => {}
            }
            return Ok(());
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

            // Text insertion — T026
            Action::InsertChar(c) => self.insert_char(c),
            Action::InsertNewline => self.insert_newline(),

            // Deletion — T027
            Action::Backspace => self.delete_backward(),
            Action::Delete => self.delete_forward(),

            // Save prompt responses (T033)
            Action::Save => self.handle_save_action(),

            // T013 — Save As Encoding dialog trigger
            Action::SaveAsEncoding => {
                if !self.buffers.is_empty() {
                    let idx = Self::encoding_to_idx(self.buffers[self.active_idx].encoding);
                    self.pending_encoding_select = Some(idx);
                }
            }

            // Search and replace (T055 / T057)
            Action::Find => {
                self.search_state = SearchState::default();
                // TODO: show FindDialog overlay (full modal input in future task)
                log::debug!("Find action: search state reset");
            }
            Action::FindNext => self.find_next(),
            Action::FindPrev => self.find_prev(),
            Action::FindReplace => {
                // TODO: show ReplaceDialog overlay (full modal input in future task)
                log::debug!("FindReplace action triggered");
            }

            // Menu navigation (T048)
            Action::MenuFile => self.menu_bar.open_menu(0),
            Action::MenuEdit => self.menu_bar.open_menu(1),
            Action::MenuSearch => self.menu_bar.open_menu(2),
            Action::MenuView => self.menu_bar.open_menu(3),
            Action::MenuOptions => self.menu_bar.open_menu(4),
            Action::MenuHelp => self.menu_bar.open_menu(5),
            Action::MenuClose => self.menu_bar.close_menu(),
            Action::Menu => self.menu_bar.open_menu(0),
            Action::MenuOpen(idx) => self.menu_bar.open_menu(idx),

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
                if self.active_idx > 0 { 1 } else { 0 }
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
                        &buf.rope, clamped_line, clamped_gcol,
                    );
                    buf.cursor = crate::buffer::CursorPos {
                        line: clamped_line,
                        grapheme_col: clamped_gcol,
                        visual_col: vcol,
                    };
                    // Apply syntax highlighting if configured.
                    if self.config.highlight {
                        if let Some(ref path) = buf.path.clone() {
                            buf.syntax = crate::highlight::detect_highlighter(path);
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

        // Replace buffers with restored set.
        self.buffers = new_buffers;

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
    fn handle_save_action(&mut self) {
        match self.active_buffer().save() {
            Ok(()) => {
                self.active_buffer_mut().modified = false;
                log::info!("Buffer saved");
            }
            Err(e) => {
                log::error!("Save failed: {}", e);
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

        // Re-clamp scroll offset so cursor stays visible after resize.
        self.clamp_scroll();
    }

    fn handle_tick(&mut self) {
        // US5 — Autosave: check EDIT_AUTOSAVE_INTERVAL env override, then write
        // a recovery file if the active buffer is modified and the interval has elapsed.
        let interval = std::env::var("EDIT_AUTOSAVE_INTERVAL")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(self.config.autosave_interval)
            .clamp(1, 300);

        // Skip if config says no-autosave.
        if self.config.no_autosave {
            return;
        }

        let buf = &self.buffers[self.active_idx];
        if buf.autosave.enabled && buf.modified {
            let elapsed = buf.autosave.last_save_at.elapsed().as_secs() as u32;
            if elapsed >= interval {
                crate::buffer::autosave::write_recovery_for_buffer(
                    &mut self.buffers[self.active_idx],
                    interval,
                );
            }
        }
    }

    // ── T025 — Cursor movement ────────────────────────────────────────────────

    /// Move the cursor one step in `dir`, clamping to valid positions and
    /// updating `scroll_offset` as necessary.
    pub fn move_cursor(&mut self, dir: Direction) {
        let buf = &self.buffers[self.active_idx];
        let line_count = buf.rope.line_count();
        let cur = buf.cursor;

        let (new_line, new_gcol) = match dir {
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
        };

        let new_vcol = CursorPos::visual_col_from_grapheme_col(
            &self.buffers[self.active_idx].rope,
            new_line,
            new_gcol,
        );

        let buf = &mut self.buffers[self.active_idx];
        buf.cursor = CursorPos {
            line: new_line,
            grapheme_col: new_gcol,
            visual_col: new_vcol,
        };

        self.clamp_scroll();
    }

    /// Move the cursor to column 0 of the current line.
    pub fn move_line_start(&mut self) {
        let buf = &mut self.buffers[self.active_idx];
        buf.cursor.grapheme_col = 0;
        buf.cursor.visual_col = 0;
        self.clamp_scroll();
    }

    /// Move the cursor to the last grapheme of the current line.
    pub fn move_line_end(&mut self) {
        let (line, rope) = {
            let buf = &self.buffers[self.active_idx];
            (buf.cursor.line, &buf.rope as *const _)
        };
        // SAFETY: we borrow the rope immutably and the buffer mutably in sequence.
        let rope: &crate::buffer::rope::EditorRope = unsafe { &*rope };
        let gcol = rope.grapheme_count_on_line(line);
        let vcol = CursorPos::visual_col_from_grapheme_col(rope, line, gcol);

        let buf = &mut self.buffers[self.active_idx];
        buf.cursor.grapheme_col = gcol;
        buf.cursor.visual_col = vcol;
        self.clamp_scroll();
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

    /// Adjust `scroll_offset` so that `cursor` is within the visible viewport.
    fn clamp_scroll(&mut self) {
        let vh = self.viewport_height();
        let buf = &mut self.buffers[self.active_idx];
        let cur_line = buf.cursor.line;

        // Vertical scroll
        if cur_line < buf.scroll_offset.0 {
            buf.scroll_offset.0 = cur_line;
        } else if cur_line >= buf.scroll_offset.0 + vh {
            buf.scroll_offset.0 = cur_line.saturating_sub(vh - 1);
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

    // ── T026 — Text insertion ─────────────────────────────────────────────────

    /// Insert a single character at the cursor. No-op when buffer is read-only.
    pub fn insert_char(&mut self, c: char) {
        if self.buffers[self.active_idx].readonly {
            return;
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

        // Advance cursor right by one grapheme
        self.move_cursor(Direction::Right);
    }

    /// Insert a newline at the cursor, placing the cursor at column 0 of the
    /// new line.
    pub fn insert_newline(&mut self) {
        if self.buffers[self.active_idx].readonly {
            return;
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

        self.clamp_scroll();
    }

    // ── T027 — Backspace and Delete ───────────────────────────────────────────

    /// Delete the grapheme cluster immediately before the cursor.
    /// No-op at the start of the buffer or when read-only.
    pub fn delete_backward(&mut self) {
        if self.buffers[self.active_idx].readonly {
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
        if self.buffers[self.active_idx].readonly {
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

        let buf = &mut self.buffers[self.active_idx];
        buf.rope
            .delete_range(del_char_idx..del_char_idx + del_char_len);
        buf.undo_stack.push(EditOp::Delete {
            at: del_char_idx,
            text: deleted_text,
        });
        buf.modified = true;
        // Cursor position stays the same after forward-delete
    }

    // ── T102 — Clipboard cut / copy / paste ──────────────────────────────────

    /// Copy selected text to the system clipboard.
    pub fn copy_selection(&self) {
        let buf = &self.buffers[self.active_idx];
        let text = match &buf.selection {
            Some(sel) => {
                let (start, end) = sel.ordered_range();
                let s_idx = self.char_idx_for(start.line, start.grapheme_col);
                let e_idx = self.char_idx_for(end.line, end.grapheme_col);
                buf.rope.to_string()[s_idx..e_idx.min(buf.rope.char_count())].to_string()
            }
            None => return,
        };
        match arboard::Clipboard::new() {
            Ok(mut cb) => {
                if let Err(e) = cb.set_text(text) {
                    log::warn!("Clipboard write failed: {}", e);
                }
            }
            Err(e) => log::warn!("Clipboard unavailable: {}", e),
        }
    }

    /// Cut selected text to the clipboard (copy + delete selection).
    pub fn cut_selection(&mut self) {
        if self.buffers[self.active_idx].readonly {
            return;
        }
        self.copy_selection();
        self.delete_selection();
    }

    /// Paste text from the system clipboard at the cursor.
    pub fn paste_clipboard(&mut self) {
        if self.buffers[self.active_idx].readonly {
            return;
        }
        let text = match arboard::Clipboard::new() {
            Ok(mut cb) => match cb.get_text() {
                Ok(t) => t,
                Err(e) => {
                    log::warn!("Clipboard read failed: {}", e);
                    return;
                }
            },
            Err(e) => {
                log::warn!("Clipboard unavailable: {}", e);
                return;
            }
        };
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
        // Advance cursor by pasted char count
        for _ in 0..char_count {
            self.move_cursor(Direction::Right);
        }
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
        let deleted: String = {
            let buf = &self.buffers[self.active_idx];
            let full = buf.rope.to_string();
            full[s_idx..e_idx.min(full.len())].to_string()
        };
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

    // ── T055 — Find next / find prev ─────────────────────────────────────────

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
            let rope = &self.buffers[self.active_idx].rope;
            self.search_state.matches =
                SearchEngine::find_all(rope, &query, regex_mode, case_sensitive);
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
            let rope = &self.buffers[self.active_idx].rope;
            self.search_state.matches =
                SearchEngine::find_all(rope, &query, regex_mode, case_sensitive);
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
            let rope = &self.buffers[self.active_idx].rope;
            self.search_state.matches =
                SearchEngine::find_all(rope, &query, regex_mode, case_sensitive);
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
        let buf = &mut self.buffers[self.active_idx];
        buf.undo_stack.push(EditOp::Composite(ops));
        buf.modified = true;

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
    pub fn next_buffer(&mut self) {
        if self.buffers.len() <= 1 {
            return;
        }
        self.active_idx = (self.active_idx + 1) % self.buffers.len();
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
                log::warn!("handle_open_file: path rejected ({:?}): {}", path, e);
                return;
            }
        };

        let default_encoding = crate::encoding::encoding_from_str(&self.config.default_encoding);

        match Buffer::open(safe_path.clone(), default_encoding) {
            Ok(buf) => {
                self.buffers.push(buf);
                self.active_idx = self.buffers.len() - 1;
                log::info!("Opened {:?} as buffer {}", safe_path, self.active_idx);
            }
            Err(e) => {
                log::error!("handle_open_file: failed to open {:?}: {}", safe_path, e);
            }
        }
    }

    // ── T111 — Mouse click cursor repositioning ───────────────────────────────

    /// Reposition the cursor when the user clicks inside the editor area.
    ///
    /// `col` and `row` are 0-based terminal coordinates.  Row 0 is the menu bar
    /// and the last row is the status bar, so editor rows are `1..terminal_rows-1`.
    pub fn handle_mouse_click(&mut self, col: u16, row: u16) {
        let (_, term_rows) = self.terminal_size;

        // Guard: only handle clicks inside the editor viewport.
        if row == 0 || row >= term_rows.saturating_sub(1) {
            return;
        }

        let buf = &self.buffers[self.active_idx];
        let scroll_line = buf.scroll_offset.0;

        // Map terminal row → document line.
        let target_line = scroll_line + (row as usize - 1);
        let line_count = buf.rope.line_count();
        if target_line >= line_count {
            return;
        }

        // Walk grapheme clusters on the target line to find which grapheme
        // the clicked column falls into.
        let line_str = buf.rope.line_slice(target_line);
        let mut visual_x: u16 = 0;
        let mut found_gcol: usize = 0;

        for (gcol, grapheme) in line_str.graphemes(true).enumerate() {
            // Use a simple width: 1 for most chars, 2 for CJK full-width.
            let w = unicode_segmentation_width(grapheme);
            if visual_x + w > col {
                found_gcol = gcol;
                break;
            }
            visual_x += w;
            found_gcol = gcol + 1; // past end → clamp at line length
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

    // ── T077 — Syntax-highlight toggle ───────────────────────────────────────

    /// Toggle syntax highlighting on the active buffer.
    ///
    /// If highlighting is currently active it is disabled.  If it is off and
    /// the buffer has a known path, the correct highlighter is re-detected and
    /// assigned.  Buffers with no path stay un-highlighted.
    pub fn toggle_highlight(&mut self) {
        let buf = self.active_buffer_mut();
        if buf.syntax.is_some() {
            buf.syntax = None;
            log::debug!("Syntax highlighting disabled");
        } else if let Some(ref path) = buf.path.clone() {
            buf.syntax = crate::highlight::detect_highlighter(path);
            log::debug!(
                "Syntax highlighting enabled: {:?}",
                buf.syntax.as_ref().map(|h| h.name())
            );
        }
    }
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
        App::new(
            Config::default(),
            vec![],
            EncodingId::Utf8,
            None,
            None,
        )
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
            app.status_message.as_deref().map_or(true, |m| !m.starts_with("Saved as")),
            "cancel must not produce a 'Saved as' message"
        );
    }

    // ── T018 — Encoding persistence ─────────────────────────────────────────

    #[test]
    fn test_encoding_persists_on_regular_save() {
        let path = std::env::temp_dir().join("edit_test_persist.txt");
        std::fs::write(&path, b"Hello").unwrap();

        let mut app = App::new(Config::default(), vec![path.clone()], EncodingId::Utf8, None, None);
        // Save as UTF-16 LE via do_save_as_encoding (Case A).
        app.do_save_as_encoding(EncodingId::Utf16Le);
        // Subsequent regular save must use the new encoding.
        app.buffers[0].save().unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(bytes[0..2], [0xFF, 0xFE], "file must start with UTF-16 LE BOM");
    }

    // ── T019 — Dialog reopens with updated preselect ─────────────────────────

    #[test]
    fn test_dialog_reopens_with_updated_preselect() {
        let path = std::env::temp_dir().join("edit_test_preselect.txt");
        std::fs::write(&path, b"Hello").unwrap();

        let mut app = App::new(Config::default(), vec![path.clone()], EncodingId::Utf8, None, None);
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
        assert_eq!(app.pending_save_as_encoding, None, "pending must be cleared");
        assert_eq!(
            app.active_buffer().encoding,
            EncodingId::Utf16Le,
            "buffer encoding must be updated even if write fails"
        );
        let _ = std::fs::remove_file(&path);
        let _ = result; // allow write failure (unnamed buf has no content path)
    }
}

/// Approximate display width of a grapheme cluster (1 for narrow, 2 for wide).
///
/// This is a simple heuristic — for full Unicode width support the `unicode-width`
/// crate is the correct tool, but that dependency is not yet in the tree.
fn unicode_segmentation_width(grapheme: &str) -> u16 {
    // Treat grapheme clusters whose first scalar is in a common CJK range as
    // double-width; everything else as single-width.
    let first = grapheme.chars().next().unwrap_or(' ');
    let cp = first as u32;
    if (0x1100..=0x115F).contains(&cp)   // Hangul Jamo
        || (0x2E80..=0x303E).contains(&cp)  // CJK Radicals / Kangxi
        || (0x3041..=0x33BF).contains(&cp)  // Hiragana / Katakana / Bopomofo
        || (0x4E00..=0x9FFF).contains(&cp)  // CJK Unified
        || (0xAC00..=0xD7AF).contains(&cp)  // Hangul Syllables
        || (0xF900..=0xFAFF).contains(&cp)  // CJK Compatibility
        || (0xFE10..=0xFE6F).contains(&cp)  // CJK Compatibility Forms
        || (0xFF01..=0xFF60).contains(&cp)  // Fullwidth Latin
        || (0xFFE0..=0xFFE6).contains(&cp)  // Fullwidth Signs
        || (0x1F300..=0x1F9FF).contains(&cp) // Emoji
        || (0x20000..=0x2A6DF).contains(&cp)
    // CJK Extension B
    {
        2
    } else {
        1
    }
}
