# Quickstart: Menu mnemonic accelerators

Validation guide for feature 013. See [spec.md](./spec.md) and
[contracts/menu-mnemonics.md](./contracts/menu-mnemonics.md) for the authoritative behavior.

## Prerequisites

```sh
make            # cargo build (debug binary at ./target/debug/edit)
```

## Manual validation (tmux or any terminal)

1. **Visible accelerators (US1)** — launch `./target/debug/edit`. The menu bar shows
   `File Edit Search View Options Help` with the first letter of each underlined. Press `Alt+F`
   (or `F10` then `→`/`Enter`) to open File; every item shows one underlined letter
   (**N**ew, **O**pen, **S**ave, Save **A**s, Save As **E**ncoding, e**X**it).
2. **Letter activates an item (US2)** — with the File menu open, press `n`. A new empty buffer
   appears and the menu closes. Re-open File, press `o` → the Open file browser appears.
3. **Top-level letter / Alt+letter (US3)** — from editing, press `Alt+V` → View opens; the letter you
   pressed (V) is the one underlined in the bar. With `F10` then the bar active, press `e` → Edit opens.
4. **Bare Alt (US3, terminal-permitting)** — tap and release `Alt` alone. On a terminal that reports
   it (Kitty-protocol capable), the bar highlights with no dropdown; then press `f` to open File. On
   other terminals nothing happens — use `F10` instead (documented fallback).
5. **No-match is inert (FR-007)** — open any menu and press a letter that is not an accelerator (e.g.
   `z` in Help). Nothing happens; the menu stays open. Press `Esc` to close.
6. **No regression (SC-006)** — with no menu open, type letters into the buffer; they insert normally.

## Automated validation

```sh
# Unit tests: mnemonic assignment, uniqueness, underline placement, letter lookup
cargo test --lib menubar

# Integration: end-to-end letter activation through App::handle_action
cargo test --test menu_mnemonics

# Smoke (tmux): open a menu, press a letter, verify clean exit
expect tests/smoke/menu_mnemonics.exp

# Full local gate
make ci-local
```

## Expected outcomes

- Every top-level menu and every actionable item shows exactly one underlined accelerator.
- Pressing an item's underlined letter while its dropdown is open runs that item's action and closes
  the menu — identical to highlighting it and pressing Enter.
- The underlined letter and the working key always agree (no drift).
- A plugin contributing menu items shows unique underlined letters for those items, assigned
  automatically with no collision against built-ins.
- All pre-existing menu/editor tests remain green.
