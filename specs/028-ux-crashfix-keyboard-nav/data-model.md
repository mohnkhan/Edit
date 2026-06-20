# Data Model: UX crash-safety and keyboard navigation hardening

This feature changes behavior, not persisted data. The "entities" are the in-memory state objects the
fixes touch.

## Wrap cache (`App.wrap_cache: Option<WrapCache>`, `App.wrap_text_gen: u64`)

- **Represents**: the soft-wrap visual-row layout for the **active** buffer at a given content
  generation and terminal width.
- **Validity rule**: a cache is valid only for the `(width, wrap_text_gen)` it was computed with;
  `WrapCache::is_stale(width, gen)` decides reuse. **New invariant**: `wrap_text_gen` MUST change
  whenever the active buffer's content identity changes (edit — already handled; **plus** restore,
  next/prev buffer, open file, close buffer — added by this feature).
- **Consumer invariant (FR-001)**: any renderer slice into a line MUST be clamped to that line's
  current byte length, so even a stale cache cannot cause an out-of-bounds slice.

## Dialog focus ring (`App.dialog_focus: usize`, `App.dialog_focus_init: bool`)

- **Represents**: the current focus stop within the open dialog. For interactive dialogs the stops are
  `[primary control, button0, button1, …]`; the number of leading field stops is
  `interactive_field_stops()`.
- **State transition (new, FR-005)**: on the first `ensure_dialog_focus()` after an interactive dialog
  opens, `dialog_focus` MUST be set to `0` (primary control). `dialog_focus_init` is the once-per-open
  guard; it resets when no dialog is open.
- **Movement (new, FR-006)**: arrow keys move `dialog_focus` via `buttons::next/prev` over the active
  ring (button dialogs: the button list; interactive dialogs: the combined ring), wrapping around —
  identical to Tab/Shift+Tab.

## Help scroll (`App.help_scroll: usize`, screen `pending_help`)

- **Represents**: the vertical scroll offset of the Help/About overlay.
- **Validity rule (FR-007)**: clamped to `[0, max(0, total_lines - body_rows)]`. New keys: Home→0,
  End→max; Up/Down by 1; PageUp/PageDown by a page — all clamped.

## Panic hook (process-global, `diagnostics::crash`)

- **Represents**: the handler run on a Rust panic.
- **Behavior (new, FR-003)**: before printing the report to stderr it MUST best-effort restore the
  terminal (leave alternate screen, disable mouse capture, show cursor, disable raw mode); it MUST
  still write the crash-log file; it MUST NOT panic itself.

## Selection slice (transient, in `copy_selection`)

- **Represents**: the byte range copied/cut from the active buffer.
- **Validity rule (FR-004)**: the slice MUST be `lo.min(hi)..hi.min(len)`-clamped so a degenerate or
  reversed range yields empty (clipboard-safe) text rather than a panic.
