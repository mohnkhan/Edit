# Data Model: Interaction completeness

Behavioral/UI-state changes only; no persisted data.

## Editor click-tracker (`App.last_editor_click: Option<(u16, u16, u8, Instant)>`)

- **Represents**: the last editor left-press as `(col, row, count, time)` for classifying the next click.
- **Rules (FR-004/005)**: a press within `DOUBLE_CLICK_MS` of the previous one **and** on the same cell
  increments `count` (1→2→3, then wraps to 1); otherwise `count = 1`. count 1 = position+anchor (existing);
  2 = select word; 3 = select line. Reset/ignored while a modal is open.

## Context menu (`App.pending_context_menu: Option<ContextMenu>`; `src/ui/contextmenu.rs`)

- **Represents**: the open editor context menu — a fixed item list `[Cut, Copy, Paste, Select All]`, a
  `focus` index, and an `anchor` (col,row).
- **Rules (FR-007..010)**: opened only when no other modal/menu is active; positioned at the click and
  clamped on-screen; Up/Down move focus, Enter/Space/click activate (routing to the existing action) then
  close; Esc/outside-click dismiss; non-applicable items are safe no-ops.

## Dialog content regions (renderer-shared geometry)

- **List rows** (encoding select, plugin manager): `row_hit(rect, col, row) -> Option<index>` maps a
  click to a visible list index using the renderer's inner-list origin (FR-001).
- **Text-field interior + caret** (Find query/replacement, Go-to-Line, file-browser Name/path):
  `field_caret_at(field_rect, visible_text, click_col) -> caret_grapheme`, walked by
  `ui::width::display_width` and clamped to the value length (FR-002).

## Selection (`Buffer.selection: Option<Selection>`)

- **Rule (FR-006)**: a double/triple-click sets `selection = Some(Selection { anchor, active })` on
  grapheme columns; Copy/Cut consume it unchanged.

## Keymap (`KeybindingMap` default map)

- **FR-011/012**: adds `F6→NextBuffer`, `Shift+F6→PrevBuffer`, `F8→Cut`, `F9→Copy`, `F11→Paste`;
  existing bindings unchanged.
