//! Plugin manager and consent dialog body builders (Feature 008).
//!
//! These produce the multi-line text rendered by the centered overlays in [`crate::ui`].
//! All plugin-provided strings (name, publisher, permission paths) are stripped of terminal
//! escape sequences before display (Principle VII / FR-011).

use crate::plugin::types::{Plugin, PluginType};
use crate::plugin::PluginHost;
use crate::security::sanitize::strip_escape_sequences;

fn types_label(types: &[PluginType]) -> String {
    types
        .iter()
        .map(|t| match t {
            PluginType::Highlighter => "highlighter",
            PluginType::Keybinding => "keybinding",
            PluginType::Menu => "menu",
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Build the plugin manager list body, with a `>` cursor marker and `[x]/[ ]` enabled state.
pub fn manager_body(host: &PluginHost, cursor: usize) -> String {
    if host.no_plugins() {
        return "  Plugins disabled (--no-plugins) for this session.".to_string();
    }
    let insts = &host.registry.instances;
    if insts.is_empty() {
        return "  No plugins installed.\n\n  [Esc] Close".to_string();
    }
    let mut lines = Vec::new();
    for (i, inst) in insts.iter().enumerate() {
        let marker = if i == cursor { ">" } else { " " };
        let check = if inst.enabled { "[x]" } else { "[ ]" };
        let faulted = if host.registry.is_disabled(&inst.plugin.id) {
            " (faulted)"
        } else {
            ""
        };
        let name = strip_escape_sequences(&inst.plugin.name);
        lines.push(format!(
            "{marker} {check} {name}  v{}  [{}]{faulted}",
            inst.plugin.version,
            types_label(&inst.plugin.types),
        ));
    }
    lines.push(String::new());
    lines.push("  [Up/Down] Navigate   [Space] Toggle   [Esc] Close".to_string());
    lines.join("\n")
}

/// Build the first-run consent dialog body for a pending plugin.
pub fn consent_body(p: &Plugin) -> String {
    let name = strip_escape_sequences(&p.name);
    let mut lines = vec![format!("  Plugin: {name}   v{}", p.version)];
    if let Some(publisher) = &p.publisher {
        lines.push(format!(
            "  Publisher: {}",
            strip_escape_sequences(publisher)
        ));
    }
    lines.push(format!("  Provides: {}", types_label(&p.types)));
    lines.push(String::new());
    lines.push("  Requested permissions:".to_string());
    for perm in p.permission_summary() {
        lines.push(format!("    - {}", strip_escape_sequences(&perm)));
    }
    lines.push(String::new());
    lines.push("  Allow this plugin to run?   [Enter] Allow   [Esc] Deny".to_string());
    lines.join("\n")
}

/// Number of text rows in a body string (for sizing the overlay).
pub fn line_count(body: &str) -> u16 {
    body.lines().count().max(1) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consent_body_lists_permissions_and_sanitizes() {
        let p = Plugin {
            id: "x".into(),
            name: "Ev\x1b[31mil".into(), // contains an escape sequence
            version: semver::Version::new(1, 0, 0),
            host_api: semver::VersionReq::parse("^1").unwrap(),
            types: vec![PluginType::Menu],
            extensions: vec![],
            permissions: vec![],
            publisher: None,
            description: None,
            keybindings: vec![],
            menu_items: vec![],
            script_path: "/tmp/x/plugin.rhai".into(),
            manifest_path: "/tmp/x/plugin.toml".into(),
        };
        let body = consent_body(&p);
        assert!(body.contains("Evil")); // escape stripped, text preserved
        assert!(!body.contains('\x1b'));
        assert!(body.contains("No filesystem access"));
    }
}
