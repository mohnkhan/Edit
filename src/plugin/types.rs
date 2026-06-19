//! Shared types for the Rhai-based plugin subsystem (Feature 008).
//!
//! These types are engine-agnostic where possible; the only Rhai-specific item is the
//! compiled [`rhai::AST`] held by a [`PluginInstance`].

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Arc;

/// Compile-time plugin ABI generation. A plugin's manifest `host_api` semver requirement
/// must be satisfied by this value (treated as `<HOST_PLUGIN_API_VERSION>.0.0`).
pub const HOST_PLUGIN_API_VERSION: i32 = 1;

/// The extension surfaces a plugin may provide. A single plugin may declare several.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    Highlighter,
    Keybinding,
    Menu,
}

/// Extra capability a plugin declares in its manifest beyond the default-deny sandbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    /// Read access to a specific file or directory path.
    ReadPath(PathBuf),
    /// Write access to a specific directory (not granted by default; reserved for future use).
    WriteDir(PathBuf),
}

/// Semantic token category a highlighter plugin assigns to a span. Plugins express these
/// as lowercase strings in their returned maps; the host parses them via [`TokenKind::from_name`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Default,
    Keyword,
    String,
    Comment,
    Number,
    Operator,
    Type,
}

impl TokenKind {
    /// Map a lowercase script string to a [`TokenKind`]. Unknown strings return `None`,
    /// which causes the host to discard the whole token array (defensive default).
    pub fn from_name(s: &str) -> Option<TokenKind> {
        match s {
            "default" => Some(TokenKind::Default),
            "keyword" => Some(TokenKind::Keyword),
            "string" => Some(TokenKind::String),
            "comment" => Some(TokenKind::Comment),
            "number" => Some(TokenKind::Number),
            "operator" => Some(TokenKind::Operator),
            "type" => Some(TokenKind::Type),
            _ => None,
        }
    }

    /// Stable index used to look up the corresponding theme colour.
    pub fn index(self) -> usize {
        match self {
            TokenKind::Default => 0,
            TokenKind::Keyword => 1,
            TokenKind::String => 2,
            TokenKind::Comment => 3,
            TokenKind::Number => 4,
            TokenKind::Operator => 5,
            TokenKind::Type => 6,
        }
    }
}

/// A single coloured span returned by a highlighter plugin, validated by the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightToken {
    pub byte_start: u32,
    pub byte_end: u32,
    pub kind: TokenKind,
}

/// A menu item a plugin contributes to a (possibly new) top-level menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMenuItem {
    pub menu: String,
    pub item: String,
    pub item_id: String,
    pub plugin_id: String,
    pub position: Option<u32>,
}

/// A plugin's declared identity and capabilities, parsed from `plugin.toml` before any
/// script code runs (required for the consent flow).
#[derive(Debug, Clone)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: semver::Version,
    pub host_api: semver::VersionReq,
    pub types: Vec<PluginType>,
    pub extensions: Vec<String>,
    pub permissions: Vec<Permission>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub keybindings: Vec<(String, String)>,
    pub menu_items: Vec<PluginMenuItem>,
    pub script_path: PathBuf,
    pub manifest_path: PathBuf,
}

impl Plugin {
    pub fn has_type(&self, t: PluginType) -> bool {
        self.types.contains(&t)
    }

    /// Human-readable summary of declared permissions, for the consent dialog.
    pub fn permission_summary(&self) -> Vec<String> {
        if self.permissions.is_empty() {
            return vec!["No filesystem access (default sandbox)".to_string()];
        }
        self.permissions
            .iter()
            .map(|p| match p {
                Permission::ReadPath(p) => format!("Read: {}", p.display()),
                Permission::WriteDir(p) => format!("Write: {}", p.display()),
            })
            .collect()
    }
}

/// Runtime state of a loaded plugin. The compiled `ast` is reused for every call; the
/// `disabled` and `fs_violations` flags are shared (`Arc`) with any highlighter wrapper
/// handed to a buffer so a runtime fault is reflected everywhere.
pub struct PluginInstance {
    pub plugin: Plugin,
    pub ast: Arc<rhai::AST>,
    pub disabled: Arc<AtomicBool>,
    pub fs_violations: Arc<AtomicU32>,
    /// Whether the user has the plugin enabled (manager toggle); distinct from a runtime fault.
    pub enabled: bool,
}

impl PluginInstance {
    pub fn is_active(&self) -> bool {
        self.enabled && !self.disabled.load(std::sync::atomic::Ordering::Relaxed)
    }
}
