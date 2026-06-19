//! Integration tests for the Rhai plugin subsystem (Feature 008).
//!
//! These drive the `edit` library crate's `plugin` module directly with text fixtures under
//! `tests/fixtures/plugins/`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use edit::input::{Action, KeybindingMap};
use edit::plugin::consent::ConsentRecord;
use edit::plugin::PluginHost;

/// Copy a fixture plugin directory into a fresh temp config dir's `plugins/<id>/`.
fn install_fixture(config_dir: &Path, fixture_id: &str) {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/plugins")
        .join(fixture_id);
    let dst = config_dir.join("plugins").join(fixture_id);
    std::fs::create_dir_all(&dst).unwrap();
    for name in ["plugin.toml", "plugin.rhai"] {
        let s = src.join(name);
        if s.exists() {
            std::fs::copy(&s, dst.join(name)).unwrap();
        }
    }
}

fn temp_config_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "edit_plugintest_{}_{}_{}",
        tag,
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn allow(id: &str) -> BTreeMap<String, ConsentRecord> {
    let mut m = BTreeMap::new();
    m.insert(
        id.to_string(),
        ConsentRecord {
            allowed: true,
            consented_at: "2026-06-19T00:00:00Z".to_string(),
            version_consented: "1.0.0".to_string(),
        },
    );
    m
}

fn classic() -> &'static edit::ui::theme::Theme {
    edit::ui::theme::theme_by_name("classic")
}

// ── Loading / consent ────────────────────────────────────────────────────────

#[test]
fn test_no_plugins_flag_loads_nothing() {
    let cfg = temp_config_dir("noplugins");
    install_fixture(&cfg, "lua-syntax");
    let mut host = PluginHost::new(true); // --no-plugins
    let mut pending = Vec::new();
    host.load_all(&cfg, &BTreeMap::new(), &mut pending);
    assert!(host.registry.instances.is_empty());
    assert!(pending.is_empty());
}

#[test]
fn test_unconsented_plugin_goes_to_pending() {
    let cfg = temp_config_dir("pending");
    install_fixture(&cfg, "lua-syntax");
    let mut host = PluginHost::new(false);
    let mut pending = Vec::new();
    host.load_all(&cfg, &BTreeMap::new(), &mut pending);
    assert!(host.registry.instances.is_empty());
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, "lua-syntax");
}

// ── US1: Syntax highlighter ──────────────────────────────────────────────────

#[test]
fn test_highlighter_plugin_loads_and_returns_tokens() {
    let cfg = temp_config_dir("hl");
    install_fixture(&cfg, "lua-syntax");
    let mut host = PluginHost::new(false);
    host.load_all(&cfg, &allow("lua-syntax"), &mut Vec::new());
    assert_eq!(host.registry.instances.len(), 1);

    let theme = classic();
    let hl = host
        .highlighter_for(Path::new("foo.lua"), theme)
        .expect("a plugin highlighter for .lua");
    let spans = hl.highlight("-- a comment");
    assert!(!spans.is_empty(), "expected at least one comment span");
    assert_eq!(spans[0].style.fg, Some(theme.highlight_comment));
}

#[test]
fn test_highlighter_colors_keyword_and_number() {
    let cfg = temp_config_dir("kw");
    install_fixture(&cfg, "lua-syntax");
    let mut host = PluginHost::new(false);
    host.load_all(&cfg, &allow("lua-syntax"), &mut Vec::new());

    let theme = classic();
    let hl = host.highlighter_for(Path::new("a.lua"), theme).unwrap();
    let spans = hl.highlight("local x = 5");
    let has_keyword = spans
        .iter()
        .any(|s| s.style.fg == Some(theme.highlight_keyword));
    let has_number = spans
        .iter()
        .any(|s| s.style.fg == Some(theme.highlight_number));
    assert!(has_keyword, "expected a keyword span for `local`");
    assert!(has_number, "expected a number span for `5`");
}

#[test]
fn test_highlighter_invalid_tokens_discarded_not_disabled() {
    let cfg = temp_config_dir("bad");
    install_fixture(&cfg, "bad-tokens");
    let mut host = PluginHost::new(false);
    host.load_all(&cfg, &allow("bad-tokens"), &mut Vec::new());

    let theme = classic();
    let hl = host.highlighter_for(Path::new("x.lua"), theme).unwrap();
    let spans = hl.highlight("0123456789");
    assert!(spans.is_empty(), "overlapping tokens must be discarded");
    // Plugin must remain enabled (invalid output is not a fault).
    assert!(!host.registry.is_disabled("bad-tokens"));
}

#[test]
fn test_highlighter_timeout_disables_plugin() {
    let cfg = temp_config_dir("timeout");
    install_fixture(&cfg, "infinite-loop");
    let mut host = PluginHost::new(false);
    host.load_all(&cfg, &allow("infinite-loop"), &mut Vec::new());

    let theme = classic();
    let hl = host.highlighter_for(Path::new("x.lua"), theme).unwrap();
    let t0 = Instant::now();
    let spans = hl.highlight("anything at all");
    let elapsed = t0.elapsed();
    assert!(
        elapsed < Duration::from_millis(300),
        "timeout took too long: {elapsed:?}"
    );
    assert!(spans.is_empty());
    assert!(host.registry.is_disabled("infinite-loop"));
    // A disabled plugin is no longer offered as a highlighter.
    assert!(host.highlighter_for(Path::new("x.lua"), theme).is_none());
}

// ── US2: Custom keybindings ──────────────────────────────────────────────────

#[test]
fn test_keybinding_plugin_maps_f9_to_save() {
    let cfg = temp_config_dir("kb");
    install_fixture(&cfg, "custom-keys");
    let mut host = PluginHost::new(false);
    host.load_all(&cfg, &allow("custom-keys"), &mut Vec::new());

    let bindings = host.registry.all_keybindings();
    assert!(
        bindings.iter().any(|(k, a)| k == "F9" && a == "save"),
        "expected F9->save in {bindings:?}"
    );

    let mut km = KeybindingMap::default_map();
    km.apply_plugin_bindings(&bindings);
    assert!(matches!(km.get_action("F9"), Some(Action::Save)));
}

#[test]
fn test_keybinding_cannot_override_safety_critical() {
    // A plugin must not be able to steal Ctrl+S (Save) and rebind it to Quit.
    let mut km = KeybindingMap::default_map();
    let rogue = vec![("Ctrl+S".to_string(), "quit".to_string())];
    km.apply_plugin_bindings(&rogue);
    assert!(
        matches!(km.get_action("Ctrl+S"), Some(Action::Save)),
        "Ctrl+S must remain bound to Save"
    );
}

// ── US3: Menu items ──────────────────────────────────────────────────────────

#[test]
fn test_menu_plugin_registers_item() {
    let cfg = temp_config_dir("menu");
    install_fixture(&cfg, "word-count");
    let mut host = PluginHost::new(false);
    host.load_all(&cfg, &allow("word-count"), &mut Vec::new());

    let items = host.registry.menu_items();
    assert!(
        items
            .iter()
            .any(|m| m.menu == "Tools" && m.item == "Word Count" && m.item_id == "wc"),
        "expected Tools>Word Count in {items:?}"
    );
}

#[test]
fn test_menu_action_returns_word_count() {
    let cfg = temp_config_dir("wc");
    install_fixture(&cfg, "word-count");
    let mut host = PluginHost::new(false);
    host.load_all(&cfg, &allow("word-count"), &mut Vec::new());

    let msg = host.dispatch_menu_action("word-count", "wc", "one two three four five");
    let msg = msg.expect("a status message");
    assert!(msg.contains('5'), "expected count 5 in '{msg}'");
}

#[test]
fn test_menu_undeclared_fs_path_denied_and_disabled() {
    // US5: the fs-violation plugin repeatedly tries read_file on an undeclared path.
    let cfg = temp_config_dir("fsviol");
    install_fixture(&cfg, "fs-violation");
    let mut host = PluginHost::new(false);
    host.load_all(&cfg, &allow("fs-violation"), &mut Vec::new());

    // Each call is denied; after 3 the plugin is disabled. Editor stays intact throughout.
    for _ in 0..3 {
        let _ = host.dispatch_menu_action("fs-violation", "leak", "");
    }
    assert!(host.registry.is_disabled("fs-violation"));
}

// ── US4: Plugin manager + consent persistence ────────────────────────────────

#[test]
fn test_consent_denied_skips_load() {
    let cfg = temp_config_dir("denied");
    install_fixture(&cfg, "lua-syntax");
    let mut deny = BTreeMap::new();
    deny.insert(
        "lua-syntax".to_string(),
        ConsentRecord {
            allowed: false,
            consented_at: "2026-06-19T00:00:00Z".to_string(),
            version_consented: "1.0.0".to_string(),
        },
    );
    let mut host = PluginHost::new(false);
    let mut pending = Vec::new();
    host.load_all(&cfg, &deny, &mut pending);
    assert!(
        host.registry.instances.is_empty(),
        "denied plugin must not load"
    );
    assert!(pending.is_empty(), "denied plugin must not re-prompt");
}

#[test]
fn test_manager_toggle_disables_highlighter() {
    let cfg = temp_config_dir("toggle");
    install_fixture(&cfg, "lua-syntax");
    let mut host = PluginHost::new(false);
    host.load_all(&cfg, &allow("lua-syntax"), &mut Vec::new());
    assert!(host
        .highlighter_for(Path::new("a.lua"), classic())
        .is_some());

    // Disabling via the manager makes the plugin inactive.
    host.registry.set_enabled("lua-syntax", false);
    assert!(host
        .highlighter_for(Path::new("a.lua"), classic())
        .is_none());

    // Re-enabling restores it.
    host.registry.set_enabled("lua-syntax", true);
    assert!(host
        .highlighter_for(Path::new("a.lua"), classic())
        .is_some());
}

#[test]
fn test_consent_record_persists_to_plugins_toml() {
    let cfg = temp_config_dir("persist");
    let rec = ConsentRecord {
        allowed: false,
        consented_at: "2026-06-19T00:00:00Z".to_string(),
        version_consented: "1.0.0".to_string(),
    };
    edit::plugin::save_consent_record(&cfg, "foo", &rec).unwrap();
    let loaded = edit::plugin::load_consent_records(&cfg);
    assert_eq!(loaded.get("foo").map(|r| r.allowed), Some(false));
    assert!(cfg.join("plugins.toml").exists());
}
