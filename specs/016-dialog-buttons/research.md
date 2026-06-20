# Phase 0 Research: Focusable dialog buttons

## R1. Boxed-button geometry (shared by render + hit-test)

**Decision**: `button_rects(area, labels) -> Vec<Rect>` lays out a horizontal row of 3-row boxes along
the bottom interior of `area`. Each button width = display-width(label) + 4 (2 borders + 2 padding); a
1-column gap separates buttons; the whole row is centered horizontally; the row sits on the last 3
interior rows of the dialog. The same `Vec<Rect>` is used to draw (render_buttons) and to hit-test
(hit_test_buttons), so the clickable region always equals the drawn box. If the row is wider than the
dialog, buttons clamp/are dropped from the right with the layout never panicking (small-terminal safe).

**Rationale**: One geometry source eliminates the drift class of bug (see Learnings P2). 3-row boxes
match the clarified DOS-window style.

**Alternatives**: per-dialog ad-hoc button math — rejected (drift, duplication).

## R2. Focused-button styling

**Decision**: the focused button is drawn distinctly using the existing selection style
(`menu_selected_bg`, inverted fg/bg) plus a `▶`/heavier marker in its border, so focus is visible even
without color. Unfocused buttons use the normal dialog (menubar) style with a plain border. Degrades
gracefully where box-drawing/colour is limited (label stays readable).

**Rationale**: reuses established theme styling (no new config); the marker keeps focus legible on
monochrome terminals (Constitution I graceful degradation).

## R3. Mouse handler restructuring

**Decision**: Replace the "modal dialogs win: ignore all mouse" early-return in `handle_mouse_event`
with a dialog-button mouse path: when an in-scope dialog is open, compute its `button_rects` for the
current frame and `hit_test_buttons`; a hit activates that button (same as Enter on it); a click outside
the dialog box cancels where a safe cancel exists, else is inert; a click inside-but-not-on-a-button is
inert. The file-browser and menu mouse paths are untouched (file browser already had its own modal
mouse path above the guard).

**Rationale**: dialogs currently ignore the mouse entirely — this is the core fix. Mirrors the
file-browser/menu hit-test pattern.

## R4. Tab disambiguation vs. feature 015

**Decision**: `Tab` is context-dependent. In the Find/Replace dialog (feat 015) `Tab` switches text
fields (unchanged; that dialog is deferred from this feature). In the in-scope dialogs `Tab` moves
button focus and `Shift+Tab` moves it backward. `Shift+Tab` arrives as crossterm `KeyCode::BackTab`;
add a mapping `BackTab -> Action::FocusPrevField` (mirror of feat-015 `Tab -> FocusNextField`). Activation
is `Enter`/`Space`; `Space` is *not* used for activation in the plugin-manager dialog (there it toggles
the list item) — that dialog activates via Enter/click only.

**Rationale**: avoids clobbering feat-015 field switching and the plugin-manager Space-toggle while
giving every in-scope dialog a consistent Tab order.

## R5. Single focus field on App

**Decision**: add `App.dialog_focus: usize`, reset to each dialog's default index when that dialog opens.
Because only one modal is open at a time, one field suffices (no per-dialog focus state needed).
Per-dialog helpers `dialog_button_labels(&self) -> Vec<&'static str>` and
`activate_dialog_button(&mut self, idx)` centralize the button set and the action mapping.

**Rationale**: minimal state; one place defines each dialog's buttons and what they do.

## R6. Default-focused button per dialog

**Decision**: focus the **safe** default — Cancel/No/Keep for destructive prompts (so an accidental
Enter doesn't discard/overwrite), and the affirmative for non-destructive ones (e.g. Help/About →
Close; session restore → Restore). Documented per dialog in the contract.

**Rationale**: data-loss safety (consistent with the project's confirm-before-destroy posture).

## R7. Staging & deferral

**Decision**: implement the shared component + the confirm/dismiss dialogs (save prompt, session
restore, revert, external change, plugin consent, Help/About) and the list dialogs (encoding select,
plugin manager, with added OK/Cancel). **Defer** the Find/Replace dialog (shipped in feat 015; needs a
combined field+button focus ring) and the file browser (already fully mouse+keyboard navigable) to a
follow-up — filed as a GitHub `follow-up` issue + a `ROADMAP.md` row before merge.

**Rationale**: covers every dialog that is currently *not* mouse-navigable (the literal complaint) with
low regression risk; the two deferred dialogs already have rich interaction, so deferring them is safe
and avoids destabilizing freshly shipped work. Honors the project deferral rule.
