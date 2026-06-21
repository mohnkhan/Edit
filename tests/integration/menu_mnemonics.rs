//! Integration tests for DOS-style menu mnemonic accelerators (Feature 013).
//!
//! Drives the `edit` library `App` (`handle_action`) to verify that typing an
//! item's accelerator letter while a dropdown is open activates that item, that
//! a non-matching letter is inert, that the menu-inactive insert path is
//! unchanged, and that plugin menu items get working auto-assigned accelerators.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;
use edit::plugin::consent::ConsentRecord;
use edit::ui::menubar::{resolve_menus, MenuState};

// ── Test helpers (mirror tests/integration/menu_activation.rs) ────────────────

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
        "edit_mnem_{}_{}_{}",
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
            consented_at: "2026-06-20T00:00:00Z".to_string(),
            version_consented: "1.0.0".to_string(),
        },
    );
    m
}

fn plain_app() -> App {
    App::new(Config::default(), vec![], EncodingId::Utf8, None, None)
}

// ── US2: letter activation inside an open dropdown ────────────────────────────

#[test]
fn file_new_by_accelerator_creates_buffer_and_closes() {
    let mut app = plain_app();
    let n_before = app.buffers.len();
    app.handle_action(Action::MenuFile).unwrap(); // open File dropdown
    app.handle_action(Action::InsertChar('n')).unwrap(); // 'New'
    assert_eq!(app.buffers.len(), n_before + 1, "New created a buffer");
    assert!(!app.menu_bar.is_active(), "menu closed after activation");
}

#[test]
fn accelerator_is_case_insensitive() {
    let mut app = plain_app();
    let n_before = app.buffers.len();
    app.handle_action(Action::MenuFile).unwrap();
    app.handle_action(Action::InsertChar('N')).unwrap(); // uppercase
    assert_eq!(app.buffers.len(), n_before + 1);
    assert!(!app.menu_bar.is_active());
}

#[test]
fn view_soft_wrap_by_accelerator_toggles() {
    let mut app = plain_app();
    let before = app.active_buffer().soft_wrap;
    app.handle_action(Action::MenuView).unwrap();
    app.handle_action(Action::InsertChar('w')).unwrap(); // 'Soft Wrap (ext)'
    assert_ne!(
        app.active_buffer().soft_wrap,
        before,
        "soft wrap toggled via accelerator"
    );
    assert!(!app.menu_bar.is_active());
}

#[test]
fn non_accelerator_letter_keeps_menu_open_and_buffer_untouched() {
    let mut app = plain_app();
    let n_before = app.buffers.len();
    app.handle_action(Action::MenuFile).unwrap();
    // 'z' is not a File accelerator.
    app.handle_action(Action::InsertChar('z')).unwrap();
    assert!(
        matches!(app.menu_bar.state, MenuState::DropDown { top_idx: 0, .. }),
        "menu stays open on non-matching letter"
    );
    assert_eq!(app.buffers.len(), n_before, "no buffer created");
    assert_eq!(
        app.active_buffer().rope.to_string(),
        "",
        "no character inserted into the buffer"
    );
}

// FR-011: with no menu active, letters insert into the buffer as before.
#[test]
fn letters_insert_normally_when_no_menu_active() {
    let mut app = plain_app();
    app.handle_action(Action::InsertChar('x')).unwrap();
    app.handle_action(Action::InsertChar('y')).unwrap();
    assert_eq!(app.active_buffer().rope.to_string(), "xy");
    assert!(!app.menu_bar.is_active());
}

// US3: with the bar active (no dropdown), a top-level letter opens that menu.
#[test]
fn top_level_letter_opens_menu_when_bar_active() {
    let mut app = plain_app();
    app.handle_action(Action::Menu).unwrap(); // activate bar (TopActive)
    app.handle_action(Action::InsertChar('e')).unwrap(); // Edit
    assert!(matches!(
        app.menu_bar.state,
        MenuState::DropDown { top_idx: 1, .. }
    ));
}

// ── US4: plugin item activation by auto-assigned accelerator ──────────────────

#[test]
fn plugin_item_activates_by_accelerator() {
    let cfg = temp_config_dir("wc");
    install_fixture(&cfg, "word-count");
    let mut app = plain_app();
    app.plugin_host
        .load_all(&cfg, &allow("word-count"), &mut Vec::new());

    // Five-word buffer (typed before opening the menu).
    for c in "one two three four five".chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }

    let menus = resolve_menus(
        &app.plugin_host.registry.menu_items(),
        &app.recent_files.paths,
    );
    let tools_idx = menus
        .iter()
        .position(|m| m.label == "Tools")
        .expect("Tools menu present after loading word-count");
    let wc = menus[tools_idx]
        .items
        .iter()
        .find(|i| i.label == "Word Count")
        .expect("Word Count item present");
    let accel = wc
        .mnemonic
        .expect("plugin item has an auto-assigned accelerator");

    app.handle_action(Action::MenuOpen(tools_idx)).unwrap();
    app.handle_action(Action::InsertChar(accel)).unwrap();

    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.contains('5'),
        "status bar should show word count: {msg:?}"
    );
    assert!(
        !app.menu_bar.is_active(),
        "menu closed after plugin activation"
    );
}
