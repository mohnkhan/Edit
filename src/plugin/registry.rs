//! In-memory catalogue of loaded plugins for the session (Feature 008).

use std::sync::atomic::Ordering;

use crate::plugin::types::{PluginInstance, PluginMenuItem, PluginType};

#[derive(Default)]
pub struct PluginRegistry {
    pub instances: Vec<PluginInstance>,
    /// Ids disabled this session due to a trap/timeout/repeated violation.
    pub disabled: Vec<String>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Active highlighter instances matching `ext` (e.g. ".lua"), in load order.
    /// The caller uses the first (first-wins per spec).
    pub fn highlighters_for(&self, ext: &str) -> Vec<&PluginInstance> {
        self.instances
            .iter()
            .filter(|i| {
                i.is_active()
                    && i.plugin.has_type(PluginType::Highlighter)
                    && i.plugin.extensions.iter().any(|e| e == ext)
            })
            .collect()
    }

    /// Aggregated `(key_seq, action_name)` from all active keybinding plugins.
    pub fn all_keybindings(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for i in self.instances.iter().filter(|i| i.is_active()) {
            if i.plugin.has_type(PluginType::Keybinding) {
                out.extend(i.plugin.keybindings.iter().cloned());
            }
        }
        out
    }

    /// Aggregated menu items from all active menu plugins.
    pub fn menu_items(&self) -> Vec<PluginMenuItem> {
        let mut out = Vec::new();
        for i in self.instances.iter().filter(|i| i.is_active()) {
            if i.plugin.has_type(PluginType::Menu) {
                out.extend(i.plugin.menu_items.iter().cloned());
            }
        }
        out
    }

    pub fn find(&self, plugin_id: &str) -> Option<&PluginInstance> {
        self.instances.iter().find(|i| i.plugin.id == plugin_id)
    }

    /// Mark a plugin disabled for the session (runtime fault).
    pub fn disable(&mut self, plugin_id: &str) {
        if let Some(i) = self.instances.iter().find(|i| i.plugin.id == plugin_id) {
            i.disabled.store(true, Ordering::Relaxed);
        }
        if !self.disabled.iter().any(|d| d == plugin_id) {
            self.disabled.push(plugin_id.to_string());
        }
    }

    /// Set the user-controlled enabled flag (plugin manager toggle).
    pub fn set_enabled(&mut self, plugin_id: &str, enabled: bool) {
        if let Some(i) = self.instances.iter_mut().find(|i| i.plugin.id == plugin_id) {
            i.enabled = enabled;
        }
    }

    /// Whether a plugin has been disabled this session (runtime fault) or not found.
    pub fn is_disabled(&self, plugin_id: &str) -> bool {
        self.find(plugin_id)
            .map(|i| i.disabled.load(Ordering::Relaxed))
            .unwrap_or(false)
    }
}
