//! Rhai-based plugin subsystem (Feature 008).
//!
//! Plugins are directories under `$XDG_CONFIG_HOME/edit/plugins/<id>/` containing a
//! `plugin.toml` manifest and (for highlighter/menu plugins) a `plugin.rhai` script. The
//! [`PluginHost`] scans, validates, consent-gates, compiles, and dispatches to them inside a
//! default-deny sandbox with a 50 ms per-call wall-clock limit.

pub mod api;
pub mod consent;
pub mod highlighter;
pub mod manifest;
pub mod registry;
pub mod sandbox;
pub mod types;

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use ratatui::style::Color;
use rhai::{Array, Engine, Map};

use crate::highlight::Highlighter;
use crate::plugin::api::{new_host_state, HostState};
use crate::plugin::consent::ConsentRecord;
use crate::plugin::manifest::{parse_manifest, PluginLoadError};
use crate::plugin::registry::PluginRegistry;
use crate::plugin::sandbox::{arm_deadline, build_engine, Deadline};
use crate::plugin::types::{HighlightToken, Plugin, PluginInstance, PluginType, TokenKind};
use crate::ui::theme::Theme;

pub use crate::plugin::consent::{
    edit_config_dir, is_allowed, load_consent_records, save_consent_record, utc_now_rfc3339,
};
pub use crate::plugin::types::Plugin as PluginMeta;

/// Validate a Rhai-returned token array against the line bounds and convert it. Returns
/// `None` (discard the whole array) on any invalid token, out-of-bounds offset, overlap, or
/// unknown kind — the plugin is NOT disabled for invalid output (per the API contract).
pub fn validate_tokens(array: Array, line: &str) -> Option<Vec<HighlightToken>> {
    let line_len = line.len();
    let mut tokens: Vec<HighlightToken> = Vec::with_capacity(array.len());
    for el in array {
        let map = el.try_cast::<Map>()?;
        let start = map.get("start")?.as_int().ok()?;
        let end = map.get("end")?.as_int().ok()?;
        let kind_str = map.get("kind")?.clone().into_string().ok()?;
        let kind = TokenKind::from_name(&kind_str)?;
        if start < 0 || end < 0 || start >= end {
            return None;
        }
        let (start, end) = (start as usize, end as usize);
        if end > line_len {
            return None;
        }
        // UTF-8 safety (Principle II): reject boundaries that fall mid-codepoint so the
        // renderer never slices a multibyte character.
        if !line.is_char_boundary(start) || !line.is_char_boundary(end) {
            return None;
        }
        tokens.push(HighlightToken {
            byte_start: start as u32,
            byte_end: end as u32,
            kind,
        });
    }
    // Reject overlaps: sort by start, ensure each token begins at/after the previous end.
    tokens.sort_by_key(|t| t.byte_start);
    let mut prev_end = 0u32;
    for t in &tokens {
        if t.byte_start < prev_end {
            return None;
        }
        prev_end = t.byte_end;
    }
    Some(tokens)
}

fn theme_colors(theme: &Theme) -> [Color; 7] {
    [
        theme.foreground,         // Default
        theme.highlight_keyword,  // Keyword
        theme.highlight_string,   // String
        theme.highlight_comment,  // Comment
        theme.highlight_number,   // Number
        theme.highlight_operator, // Operator
        theme.highlight_type,     // Type
    ]
}

/// Owns the shared sandboxed engine and the plugin registry for a session.
pub struct PluginHost {
    no_plugins: bool,
    engine: Arc<Engine>,
    deadline: Deadline,
    host: HostState,
    pub registry: PluginRegistry,
}

impl PluginHost {
    pub fn new(no_plugins: bool) -> Self {
        let host = new_host_state();
        let (engine, deadline) = build_engine(host.clone());
        Self {
            no_plugins,
            engine: Arc::new(engine),
            deadline,
            host,
            registry: PluginRegistry::new(),
        }
    }

    pub fn no_plugins(&self) -> bool {
        self.no_plugins
    }

    /// Scan the plugin directory, validate manifests, apply consent, and load allowed plugins.
    /// Plugins with no recorded decision are pushed to `pending_consent` for the App to prompt.
    pub fn load_all(
        &mut self,
        config_dir: &Path,
        consent_records: &BTreeMap<String, ConsentRecord>,
        pending_consent: &mut Vec<Plugin>,
    ) {
        if self.no_plugins {
            return;
        }
        let plugins_dir = config_dir.join("plugins");
        let Ok(entries) = std::fs::read_dir(&plugins_dir) else {
            return; // absent dir == zero plugins (not an error)
        };

        let mut dirs: Vec<_> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort(); // deterministic first-wins ordering

        for dir in dirs {
            let manifest_path = dir.join("plugin.toml");
            if !manifest_path.exists() {
                continue;
            }
            let plugin = match parse_manifest(&manifest_path) {
                Ok(p) => p,
                Err(e) => {
                    log::warn!("skipping plugin in {}: {e}", dir.display());
                    continue;
                }
            };
            match is_allowed(consent_records, &plugin.id) {
                Some(true) => {
                    if let Err(e) = self.load_plugin_now(plugin) {
                        log::warn!("failed to load plugin: {e}");
                    }
                }
                Some(false) => {
                    log::info!("plugin '{}' disabled by consent record", plugin.id);
                }
                None => pending_consent.push(plugin),
            }
        }
    }

    /// Compile and register a single (already consent-approved) plugin.
    pub fn load_plugin_now(&mut self, plugin: Plugin) -> Result<(), PluginLoadError> {
        let needs_script =
            plugin.has_type(PluginType::Highlighter) || plugin.has_type(PluginType::Menu);

        let ast = if needs_script {
            let text = std::fs::read_to_string(&plugin.script_path).map_err(|e| {
                PluginLoadError::ScriptParseError(format!("{}: {e}", plugin.script_path.display()))
            })?;
            self.engine
                .compile(&text)
                .map_err(|e| PluginLoadError::ScriptParseError(e.to_string()))?
        } else {
            // Keybinding-only plugins have no script.
            self.engine
                .compile("")
                .map_err(|e| PluginLoadError::ScriptParseError(e.to_string()))?
        };

        log::info!("loaded plugin '{}' v{}", plugin.id, plugin.version);
        self.registry.instances.push(PluginInstance {
            plugin,
            ast: Arc::new(ast),
            disabled: Arc::new(AtomicBool::new(false)),
            fs_violations: Arc::new(AtomicU32::new(0)),
            enabled: true,
        });
        Ok(())
    }

    /// Build a boxed highlighter for `path`'s extension if an active plugin handles it.
    /// Returned highlighter takes precedence over the built-in (caller decides ordering).
    pub fn highlighter_for(&self, path: &Path, theme: &Theme) -> Option<Box<dyn Highlighter>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))?;
        let inst = self.registry.highlighters_for(&ext).into_iter().next()?;
        Some(highlighter::make_highlighter(
            inst.plugin.id.clone(),
            ext,
            self.engine.clone(),
            inst.ast.clone(),
            self.deadline.clone(),
            self.host.clone(),
            inst.plugin.permissions.clone(),
            inst.disabled.clone(),
            inst.fs_violations.clone(),
            theme_colors(theme),
        ))
    }

    /// Invoke a plugin menu item. Returns the status-bar message to show, if any.
    pub fn dispatch_menu_action(
        &mut self,
        plugin_id: &str,
        item_id: &str,
        buf_content: &str,
    ) -> Option<String> {
        let inst = self.registry.find(plugin_id)?;
        if !inst.is_active() {
            return None;
        }
        let ast = inst.ast.clone();
        let perms = inst.plugin.permissions.clone();
        let fs_violations = inst.fs_violations.clone();

        arm_deadline(&self.deadline);
        if let Ok(mut h) = self.host.lock() {
            h.status_queue.clear();
            h.current_perms = perms;
            h.current_plugin_id = plugin_id.to_string();
            h.fs_violation = Some(fs_violations.clone());
        }

        let engine = self.engine.clone();
        let item = item_id.to_string();
        let buf = buf_content.to_string();
        let call = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut scope = rhai::Scope::new();
            engine.call_fn::<Map>(&mut scope, &ast, "menu_action", (item, buf))
        }));

        // Check for repeated FS violations -> disable.
        if fs_violations.load(Ordering::Relaxed) >= 3 {
            self.registry.disable(plugin_id);
            return Some(format!(
                "Plugin '{plugin_id}' disabled: repeated filesystem violations"
            ));
        }

        match call {
            Ok(Ok(map)) => {
                let queued = self.drain_status();
                let message = map
                    .get("message")
                    .and_then(|m| m.clone().into_string().ok())
                    .map(|s| sanitize_display(&s));
                message.or(queued)
            }
            Ok(Err(e)) => {
                log::warn!("[plugin {plugin_id}] menu_action error; disabling: {e}");
                self.registry.disable(plugin_id);
                Some(format!("Plugin '{plugin_id}' error and was disabled"))
            }
            Err(_) => {
                log::error!("[plugin {plugin_id}] menu_action panicked; disabling");
                self.registry.disable(plugin_id);
                Some(format!("Plugin '{plugin_id}' crashed and was disabled"))
            }
        }
    }

    /// Drain any messages a script queued via `status_bar` into one display string.
    pub fn drain_status(&self) -> Option<String> {
        let mut h = self.host.lock().ok()?;
        if h.status_queue.is_empty() {
            return None;
        }
        let joined = h.status_queue.join("  ");
        h.status_queue.clear();
        Some(joined)
    }
}

fn sanitize_display(s: &str) -> String {
    crate::security::sanitize::strip_escape_sequences(s)
        .chars()
        .take(120)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tokens_discards_overlapping() {
        // Two overlapping tokens on a 10-byte line.
        let mut a = Array::new();
        let mut m1 = Map::new();
        m1.insert("start".into(), (0_i64).into());
        m1.insert("end".into(), (5_i64).into());
        m1.insert("kind".into(), "keyword".into());
        let mut m2 = Map::new();
        m2.insert("start".into(), (3_i64).into());
        m2.insert("end".into(), (8_i64).into());
        m2.insert("kind".into(), "string".into());
        a.push(m1.into());
        a.push(m2.into());
        assert!(validate_tokens(a, "0123456789").is_none());
    }

    #[test]
    fn test_validate_tokens_accepts_valid() {
        let mut a = Array::new();
        let mut m = Map::new();
        m.insert("start".into(), (0_i64).into());
        m.insert("end".into(), (2_i64).into());
        m.insert("kind".into(), "comment".into());
        a.push(m.into());
        let toks = validate_tokens(a, "0123456789").unwrap();
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, TokenKind::Comment);
    }

    #[test]
    fn test_validate_tokens_rejects_out_of_bounds() {
        let mut a = Array::new();
        let mut m = Map::new();
        m.insert("start".into(), (0_i64).into());
        m.insert("end".into(), (50_i64).into());
        m.insert("kind".into(), "number".into());
        a.push(m.into());
        assert!(validate_tokens(a, "0123456789").is_none());
    }

    #[test]
    fn test_no_plugins_skips_load() {
        let mut host = PluginHost::new(true);
        let mut pending = Vec::new();
        host.load_all(Path::new("/nonexistent"), &BTreeMap::new(), &mut pending);
        assert!(host.registry.instances.is_empty());
        assert!(pending.is_empty());
    }
}
