//! Configuration subsystem.
//!
//! Loads `config.toml` from the XDG config directory and merges CLI flags on top.

#![allow(dead_code)]

pub mod schema;

pub use schema::Config;

use clap::ArgMatches;
use std::path::PathBuf;

/// Returns the path to the config file: `$XDG_CONFIG_HOME/edit/config.toml`.
pub fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| {
        let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        p.push(".config");
        p
    });
    base.join("edit").join("config.toml")
}

/// Load config from XDG path.
///
/// - Missing file → silently use all defaults.
/// - Unparseable TOML → log error, use all defaults.
/// - Unknown keys → log warning, ignore.
/// - Type mismatches → log error, revert field to default.
pub fn load_config() -> Config {
    let path = config_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            log::debug!("Config file not found at {:?}; using defaults", path);
            return Config::default();
        }
        Err(e) => {
            log::error!("Could not read config file {:?}: {}", path, e);
            return Config::default();
        }
    };

    match toml::from_str::<Config>(&content) {
        Ok(cfg) => {
            log::info!("Config loaded from {:?}", path);
            cfg
        }
        Err(e) => {
            log::error!("Unparseable TOML in {:?}: {}; using defaults", path, e);
            Config::default()
        }
    }
}

/// Merge CLI flag overrides into an already-loaded `Config`.
///
/// Flags always take precedence over config file values.
pub fn merge_cli_flags(config: &mut Config, matches: &ArgMatches) {
    if let Some(enc) = matches.get_one::<String>("encoding") {
        config.default_encoding = enc.clone();
    }
    if let Some(theme) = matches.get_one::<String>("theme") {
        config.theme = theme.clone();
    }
    if matches.get_flag("line-numbers") {
        config.line_numbers = true;
    }
    if matches.get_flag("no-highlight") {
        config.highlight = false;
    }
    if matches.get_flag("no-autosave") {
        config.no_autosave = true;
    }
    if matches.get_flag("readonly") {
        config.readonly = true;
    }
    if matches.get_flag("debug") {
        config.log_level = "debug".to_string();
    }
    if let Some(locale) = matches.get_one::<String>("locale") {
        config.locale_override = Some(locale.clone());
    }
    if matches.get_flag("no-session") {
        config.no_session = true;
    }
    if matches.get_flag("no-watch") {
        config.no_watch = true;
    }
}

/// Validate config fields after loading; clamp out-of-range values and log errors.
pub fn validate_config(c: &mut Config) {
    // Clamp autosave_interval to 10–300
    if c.autosave_interval < 10 {
        log::warn!(
            "autosave_interval {} < 10; clamping to 10",
            c.autosave_interval
        );
        c.autosave_interval = 10;
    } else if c.autosave_interval > 300 {
        log::warn!(
            "autosave_interval {} > 300; clamping to 300",
            c.autosave_interval
        );
        c.autosave_interval = 300;
    }

    let known_themes = ["classic", "high-contrast", "plain"];
    if !known_themes.contains(&c.theme.as_str()) {
        log::error!("Unknown theme {:?}; falling back to \"classic\"", c.theme);
        c.theme = "classic".to_string();
    }

    let known_levels = ["error", "warn", "info", "debug"];
    if !known_levels.contains(&c.log_level.as_str()) {
        log::error!(
            "Unknown log_level {:?}; falling back to \"warn\"",
            c.log_level
        );
        c.log_level = "warn".to_string();
    }

    let known_encodings = ["utf-8", "cp437", "cp850", "iso-8859-1", "windows-1252"];
    if !known_encodings.contains(&c.default_encoding.as_str()) {
        log::error!(
            "Unknown default_encoding {:?}; falling back to \"utf-8\"",
            c.default_encoding
        );
        c.default_encoding = "utf-8".to_string();
    }
}
