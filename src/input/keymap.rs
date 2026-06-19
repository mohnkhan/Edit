// src/input/keymap.rs
// Task T017: Action enum and KeybindingMap
// EDIT.COM keybinding model — default DOS bindings plus user override support.

use std::collections::HashMap;

/// All editor actions that can be bound to a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // File operations
    Save,
    SaveAs,
    SaveAsEncoding,
    Open,
    Close,
    Quit,

    // Edit operations
    Cut,
    Copy,
    Paste,
    Undo,
    Redo,
    SelectAll,

    // Search / replace
    Find,
    FindNext,
    FindPrev,
    FindReplace,

    // Menu navigation
    Menu,
    MenuOpen(usize),
    MenuClose,
    MenuFile,
    MenuEdit,
    MenuSearch,
    MenuView,
    MenuOptions,
    MenuHelp,

    // Help
    Help,

    // View toggles
    ToggleLineNumbers,
    ToggleHighlight,
    ToggleTheme,
    ToggleSoftWrap,
    SplitView,
    NextBuffer,
    PrevBuffer,

    // Cursor movement
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveLineStart,
    MoveLineEnd,
    MovePageUp,
    MovePageDown,
    MoveDocStart,
    MoveDocEnd,

    // Text input / deletion
    InsertChar(char),
    Backspace,
    Delete,
    InsertNewline,

    // Terminal resize event
    Resize(u16, u16),

    // Periodic tick (used for cursor blink, etc.)
    Tick,

    // Explicit no-op
    Noop,

    // Feature 007 — External file modification detection
    /// User chose to reload buffer from disk after external modification.
    ReloadFile,
    /// User chose to keep in-editor version after external modification.
    DismissExternalChange,

    // Feature 008 — Plugin API
    /// Open the Options > Plugins manager dialog.
    OpenPluginManager,
    /// Activate a plugin-contributed menu item: `(plugin_id, item_id)`.
    PluginMenuActivated(String, String),
}

/// Maps canonical key-chord strings (e.g. `"Ctrl+S"`) to [`Action`] variants.
pub struct KeybindingMap {
    map: HashMap<String, Action>,
}

impl KeybindingMap {
    /// Construct the map that matches EDIT.COM's default DOS key bindings.
    pub fn default_map() -> KeybindingMap {
        let mut map: HashMap<String, Action> = HashMap::new();

        // File
        map.insert("Ctrl+S".to_string(), Action::Save);
        map.insert("F5".to_string(), Action::Save);
        map.insert("F12".to_string(), Action::SaveAsEncoding);
        map.insert("Ctrl+O".to_string(), Action::Open);
        map.insert("Ctrl+Q".to_string(), Action::Quit);

        // Edit
        map.insert("Ctrl+X".to_string(), Action::Cut);
        map.insert("Ctrl+C".to_string(), Action::Copy);
        map.insert("Ctrl+V".to_string(), Action::Paste);
        map.insert("Ctrl+Z".to_string(), Action::Undo);
        map.insert("Ctrl+Y".to_string(), Action::Redo);
        map.insert("Ctrl+A".to_string(), Action::SelectAll);

        // Search
        map.insert("Ctrl+F".to_string(), Action::Find);
        map.insert("F3".to_string(), Action::FindNext);
        map.insert("F2".to_string(), Action::FindPrev);
        map.insert("Ctrl+H".to_string(), Action::FindReplace);

        // Help / menu
        map.insert("F1".to_string(), Action::Help);
        map.insert("F10".to_string(), Action::Menu);
        // Escape closes an open menu/dropdown and cancels modal dialogs
        // (DOS-faithful: ESC always backs out of the current context).
        map.insert("Esc".to_string(), Action::MenuClose);

        // Soft-wrap toggle (Feature 005)
        map.insert("Alt+Z".to_string(), Action::ToggleSoftWrap);

        // Alt-key menu shortcuts
        map.insert("Alt+F".to_string(), Action::MenuFile);
        map.insert("Alt+E".to_string(), Action::MenuEdit);
        map.insert("Alt+S".to_string(), Action::MenuSearch);
        map.insert("Alt+V".to_string(), Action::MenuView);
        map.insert("Alt+O".to_string(), Action::MenuOptions);
        map.insert("Alt+H".to_string(), Action::MenuHelp);

        KeybindingMap { map }
    }

    /// Overlay user-supplied overrides on top of the current map.
    ///
    /// For each `(key_str, action_str)` pair:
    /// - If the key is already bound, a WARN is logged and the user binding wins.
    /// - If `action_str` does not correspond to a known action, an ERROR is logged
    ///   and that entry is skipped.
    pub fn apply_user_overrides(&mut self, overrides: &HashMap<String, String>) {
        for (key_str, action_str) in overrides {
            match action_from_str(action_str) {
                Some(action) => {
                    if self.map.contains_key(key_str.as_str()) {
                        log::warn!(
                            "Key binding conflict: '{}' was already bound to '{:?}'; \
                             user override '{}' wins.",
                            key_str,
                            self.map[key_str.as_str()],
                            action_str
                        );
                    }
                    self.map.insert(key_str.clone(), action);
                }
                None => {
                    log::error!(
                        "Unknown action '{}' for key '{}'; skipping override.",
                        action_str,
                        key_str
                    );
                }
            }
        }
    }

    /// Merge plugin-provided keybindings (Feature 008).
    ///
    /// Plugin bindings take precedence over built-ins (a conflict is logged at WARN),
    /// EXCEPT safety-critical built-in bindings (Quit, Save) which a plugin may not steal —
    /// such attempts are logged and discarded. Unknown action names are logged and skipped.
    pub fn apply_plugin_bindings(&mut self, bindings: &[(String, String)]) {
        for (key, action_str) in bindings {
            match plugin_action_from_str(action_str) {
                Some(action) => {
                    if let Some(existing) = self.map.get(key.as_str()) {
                        if is_safety_critical(existing) {
                            log::warn!(
                                "Plugin attempted to override safety-critical binding '{}' \
                                 ({:?}); ignored.",
                                key,
                                existing
                            );
                            continue;
                        }
                        log::warn!("Plugin overrides built-in binding for '{}'.", key);
                    }
                    self.map.insert(key.clone(), action);
                }
                None => log::error!(
                    "Unknown plugin action '{}' for key '{}'; skipping.",
                    action_str,
                    key
                ),
            }
        }
    }

    /// Look up the action bound to a canonical key-chord string.
    pub fn get_action(&self, key: &str) -> Option<&Action> {
        self.map.get(key)
    }
}

/// Built-in actions a plugin must never be allowed to rebind.
fn is_safety_critical(action: &Action) -> bool {
    matches!(action, Action::Quit | Action::Save)
}

/// Resolve a plugin action name, accepting both PascalCase (config style) and the
/// lowercase form used in plugin manifests (e.g. `"save"`).
fn plugin_action_from_str(s: &str) -> Option<Action> {
    if let Some(a) = action_from_str(s) {
        return Some(a);
    }
    match s.to_ascii_lowercase().as_str() {
        "save" => Some(Action::Save),
        "saveas" => Some(Action::SaveAs),
        "quit" => Some(Action::Quit),
        "open" => Some(Action::Open),
        "close" => Some(Action::Close),
        "cut" => Some(Action::Cut),
        "copy" => Some(Action::Copy),
        "paste" => Some(Action::Paste),
        "undo" => Some(Action::Undo),
        "redo" => Some(Action::Redo),
        "find" => Some(Action::Find),
        "findnext" => Some(Action::FindNext),
        "findprev" => Some(Action::FindPrev),
        "selectall" => Some(Action::SelectAll),
        _ => None,
    }
}

/// Parse a human-readable action name into an [`Action`] variant.
///
/// Supports all variants that carry no runtime data (i.e. everything except
/// `InsertChar`, `MenuOpen`, and `Resize`).  Returns `None` for unrecognised
/// strings so callers can emit a diagnostic.
fn action_from_str(s: &str) -> Option<Action> {
    match s {
        "Save" => Some(Action::Save),
        "SaveAs" => Some(Action::SaveAs),
        "SaveAsEncoding" => Some(Action::SaveAsEncoding),
        "Open" => Some(Action::Open),
        "Close" => Some(Action::Close),
        "Quit" => Some(Action::Quit),
        "Cut" => Some(Action::Cut),
        "Copy" => Some(Action::Copy),
        "Paste" => Some(Action::Paste),
        "Undo" => Some(Action::Undo),
        "Redo" => Some(Action::Redo),
        "SelectAll" => Some(Action::SelectAll),
        "Find" => Some(Action::Find),
        "FindNext" => Some(Action::FindNext),
        "FindPrev" => Some(Action::FindPrev),
        "FindReplace" => Some(Action::FindReplace),
        "Menu" => Some(Action::Menu),
        "MenuClose" => Some(Action::MenuClose),
        "MenuFile" => Some(Action::MenuFile),
        "MenuEdit" => Some(Action::MenuEdit),
        "MenuSearch" => Some(Action::MenuSearch),
        "MenuView" => Some(Action::MenuView),
        "MenuOptions" => Some(Action::MenuOptions),
        "MenuHelp" => Some(Action::MenuHelp),
        "Help" => Some(Action::Help),
        "ToggleLineNumbers" => Some(Action::ToggleLineNumbers),
        "ToggleHighlight" => Some(Action::ToggleHighlight),
        "ToggleTheme" => Some(Action::ToggleTheme),
        "ToggleSoftWrap" => Some(Action::ToggleSoftWrap),
        "SplitView" => Some(Action::SplitView),
        "NextBuffer" => Some(Action::NextBuffer),
        "PrevBuffer" => Some(Action::PrevBuffer),
        "MoveUp" => Some(Action::MoveUp),
        "MoveDown" => Some(Action::MoveDown),
        "MoveLeft" => Some(Action::MoveLeft),
        "MoveRight" => Some(Action::MoveRight),
        "MoveLineStart" => Some(Action::MoveLineStart),
        "MoveLineEnd" => Some(Action::MoveLineEnd),
        "MovePageUp" => Some(Action::MovePageUp),
        "MovePageDown" => Some(Action::MovePageDown),
        "MoveDocStart" => Some(Action::MoveDocStart),
        "MoveDocEnd" => Some(Action::MoveDocEnd),
        "Backspace" => Some(Action::Backspace),
        "Delete" => Some(Action::Delete),
        "InsertNewline" => Some(Action::InsertNewline),
        "Tick" => Some(Action::Tick),
        "Noop" => Some(Action::Noop),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_map_contains_save() {
        let km = KeybindingMap::default_map();
        assert_eq!(km.get_action("Ctrl+S"), Some(&Action::Save));
        assert_eq!(km.get_action("F5"), Some(&Action::Save));
    }

    #[test]
    fn default_map_contains_quit() {
        let km = KeybindingMap::default_map();
        assert_eq!(km.get_action("Ctrl+Q"), Some(&Action::Quit));
    }

    #[test]
    fn default_map_binds_ctrl_o_to_open() {
        // Docs promised Ctrl+O = Open file dialog, but it was never bound.
        let km = KeybindingMap::default_map();
        assert_eq!(km.get_action("Ctrl+O"), Some(&Action::Open));
    }

    #[test]
    fn default_map_binds_escape_to_menu_close() {
        // Regression: Escape produced no Action because "Esc" was unbound,
        // so it could never close a menu or cancel a modal dialog.
        let km = KeybindingMap::default_map();
        assert_eq!(km.get_action("Esc"), Some(&Action::MenuClose));
    }

    #[test]
    fn default_map_contains_menu_shortcuts() {
        let km = KeybindingMap::default_map();
        assert_eq!(km.get_action("Alt+F"), Some(&Action::MenuFile));
        assert_eq!(km.get_action("Alt+E"), Some(&Action::MenuEdit));
        assert_eq!(km.get_action("Alt+S"), Some(&Action::MenuSearch));
        assert_eq!(km.get_action("Alt+V"), Some(&Action::MenuView));
        assert_eq!(km.get_action("Alt+O"), Some(&Action::MenuOptions));
        assert_eq!(km.get_action("Alt+H"), Some(&Action::MenuHelp));
    }

    #[test]
    fn default_map_contains_search_keys() {
        let km = KeybindingMap::default_map();
        assert_eq!(km.get_action("Ctrl+F"), Some(&Action::Find));
        assert_eq!(km.get_action("F3"), Some(&Action::FindNext));
        assert_eq!(km.get_action("F2"), Some(&Action::FindPrev));
        assert_eq!(km.get_action("Ctrl+H"), Some(&Action::FindReplace));
    }

    #[test]
    fn user_override_replaces_binding() {
        let mut km = KeybindingMap::default_map();
        let mut overrides = HashMap::new();
        overrides.insert("Ctrl+S".to_string(), "SaveAs".to_string());
        km.apply_user_overrides(&overrides);
        assert_eq!(km.get_action("Ctrl+S"), Some(&Action::SaveAs));
    }

    #[test]
    fn user_override_unknown_action_is_skipped() {
        let mut km = KeybindingMap::default_map();
        let mut overrides = HashMap::new();
        overrides.insert("Ctrl+S".to_string(), "LaunchRocket".to_string());
        km.apply_user_overrides(&overrides);
        // Original binding must be preserved because the override was invalid.
        assert_eq!(km.get_action("Ctrl+S"), Some(&Action::Save));
    }

    #[test]
    fn get_action_returns_none_for_unbound_key() {
        let km = KeybindingMap::default_map();
        assert_eq!(km.get_action("Ctrl+Shift+Z"), None);
    }

    #[test]
    fn action_from_str_round_trips() {
        let cases = [
            ("Save", Action::Save),
            ("Quit", Action::Quit),
            ("Cut", Action::Cut),
            ("Copy", Action::Copy),
            ("Paste", Action::Paste),
            ("Undo", Action::Undo),
            ("Redo", Action::Redo),
            ("Find", Action::Find),
            ("FindNext", Action::FindNext),
            ("FindPrev", Action::FindPrev),
            ("FindReplace", Action::FindReplace),
            ("Help", Action::Help),
            ("Menu", Action::Menu),
            ("MenuFile", Action::MenuFile),
            ("Noop", Action::Noop),
        ];
        for (s, expected) in cases {
            assert_eq!(action_from_str(s), Some(expected), "failed for '{s}'");
        }
    }

    #[test]
    fn action_from_str_returns_none_for_unknown() {
        assert_eq!(action_from_str("DoSomethingCrazy"), None);
    }

    #[test]
    fn test_f12_maps_to_save_as_encoding() {
        let km = KeybindingMap::default_map();
        assert_eq!(km.get_action("F12"), Some(&Action::SaveAsEncoding));
    }

    #[test]
    fn test_save_as_encoding_round_trips_action_from_str() {
        assert_eq!(
            action_from_str("SaveAsEncoding"),
            Some(Action::SaveAsEncoding)
        );
    }

    #[test]
    fn test_alt_z_maps_to_toggle_soft_wrap() {
        let km = KeybindingMap::default_map();
        assert_eq!(km.get_action("Alt+Z"), Some(&Action::ToggleSoftWrap));
    }

    #[test]
    fn test_toggle_soft_wrap_round_trips_action_from_str() {
        assert_eq!(
            action_from_str("ToggleSoftWrap"),
            Some(Action::ToggleSoftWrap)
        );
    }
}
