# Learnings

A running log of bugs, near-misses, and weird behaviors found while working on `edit`, organized so
the **recurring patterns** are reusable. When you fix something subtle, add it here — especially if it
took a while to find or surprised you. Newest patterns/sessions near the top of each section.

---

## Recurring bug patterns (read this first)

These classes of bug have each bitten us more than once. Check for them in review.

### P8 — Changing a default focus/Enter target silently changes Enter semantics

When you add focusable buttons with a *safe* default to a dialog that previously hard-coded `Enter` =
confirm, `Enter` now activates the focused (often Cancel) button instead. This is usually the desired
safer behavior, but it **changes existing behavior and tests**. (Feat 016: the Revert dialog's `Enter`
went from "confirm revert" to "Cancel"; its feat-014 test had to switch to the `Y` shortcut.) When
introducing focus/default to an existing confirm flow, audit every test/handler that assumed `Enter`
meant the affirmative, and keep a letter shortcut for the affirmative.

### P9 — Same key, different meaning per dialog (disambiguate before reusing)

`Tab`, `Enter`, `Space`, `Ctrl+A` already carry meaning in some dialogs (feat 015 `Tab` switches
Find/Replace fields; plugin-manager `Space` toggles a list item; `Ctrl+A` is Select-All globally).
Before binding one of these to a new cross-dialog behavior (button focus/activate), confirm it doesn't
clobber an existing per-dialog meaning — scope the new behavior to the dialogs that need it, or exclude
the conflicting one. (Feat 016 excluded `Space` activation in plugin-manager and deferred the
field-rich dialogs precisely to avoid this.)

### P1 — "Implemented but never wired up"

A widget, state field, action, or handler exists in the code but is never connected to the event loop,
the render path, or a handler arm — so the feature silently does nothing.

- **Menu items were no-ops (feat 010/011):** `Undo/Redo/Cut/Copy/Paste/Select All/Save As/Toggle Line
  Nos/New/Open/Help` dispatched an `Action` that had **no arm** in `handle_action`, so they fell
  through to a debug-log catch-all. This *also* silently broke the bound `Ctrl+Z/Y/X/C/V/A`.
- **File ▸ Open did nothing (feat 010):** the `OpenFileDialog` widget and `handle_open_file` both
  existed but were never invoked; `Action::Open` had no handler.
- **Escape did nothing (feat 010):** `Action::MenuClose` handling existed, but `"Esc"` was never bound
  in the keymap and there was no fallback in the dispatcher.
- **`Ctrl+O` did nothing (feat 010):** documented in `CAPABILITIES.md` but never added to the keymap.
- **Dropdowns invisible (feat 009):** menus were rendered *under* the editor content.
- **Status messages invisible (feat 009):** `status_message` was set but never rendered.
- **Find/Replace were stubs (feat 015):** the menu actions reset state / logged; the dialog widgets
  existed but had no text input, no routing, and were never opened.

**Guard:** when adding an `Action`, grep for a handler arm AND a keymap binding AND (if UI) a render
site. A feature isn't done until the keypress → action → handler → render path is complete end-to-end.
Prefer an exhaustive `match` over a catch-all `_ => log::debug!(...)` so unhandled actions fail loudly.

### P2 — Two sources of truth for geometry / state

When the same quantity is computed in two places, they drift.

- **Stale `terminal_size` (this session, bug 2):** `render()` drew the centered file-browser box using
  `frame.size()` (the real frame), but mouse `hit_test` used `self.terminal_size`, which was only
  updated on a `Resize` event — so it stayed at the `(80,24)` default until the terminal was resized.
  On any other size, clicks inside the visible box mapped to "outside" and **closed the dialog**. Fix:
  `render()` now syncs `self.terminal_size = frame.size()` every frame.
- General: hit-testing must use the *same* geometry the renderer used (the menu bar already learned this
  in feat 009/011 via shared `hit_test_menu` / `dropdown_layout`). The file browser shares
  `compute_layout` between draw and `hit_test` — good — but the *area* fed to both must match.

**Guard:** hit-testing and rendering must derive geometry from one shared source for the same frame.

### P3 — Unsigned subtraction underflow (`usize` panics)

`a - b` on `usize` panics in debug builds when `b > a`. Easy to hit with viewport/length math.

- **`clamp_scroll` crash (this session):** computed `viewport_height() - 1`; when the visible height
  was 0 (possible once `terminal_size` tracked the real frame, P2 fix above), it underflowed and
  **panicked on basic editing**. This reproduced on `master`, not just the feature branch. Fix: floor
  the height at 1 (`viewport_height().max(1)`).

**Guard:** prefer `saturating_sub`, or clamp the operand (`.max(1)`) before subtracting. Be suspicious
of every `- 1` / `- 2` involving a height, width, length, count, or index that could be 0/empty.

### P4 — Derived state computed the wrong way

- **`[Modified]` was wrong on undo (this session, bug 1/feat 014):** `modified` was forced `true` on
  every edit *and* on every undo/redo, so undoing back to the saved content still showed `[Modified]`.
  The flag should reflect **"content == saved baseline"**, not "an edit happened". Fix: a saved-point
  marker in the undo history (`UndoStack.saved`), with `modified = !is_at_saved()`.
  - **Subtlety that bites the naive fix:** counting edits since save (or comparing undo depth) gives a
    **false-clean** after a divergent edit (save → undo → retype reaches the same depth with different
    content). The marker must be **invalidated** when the divergent `push` discards the branch that
    held it.

**Guard:** for "is X dirty/derived", define it as a function of current state vs. a baseline, not as a
flag toggled by events.

### P5 — Input dispatch only produces actions for bound or fallback keys

`dispatch_key` maps a key to an `Action` only if (a) it's in the keymap, or (b) it's a plain
char/Shift+char/known special. **An unbound `Alt+<letter>` or `Tab` yields `None`** — the app never
sees it.

- **Search-option toggles (feat 015):** `Alt+C/A/R/W` had to be added as real `Action` variants *and*
  keymap bindings; without bindings the dialog would never receive them.
- **`Tab` was unbound:** had to bind `Tab → Action::FocusNextField` for the Replace dialog field
  switch. (It was a no-op before, so binding it to an inert-outside-dialog action is safe.)

**Guard:** any new modifier chord you want a modal to handle must have a keymap entry + `Action`
variant. Modal-only keys should be inert no-ops everywhere else.

### P6 — Context-dependent key meaning

- **`Ctrl+A` (feat 015):** Select-All globally, but Replace-All while the Replace dialog is open.
  Handled by intercepting it in the dialog guard (which runs before the normal action match) with a
  match guard on dialog mode.

**Guard:** modal intercepts run before the global match and must `return` early; keep the global meaning
intact for when no modal is open (add a regression test asserting the global behavior still works).

### P7 — UX convention mismatch (single vs double click)

- **File-browser double-click (this session, bug 1):** a *single* click activated an entry, so
  double-clicking a folder entered it on click 1 and then opened whatever file was now under the cursor
  on click 2 — closing the dialog. Fix: single click selects, double-click activates (track last
  click index + time; 400 ms window).

**Guard:** match platform/host conventions (single = select, double = open) for pointer interactions.

---

## Session log

### 2026-06-20 — file browser, undo/revert, find/replace (features 012-follow-up, 014, 015)

| # | Symptom | Root cause | Pattern | Fix / PR |
|---|---|---|---|---|
| 1 | Open dialog: double-clicking a folder closed it / opened a stray file | Single click activated; 2nd click hit a different row in the reloaded listing | P7 | single-click selects, double-click activates (#32) |
| 2 | Open dialog: a single click anywhere closed it (on non-80×24 terminals) | `hit_test` used stale `self.terminal_size`; render used real `frame.size()` | P2 | sync `terminal_size` in `render()` (#34) |
| 3 | Undo never cleared `[Modified]` | flag forced true on edit *and* undo/redo | P4 | saved-point marker in undo history (#35) |
| 4 | (added) File ▸ Revert | missing feature | — | reuse `reload_from_disk` + confirm modal (#35) |
| 5 | Search ▸ Find / Replace did nothing | stub handlers, no dialog/input | P1 | interactive Find/Replace dialogs (#36) |
| 6 | Crash ("subtract with overflow") on basic editing in a tiny frame | `viewport_height() - 1` underflow when height 0 | P3 | floor at 1 (#36); pre-existing on master |

### Earlier sessions (from the changelog ledger)

- **feat 011:** most Edit/View/File menu items and their `Ctrl`-shortcuts were no-ops (P1); mouse
  clicks past "File" and on dropdown items did nothing (mouse events were flattened to an action
  without coordinates/state before the app saw them).
- **feat 010:** Escape unbound (P1); File ▸ Open / `Ctrl+O` unwired (P1).
- **feat 009:** dropdowns drawn under the editor; transient status messages never rendered (P1).
- **#31:** Help ▸ About credited "the MyOS project" instead of the actual author — wrong attribution
  in a user-visible string; watch generated/boilerplate text.

---

## Testing gotchas

- **`render()` bails out below the minimum terminal size.** The editor draws a "Terminal too small"
  message and returns early when the frame is `< 80×24` (`MIN_WIDTH`/`MIN_HEIGHT`). A `TestBackend`
  render test sized smaller than that renders nothing real — assertions on cell content/styles will
  fail confusingly. Size render-test backends at **≥ 80×24** (feat 016/017 render tests). The
  `terminal_size`-sync test is the exception (it only checks the synced field, not drawn content).

## Weird behaviors & environment gotchas

These are not code bugs in `edit` but cost real time; know them before debugging.

- **Smoke tests are flaky in this dev sandbox.** `tests/smoke/*.exp` drive the editor through a PTY
  with `expect`. In the sandbox they fail/pass non-deterministically:
  - The shell locale is `en_IN` (non-UTF-8); the editor prints a locale warning at startup. Tests are
    meant to run with `LC_ALL=C.UTF-8 LANG=C.UTF-8`.
  - There is a **startup race**: the first keystroke sent shortly after launch can be lost (interacts
    with terminal queries at init). `file_browser.exp` sends `Ctrl+O` as its *first* keystroke and is
    especially sensitive; exps that type into the buffer first are not.
  - **Proof it's environmental, not a regression:** `file_browser.exp` failed 5/5 in the repo working
    dir on a *clean `origin/master` checkout* (changes stashed), yet the *same commit* passed 5/5 in a
    `/tmp` git worktree. Same code, different result → environment/timing.
  - **Takeaway:** don't treat a single sandbox smoke failure as a code regression. Reproduce on a clean
    base in the same dir, and compare across ≥5 runs and a `/tmp` worktree before concluding.
- **Crash-report line numbers can be misleading.** The panic report pointed at `src/app.rs:1527:63`,
  which was a `}` — the attribution didn't match the real site. Use `RUST_BACKTRACE=1` and/or a
  deterministic repro (a unit test that forces the bad input) instead of trusting the reported line.
  Crash reports land in `$XDG_STATE_HOME/edit/crash-<ts>.log` (default `~/.local/state/edit/`).
- **Mouse testing over a PTY** works with tmux + raw SGR sequences: press
  `ESC [ < 0 ; <col> ; <row> M`, release `... m` (1-based coords). Verify *styling* (underline, match
  highlight) with `tmux capture-pane -pe` and grep for SGR codes (`[4m` underline, `48;5;NNm` bg).
- **`Date::now`/`Math.random` are unavailable in workflow scripts** (would break resume) — unrelated to
  the editor but relevant when scripting orchestration.

---

## Process learnings (Spec Kit / git)

- **Every feature PR conflicts on the same anchor files.** Parallel feature branches all edit
  `CHANGELOG.md` `[Unreleased]`, `docs/STATUS.md`, `Cargo.toml` `[[test]]`, `.specify/feature.json`, and
  the `CLAUDE.md` `<!-- SPECKIT START -->` plan pointer. After merging one feature, the next branch
  conflicts there. Resolution is mechanical: **keep both** changelog/status/test entries; take the
  current branch's `feature.json` / `CLAUDE.md` pointer. Expect this and budget for it.
- **Branch number vs. feature number.** A fix branch was named `014-fix-mouse-hit-geometry` but was not
  a ledger feature; the next *feature* then also wanted "014". Keep feature numbers (CHANGELOG ledger
  max + 1) distinct from fix-branch names, or don't number fix branches.
- **`[no-docs]`** in a commit message skips the docs gate for docs-only/infra-only PRs (like this one).
- **Resolve every `/speckit-analyze` finding before implementing** (all severities) — e.g. analyze for
  feat 015 caught that unbound `Alt+<letter>` produce no action (P5) *before* coding, avoiding rework.
- **Verify deterministically, then live.** `cargo fmt --check` + `clippy --all-targets -D warnings` +
  `cargo test` is the trustworthy gate here; tmux/smoke is for a final human-like sanity check, not the
  source of truth (see sandbox flakiness above).
