# Phase 1 Data Model: Field affordance + Help redesign

## A. File-dialog input box — `src/ui/file_browser.rs`

- `FileBrowser.filename: String` (existing) backs the field in both modes (Save: filename; Open: jump
  path). No new state needed for rendering.
- `compute_layout` reserves a **field region** of label row + a 3-row bordered box (4 rows total) above
  the footer, in BOTH modes (Open previously reserved none). The entry list shrinks accordingly.
- Render: draw the box (Block::ALL), a label ("Name:" in Save, "Go to path:" in Open) on the row above,
  and the field text inside with an always-visible caret (`▏` or block) after the text; long text is
  shown right-anchored/truncated within the box (grapheme-correct).
- Hints updated to mention typing in Open mode.

## B. Help table — `src/ui/mod.rs` + `src/app.rs`

- A static, grouped table model:
  `const HELP_SECTIONS: &[(&str, &[(&str, &str)])]` = sections (title → rows of (key, action)) for
  File / Edit / Search / Selection / View / Menus / Dialogs.
- Render: for each section, a heading line then aligned `KEY   ACTION` rows (key column width = max key
  width across the table, clamped). Built into a `Vec<Line>`; only `[scroll .. scroll+visible]` rows are
  drawn inside the box; a footer cue (e.g. `↑↓ scroll · Esc close` and `▼ more`) shows when scrollable.
- `App`: add `help_scroll: usize` (reset to 0 when Help opens). In the help intercept, `Up`/`Down`/
  `PageUp`/`PageDown` adjust `help_scroll` (clamped); Esc/other dismiss keys close as today. About is
  unchanged (or shares the same scrollable box; no scroll needed as it fits).

## Invariants

- File-dialog field box visible in both modes; caret always shown; text grapheme-correct; typing/confirm
  unchanged (FR-001/002/003/004).
- Help shows all rows via scroll (no silent truncation); aligned columns; modal + Esc closes
  (FR-005/006/007).
- No panic at small sizes; no regression to browse/select or other dialogs (FR-008).
