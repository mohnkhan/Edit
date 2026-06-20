# Phase 0 Research: Menu mnemonic accelerators

## R1. Visual indicator: how to render the underlined accelerator

**Decision**: Render the accelerator glyph with ratatui's `Modifier::UNDERLINED` added to the cell's
existing `Style`, leaving all other cells of the label unchanged.

**Rationale**: Theme-independent (works on every theme without a new color field), trivially testable
(`buf.get(x, y).style().add_modifier` contains `UNDERLINED`), and the DOS-faithful "highlighted hotkey"
read. crossterm emits SGR `4` (underline); terminals that don't support underline simply ignore the
attribute and still show the full readable label — exactly the graceful degradation FR-001/FR-002 want.

**Alternatives considered**:
- *Distinct foreground color*: needs a new `Theme` field per theme and depends on color support; more
  surface area, and on the cyan/black classic bar a second color competes with the selection highlight.
- *Both underline + color*: maximal noise, two concerns to maintain — rejected by the clarification.

## R2. Lone-`Alt` detection (FR-005a, bare Alt activates the bar)

**Decision**: Best-effort. At terminal init, if `crossterm::terminal::supports_keyboard_enhancement()`
returns `Ok(true)`, push `KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES` (and pop it on
teardown). In `dispatch_key`, map a `Press` of `KeyCode::Modifier(ModifierKeyCode::LeftAlt | RightAlt)`
to `Action::Menu` (the same action `F10` produces). On terminals without enhancement support, the flag
is not pushed, the lone-`Alt` event is never delivered, and `F10` / `Alt+letter` remain the entry path
(graceful degradation).

**Rationale**: Standard terminals never report a modifier key pressed by itself — only modifier+key
combinations generate input. The Kitty keyboard protocol (crossterm `REPORT_ALL_KEYS_AS_ESCAPE_CODES`)
is the only portable way to receive a standalone `Alt`. Gating on `supports_keyboard_enhancement()`
keeps existing behavior byte-for-byte on unsupported terminals (including the tmux/expect smoke
harness), so there is no regression risk; supported terminals gain the convenience.

**Notes / risk**: `dispatch_key` already ignores `KeyEventKind::Release`; we keep that. We do **not**
enable repeat-event reporting. `Alt+letter` continues to arrive as `Char(c)` + `ALT` and is unchanged.
The lone-`Alt` → `Menu` mapping is unit-testable independent of terminal support by synthesizing the
`KeyEvent`. Bare-Alt is the lowest-priority slice (US3 support); the underline + letter-activation core
(US1/US2) does not depend on it.

**Alternatives considered**: Treating the ESC byte as "Alt" — rejected (collides with `Esc` =
`MenuClose` and with escape-sequence prefixes). Polling raw modifier state — not available via
crossterm's event model.

## R3. Deterministic auto-assignment for plugin entries

**Decision**: `auto_mnemonic(label, used: &mut HashSet<char>) -> Option<char>`: iterate the label's
chars in order; for the first alphanumeric char whose lowercase form is not already in `used`, insert
it and return it; if none qualifies, return `None`. The `used` set is seeded per scope before plugin
assignment (top-level: the six built-in menu letters; dropdown: the built-in items' letters of the
menu the plugin items merge into). Lowercasing uses `char::to_lowercase().next()` for Unicode safety.

**Rationale**: Deterministic (same input → same output, FR-008), unique within scope (FR-003), prefers
the natural first letter (DOS habit), and yields `None` cleanly when no letter is free (FR-006, entry
shown without an accelerator). Pure function over the label — no I/O, no global state.

**Alternatives considered**: "prefer first consonant", weighting by frequency — more complex with no
user-visible benefit (YAGNI). Hashing label → letter — non-intuitive accelerators.

## R4. DOS-faithful built-in accelerator letters

**Decision**: Hand-author one accelerator per built-in item, all unique within their menu:

| Menu | Items → accelerator |
|---|---|
| **File** (F) | New=**N**, Open=**O**, Save=**S**, Save **A**s, Save As **E**ncoding…, e**X**it |
| **Edit** (E) | **U**ndo, **R**edo, **C**ut, C**o**py, **P**aste, **S**elect All |
| **Search** (S) | **F**ind, Find **N**ext, Find **P**rev, Find **R**eplace |
| **View** (V) | **S**plit View, **N**ext Buffer, **P**rev Buffer, **T**oggle Line Nos, Soft **W**rap (ext) |
| **Options** (O) | Toggle **H**ighlight, **P**lugins… |
| **Help** (H) | **H**elp, **A**bout |

**Rationale**: Matches MS-DOS EDIT / common Windows menus where possible (File: N/O/S/A/X; Edit:
U/C/P; eXit by convention uses X). Each set is collision-free within its menu. "Copy" takes **o**
because **C**ut owns C; "Find Next/Prev/Replace" take N/P/R because **F**ind owns F. These letters are
stored as the canonical lowercase `char` on each `MenuItem`.

**Alternatives considered**: Auto-computing built-ins too — rejected in clarification (would pick
non-conventional letters such as Save As→`v` and drift from DOS).

## R5. Underline position for a given mnemonic

**Decision**: The accelerator is stored as a canonical lowercase `char`; the rendered underline is the
**first** position in the label whose lowercased char equals it. Position is computed in display
columns using the existing wide-char width helper so the underline lands on the correct cell even with
wide/combining characters; if the char is not present in the label (shouldn't happen for authored
data, possible for malformed plugin data) no underline is drawn and the label still renders.

**Rationale**: Avoids storing a separate index that could drift from the label; robust to UTF-8
(FR-010). The "first match" rule is what users expect (e.g. Save **A**s underlines the A in "As").
