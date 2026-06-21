//! Application state machine and top-level event dispatch.
//!
//! [`App`] owns all editor state and drives the main event loop.

#![allow(dead_code, unused_variables, unused_imports)]
// Feature 042 (#72): forbid panic-prone `unwrap()`/`expect()` on fallible values in the editor's
// core input/dialog code. This inner attribute propagates to all `src/app/*` submodules, so a new
// guarded unwrap anywhere in the App tree fails `clippy -D warnings`. Test code re-allows it (see the
// `#![allow]` at the top of `src/app/tests.rs` and the `#[allow]` on the inline debug modules below).
// Out of scope by design (FR-006): `Regex::new("<literal>").unwrap()` in `highlight/languages/*` and
// best-effort `let _ =` cleanup live in other modules and are unaffected.
#![deny(clippy::unwrap_used, clippy::expect_used)]

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

/// The single foreground modal layer (Feature 039). At most one overlay is ever
/// open — the enum makes any other combination unrepresentable, replacing the prior
/// bag of independent `Option`/`bool` flags. Key dispatch, mouse dispatch, and paint
/// all derive which overlay is active from this one value.
///
/// State that legitimately coexists with editing (`menu_bar`, pointer drags) or that
/// is adjunct focus/flow state (`dialog_focus`, `pending_save_as_encoding`) stays in
/// its own field — see `data-model.md`.
#[derive(Default)]
pub(crate) enum Modal {
    /// No overlay open — normal editing (possibly with the menu bar active).
    #[default]
    None,
    /// Right-click editor context menu (Feature 030).
    ContextMenu(crate::ui::contextmenu::ContextMenu),
    /// Session-restore confirmation (Feature 003).
    SessionRestore(crate::session::SessionData),
    /// Unsaved-changes save-before-quit prompt.
    SavePrompt,
    /// Revert confirmation for the given (modified) buffer index (Feature 014).
    RevertConfirm(usize),
    /// Tab `[x]` close confirmation for the given buffer index (Feature 027).
    CloseConfirm(usize),
    /// Interactive Find/Replace dialog (Feature 015).
    FindReplace(FindReplaceDialog),
    /// Go-to-Line prompt: in-progress 1-based digits + caret index (Feature 025/031).
    GotoLine { digits: String, caret: usize },
    /// Encoding selection dialog: highlighted row in `ENCODING_OPTIONS` (US4).
    EncodingSelect { row: usize },
    /// Navigable Open/Save file browser (Feature 012).
    FileBrowser(FileBrowser),
    /// Help/About overlay + scroll offset (Feature 011/018).
    Help { screen: HelpScreen, scroll: usize },
    /// Options ▸ Plugins manager overlay + list cursor (Feature 008).
    PluginManager { cursor: usize },
}

// Note (Feature 039): two overlays are intentionally NOT `Modal` variants because
// they are set *asynchronously* and can be pending *underneath* a user-opened
// overlay (a priority stack the original relied on — preserving FR-010):
//   * `pending_external_change` — the file watcher (`handle_tick`, every 500ms)
//     can detect a change while a dialog is open; it preempts by precedence and
//     the dialog survives underneath until it is dismissed.
//   * `pending_plugin_consent` — a startup queue that can sit behind a
//     session-restore prompt and surface once it closes.
// A single `Modal` value cannot hold both at once, so these stay independent
// fields, like `pending_save_as_encoding` (flow state) and `dialog_focus`
// (adjunct focus state). They keep their existing precedence slots in dispatch.

/// Feature 039: the stacked UI layers, from **topmost to bottommost**.
///
/// This is the single declared source of stacking precedence. Mouse hit-testing
/// resolves a click to the topmost *active* layer occupying the cell (see
/// [`App::top_row_owner`]); the renderer paints in the **reverse** of this order
/// (editor first, modal last) so the on-screen z-order matches. Keeping both
/// sides derived from one ordering is what prevents the paint-vs-hit-test drift
/// that produced the tab-bar/menu-dropdown bugs (033/038).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Layer {
    /// A foreground [`Modal`] overlay — drawn last, hit-tested first.
    Modal,
    /// An open menu dropdown (overlays the tab-bar row and below).
    MenuDropDown,
    /// The menu bar itself (row 0).
    MenuBar,
    /// The buffer tab bar (row 1, only with 2+ buffers).
    TabBar,
    /// The editor text area (bottom of the stack).
    Editor,
}

/// The declared layer precedence, topmost first. Consumed by mouse hit-testing
/// (forward) and the render paint order (reverse).
const LAYER_PRECEDENCE: [Layer; 5] = [
    Layer::Modal,
    Layer::MenuDropDown,
    Layer::MenuBar,
    Layer::TabBar,
    Layer::Editor,
];

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
    /// Feature 039: the single foreground overlay (at most one open at a time).
    /// Replaces the former bag of `pending_*`/`file_browser` flags. Key/mouse/paint
    /// all derive the active overlay from this field.
    modal: Modal,
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
    /// Pull-down menu state machine (T041).
    pub menu_bar: MenuBarState,
    /// Whether the terminal is too small to render the editor.
    pub too_small: bool,
    /// Current search-and-replace session state (T055).
    pub search_state: SearchState,
    /// Transient message shown in the status bar (e.g. "Match 2/5").
    pub status_message: Option<String>,
    /// Default encoding resolved from config/CLI at startup.
    pub default_encoding: EncodingId,
    /// Last file-browser entry click (index + time) for double-click detection.
    /// A single click selects the row; a second click on the same row within
    /// [`DOUBLE_CLICK_MS`] activates it (enter folder / open file) — Feature 012.
    pub last_browser_click: Option<(usize, Instant)>,
    /// Feature 030: last editor left-press `(col, row, count, time)` for
    /// double/triple-click detection (count 1=single, 2=word, 3=line).
    pub last_editor_click: Option<(u16, u16, u8, Instant)>,
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

    // ── Feature 008: plugin subsystem ────────────────────────────────────────
    /// The Rhai plugin host owning the engine and registry for this session.
    pub plugin_host: crate::plugin::PluginHost,
    /// Plugins awaiting a first-run consent decision; the front item is prompted.
    /// Async queue kept as a field (not a `Modal` variant) — see the note on `Modal`.
    pub pending_plugin_consent: Vec<crate::plugin::PluginMeta>,
}

// ── App impl (split across `src/app/*.rs` submodules — Feature 041) ───────────

mod actions;
mod dialogs;
mod dispatch;
mod editing;
mod fileops;
mod mouse;
mod search;
mod softwrap;

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
            // Feature 039: a pending session-restore is the only overlay that can
            // be open at construction time.
            modal: session.map(Modal::SessionRestore).unwrap_or(Modal::None),
            config,
            keymap,
            buffers,
            active_idx: 0,
            running: true,
            terminal_size: (80, 24),
            theme,
            split_mode: crate::ui::SplitMode::Single,
            menu_bar: MenuBarState::new(),
            too_small: false,
            search_state: SearchState::default(),
            status_message,
            default_encoding,
            last_browser_click: None,
            last_editor_click: None,
            pending_save_as_encoding: None,
            soft_wrap: soft_wrap_initial,
            wrap_cache: None,
            wrap_text_gen: 0,
            file_watcher,
            self_write_times: std::collections::HashMap::new(),
            pending_external_change: None,
            watcher_notice,
            dialog_focus: 0,
            dialog_focus_init: false,
            drag_anchor: None,
            scrollbar_drag: None,
            plugin_host,
            pending_plugin_consent,
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

    // ── Feature 039: foreground modal accessors ──────────────────────────────
    // The single source of truth for "which overlay is open". All key/mouse/paint
    // dispatch reads through these; nothing reads a per-overlay flag any more.

    /// The current foreground overlay (read-only). Crate-internal: `Modal` is a
    /// private type, so integration tests use the typed `*()`/`*_is_open()`
    /// accessors below instead of matching on this directly.
    pub(crate) fn modal(&self) -> &Modal {
        &self.modal
    }

    /// True iff a foreground overlay is open (≠ `Modal::None`).
    pub fn modal_is_open(&self) -> bool {
        !matches!(self.modal, Modal::None)
    }

    /// Close any open overlay (set to `Modal::None`). Idempotent. The pre-render
    /// `clamp_all_cursors()` still runs, preserving the cursor-bounds invariant.
    pub(crate) fn close_modal(&mut self) {
        self.modal = Modal::None;
    }

    /// The open Find/Replace dialog, if that overlay is open.
    pub fn find_replace(&self) -> Option<&FindReplaceDialog> {
        match &self.modal {
            Modal::FindReplace(d) => Some(d),
            _ => None,
        }
    }

    /// Mutable access to the open Find/Replace dialog.
    pub fn find_replace_mut(&mut self) -> Option<&mut FindReplaceDialog> {
        match &mut self.modal {
            Modal::FindReplace(d) => Some(d),
            _ => None,
        }
    }

    /// Open the interactive Find/Replace dialog with the given dialog state.
    pub fn open_find_replace(&mut self, dialog: FindReplaceDialog) {
        self.modal = Modal::FindReplace(dialog);
    }

    /// The open file browser, if that overlay is open.
    pub fn file_browser(&self) -> Option<&FileBrowser> {
        match &self.modal {
            Modal::FileBrowser(b) => Some(b),
            _ => None,
        }
    }

    /// Mutable access to the open file browser.
    pub fn file_browser_mut(&mut self) -> Option<&mut FileBrowser> {
        match &mut self.modal {
            Modal::FileBrowser(b) => Some(b),
            _ => None,
        }
    }

    /// Open the file browser (Open/Save) with the given browser state.
    pub fn open_file_browser(&mut self, browser: FileBrowser) {
        self.modal = Modal::FileBrowser(browser);
    }

    /// The open editor context menu, if that overlay is open.
    pub fn context_menu(&self) -> Option<&crate::ui::contextmenu::ContextMenu> {
        match &self.modal {
            Modal::ContextMenu(m) => Some(m),
            _ => None,
        }
    }

    /// True iff the save-before-quit prompt is the open overlay.
    pub fn is_save_prompt_open(&self) -> bool {
        matches!(self.modal, Modal::SavePrompt)
    }

    /// True iff the session-restore prompt is the open overlay.
    pub fn is_session_restore_open(&self) -> bool {
        matches!(self.modal, Modal::SessionRestore(_))
    }

    /// Buffer index awaiting a Revert confirmation, if that overlay is open.
    pub fn revert_confirm_target(&self) -> Option<usize> {
        match self.modal {
            Modal::RevertConfirm(i) => Some(i),
            _ => None,
        }
    }

    /// Buffer index awaiting a tab-close confirmation, if that overlay is open.
    pub fn close_confirm_target(&self) -> Option<usize> {
        match self.modal {
            Modal::CloseConfirm(i) => Some(i),
            _ => None,
        }
    }

    /// Highlighted row of the encoding-select dialog, if that overlay is open.
    pub fn encoding_select_row(&self) -> Option<usize> {
        match self.modal {
            Modal::EncodingSelect { row } => Some(row),
            _ => None,
        }
    }

    /// Open (or re-point) the encoding-select dialog at `row`.
    pub fn set_encoding_select(&mut self, row: usize) {
        self.modal = Modal::EncodingSelect { row };
    }

    /// True iff the plugin-manager overlay is open.
    pub fn is_plugin_manager_open(&self) -> bool {
        matches!(self.modal, Modal::PluginManager { .. })
    }

    /// Open the Options ▸ Plugins manager overlay (cursor at the top).
    pub fn open_plugin_manager(&mut self) {
        self.modal = Modal::PluginManager { cursor: 0 };
    }

    /// Open the Go-to-Line prompt with the given in-progress digits and caret.
    pub fn open_goto_line(&mut self, digits: String, caret: usize) {
        self.modal = Modal::GotoLine { digits, caret };
    }

    /// Open the Help/About overlay at the given screen (scroll reset to top).
    pub fn open_help(&mut self, screen: HelpScreen) {
        self.modal = Modal::Help { screen, scroll: 0 };
    }

    /// The Go-to-Line digits (in progress), if that prompt is open.
    pub fn goto_line_digits(&self) -> Option<&str> {
        match &self.modal {
            Modal::GotoLine { digits, .. } => Some(digits.as_str()),
            _ => None,
        }
    }

    /// Feature 039 (FR-006): the Go-to-Line dialog box rect — the single geometry
    /// source shared by the renderer and mouse hit-testing, so a click can never
    /// diverge from the drawn box. `None` when the prompt is closed.
    pub(crate) fn goto_line_rect(&self) -> Option<ratatui::layout::Rect> {
        let digits = self.goto_line_digits()?;
        let (w, h) = self.terminal_size;
        // Width fits "Go to line: " (12) + digits + the caret glyph + borders,
        // clamped to the terminal; centered. (Equivalent to the prior render-side
        // `body.len() + 4`, since the caret glyph "▏" is 3 bytes.)
        let dw = ((19 + digits.len()) as u16).clamp(20, w.max(1));
        let dh = 3u16.min(h.max(1));
        let dx = w.saturating_sub(dw) / 2;
        let dy = h.saturating_sub(dh) / 2;
        Some(ratatui::layout::Rect::new(dx, dy, dw, dh))
    }

    /// The digit-input field rect inside the Go-to-Line box (where a caret click
    /// lands). Derived from [`Self::goto_line_rect`] so render and hit-test agree.
    pub(crate) fn goto_line_field_rect(&self) -> Option<ratatui::layout::Rect> {
        let rect = self.goto_line_rect()?;
        let value_x = rect.x + 1 + "Go to line: ".len() as u16;
        let field_w = rect.width.saturating_sub(2 + 12);
        Some(ratatui::layout::Rect::new(value_x, rect.y + 1, field_w, 1))
    }

    /// The Go-to-Line caret index (0 when the prompt is not open).
    pub fn goto_line_caret(&self) -> usize {
        match self.modal {
            Modal::GotoLine { caret, .. } => caret,
            _ => 0,
        }
    }

    /// The open Help/About overlay screen, if that overlay is open.
    pub fn help_screen(&self) -> Option<HelpScreen> {
        match self.modal {
            Modal::Help { screen, .. } => Some(screen),
            _ => None,
        }
    }

    /// The Help overlay scroll offset (0 when Help is not open).
    pub fn help_scroll(&self) -> usize {
        match self.modal {
            Modal::Help { scroll, .. } => scroll,
            _ => 0,
        }
    }

    /// Mutable Help scroll offset, if the Help overlay is open.
    pub(crate) fn help_scroll_mut(&mut self) -> Option<&mut usize> {
        match &mut self.modal {
            Modal::Help { scroll, .. } => Some(scroll),
            _ => None,
        }
    }

    /// Plugin-manager list cursor (0 when the overlay is not open — callers read
    /// it only inside manager-open contexts).
    pub(crate) fn plugin_manager_cursor(&self) -> usize {
        match self.modal {
            Modal::PluginManager { cursor } => cursor,
            _ => 0,
        }
    }

    /// Mutable plugin-manager list cursor, if the overlay is open.
    pub(crate) fn plugin_manager_cursor_mut(&mut self) -> Option<&mut usize> {
        match &mut self.modal {
            Modal::PluginManager { cursor } => Some(cursor),
            _ => None,
        }
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

    /// Whether a given layer is currently active (present and able to own clicks).
    /// `Modal` is excluded here — modal overlays are dispatched earlier in the
    /// mouse handler and return before the top-bar rows are considered.
    fn layer_active(&self, layer: Layer) -> bool {
        match layer {
            Layer::Modal => self.modal_is_open(),
            Layer::MenuDropDown => matches!(self.menu_bar.state, MenuState::DropDown { .. }),
            // The menu *bar* lives on row 0, not the tab-bar row, so it never wins
            // the tab row by precedence; it is handled by `hit_test_menu` directly.
            Layer::MenuBar => false,
            Layer::TabBar => self.tab_bar_visible(),
            Layer::Editor => true,
        }
    }

    /// Feature 039: the topmost active layer occupying the tab-bar row, resolved by
    /// scanning the single [`LAYER_PRECEDENCE`] order. When this is not
    /// [`Layer::TabBar`] (i.e. an open dropdown sits above it), the tab bar yields
    /// those clicks to the higher layer — replacing the former ad-hoc
    /// `!dropdown_open` guard with a precedence-derived decision so paint and
    /// hit-test cannot drift (the 033/038 bug class).
    fn top_row_owner(&self) -> Layer {
        LAYER_PRECEDENCE
            .iter()
            .copied()
            .find(|&l| self.layer_active(l))
            .unwrap_or(Layer::Editor)
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
        if let Some(screen) = self.help_screen() {
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
                    offset: self.help_scroll().min(content),
                    target: ScrollTarget::Help,
                });
            }
        } else if let Modal::EncodingSelect { row: idx } = self.modal {
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
        } else if let Some(fb) = self.file_browser() {
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
        } else if self.is_plugin_manager_open() {
            let rect = crate::ui::plugin_manager::manager_rect(
                &self.plugin_host,
                self.plugin_manager_cursor(),
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
                    offset: self.plugin_manager_cursor(),
                    target: ScrollTarget::Plugin,
                });
            }
        } else if self.find_replace().is_some() || self.goto_line_digits().is_some() {
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
                if let Some(fb) = self.file_browser_mut() {
                    fb.set_scroll(offset, viewport);
                }
            }
            ScrollTarget::Help => {
                if let Some(s) = self.help_scroll_mut() {
                    *s = offset;
                }
            }
            ScrollTarget::Encoding => {
                let n = crate::ui::dialog::ENCODING_OPTIONS.len();
                self.set_encoding_select(offset.min(n.saturating_sub(1)));
            }
            ScrollTarget::Plugin => {
                let n = self.plugin_host.registry.instances.len();
                if let Some(c) = self.plugin_manager_cursor_mut() {
                    *c = offset.min(n.saturating_sub(1));
                }
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
                    let rope = &self.active_buffer().rope;
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
        // Feature 034: guarantee every buffer's cursor is in range before the
        // renderer reads it. The renderer indexes lines by `cursor.line`; a stale
        // cursor (past the content) would otherwise slice out of bounds. This is
        // the belt to the rope's suspenders (`line_slice` also clamps) and keeps a
        // crash from ever reaching the screen on session restore / buffer switch.
        self.clamp_all_cursors();

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
        let idx = self.plugin_manager_cursor();
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
mod tests;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod f032_dbg2 {
    use super::*;
    fn mk() -> App {
        App::new(Config::default(), vec![], EncodingId::Utf8, None, None)
    }
    #[test]
    fn no_termsize() {
        let mut a = mk();
        a.buffers[0].rope = crate::buffer::rope::EditorRope::from_str("foo bar baz\n");
        a.active_idx = 0;
        a.buffers[0].cursor = crate::buffer::CursorPos {
            line: 0,
            grapheme_col: 3,
            visual_col: 3,
        };
        eprintln!(
            "DBG2 npw={:?} termsize={:?}",
            a.next_word_pos(Direction::Left),
            a.terminal_size
        );
        a.delete_word(Direction::Left);
        eprintln!("DBG2 after={:?}", a.buffers[0].rope.line_slice(0));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod f034_repro2 {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn render(a: &mut App) {
        let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
        t.draw(|f| a.render(f)).unwrap();
    }
    fn build() -> App {
        let mut a = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
        a.terminal_size = (80, 24);
        a.buffers.clear();
        for content in ["x", "a\nb\nc", ""] {
            let mut b = crate::buffer::Buffer::new_empty();
            b.rope = crate::buffer::rope::EditorRope::from_str(content);
            b.path = Some(std::path::PathBuf::from("f"));
            a.buffers.push(b);
        }
        a.buffers[1].cursor = crate::buffer::CursorPos {
            line: 2,
            grapheme_col: 0,
            visual_col: 0,
        };
        a.active_idx = 0;
        let menus = a.resolved_menus();
        a.menu_bar.open_menu(2, &menus);
        a
    }

    #[test]
    fn repro_menu_click_over_tabs() {
        for row in 0..4u16 {
            for col in 0..80u16 {
                let mut a = build();
                render(&mut a);
                a.handle_mouse_event(crossterm::event::MouseEvent {
                    kind: crossterm::event::MouseEventKind::Down(
                        crossterm::event::MouseButton::Left,
                    ),
                    column: col,
                    row,
                    modifiers: crossterm::event::KeyModifiers::NONE,
                })
                .unwrap();
                render(&mut a);
            }
        }
    }
}
