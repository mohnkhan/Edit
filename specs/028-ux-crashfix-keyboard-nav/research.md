# Research: UX crash-safety and keyboard navigation hardening

No NEEDS CLARIFICATION remained in Technical Context; this records the decisions behind each fix,
grounded in the current code.

## D1 — Session-restore crash root cause & fix strategy

**Finding**: The soft-wrap renderer (`src/ui/editor.rs`) iterates the cached per-line `wrap_starts`
and slices `let seg_str = &line_str[seg_start..seg_end];` where `seg_end` comes from the cached
segment boundaries. `do_restore_session` (`src/app.rs`) replaces `self.buffers` but never bumps
`wrap_text_gen` / clears `wrap_cache`. The main loop only rebuilds the cache when
`cache.is_stale(width, wrap_text_gen)` is true, so a same-generation cache built for the *old* buffer
is reused against the *new* buffer's lines → `end byte index N is out of bounds for string of length 0`
(observed crash, `crash-1781946189.log`). The same staleness affects `next_buffer`, `prev_buffer`,
`handle_open_file`, and `close_buffer_at`/`close_active_buffer`, none of which bump the generation.

**Decision**: Two complementary fixes.
- **Defense-in-depth (must-not-panic)**: clamp `seg_start`/`seg_end` to `line_str.len()` (and verify
  the non-wrap render path's slices) so a stale or mismatched cache can never panic — it renders a
  truncated/blank line at worst.
- **Correctness (root cause)**: add a single `invalidate_wrap_cache()` helper that bumps
  `wrap_text_gen`, and call it from every place the active buffer's content identity changes (restore,
  switch next/prev, open, close). This reuses the existing generation-based staleness mechanism.

**Rationale**: Clamping alone would hide a real correctness bug (wrong wrap layout for one frame);
invalidation alone would still leave the renderer one stale frame away from a panic on any future
code path. Both together give "never panic" + "always correct."

**Alternatives considered**: (a) Recompute the cache eagerly inside each switch site — rejected:
duplicates the width/gen logic already centralized in the render loop. (b) Store the owning buffer
index in the cache and treat a mismatch as stale — heavier; the generation bump is simpler and already
the established invalidation channel.

## D2 — Terminal restoration on panic

**Finding**: `install_panic_hook` in `src/diagnostics/crash.rs` writes the crash file and prints to
stderr but never restores the terminal. Normal teardown (`disable_raw_mode` + `LeaveAlternateScreen`,
done in `App::run`'s cleanup ~`src/app.rs:504`) is bypassed on panic, so the terminal stays in raw
mode on the alternate screen — the "hangs badly" symptom; the printed report is also invisible/garbled.

**Decision**: At the top of the panic hook, best-effort restore the terminal **before** writing to
stderr: `execute!(stdout, LeaveAlternateScreen, DisableMouseCapture, Show)` then `disable_raw_mode()`,
ignoring errors. Keep the crash-file write unchanged.

**Rationale**: Mirrors the exact teardown the app already performs on clean exit; ignoring errors is
correct because the hook must not itself panic (and may run when the terminal was never initialized,
e.g. headless tests).

**Alternatives considered**: Wrapping `run()` in `catch_unwind` and restoring there — rejected: the
panic hook is process-wide and also covers panics outside `run`; a global hook is the canonical place.

## D3 — Interactive-dialog focus reset (Save-As typing)

**Finding**: `ensure_dialog_focus()` only initializes `dialog_focus` for **button** dialogs
(`open_button_dialog()`); for **interactive** dialogs (`InteractiveDialog`: EncodingSelect,
PluginManager, FindReplace, FileBrowser) it just clears the init flag, so `dialog_focus` carries over
from a previous dialog. `interactive_field_stops()` for the file browser is 1 (stop 0 = field). If
`dialog_focus >= 1` on open (left over from e.g. a SavePrompt that set focus to 2), the file browser
opens with a *button* focused; `handle_action`'s file-browser branch then routes keys to the button
path, which ignores `InsertChar`, so typing is swallowed and the caret (drawn at the focused field)
is not shown.

**Decision**: Initialize `dialog_focus` to `0` (the primary control) when an interactive dialog
becomes open and hasn't been initialized yet — i.e. extend `ensure_dialog_focus()` to also cover the
interactive-dialog case (reuse `dialog_focus_init` as the once-per-open guard).

**Rationale**: Single, central fix that makes every interactive dialog open on its primary control,
matching user expectation; no per-call-site changes.

**Alternatives considered**: Resetting focus inside each `file_browser = Some(...)` / dialog-open site —
rejected: scattered and easy to miss; centralizing in `ensure_dialog_focus` covers all four dialogs.

## D4 — Arrow-key movement between dialog buttons

**Finding**: Both rings move focus only on Tab/Shift+Tab. Feature 016 (confirm dialogs) handles
`DialogFocusNext`/`Prev` via `buttons::next/prev`; feature 020 (interactive) does the same over the
combined ring. `Action::MoveLeft/MoveRight/MoveUp/MoveDown` are currently ignored (016) or routed to
the primary control only (020).

**Decision**: In the 016 button-dialog intercept, treat `MoveRight`/`MoveDown` as focus-next and
`MoveLeft`/`MoveUp` as focus-prev (same `buttons::next/prev` calls as Tab). In the 020 intercept, when
focus is on a button, treat the arrows the same way over the ring; when focus is on the primary
control, preserve existing behavior (arrows drive the list/field). Buttons are a single horizontal row,
so Left/Right and Up/Down are equivalent prev/next.

**Rationale**: Reuses the existing wrap-around helpers; consistent, minimal, and discoverable.

**Alternatives considered**: Only Left/Right (not Up/Down) — rejected: confirm dialogs are a single
row and users press either; supporting both is free and friendlier.

## D5 — Help/About keyboard scroll & dismissal

**Finding**: The Help intercept already handles `MoveUp`/`MoveDown`/`PageUp`/`PageDown`/Esc/Enter for
`help_scroll`; it does **not** handle `Home`/`End`, and once Home/End are bound globally (D6) they
should jump to top/bottom. The Close button is rendered (feature 021) but only clickable. Esc/Enter
already dismiss from the keyboard.

**Decision**: Add `MoveLineStart`(Home) → `help_scroll = 0` and `MoveLineEnd`(End) → clamp to last
page in the Help intercept; keep Esc/Enter dismissal. Confirm Up/Down/PageUp/PageDown clamp to content
(reuse `help_total_lines` for the max). The Close button stays mouse-clickable and is *also* reachable
because Esc/Enter dismiss — no new focus ring needed for a single dismiss action.

**Rationale**: Smallest change that satisfies "scroll and close from the keyboard"; avoids adding a
focus ring to an overlay whose only action is dismiss (already keyboard-served by Esc/Enter).

**Alternatives considered**: A full focus ring with a focusable Close button — rejected as YAGNI
(Principle VI): the overlay has exactly one actionable control and Esc/Enter already trigger it.

## D6 — Home/End key bindings

**Finding**: `default_map()` in `src/input/keymap.rs` binds `Shift+Home`/`Shift+End` to selection but
leaves plain `Home`/`End` unbound; `Action::MoveLineStart`/`MoveLineEnd` exist and are handled.

**Decision**: Bind `Home` → `MoveLineStart`, `End` → `MoveLineEnd`.

**Rationale**: The actions already exist and are handled in the editor and (via D5) Help; this is a
one-line-each keymap addition.

## D7 — PageUp/PageDown in lists

**Finding**: `FileBrowser` exposes `move_up(rows)`/`move_down(rows)` (already used for wheel paging);
encoding-select and plugin-manager lists move one item per Up/Down. None handle `PageUp`/`PageDown`
from the keyboard.

**Decision**: In each list's key intercept, map `PageUp`/`PageDown` to a page jump (file browser:
`move_up/move_down(visible_rows)`; encoding/plugin: clamp `cursor ± visible_page`). Use the same
visible-rows source already used for wheel/selection so paging matches what is drawn.

**Rationale**: Reuses existing movement methods; consistent with the editor's paging.

## D8 — Remaining hardening (copy_selection, file-browser scroll)

**Finding**: `copy_selection` slices `to_string()[s_idx..e_idx]`; although `ordered_range()` is used,
the slice is unguarded against `s_idx > e_idx` or out-of-range indices. `file_browser` scroll computes
`self.selected + 1 - visible_rows`, guarded but fragile (underflow if the guard ever changes).

**Decision**: Clamp the copy slice to `lo.min(hi)..hi.min(len)` (empty/clipboard-safe when degenerate)
and switch the file-browser scroll to `(self.selected + 1).saturating_sub(visible_rows)`.

**Rationale**: Defense-in-depth consistent with FR-001/FR-004; cheap and removes latent panics.

## Testing approach

TDD per Principle V: write failing tests first for each fix. Unit tests inline (renderer no-panic with
a deliberately stale/oversized wrap cache; `invalidate_wrap_cache` bumps generation; focus reset;
button arrow movement via `buttons::next/prev` round-trips; copy guard; file-browser saturating
scroll). Integration tests in `tests/integration/ux_hardening.rs` drive `handle_action`/render for:
session-restore-with-soft-wrap no panic, Save-As typing accumulates + caret, arrow-key button movement
per dialog family, Help keyboard scroll clamping, Home/End cursor movement, list PageUp/Down. The
panic-hook terminal restore is covered by a unit test asserting the restore sequence is invoked
best-effort without panicking (the crash module already tests `write_report` independently).
