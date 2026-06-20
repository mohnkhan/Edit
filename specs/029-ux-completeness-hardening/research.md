# Research: UX completeness hardening (round 2)

All items are verified defects from the audit (with confirmed `file:line`); no NEEDS CLARIFICATION
remained. This records the chosen approach per fix.

## D1 — Char-safe text extraction (delete/cut, copy already fixed)

**Finding**: `delete_selection` (src/app.rs ~2549) does `full[s_idx..e_idx.min(full.len())]` where
`s_idx`/`e_idx` are CHAR indices from `char_idx_for` — byte-slicing a `String` with char indices panics
on multibyte text and captures the wrong undo text. `copy_selection` was already fixed (feature 028) via
`selection_text()`.
**Decision**: Reuse the existing char-safe `selection_text()` (or a shared char-range extractor) to
compute the deleted text; clamp `lo.min(hi)..hi.min(total_chars)`. Keep the rope `delete_range` call
(it already takes the same indices) unchanged.
**Alternatives**: Convert to byte indices and slice — rejected; the char-based path already exists and is
correct.

## D2 — Recovery-dialog path truncation

**Finding**: src/ui/dialog.rs ~856 `&self.path[self.path.len()-47..]` byte-slices a path; a Unicode path
>50 bytes can cut mid-character → panic.
**Decision**: Truncate by the shared width/char helper (keep the last N display columns or chars with a
leading `...`), never slicing on a byte boundary.

## D3 — `byte_to_char` boundary safety

**Finding**: src/buffer/rope.rs ~124 `s[..byte_idx].chars().count()` panics if `byte_idx` is not on a
char boundary.
**Decision**: Clamp `byte_idx` down to the nearest char boundary (or saturate to `s.len()`) before
slicing — e.g. walk `char_indices()`/`floor_char_boundary`-style — returning a correct count without
panic.
**Alternatives**: `is_char_boundary` assert — rejected (still panics); we want graceful clamping.

## D4 — File-size guard

**Finding**: src/buffer/mod.rs reads the whole file with `std::fs::read` — a multi-GB file → OOM.
**Decision**: Before reading, `stat` the file; if it exceeds a documented constant (default a few hundred
MB) return a `BufferError` "file too large" that the app surfaces as a status message; do not read.
**Alternatives**: Streaming/partial load — out of scope (rope already holds the whole file); a guard is
the minimal safe fix.

## D5 — Save feedback & failure surfacing

**Finding**: `handle_save_action` (~1829) logs success/failure only; no status message. Save-As sets
"Saved as …".
**Decision**: On success set `status_message = "Saved"` (or "Saved <name>"); on failure set
`status_message = "Save failed: <reason>"` and leave `modified = true`. Mirror for autosave (D6).
**Alternatives**: A modal on failure — rejected as too heavy for a transient; status line is consistent
with the rest of the app.

## D6 — Autosave/recovery failure notice

**Finding**: autosave write failures (src/buffer/autosave.rs) are logged and dropped.
**Decision**: Propagate a failure signal so the app shows a non-intrusive `watcher_notice`/status notice
("Autosave failed"); keep logging.

## D7 — SavePrompt Esc

**Finding**: the save-before-quit intercept (src/app.rs ~981) handles only S/D/C, not `MenuClose`, though
the button says `Cancel (Esc)`. RevertConfirm/CloseConfirm/ExternalChange already handle Esc.
**Decision**: Add `Action::MenuClose | Action::Quit => prompt_cancel_quit()` to the SavePrompt intercept.

## D8 — Save-As encoding via the file browser

**Finding**: `apply_browse_outcome` → `do_save_as(path)` ignores `pending_save_as_encoding`, so a chosen
encoding is dropped; only `handle_save_as` honors it.
**Decision**: In the browser save path, apply `pending_save_as_encoding` (set the buffer encoding) before
`do_save_as`, or route through the encoding-aware path. Clear the pending encoding after.

## D9 — Click → column mapping (gutter + horizontal scroll)

**Finding**: `handle_mouse_click` maps raw terminal `col` to text column without subtracting the
line-number gutter (4 cols when enabled) or adding `scroll_offset.1`. Clicks land ~4 cols off with line
numbers on.
**Decision**: Subtract the gutter width (same `config.line_numbers ? 4 : 0` used by render/viewport) and
account for the horizontal scroll offset in both the soft-wrap and non-wrap click branches. Clicks on the
gutter columns clamp to column 0 and don't mis-place the cursor.

## D10 — Unified display width (the core correctness fix)

**Finding**: Two custom width helpers — `file_browser::grapheme_width` and
`app::unicode_segmentation_width` — return 1 for combining marks (should be 0) and mishandle emoji; the
editor/buttons use `unicode-width` directly. Divergence causes cursor/scroll/truncation misalignment.
**Decision**: Add `src/ui/width.rs` with `display_width(grapheme: &str) -> usize` (sum of
`unicode_width::UnicodeWidthStr::width`, which already yields 0 for combining marks and 2 for wide) and
`str_width(&str)`. Replace both custom helpers and the editor click width call with it. Keep behavior for
control chars consistent with today (tabs handled where they already are).
**Rationale**: One source of truth satisfies the constitution's width mandate and removes three latent
mis-measurements at once.
**Alternatives**: Fix each helper in place — rejected (leaves two copies to drift again).

## D11 — Ctrl+W → Close + File ▸ Close menu

**Finding**: `Action::Close` exists and is handled but is unbound and absent from menus, while
`docs/CAPABILITIES.md` claims `Ctrl+W` closes the buffer.
**Decision**: Bind `Ctrl+W` → `Action::Close` in `default_map`; add a `File ▸ Close` menu item routing to
the same action.

## D12 — Light-theme selected menu legibility

**Finding**: the light theme sets `menu_selected_bg = White` while the selected style uses a white
foreground → white-on-white (invisible). The classic theme is fine (cyan-on-black).
**Decision**: Give the light theme a contrasting `menu_selected_bg` (e.g. Blue or Black) so the selected
item is legible; verify both themes by a render assertion.

## D13 — Go-to-Line modal guard

**Finding**: the Go-to-Line open path doesn't check `menu_bar.is_active()`, so it can open over a menu.
**Decision**: Guard the open like the other modals (no-op or close the menu first), consistent with
modal precedence.

## D14 — Action feedback (copy/cut/paste, read-only, file-open)

**Finding**: copy/cut/paste set no status; clipboard failures and empty-clipboard paste are silent;
read-only edits silently `return`; file-open failures (startup + Ctrl+O) silently yield a blank buffer.
**Decision**: Add concise `status_message` feedback: "Copied"/"Cut"/"Pasted", "Clipboard unavailable",
"Nothing to paste", "Buffer is read-only", and "Open failed: <path> — <reason>". For read-only, set the
message at the central edit-guard points. For file open, surface the error instead of substituting a
silent empty buffer (still allow the new-file NotFound case).

## Testing approach

TDD per Principle V. Unit tests inline: char-safe delete on multibyte; recovery-path truncation no-panic;
`byte_to_char` on a non-boundary offset; file-size guard returns the error; SavePrompt Esc cancels;
browser Save-As applies the pending encoding; click maps with gutter+scroll; `width::display_width`
(combining=0, CJK=2, emoji); Ctrl+W bound to Close; both themes render a legible selected item; GoToLine
not opening over a menu; save/copy/cut/paste/read-only/open feedback strings. Integration tests in
`tests/integration/ux_round2.rs` drive `handle_action`/render end to end. A headless render test covers
the theme legibility.
