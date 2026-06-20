# Phase 1 Data Model: Menu mnemonic accelerators

All types extend existing structures in `src/ui/menubar.rs`. The `mnemonic` is the **canonical
lowercase `char`** that activates the entry; `None` means "no accelerator" (FR-006).

## Static (compile-time) types

### `MenuItem` (built-in dropdown item) — extended

| Field | Type | Notes |
|---|---|---|
| `label` | `&'static str` | unchanged |
| `action` | `Action` | unchanged |
| `mnemonic` | `Option<char>` | **new** — hand-authored DOS letter, lowercase (e.g. `Some('n')` for "New"). `None` only for non-letter rows (none exist today). |

Authored values per R4 of [research.md](./research.md).

### Top-level menu labels — extended

The six top-level menus keep their first-letter accelerators (File→`f`, Edit→`e`, Search→`s`,
View→`v`, Options→`o`, Help→`h`), matching the existing `Alt+letter` bindings. Stored alongside the
existing `BarLabel` table (an added `mnemonic: char`) or derived as "first alphanumeric char of the
label, lowercased" — both yield the same letters; the implementation MAY derive to avoid duplication,
but plugin top-level menus MUST use the auto-assignment path (below).

## Resolved (runtime) types

### `ResolvedItem` — extended

| Field | Type | Notes |
|---|---|---|
| `label` | `String` | unchanged |
| `action` | `Action` | unchanged |
| `mnemonic` | `Option<char>` | **new** — canonical lowercase accelerator, or `None` |

### `ResolvedMenu` — extended

| Field | Type | Notes |
|---|---|---|
| `label` | `String` | unchanged |
| `items` | `Vec<ResolvedItem>` | unchanged element type aside from the new field |
| `mnemonic` | `Option<char>` | **new** — top-level accelerator, canonical lowercase |

## Assignment rules (`resolve_menus`)

Computed deterministically every time the model is built (FR-008):

1. **Built-in top-level menus**: `mnemonic` = the built-in letter (f/e/s/v/o/h). Items copy their
   authored `MenuItem.mnemonic`.
2. **Plugin items merged into a built-in menu**: seed a `used` set with that menu's existing item
   mnemonics, then `auto_mnemonic(label, &mut used)` for each plugin item in order.
3. **New plugin top-level menu**: seed a top-level `used` set with all current top-level mnemonics,
   `auto_mnemonic(menu_label, &mut used_toplevel)` for the menu; then assign its items from a fresh
   per-menu `used` set.
4. `auto_mnemonic` returns `None` when no free alphanumeric letter remains → entry has no accelerator.

## Invariants

- **Per-scope uniqueness (FR-003)**: within the bar, all `Some` top-level mnemonics are distinct;
  within each dropdown, all `Some` item mnemonics are distinct (case-insensitive — they are stored
  lowercase, so plain `!=`).
- **Determinism (FR-008)**: identical menu input (built-ins, or built-ins + a fixed plugin set) always
  produces identical mnemonics.
- **Canonical form**: every stored mnemonic is lowercase; key matching lowercases the pressed char
  before comparing, making activation case-insensitive (spec edge case).
- **No-plugin parity**: `resolve_menus(&[])` yields the six built-in menus with their authored
  mnemonics; item/menu ordering and labels are byte-for-byte unchanged from feature 009 (only the new
  field is added).

## Derived: underline index

`underline_col(label, mnemonic) -> Option<u16>`: display-column offset of the **first** char in
`label` whose `to_lowercase().next() == Some(mnemonic)`, measured with the existing wide-char width
function. `None` if `mnemonic` is `None` or the char is absent. Used only by the renderer.

## Lookup methods (`MenuBarState`)

- `open_menu_by_mnemonic(&mut self, menus, ch) -> bool`: when bar is active (`TopActive`/`DropDown`),
  if some top-level menu has `mnemonic == lc(ch)`, switch to its dropdown (`item_idx = 0`) and return
  `true`; else `false`.
- `select_item_by_mnemonic(&mut self, menus, ch) -> Option<Action>`: when in `DropDown`, if some item
  in the open menu has `mnemonic == lc(ch)`, set state `Inactive` and return its `action`; else `None`.

Where `lc(ch) = ch.to_lowercase().next()`.
