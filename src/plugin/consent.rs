//! Persistent per-plugin consent decisions, stored in `$XDG_CONFIG_HOME/edit/plugins.toml`
//! (Feature 008). The file is human-readable so users can inspect and hand-edit decisions.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// One persisted consent decision, keyed by plugin id in `plugins.toml`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsentRecord {
    pub allowed: bool,
    pub consented_at: String,
    pub version_consented: String,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct ConsentFile {
    #[serde(default)]
    plugins: BTreeMap<String, ConsentRecord>,
}

/// The editor config directory (`$XDG_CONFIG_HOME/edit/`), used for `plugins.toml`.
pub fn edit_config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| {
        let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        p.push(".config");
        p
    });
    base.join("edit")
}

fn plugins_toml_path(config_dir: &Path) -> PathBuf {
    config_dir.join("plugins.toml")
}

/// Load all consent records; returns an empty map if the file is absent or unreadable.
pub fn load_consent_records(config_dir: &Path) -> BTreeMap<String, ConsentRecord> {
    let path = plugins_toml_path(config_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return BTreeMap::new();
    };
    match toml::from_str::<ConsentFile>(&text) {
        Ok(f) => f.plugins,
        Err(e) => {
            log::warn!("plugins.toml parse error ({}): {e}", path.display());
            BTreeMap::new()
        }
    }
}

/// Persist (insert or update) a single consent record, writing atomically.
pub fn save_consent_record(
    config_dir: &Path,
    plugin_id: &str,
    record: &ConsentRecord,
) -> std::io::Result<()> {
    let mut current = ConsentFile {
        plugins: load_consent_records(config_dir),
    };
    current
        .plugins
        .insert(plugin_id.to_string(), record.clone());

    std::fs::create_dir_all(config_dir)?;
    let serialized = toml::to_string_pretty(&current).map_err(std::io::Error::other)?;

    let path = plugins_toml_path(config_dir);
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, serialized)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// `Some(true)` = allowed, `Some(false)` = denied, `None` = undecided (needs consent prompt).
pub fn is_allowed(records: &BTreeMap<String, ConsentRecord>, plugin_id: &str) -> Option<bool> {
    records.get(plugin_id).map(|r| r.allowed)
}

/// Current UTC time as an RFC-3339 string (`YYYY-MM-DDTHH:MM:SSZ`), dependency-free.
pub fn utc_now_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, h, mi, s) = civil_from_unix(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// Convert Unix seconds to civil (UTC) Y/M/D H:M:S using Howard Hinnant's algorithm.
fn civil_from_unix(secs: u64) -> (i64, u32, u32, u32, u32, u32) {
    let days = (secs / 86_400) as i64;
    let rem = (secs % 86_400) as u32;
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    // days since 1970-01-01 -> civil date
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d, h, mi, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("edit_consent_{}_{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn test_load_returns_empty_map_when_file_absent() {
        let d = tmpdir("absent");
        assert!(load_consent_records(&d).is_empty());
    }

    #[test]
    fn test_round_trip_persist_and_load() {
        let d = tmpdir("roundtrip");
        let rec = ConsentRecord {
            allowed: true,
            consented_at: "2026-06-19T00:00:00Z".to_string(),
            version_consented: "1.0.0".to_string(),
        };
        save_consent_record(&d, "lua-syntax", &rec).unwrap();
        let loaded = load_consent_records(&d);
        assert_eq!(is_allowed(&loaded, "lua-syntax"), Some(true));
    }

    #[test]
    fn test_is_allowed_returns_none_for_unknown_plugin() {
        let d = tmpdir("unknown");
        let loaded = load_consent_records(&d);
        assert_eq!(is_allowed(&loaded, "nope"), None);
    }

    #[test]
    fn test_utc_now_format() {
        let s = utc_now_rfc3339();
        assert_eq!(s.len(), 20);
        assert!(s.ends_with('Z'));
        // 1970-... sanity via a known epoch
        let (y, mo, d, _, _, _) = civil_from_unix(0);
        assert_eq!((y, mo, d), (1970, 1, 1));
        let (y2, mo2, d2, h2, mi2, s2) = civil_from_unix(1_750_000_000);
        assert_eq!((y2, mo2, d2, h2, mi2, s2), (2025, 6, 15, 15, 6, 40));
    }
}
