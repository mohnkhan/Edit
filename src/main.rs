//! `edit` — Linux-compatible reimplementation of MS-DOS EDIT.COM.
//!
//! Entry point: parse CLI flags → bootstrap logging → load config → run editor.

#![allow(dead_code)]

mod app;
mod buffer;
mod config;
mod diagnostics;
mod encoding;
mod highlight;
mod input;
mod search;
mod security;
mod session;
mod ui;
mod watcher;

use std::path::PathBuf;
use std::process;

use clap::{Arg, ArgAction, Command};

use config::{load_config, merge_cli_flags, validate_config};
use diagnostics::{crash::install_panic_hook, logging::init_logging};

fn main() {
    let matches = build_cli().get_matches();

    // ── Locale detection + UTF-8 enforcement ────────────────────────────────
    enforce_utf8_locale(&matches);

    // ── Crash handler ───────────────────────────────────────────────────────
    install_panic_hook();
    diagnostics::crash::install_signal_handler();

    // ── Config loading ──────────────────────────────────────────────────────
    let mut config = load_config();
    merge_cli_flags(&mut config, &matches);
    validate_config(&mut config);

    // ── Logging ─────────────────────────────────────────────────────────────
    init_logging(config.resolved_log_level());
    log::info!("edit {} starting up", env!("CARGO_PKG_VERSION"));

    if config.log_level == "debug" {
        log::debug!("Config: {:?}", config);
        log::debug!(
            "XDG_CONFIG_HOME: {:?}",
            std::env::var("XDG_CONFIG_HOME").ok()
        );
        log::debug!("XDG_STATE_HOME: {:?}", std::env::var("XDG_STATE_HOME").ok());
        log::debug!(
            "XDG_RUNTIME_DIR: {:?}",
            std::env::var("XDG_RUNTIME_DIR").ok()
        );
        log::debug!("TERM: {:?}", std::env::var("TERM").ok());
    }

    // ── Collect file arguments ───────────────────────────────────────────────
    let files: Vec<PathBuf> = matches
        .get_many::<String>("FILE")
        .unwrap_or_default()
        .map(PathBuf::from)
        .collect();

    // T084 — Enhanced debug logging: resolved runtime parameters.
    if config.log_level == "debug" {
        log::debug!("Files to open: {}", files.len());
        log::debug!("Theme: {}", config.theme);
        log::debug!(
            "Autosave: enabled={}, interval={}s",
            !config.no_autosave,
            config.autosave_interval
        );
        log::debug!("Highlight: {}", config.highlight);
        log::debug!("Read-only: {}", config.readonly);
    }

    // ── Resolve encoding flag ─────────────────────────────────────────────────
    // `merge_cli_flags` already stored any --encoding value in `config.default_encoding`.
    // Convert it here so App::new can use the resolved EncodingId.
    let default_encoding = encoding::encoding_from_str(&config.default_encoding);
    log::debug!("Default encoding: {:?}", default_encoding);

    // ── Session restore: only when no explicit file arguments and --no-session not set
    let (session, session_warning) = if files.is_empty() && !config.no_session {
        match session::load_session() {
            Ok(Some(data)) => (Some(data), None),
            Err(msg) => (None, Some(msg)),
            Ok(None) => (None, None),
        }
    } else {
        (None, None)
    };

    // ── Launch editor ────────────────────────────────────────────────────────
    let app = app::App::new(config, files, default_encoding, session, session_warning);
    if let Err(e) = app.run() {
        eprintln!("edit: fatal error: {}", e);
        process::exit(1);
    }
}

// ── CLI definition ───────────────────────────────────────────────────────────

fn build_cli() -> Command {
    Command::new("edit")
        .version(env!("CARGO_PKG_VERSION"))
        .about("MS-DOS EDIT.COM reimplementation for Linux")
        .arg(
            Arg::new("FILE")
                .help("File(s) to open; omit for a new empty buffer")
                .num_args(0..)
                .value_name("FILE"),
        )
        .arg(
            Arg::new("encoding")
                .short('e')
                .long("encoding")
                .value_name("ENC")
                .help("Force file encoding (utf-8, cp437, cp850, iso-8859-1, windows-1252)"),
        )
        .arg(
            Arg::new("locale")
                .long("locale")
                .value_name("LOCALE")
                .help("Override locale for this session (e.g. C.UTF-8)"),
        )
        .arg(
            Arg::new("readonly")
                .short('r')
                .long("readonly")
                .action(ArgAction::SetTrue)
                .help("Open all files in read-only mode"),
        )
        .arg(
            Arg::new("no-autosave")
                .long("no-autosave")
                .action(ArgAction::SetTrue)
                .help("Disable auto-save and crash recovery"),
        )
        .arg(
            Arg::new("line-numbers")
                .short('n')
                .long("line-numbers")
                .action(ArgAction::SetTrue)
                .help("Show line numbers in gutter"),
        )
        .arg(
            Arg::new("theme")
                .long("theme")
                .value_name("NAME")
                .help("Color theme (classic, high-contrast, plain)"),
        )
        .arg(
            Arg::new("no-highlight")
                .long("no-highlight")
                .action(ArgAction::SetTrue)
                .help("Disable syntax highlighting"),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .action(ArgAction::SetTrue)
                .help("Enable verbose diagnostic logging"),
        )
        .arg(
            Arg::new("no-session")
                .long("no-session")
                .action(ArgAction::SetTrue)
                .help("Skip session restore prompt on startup"),
        )
        .arg(
            Arg::new("no-watch")
                .long("no-watch")
                .action(ArgAction::SetTrue)
                .help("Disable external file modification watching"),
        )
}

// ── Locale enforcement ───────────────────────────────────────────────────────

/// Detect and enforce a UTF-8 locale.
///
/// Logs a warning (and in a later task shows a dialog) if the resolved locale
/// is not UTF-8, since the editor requires UTF-8 for correct operation.
fn enforce_utf8_locale(matches: &clap::ArgMatches) {
    // Priority: --locale flag > EDIT_LOCALE env var > system LANG
    let locale = matches
        .get_one::<String>("locale")
        .cloned()
        .or_else(|| std::env::var("EDIT_LOCALE").ok())
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_else(|| "C.UTF-8".to_string());

    let is_utf8 = locale.to_uppercase().contains("UTF-8") || locale.to_uppercase().contains("UTF8");

    if !is_utf8 {
        eprintln!(
            "edit: WARNING: locale {:?} is not UTF-8; Unicode display may be incorrect. \
             Set LANG=C.UTF-8 or use --locale C.UTF-8.",
            locale
        );
    }

    log::debug!("Resolved locale: {:?} (utf8={})", locale, is_utf8);
}
