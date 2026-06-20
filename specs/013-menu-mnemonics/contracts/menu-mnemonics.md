# Contract: Menu mnemonic accelerators

Extends the feature-009/011 menu contracts. "Bar active" means `MenuBarState::is_active()`.

## Assignment contract (`resolve_menus`)

- Every built-in top-level menu and every built-in item has a deterministic, hand-authored accelerator
  (canonical lowercase `char`); see `data-model.md` / `research.md` R4.
- Plugin items and plugin top-level menus receive accelerators via `auto_mnemonic`, seeded with the
  already-used letters of their scope so they never collide with built-ins.
- Within the bar, `Some` top-level mnemonics are unique; within each dropdown, `Some` item mnemonics
  are unique (FR-003). An entry with no free letter has `mnemonic == None` (FR-006).
- `resolve_menus(&[])` is unchanged from feature 009 except each entry now carries its `mnemonic`.

## Rendering contract (`MenuBarWidget`)

| Element | Rendering |
|---|---|
| Top-level menu label | Exactly one glyph — the accelerator — drawn with `Modifier::UNDERLINED` added to its existing style; all other glyphs unchanged. Applies whether or not the menu is the selected/highlighted one. |
| Dropdown item label | Same: the one accelerator glyph underlined; check-mark prefix column and alignment unchanged from feature 006/009. |
| Entry with `mnemonic == None` | No underline; label rendered normally. |
| Wide / multi-byte labels | Underline lands on the correct display cell of the **first** matching char; never splits a character (FR-010). |

The underline is the only visual change; geometry (columns, widths, dropdown box, selection highlight)
is identical to feature 011.

## Keyboard contract (while bar active; via `App` menu intercept)

| Key (state) | Effect |
|---|---|
| printable letter, in `DropDown` | If it matches an item's mnemonic (case-insensitive) → activate that item exactly as Enter would (run its action) and close the menu. If no match → consumed no-op, menu stays open (FR-004, FR-007). |
| printable letter, in `TopActive` | If it matches a top-level menu's mnemonic → open that menu's dropdown (item 0). If no match → consumed no-op (FR-005). |
| `Alt` + top-level mnemonic (any state) | Open that menu's dropdown (existing `Alt+F/E/S/V/O/H`, now guaranteed to equal the underlined letter; FR-005). |
| `Alt` tapped alone | Activate the bar at top level (no dropdown), identical to `F10` — best-effort, only on terminals reporting lone-modifier keys; otherwise no-op (FR-005a). |
| all existing menu keys | `F10`, arrows, `Enter`, `Esc`, mouse — unchanged (FR-012). |

Letter activation is **only** active while the bar is active. In normal editing, every letter key types
its character exactly as before (FR-011).

## Invariants

- The underlined letter for any entry is exactly the key that activates it (SC-002).
- No regression: with the bar inactive, input behavior is byte-for-byte unchanged (SC-006).
- Activating an item by letter and activating it by Enter produce identical results (FR-004).
