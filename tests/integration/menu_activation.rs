//! Integration tests for live menu-bar keyboard activation (Feature 009).
//!
//! Drives the `edit` library crate's `App` event loop (`handle_action`) to verify
//! keyboard navigation and activation of built-in and plugin-contributed menus.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;
use edit::plugin::consent::ConsentRecord;
use edit::ui::menubar::{resolve_menus, MenuState};

// ── Test helpers (mirrors tests/integration/plugin_api.rs) ────────────────────

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
        "edit_menuact_{}_{}_{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
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

fn plain_app() -> App {
    App::new(Config::default(), vec![], EncodingId::Utf8, None, None)
}

fn type_str(app: &mut App, s: &str) {
    for c in s.chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
}

// ── US1: built-in menu activation by keyboard (T008) ──────────────────────────

#[test]
fn test_keyboard_open_navigate_activate_builtin_save() {
    let dir = temp_config_dir("save");
    let path = dir.join("save.txt");
    std::fs::write(&path, "hello").unwrap();

    let mut app = App::new(
        Config::default(),
        vec![path.clone()],
        EncodingId::Utf8,
        None,
        None,
    );
    // Modify the buffer so we can confirm Save wrote it.
    app.handle_action(Action::InsertChar('X')).unwrap();

    // Open File dropdown, navigate to "Save" (New=0, Open=1, Save=2), activate.
    app.handle_action(Action::MenuFile).unwrap();
    app.handle_action(Action::MoveDown).unwrap(); // -> 1
    app.handle_action(Action::MoveDown).unwrap(); // -> 2 (Save)
    app.handle_action(Action::InsertNewline).unwrap();

    assert!(
        !app.menu_bar.is_active(),
        "menu must close after activation"
    );
    let written = std::fs::read_to_string(&path).unwrap();
    assert!(
        written.contains('X'),
        "Save via keyboard must persist the buffer (got {written:?})"
    );
}

#[test]
fn test_f10_enters_top_active_then_down_opens_dropdown() {
    let mut app = plain_app();
    // F10 → top-level highlight, NO dropdown (FR-015 / remediation H1).
    app.handle_action(Action::Menu).unwrap();
    assert_eq!(app.menu_bar.state, MenuState::TopActive(0));
    // Down then opens File's dropdown.
    app.handle_action(Action::MoveDown).unwrap();
    assert_eq!(
        app.menu_bar.state,
        MenuState::DropDown {
            top_idx: 0,
            item_idx: 0
        }
    );
}

#[test]
fn test_escape_closes_without_action() {
    let mut app = plain_app();
    type_str(&mut app, "abc");
    let before = app.active_buffer().rope.to_string();
    app.handle_action(Action::MenuFile).unwrap();
    app.handle_action(Action::MoveDown).unwrap();
    app.handle_action(Action::MenuClose).unwrap();
    assert!(!app.menu_bar.is_active());
    assert_eq!(app.active_buffer().rope.to_string(), before);
}

#[test]
fn test_navigation_does_not_mutate_buffer() {
    let mut app = plain_app();
    type_str(&mut app, "hello world");
    let text_before = app.active_buffer().rope.to_string();
    let col_before = app.active_buffer().cursor.grapheme_col;
    let line_before = app.active_buffer().cursor.line;

    app.handle_action(Action::MenuFile).unwrap();
    for a in [
        Action::MoveDown,
        Action::MoveUp,
        Action::MoveLeft,
        Action::MoveRight,
    ] {
        app.handle_action(a).unwrap();
    }
    // Navigation must not touch buffer text or editor cursor (FR-006).
    assert_eq!(app.active_buffer().rope.to_string(), text_before);
    assert_eq!(app.active_buffer().cursor.grapheme_col, col_before);
    assert_eq!(app.active_buffer().cursor.line, line_before);
}

// ── US2: plugin menu activation by keyboard (T011) ────────────────────────────

#[test]
fn test_plugin_menu_keyboard_activation_sets_status() {
    let cfg = temp_config_dir("wcmenu");
    install_fixture(&cfg, "word-count");
    let mut app = plain_app();
    app.plugin_host
        .load_all(&cfg, &allow("word-count"), &mut Vec::new());

    // Five-word buffer.
    type_str(&mut app, "one two three four five");

    let menus = resolve_menus(&app.plugin_host.registry.menu_items());
    let tools_idx = menus
        .iter()
        .position(|m| m.label == "Tools")
        .expect("Tools menu present after loading word-count");

    app.handle_action(Action::MenuOpen(tools_idx)).unwrap();
    app.handle_action(Action::InsertNewline).unwrap(); // activate "Word Count"

    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.contains('5'),
        "status bar should show word count: {msg:?}"
    );
    assert!(!app.menu_bar.is_active());
}

#[test]
fn test_plugin_menu_renders_between_options_and_help() {
    let cfg = temp_config_dir("place");
    install_fixture(&cfg, "word-count");
    let mut app = plain_app();
    app.plugin_host
        .load_all(&cfg, &allow("word-count"), &mut Vec::new());

    let menus = resolve_menus(&app.plugin_host.registry.menu_items());
    let labels: Vec<&str> = menus.iter().map(|m| m.label.as_str()).collect();
    let tools = labels.iter().position(|l| *l == "Tools").unwrap();
    let help = labels.iter().position(|l| *l == "Help").unwrap();
    let options = labels.iter().position(|l| *l == "Options").unwrap();
    assert!(options < tools && tools < help, "order: {labels:?}");
    assert_eq!(help, labels.len() - 1, "Help stays rightmost");
}

#[test]
fn test_no_plugins_menu_bar_unchanged() {
    let app = plain_app(); // no fixtures loaded
    let menus = resolve_menus(&app.plugin_host.registry.menu_items());
    let labels: Vec<&str> = menus.iter().map(|m| m.label.as_str()).collect();
    assert_eq!(
        labels,
        ["File", "Edit", "Search", "View", "Options", "Help"]
    );
}

#[test]
fn test_no_plugins_flag_yields_no_plugin_menus() {
    let cfg = Config {
        no_plugins: true,
        ..Config::default()
    };
    let app = App::new(cfg, vec![], EncodingId::Utf8, None, None);
    let menus = resolve_menus(&app.plugin_host.registry.menu_items());
    assert_eq!(menus.len(), 6, "--no-plugins → built-in menus only");
}

#[test]
fn test_disabled_plugin_contributes_no_menu() {
    let cfg = temp_config_dir("disabled");
    install_fixture(&cfg, "word-count");
    let mut app = plain_app();
    app.plugin_host
        .load_all(&cfg, &allow("word-count"), &mut Vec::new());
    app.plugin_host.registry.set_enabled("word-count", false);

    let menus = resolve_menus(&app.plugin_host.registry.menu_items());
    assert!(
        !menus.iter().any(|m| m.label == "Tools"),
        "disabled plugin must contribute no menu"
    );
}

// ── US3: navigation semantics & resilience (T013) ─────────────────────────────

#[test]
fn test_left_right_ring_includes_plugin_menu() {
    let cfg = temp_config_dir("ring");
    install_fixture(&cfg, "word-count");
    let mut app = plain_app();
    app.plugin_host
        .load_all(&cfg, &allow("word-count"), &mut Vec::new());

    // Resolved order: File,Edit,Search,View,Options,Tools,Help (7).
    app.handle_action(Action::MenuOptions).unwrap(); // DropDown{4,_}
    assert!(matches!(
        app.menu_bar.state,
        MenuState::DropDown { top_idx: 4, .. }
    ));
    app.handle_action(Action::MoveRight).unwrap(); // Tools (5)
    assert!(matches!(
        app.menu_bar.state,
        MenuState::DropDown { top_idx: 5, .. }
    ));
    app.handle_action(Action::MoveRight).unwrap(); // Help (6)
    assert!(matches!(
        app.menu_bar.state,
        MenuState::DropDown { top_idx: 6, .. }
    ));
    app.handle_action(Action::MoveRight).unwrap(); // wrap to File (0)
    assert!(matches!(
        app.menu_bar.state,
        MenuState::DropDown { top_idx: 0, .. }
    ));
}

#[test]
fn test_modal_precedence_over_menu() {
    // When a modal dialog is active, its guard runs before the menu guard.
    let mut app = plain_app();
    app.open_plugin_manager();
    app.menu_bar.activate_bar(); // pretend the menu is also active
    app.handle_action(Action::MoveDown).unwrap();
    // The plugin-manager guard consumed the key; menu state is untouched.
    assert_eq!(app.menu_bar.state, MenuState::TopActive(0));
}

#[test]
fn test_plugin_menu_dispatch_failure_surfaces_warning() {
    // Remediation M1 / FR-013 / SC-006: a misbehaving plugin item, activated via
    // the menu, leaves the editor responsive, surfaces a warning, and the plugin
    // is disabled by the dispatch layer — without corrupting the buffer.
    let cfg = temp_config_dir("fail");
    install_fixture(&cfg, "fs-violation");
    let mut app = plain_app();
    app.plugin_host
        .load_all(&cfg, &allow("fs-violation"), &mut Vec::new());
    type_str(&mut app, "keep me");
    let buf_before = app.active_buffer().rope.to_string();

    // The fs-violation plugin denies on each call and disables after 3.
    for _ in 0..3 {
        let menus = resolve_menus(&app.plugin_host.registry.menu_items());
        if let Some(idx) = menus.iter().position(|m| m.label == "Tools") {
            app.handle_action(Action::MenuOpen(idx)).unwrap();
            app.handle_action(Action::InsertNewline).unwrap();
        }
    }

    assert!(
        app.plugin_host.registry.is_disabled("fs-violation"),
        "repeated FS violations must disable the plugin"
    );
    assert!(
        app.status_message.is_some(),
        "a warning must be surfaced in the status bar"
    );
    assert_eq!(
        app.active_buffer().rope.to_string(),
        buf_before,
        "buffer must be intact after a failing plugin activation"
    );
    assert!(!app.menu_bar.is_active());
}
