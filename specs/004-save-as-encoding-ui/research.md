# Phase 0 Research: Save-As Encoding Selection UI

**Feature**: 004-save-as-encoding-ui | **Date**: 2026-06-19

---

## Decision 1: Dialog State Representation

**Decision**: Store the dialog's open/closed state and the highlighted row index as a single `Option<usize>` field (`pending_encoding_select: Option<usize>`) on `App`.

**Rationale**: Follows the exact pattern used by `pending_session_restore: Option<SessionData>` from feature 003. `None` = dialog closed; `Some(idx)` = dialog open with row `idx` highlighted. The index directly addresses the `ENCODING_OPTIONS` static slice. No extra struct needed.

**Alternatives considered**:
- Separate `encoding_dialog_open: bool` + `encoding_dialog_cursor: usize` — two fields to keep in sync; `Option<usize>` unifies them.
- A dedicated `EncodingSelectState` struct — over-engineering for a 7-item listbox with a single usize of state.

---

## Decision 2: Encoding List (Static Slice)

**Decision**: Define a `const ENCODING_OPTIONS: &[(EncodingId, &str)]` in `src/ui/dialog.rs` listing all seven registry entries in a fixed order: UTF-8, UTF-16 LE, UTF-16 BE, CP437, CP850, ISO-8859-1, Windows-1252.

**Rationale**: All seven `EncodingId` variants are appropriate to expose (per spec assumptions). A static const slice is the correct Rust idiom for a fixed, ordered list; it compiles to zero overhead and requires no heap allocation. The display label strings match the human-readable names already used in the status bar and encode pipeline.

**Alternatives considered**:
- Iterating `ENCODING_REGISTRY` at runtime — order is undefined and may vary; a fixed order is better UX.
- Pulling labels from `EncodingProfile::label` — labels are canonical encoding names (e.g. `"UTF-16LE"`), not user-friendly; we want `"UTF-16 LE"` with a space.

---

## Decision 3: F-Key Assignment

**Decision**: Bind `Action::SaveAsEncoding` to `F12`.

**Rationale**: F12 is the only standard F-key in the 1–12 range not already bound in the default keymap. F5 = Save, F10 = Menu, F3/F2 = Find Next/Prev, F1 = Help — all taken. F12 is commonly used for "Save As" variants in other DOS-lineage editors. The menu item is the primary discovery mechanism; F12 is an accelerator for power users.

**Alternatives considered**:
- Ctrl+Shift+S — multi-modifier chords are not DOS-faithful and are harder to type.
- No F-key binding, menu-only — reduces discoverability for keyboard-first users; F12 is the right DOS-style fit.

---

## Decision 4: Dialog Widget Dimensions

**Decision**: `EncodingSelectDialog` renders at 40 columns × 11 rows (7 encoding rows + 1 blank + 1 hint row + 2 border rows = 11). Clamped to terminal size.

**Rationale**: Seven items × 1 row each = 7 data rows. Plus border top/bottom = 2, hint line = 1, blank separator = 1 → 11 total. 40 columns is enough for the longest label ("Windows-1252") plus selection markers and padding. Matches the style of `SavePromptDialog` (52 cols × 5 rows).

**Alternatives considered**:
- Scrollable listbox — unnecessary for 7 fixed items; scrollbars add complexity.
- 60-column width — wider than needed, wastes screen real estate on narrow terminals.

---

## Decision 5: Save Path for New (Unnamed) Buffers

**Decision**: When the active buffer has no `path` (new, unsaved), `do_save_as_encoding` transitions to the existing Save-As filename flow by calling the same path the existing `handle_save_as` uses, then sets `buf.encoding` to the selected encoding before writing.

**Rationale**: Reuses the existing filename-prompt dialog and `OpenFileDialog` widget (which doubles as a path-input widget). No new UI component needed. The encoding selection precedes the filename prompt; if the user cancels the filename prompt, the encoding selection is discarded.

**Alternatives considered**:
- Combining filename + encoding into a single dialog — requires a new two-field dialog widget; unnecessary complexity for an edge case (most uses are on already-named files).
- Encoding dialog after filename prompt — counterintuitive; the user selects what format they want before naming the file.

---

## Decision 6: Status Bar Message Format

**Decision**: On successful save, set `self.status_message = Some(format!("Saved as {}", label))` where `label` is the human-readable encoding name from `ENCODING_OPTIONS`.

**Rationale**: Consistent with existing status messages ("session: no files could be restored", etc.) — all use lowercase prefixes or descriptive phrases. The encoding name in the status bar is already rendered by the `StatusBar` widget; this message is transient (overwritten by the next keypress status update).

**Alternatives considered**:
- "Encoded and saved as UTF-16 LE" — too verbose for a status bar.
- Just updating the encoding indicator without a message — silent; user won't know the save actually happened.

---

## Decision 7: Integration Tests Location

**Decision**: New integration test file `tests/integration/encoding_select.rs`; registered in `Cargo.toml` as `[[test]] name = "encoding_select"`.

**Rationale**: Follows the exact pattern of `tests/integration/session.rs` (feature 003). Each feature gets its own integration test file for clear ownership and independent `cargo test --test <name>` invocation.

**Alternatives considered**:
- Adding tests to `tests/integration/session.rs` — wrong ownership; encoding select is an independent feature.
- Unit tests only — encoding-dialog integration (Action dispatch → encoding change → file write → status message) is better exercised at the integration layer.

---

## Summary: No NEEDS CLARIFICATION Items

All technical decisions are resolved. The feature uses only existing infrastructure:
- `EncodingId` enum and `ENCODING_REGISTRY` — from `src/encoding/detect.rs`
- `encode()` / `Buffer::save()` — from `src/encoding/transcode.rs` + `src/buffer/mod.rs`
- `ratatui` widget system — from `src/ui/dialog.rs`
- `Action` dispatch — from `src/input/keymap.rs`
- `App` state machine — from `src/app.rs`
- Atomic write via tmp-rename — already in `Buffer::save()`
