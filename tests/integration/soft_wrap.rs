//! Integration tests for Feature 005 — Soft-Wrap Mode.

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::Action;

#[test]
fn test_toggle_on_off() {
    let mut app = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    app.terminal_size = (80, 24);

    // Toggle on.
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    assert!(app.soft_wrap, "soft_wrap must be true after first toggle");
    assert!(app.wrap_cache.is_some(), "cache must exist when wrap is on");

    // Toggle off.
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    assert!(
        !app.soft_wrap,
        "soft_wrap must be false after second toggle"
    );
    assert!(
        app.wrap_cache.is_none(),
        "cache must be None when wrap is off"
    );
    assert_eq!(
        app.buffers[0].scroll_offset.1, 0,
        "h-scroll must be 0 after toggle off"
    );
}

#[test]
fn test_toggle_off_buffer_unchanged() {
    let mut app = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    app.terminal_size = (80, 24);

    // Insert content.
    let content = "Hello, world!";
    for c in content.chars() {
        app.handle_action(Action::InsertChar(c)).unwrap();
    }
    let before: String = app.buffers[0].rope.to_string();

    // Toggle on and off.
    app.handle_action(Action::ToggleSoftWrap).unwrap();
    app.handle_action(Action::ToggleSoftWrap).unwrap();

    let after: String = app.buffers[0].rope.to_string();
    assert_eq!(
        before, after,
        "buffer content must be unchanged after toggle cycle"
    );
}

#[test]
fn test_persistence_from_config() {
    let cfg = Config {
        soft_wrap: true,
        ..Default::default()
    };
    let app = App::new(cfg, vec![], EncodingId::Utf8, None, None);
    assert!(
        app.soft_wrap,
        "App must start with soft_wrap=true when config says so"
    );
}
