//! Task T016: Theme definitions for the DOS-faithful UI.
//!
//! This module defines the `Theme` struct and the `CLASSIC` built-in theme.
//! HIGH_CONTRAST and PLAIN themes will be added in task T079.

#![allow(dead_code)]

use ratatui::style::Color;

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/// A complete color scheme for the editor UI.
///
/// Each field maps to a logical role in the rendered interface.  Individual
/// components (menu bar, status bar, editor area, syntax highlights) look up
/// the relevant fields when they paint themselves.
pub struct Theme {
    /// Human-readable theme name, e.g. `"classic"`.
    pub name: &'static str,

    // ---- Editor area -------------------------------------------------------
    /// Background color of the main editing area.
    pub background: Color,
    /// Default foreground (text) color in the editing area.
    pub foreground: Color,

    // ---- Menu bar ----------------------------------------------------------
    /// Background color of the top menu bar.
    pub menubar_bg: Color,
    /// Foreground (label) color of menu bar items.
    pub menubar_fg: Color,
    /// Background color of the currently selected menu item.
    pub menu_selected_bg: Color,

    // ---- Status bar --------------------------------------------------------
    /// Background color of the bottom status bar.
    pub status_bg: Color,
    /// Foreground (text) color of the status bar.
    pub status_fg: Color,

    // ---- Syntax highlighting -----------------------------------------------
    /// Color used for language keywords (`if`, `fn`, `struct`, …).
    pub highlight_keyword: Color,
    /// Color used for string literals.
    pub highlight_string: Color,
    /// Color used for comments.
    pub highlight_comment: Color,
    /// Color used for numeric literals.
    pub highlight_number: Color,
    /// Color used for operators (plugin-provided highlighters; Feature 008).
    pub highlight_operator: Color,
    /// Color used for type names (plugin-provided highlighters; Feature 008).
    pub highlight_type: Color,
}

// ---------------------------------------------------------------------------
// CLASSIC theme
// ---------------------------------------------------------------------------

/// The DOS-faithful blue-background theme, matching the look of the original
/// MS-DOS EDIT.COM / FreeDOS EDIT.
pub static CLASSIC: Theme = Theme {
    name: "classic",

    // Editor area: white text on blue background — the signature DOS look.
    background: Color::Blue,
    foreground: Color::White,

    // Menu bar: black text on cyan, matching the original palette.
    menubar_bg: Color::Cyan,
    menubar_fg: Color::Black,

    // Selected menu item: highlighted with black background.
    menu_selected_bg: Color::Black,

    // Status bar: matches the menu bar for visual coherence.
    status_bg: Color::Cyan,
    status_fg: Color::Black,

    // Syntax highlighting: vivid colors that remain legible on a blue field.
    highlight_keyword: Color::Yellow,
    highlight_string: Color::Green,
    highlight_comment: Color::DarkGray,
    highlight_number: Color::Cyan,
    highlight_operator: Color::White,
    highlight_type: Color::LightCyan,
};

// ---------------------------------------------------------------------------
// HIGH_CONTRAST theme (T079)
// ---------------------------------------------------------------------------

/// High-contrast white-on-black theme for accessibility and low-vision use.
pub static HIGH_CONTRAST: Theme = Theme {
    name: "high-contrast",

    // Editor area: white text on black background — maximum contrast.
    background: Color::Black,
    foreground: Color::White,

    // Menu bar: black text on white for clear separation.
    menubar_bg: Color::White,
    menubar_fg: Color::Black,

    // Selected menu item: the menu renders selected text as `fg = menubar_bg`
    // (White) on `menu_selected_bg`, so this MUST contrast with White or the
    // highlighted item is invisible (Feature 029 fix — was White → white-on-white).
    menu_selected_bg: Color::Black,

    // Status bar: same as editor area for visual continuity.
    status_bg: Color::Black,
    status_fg: Color::White,

    // Syntax highlighting: vivid accessible colors on black.
    highlight_keyword: Color::Yellow,
    highlight_string: Color::Green,
    highlight_comment: Color::Cyan,
    highlight_number: Color::Magenta,
    highlight_operator: Color::White,
    highlight_type: Color::LightYellow,
};

// ---------------------------------------------------------------------------
// PLAIN theme (T079)
// ---------------------------------------------------------------------------

/// Plain theme — all colors deferred to the terminal's own defaults.
///
/// Suitable for terminals with custom palettes or for users who want the
/// editor to inherit the terminal theme without any overrides.
pub static PLAIN: Theme = Theme {
    name: "plain",

    // All colors reset to the terminal default.
    background: Color::Reset,
    foreground: Color::Reset,
    menubar_bg: Color::Reset,
    menubar_fg: Color::Reset,
    menu_selected_bg: Color::Reset,
    status_bg: Color::Reset,
    status_fg: Color::Reset,
    highlight_keyword: Color::Reset,
    highlight_string: Color::Reset,
    highlight_comment: Color::Reset,
    highlight_number: Color::Reset,
    highlight_operator: Color::Reset,
    highlight_type: Color::Reset,
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return the theme with the given name.
///
/// Built-in themes: `"classic"`, `"high-contrast"`, `"plain"`.
/// Any unrecognised name falls back to `CLASSIC`.
pub fn theme_by_name(name: &str) -> &'static Theme {
    match name {
        "classic" => &CLASSIC,
        "high-contrast" => &HIGH_CONTRAST,
        "plain" => &PLAIN,
        _ => &CLASSIC,
    }
}

/// Report whether the current terminal supports color rendering.
///
/// Checks the `TERM` environment variable: if it is unset or set to `"dumb"`,
/// returns `false`; otherwise returns `true`.  This matches the approach used
/// by most Unix utilities and avoids a hard dependency on terminfo.
pub fn terminal_supports_color() -> bool {
    std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_name() {
        assert_eq!(CLASSIC.name, "classic");
    }

    // T029 (Feature 029): the selected menu item must be legible — the menubar
    // renders it as `fg = menubar_bg` on `menu_selected_bg`, so for any theme that
    // sets explicit colors the two must differ (high-contrast was White-on-White).
    #[test]
    fn selected_menu_item_is_legible_in_color_themes() {
        for t in [&CLASSIC, &HIGH_CONTRAST] {
            assert_ne!(
                t.menu_selected_bg, t.menubar_bg,
                "{}: selected menu item is invisible (menu_selected_bg == menubar_bg)",
                t.name
            );
        }
    }

    #[test]
    fn classic_background_is_blue() {
        assert_eq!(CLASSIC.background, Color::Blue);
    }

    #[test]
    fn classic_foreground_is_white() {
        assert_eq!(CLASSIC.foreground, Color::White);
    }

    #[test]
    fn classic_menubar_bg_is_cyan() {
        assert_eq!(CLASSIC.menubar_bg, Color::Cyan);
    }

    #[test]
    fn classic_menubar_fg_is_black() {
        assert_eq!(CLASSIC.menubar_fg, Color::Black);
    }

    #[test]
    fn classic_menu_selected_bg_is_black() {
        assert_eq!(CLASSIC.menu_selected_bg, Color::Black);
    }

    #[test]
    fn classic_status_bg_is_cyan() {
        assert_eq!(CLASSIC.status_bg, Color::Cyan);
    }

    #[test]
    fn classic_status_fg_is_black() {
        assert_eq!(CLASSIC.status_fg, Color::Black);
    }

    #[test]
    fn classic_highlight_keyword_is_yellow() {
        assert_eq!(CLASSIC.highlight_keyword, Color::Yellow);
    }

    #[test]
    fn classic_highlight_string_is_green() {
        assert_eq!(CLASSIC.highlight_string, Color::Green);
    }

    #[test]
    fn classic_highlight_comment_is_darkgray() {
        assert_eq!(CLASSIC.highlight_comment, Color::DarkGray);
    }

    #[test]
    fn classic_highlight_number_is_cyan() {
        assert_eq!(CLASSIC.highlight_number, Color::Cyan);
    }

    #[test]
    fn theme_by_name_classic() {
        let t = theme_by_name("classic");
        assert_eq!(t.name, "classic");
    }

    #[test]
    fn theme_by_name_unknown_falls_back_to_classic() {
        let t = theme_by_name("not-a-real-theme");
        assert_eq!(t.name, "classic");
    }

    #[test]
    fn terminal_supports_color_dumb_returns_false() {
        // TERM=dumb must report no color support
        std::env::set_var("TERM", "dumb");
        assert!(!terminal_supports_color());
        // Restore to a sensible value so other tests are not affected.
        std::env::set_var("TERM", "xterm-256color");
    }

    #[test]
    fn terminal_supports_color_xterm_returns_true() {
        std::env::set_var("TERM", "xterm-256color");
        assert!(terminal_supports_color());
    }

    // T079 — HIGH_CONTRAST theme tests
    #[test]
    fn high_contrast_name() {
        assert_eq!(HIGH_CONTRAST.name, "high-contrast");
    }

    #[test]
    fn high_contrast_background_is_black() {
        assert_eq!(HIGH_CONTRAST.background, Color::Black);
    }

    #[test]
    fn high_contrast_foreground_is_white() {
        assert_eq!(HIGH_CONTRAST.foreground, Color::White);
    }

    #[test]
    fn high_contrast_menubar_bg_is_white() {
        assert_eq!(HIGH_CONTRAST.menubar_bg, Color::White);
    }

    #[test]
    fn high_contrast_menubar_fg_is_black() {
        assert_eq!(HIGH_CONTRAST.menubar_fg, Color::Black);
    }

    #[test]
    fn high_contrast_status_bg_is_black() {
        assert_eq!(HIGH_CONTRAST.status_bg, Color::Black);
    }

    #[test]
    fn high_contrast_status_fg_is_white() {
        assert_eq!(HIGH_CONTRAST.status_fg, Color::White);
    }

    #[test]
    fn high_contrast_keyword_is_yellow() {
        assert_eq!(HIGH_CONTRAST.highlight_keyword, Color::Yellow);
    }

    #[test]
    fn high_contrast_string_is_green() {
        assert_eq!(HIGH_CONTRAST.highlight_string, Color::Green);
    }

    #[test]
    fn high_contrast_comment_is_cyan() {
        assert_eq!(HIGH_CONTRAST.highlight_comment, Color::Cyan);
    }

    #[test]
    fn high_contrast_number_is_magenta() {
        assert_eq!(HIGH_CONTRAST.highlight_number, Color::Magenta);
    }

    // T079 — PLAIN theme tests
    #[test]
    fn plain_name() {
        assert_eq!(PLAIN.name, "plain");
    }

    #[test]
    fn plain_background_is_reset() {
        assert_eq!(PLAIN.background, Color::Reset);
    }

    #[test]
    fn plain_foreground_is_reset() {
        assert_eq!(PLAIN.foreground, Color::Reset);
    }

    #[test]
    fn plain_menubar_bg_is_reset() {
        assert_eq!(PLAIN.menubar_bg, Color::Reset);
    }

    #[test]
    fn plain_status_bg_is_reset() {
        assert_eq!(PLAIN.status_bg, Color::Reset);
    }

    // T079 — theme_by_name routing
    #[test]
    fn theme_by_name_high_contrast() {
        let t = theme_by_name("high-contrast");
        assert_eq!(t.name, "high-contrast");
    }

    #[test]
    fn theme_by_name_plain() {
        let t = theme_by_name("plain");
        assert_eq!(t.name, "plain");
    }
}
