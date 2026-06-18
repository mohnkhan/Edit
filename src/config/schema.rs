//! Configuration schema (Task T009).
//!
//! Defines [`Config`] — the top-level struct that maps to the TOML/JSON
//! configuration file (`~/.config/edit/config.toml` by convention).
//!
//! All fields have defaults matching the documented contract; deserialising a
//! partial config file (or an empty one) will fill in every missing key.
//!
//! # Example
//!
//! ```toml
//! default_encoding = "utf-8"
//! theme            = "classic"
//! autosave_interval = 60
//! line_numbers      = true
//! highlight         = true
//! mouse             = false
//! log_level         = "info"
//!
//! [keybindings]
//! "ctrl+s" = "save"
//! "ctrl+q" = "quit"
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ── Config ───────────────────────────────────────────────────────────────────

/// Top-level editor configuration.
///
/// Serialises to / deserialises from TOML (or any `serde`-compatible format).
/// Missing fields during deserialisation are filled by the [`Default`] impl.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)] // fills every missing field from Default::default()
pub struct Config {
    /// Character encoding used when opening files that lack a BOM or
    /// declaration.  Must be a label accepted by the `encoding_rs` crate.
    ///
    /// Default: `"utf-8"`
    pub default_encoding: String,

    /// Name of the colour theme to apply.  Built-in options: `"classic"`
    /// (blue-on-white, DOS look), `"dark"`, `"light"`.
    ///
    /// Default: `"classic"`
    pub theme: String,

    /// How often (in seconds) the editor auto-saves dirty buffers.
    /// Set to `0` to disable auto-save.
    ///
    /// Default: `30`
    pub autosave_interval: u32,

    /// Show line numbers in the left gutter.
    ///
    /// Default: `false`
    pub line_numbers: bool,

    /// Enable syntax highlighting.
    ///
    /// Default: `true`
    pub highlight: bool,

    /// Enable mouse support (click-to-position, scroll wheel).
    ///
    /// Default: `true`
    pub mouse: bool,

    /// Minimum severity for messages written to the log file.
    /// Accepted values (case-insensitive): `"off"`, `"error"`, `"warn"`,
    /// `"info"`, `"debug"`, `"trace"`.
    ///
    /// Default: `"warn"`
    pub log_level: String,

    /// User-defined key bindings.  Keys are the trigger (e.g. `"ctrl+s"`,
    /// `"f2"`) and values are the command identifier (e.g. `"save"`, `"quit"`).
    ///
    /// Default: empty map
    pub keybindings: HashMap<String, String>,

    // ── Runtime-only fields (not persisted to config.toml) ───────────────────
    /// Set by `--no-autosave` CLI flag; not written to config file.
    #[serde(skip)]
    pub no_autosave: bool,

    /// Set by `--readonly` CLI flag; not written to config file.
    #[serde(skip)]
    pub readonly: bool,

    /// Set by `--locale` CLI flag or `EDIT_LOCALE` env var; not written to config.
    #[serde(skip)]
    pub locale_override: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_encoding: "utf-8".to_owned(),
            theme: "classic".to_owned(),
            autosave_interval: 30,
            line_numbers: false,
            highlight: true,
            mouse: true,
            log_level: "warn".to_owned(),
            keybindings: HashMap::new(),
            no_autosave: false,
            readonly: false,
            locale_override: None,
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

impl Config {
    /// Parses a `log::LevelFilter` from [`Config::log_level`].
    ///
    /// Returns [`log::LevelFilter::Warn`] when the stored string is not
    /// recognised so that the editor always starts with a sane log level.
    pub fn resolved_log_level(&self) -> log::LevelFilter {
        self.log_level
            .parse::<log::LevelFilter>()
            .unwrap_or(log::LevelFilter::Warn)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_match_contract() {
        let cfg = Config::default();
        assert_eq!(cfg.default_encoding, "utf-8");
        assert_eq!(cfg.theme, "classic");
        assert_eq!(cfg.autosave_interval, 30);
        assert!(!cfg.line_numbers);
        assert!(cfg.highlight);
        assert!(cfg.mouse);
        assert_eq!(cfg.log_level, "warn");
        assert!(cfg.keybindings.is_empty());
    }

    #[test]
    fn serde_round_trip_default() {
        let original = Config::default();
        // Serialise to JSON (always available via serde_json in tests, or use
        // toml — here we use serde_json because it ships with serde derive).
        let json = serde_json::to_string(&original).expect("serialise");
        let restored: Config = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(restored.default_encoding, original.default_encoding);
        assert_eq!(restored.theme, original.theme);
        assert_eq!(restored.autosave_interval, original.autosave_interval);
        assert_eq!(restored.line_numbers, original.line_numbers);
        assert_eq!(restored.highlight, original.highlight);
        assert_eq!(restored.mouse, original.mouse);
        assert_eq!(restored.log_level, original.log_level);
        assert_eq!(restored.keybindings, original.keybindings);
    }

    #[test]
    fn partial_deserialise_fills_defaults() {
        // Only override `theme`; everything else should come from Default.
        let json = r#"{"theme": "dark"}"#;
        let cfg: Config = serde_json::from_str(json).expect("deserialise partial");
        assert_eq!(cfg.theme, "dark");
        assert_eq!(cfg.default_encoding, "utf-8");
        assert_eq!(cfg.autosave_interval, 30);
        assert!(!cfg.line_numbers);
        assert!(cfg.highlight);
        assert!(cfg.mouse);
        assert_eq!(cfg.log_level, "warn");
    }

    #[test]
    fn keybindings_round_trip() {
        let json = r#"{"keybindings": {"ctrl+s": "save", "f10": "quit"}}"#;
        let cfg: Config = serde_json::from_str(json).expect("deserialise keybindings");
        assert_eq!(cfg.keybindings.get("ctrl+s"), Some(&"save".to_owned()));
        assert_eq!(cfg.keybindings.get("f10"), Some(&"quit".to_owned()));
    }

    #[test]
    fn resolved_log_level_valid() {
        let mut cfg = Config::default();
        cfg.log_level = "debug".to_owned();
        assert_eq!(cfg.resolved_log_level(), log::LevelFilter::Debug);
    }

    #[test]
    fn resolved_log_level_invalid_falls_back_to_warn() {
        let mut cfg = Config::default();
        cfg.log_level = "not-a-level".to_owned();
        assert_eq!(cfg.resolved_log_level(), log::LevelFilter::Warn);
    }

    #[test]
    fn clone_is_independent() {
        let mut original = Config::default();
        let mut cloned = original.clone();
        cloned.theme = "dark".to_owned();
        original
            .keybindings
            .insert("ctrl+z".to_owned(), "undo".to_owned());
        // Mutations are independent.
        assert_eq!(original.theme, "classic");
        assert!(cloned.keybindings.is_empty());
    }
}
