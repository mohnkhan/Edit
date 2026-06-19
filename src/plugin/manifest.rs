//! Parsing and validation of `plugin.toml` manifests (Feature 008).
//!
//! The manifest is read and fully validated *before* any plugin script is compiled or run,
//! so the host knows a plugin's identity and requested permissions ahead of the consent flow.

use std::collections::BTreeMap;
use std::path::Path;

use crate::plugin::types::{
    Permission, Plugin, PluginMenuItem, PluginType, HOST_PLUGIN_API_VERSION,
};

/// Reasons a plugin manifest may be rejected at load time.
#[derive(Debug)]
pub enum PluginLoadError {
    ManifestParseError(String),
    InvalidId(String),
    InvalidVersion(String),
    ApiVersionMismatch { plugin: String, host: i32 },
    InvalidUtf8(String),
    ScriptParseError(String),
    ConsentDenied,
}

impl std::fmt::Display for PluginLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginLoadError::ManifestParseError(s) => write!(f, "manifest parse error: {s}"),
            PluginLoadError::InvalidId(s) => write!(f, "invalid plugin id: {s}"),
            PluginLoadError::InvalidVersion(s) => write!(f, "invalid version: {s}"),
            PluginLoadError::ApiVersionMismatch { plugin, host } => write!(
                f,
                "plugin requires host_api {plugin} but editor provides API v{host}"
            ),
            PluginLoadError::InvalidUtf8(s) => write!(f, "non-UTF-8 content: {s}"),
            PluginLoadError::ScriptParseError(s) => write!(f, "script parse error: {s}"),
            PluginLoadError::ConsentDenied => write!(f, "consent denied"),
        }
    }
}

#[derive(serde::Deserialize)]
struct RawManifest {
    id: String,
    name: String,
    version: String,
    host_api: String,
    types: Vec<PluginType>,
    #[serde(default)]
    extensions: Vec<String>,
    publisher: Option<String>,
    description: Option<String>,
    #[serde(default)]
    keybindings: BTreeMap<String, String>,
    #[serde(default)]
    menu_items: Vec<RawMenuItem>,
    #[serde(default)]
    permissions: RawPermissions,
}

#[derive(serde::Deserialize)]
struct RawMenuItem {
    menu: String,
    item: String,
    item_id: String,
    position: Option<u32>,
}

#[derive(serde::Deserialize, Default)]
struct RawPermissions {
    #[serde(default)]
    read_paths: Vec<String>,
    #[serde(default)]
    write_dirs: Vec<String>,
}

/// `id` must be kebab-case: `[a-z0-9]` with internal `-` allowed, no leading/trailing hyphen.
fn valid_id(id: &str) -> bool {
    if id.is_empty() {
        return false;
    }
    let bytes = id.as_bytes();
    let ok_char = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-';
    if !bytes.iter().all(|&b| ok_char(b)) {
        return false;
    }
    bytes[0] != b'-' && bytes[bytes.len() - 1] != b'-'
}

/// Parse and validate a `plugin.toml`, returning a [`Plugin`] (script not yet compiled).
pub fn parse_manifest(manifest_path: &Path) -> Result<Plugin, PluginLoadError> {
    let bytes = std::fs::read(manifest_path)
        .map_err(|e| PluginLoadError::ManifestParseError(e.to_string()))?;
    let text = String::from_utf8(bytes)
        .map_err(|_| PluginLoadError::InvalidUtf8(manifest_path.display().to_string()))?;

    let raw: RawManifest =
        toml::from_str(&text).map_err(|e| PluginLoadError::ManifestParseError(e.to_string()))?;

    if !valid_id(&raw.id) {
        return Err(PluginLoadError::InvalidId(raw.id));
    }
    if raw.name.chars().count() > 64 {
        return Err(PluginLoadError::ManifestParseError(
            "name exceeds 64 characters".to_string(),
        ));
    }

    let version = semver::Version::parse(&raw.version)
        .map_err(|e| PluginLoadError::InvalidVersion(format!("{}: {e}", raw.version)))?;
    let host_api = semver::VersionReq::parse(&raw.host_api)
        .map_err(|e| PluginLoadError::InvalidVersion(format!("host_api {}: {e}", raw.host_api)))?;

    let host_version = semver::Version::new(HOST_PLUGIN_API_VERSION as u64, 0, 0);
    if !host_api.matches(&host_version) {
        return Err(PluginLoadError::ApiVersionMismatch {
            plugin: raw.host_api,
            host: HOST_PLUGIN_API_VERSION,
        });
    }

    if raw.types.is_empty() {
        return Err(PluginLoadError::ManifestParseError(
            "types must declare at least one of highlighter|keybinding|menu".to_string(),
        ));
    }
    if raw.types.contains(&PluginType::Highlighter) && raw.extensions.is_empty() {
        return Err(PluginLoadError::ManifestParseError(
            "highlighter plugins must declare at least one extension".to_string(),
        ));
    }

    let dir = manifest_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();

    let keybindings: Vec<(String, String)> = raw.keybindings.into_iter().collect();

    let menu_items: Vec<PluginMenuItem> = raw
        .menu_items
        .into_iter()
        .map(|m| PluginMenuItem {
            menu: m.menu,
            item: m.item,
            item_id: m.item_id,
            plugin_id: raw.id.clone(),
            position: m.position,
        })
        .collect();

    // Reject empty menu fields (UTF-8 is guaranteed by Rust String).
    for mi in &menu_items {
        if mi.menu.is_empty() || mi.item.is_empty() || mi.item_id.is_empty() {
            return Err(PluginLoadError::ManifestParseError(
                "menu_items entries must have non-empty menu, item, and item_id".to_string(),
            ));
        }
    }

    let mut permissions = Vec::new();
    for p in raw.permissions.read_paths {
        permissions.push(Permission::ReadPath(p.into()));
    }
    for p in raw.permissions.write_dirs {
        permissions.push(Permission::WriteDir(p.into()));
    }

    Ok(Plugin {
        id: raw.id,
        name: raw.name,
        version,
        host_api,
        types: raw.types,
        extensions: raw.extensions,
        permissions,
        publisher: raw.publisher,
        description: raw.description,
        keybindings,
        menu_items,
        script_path: dir.join("plugin.rhai"),
        manifest_path: manifest_path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_manifest(dir: &Path, body: &str) -> std::path::PathBuf {
        let p = dir.join("plugin.toml");
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        p
    }

    fn tmpdir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("edit_pm_{}_{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn test_valid_manifest_parses() {
        let d = tmpdir("valid");
        let p = write_manifest(
            &d,
            r#"
id = "lua-syntax"
name = "Lua Syntax"
version = "1.0.0"
host_api = "^1"
types = ["highlighter"]
extensions = [".lua"]
"#,
        );
        let plugin = parse_manifest(&p).expect("should parse");
        assert_eq!(plugin.id, "lua-syntax");
        assert!(plugin.has_type(PluginType::Highlighter));
        assert_eq!(plugin.extensions, vec![".lua".to_string()]);
    }

    #[test]
    fn test_invalid_id_rejected() {
        let d = tmpdir("badid");
        let p = write_manifest(
            &d,
            r#"
id = "-bad-"
name = "x"
version = "1.0.0"
host_api = "^1"
types = ["menu"]
"#,
        );
        assert!(matches!(
            parse_manifest(&p),
            Err(PluginLoadError::InvalidId(_))
        ));
    }

    #[test]
    fn test_missing_extensions_for_highlighter_rejected() {
        let d = tmpdir("noext");
        let p = write_manifest(
            &d,
            r#"
id = "h"
name = "x"
version = "1.0.0"
host_api = "^1"
types = ["highlighter"]
"#,
        );
        assert!(matches!(
            parse_manifest(&p),
            Err(PluginLoadError::ManifestParseError(_))
        ));
    }

    #[test]
    fn test_name_too_long_rejected() {
        let d = tmpdir("longname");
        let long = "n".repeat(65);
        let p = write_manifest(
            &d,
            &format!(
                r#"
id = "h"
name = "{long}"
version = "1.0.0"
host_api = "^1"
types = ["menu"]
"#
            ),
        );
        assert!(matches!(
            parse_manifest(&p),
            Err(PluginLoadError::ManifestParseError(_))
        ));
    }

    #[test]
    fn test_incompatible_host_api_rejected() {
        let d = tmpdir("apimismatch");
        let p = write_manifest(
            &d,
            r#"
id = "h"
name = "x"
version = "1.0.0"
host_api = "^2"
types = ["menu"]
"#,
        );
        assert!(matches!(
            parse_manifest(&p),
            Err(PluginLoadError::ApiVersionMismatch { .. })
        ));
    }
}
