# Phase 1 Data Model: Centralize Editor UI State

This refactor introduces no persisted data. The "model" here is the in-memory state types on `App`.

## Entity: `Modal` (the single foreground-overlay value)

Replaces ~14 independent fields. At most one overlay is representable at a time (FR-001, FR-002).

```rust
/// The single foreground modal layer. At most one is ever open — the enum makes
/// any other combination unrepresentable. Replaces the prior bag of Option/bool flags.
enum Modal {
    None,
    ContextMenu(crate::ui::contextmenu::ContextMenu),
    SessionRestore(crate::session::SessionData),
    SavePrompt,                                   // was `pending_save_prompt: bool`
    ExternalChange(crate::watcher::ExternalChange),
    RevertConfirm(usize),                         // buffer index
    CloseConfirm(usize),                          // buffer index
    FindReplace(FindReplaceDialog),
    GotoLine { digits: String, caret: usize },    // folds pending_goto_line + _caret
    EncodingSelect { row: usize },                // highlighted row
    FileBrowser(FileBrowser),
    Help { screen: HelpScreen, scroll: usize },   // folds pending_help + help_scroll
    PluginConsent(Vec<crate::plugin::PluginMeta>),// front item is prompted
    PluginManager { cursor: usize },              // folds pending_plugin_manager + plugin_manager_cursor
}
```

**Field→variant mapping** (old → new):

| Removed field(s)                                   | New representation                         |
|----------------------------------------------------|--------------------------------------------|
| `pending_context_menu: Option<ContextMenu>`        | `Modal::ContextMenu(_)`                     |
| `pending_session_restore: Option<SessionData>`     | `Modal::SessionRestore(_)`                  |
| `pending_save_prompt: bool`                        | `Modal::SavePrompt`                         |
| `pending_external_change: Option<ExternalChange>`  | `Modal::ExternalChange(_)`                  |
| `pending_revert_confirm: Option<usize>`            | `Modal::RevertConfirm(usize)`              |
| `pending_close_confirm: Option<usize>`             | `Modal::CloseConfirm(usize)`              |
| `pending_find_replace: Option<FindReplaceDialog>`  | `Modal::FindReplace(_)`                     |
| `pending_goto_line: Option<String>` + `_caret`     | `Modal::GotoLine { digits, caret }`        |
| `pending_encoding_select: Option<usize>`           | `Modal::EncodingSelect { row }`            |
| `file_browser: Option<FileBrowser>`                | `Modal::FileBrowser(_)`                     |
| `pending_help: Option<HelpScreen>` + `help_scroll` | `Modal::Help { screen, scroll }`           |
| `pending_plugin_consent: Vec<PluginMeta>`          | `Modal::PluginConsent(_)` (empty ⇒ `None`) |
| `pending_plugin_manager: bool` + `plugin_manager_cursor` | `Modal::PluginManager { cursor }`    |
| `menu_active: bool`                                | **deleted** (dead; use `menu_bar.is_active()`) |

**Kept as separate fields** (not overlays): `menu_bar: MenuBarState`, `drag_anchor`, `scrollbar_drag`,
`dialog_focus`, `dialog_focus_init`, `pending_save_as_encoding`. (Rationale in research.md R1.)

**Invariants**:
- INV-1: At most one foreground overlay open — guaranteed by the type (single `modal: Modal` field).
- INV-2: `modal_is_open()` ⇔ `!matches!(self.modal, Modal::None)`.
- INV-3: Closing any overlay sets `self.modal = Modal::None` via `close_modal()`; the pre-render
  `clamp_all_cursors()` still runs (FR-011).

## Entity: `Layer` (the single stacking precedence)

Drives both paint order and click resolution (FR-004, FR-005).

```rust
/// UI layers from BOTTOM (painted first) to TOP (painted last / hit-tested first).
/// Declared once; render walks ascending, mouse walks descending.
enum Layer {
    Editor,
    TabBar,        // present only when >= 2 buffers
    MenuBarTop,    // MenuState::TopActive (bar highlighted, no dropdown)
    MenuDropDown,  // MenuState::DropDown (overlays tab bar + editor)
    Modal,         // the foreground overlay (topmost)
}
```

- Active layers at any moment derive from `(self.modal, self.menu_bar.state, tab_bar_visible())`.
- Render: for each active layer ascending, draw it.
- Mouse: for each active layer descending, if its rect contains the cell, dispatch and stop.
- The previous `!dropdown_open &&` tab-bar guard is **removed**: `MenuDropDown` sits above `TabBar` in
  the order, so a click on a cell the dropdown occupies resolves to the dropdown first.

## State transitions (unchanged behavior, new representation)

```
Editing ──open X──▶ Modal::X ──Esc/confirm/cancel──▶ Editing (Modal::None)
Editing ──F10/click menu──▶ menu_bar: TopActive ──open──▶ DropDown ──Esc──▶ Inactive
```

Opening an overlay while another is open is not a transition that can occur (each open path goes through
the single `modal` field; setting a new variant replaces the old). This is the structural enforcement of
FR-001 that previously relied on dispatch ordering.

## No schema / persistence / migration

Session restore payload (`SessionData`) is unchanged on disk; only its in-memory holder moves from a
field to a `Modal` variant. No file format, config, or serialization changes.
