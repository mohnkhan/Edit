# Quickstart Validation Guide: Menu Check-State Indicator

**Feature**: 006 — Menu Check-State Indicator
**Date**: 2026-06-19

---

## Prerequisites

- Rust toolchain ≥ 1.74.0 (`rustup show`)
- Terminal supporting UTF-8 and U+2713 (`echo ✓` should display a single checkmark)
- A working build of the editor (`make` produces `./edit`)

---

## Build

```bash
make          # debug build
make check    # unit tests — all must pass
```

---

## Scenario 1: Checkmark appears when soft-wrap is ON

**Goal**: Verify FR-001, FR-002, FR-004, SC-001, SC-002.

```bash
./edit /tmp/test.txt
```

1. Press `Alt+Z` to enable soft-wrap — status bar shows `[WRAP]`.
2. Press `F10` (or `Alt+V`) to open the View menu.
3. **Expected**: "Soft Wrap (ext)" appears as `✓ Soft Wrap (ext)` in the dropdown.
4. Press `Alt+Z` again (or select "Soft Wrap (ext)" from the menu) to toggle off.
5. Press `F10` to reopen the View menu.
6. **Expected**: "Soft Wrap (ext)" appears WITHOUT the `✓` prefix.

---

## Scenario 2: Check-state reflects persisted config on startup

**Goal**: Verify US3, FR-005, SC-001.

```bash
mkdir -p ~/.config/edit
echo 'soft_wrap = true' > ~/.config/edit/config.toml
./edit /tmp/test.txt
```

1. Editor opens — status bar shows `[WRAP]` immediately.
2. Press `F10` to open the View menu.
3. **Expected**: "Soft Wrap (ext)" appears as `✓ Soft Wrap (ext)` on the FIRST open, without any toggle.

Cleanup:
```bash
rm ~/.config/edit/config.toml
```

---

## Scenario 3: Non-toggle menu items are unaffected

**Goal**: Verify FR-006, SC-003.

```bash
./edit /tmp/test.txt
```

1. Press `Alt+Z` to enable soft-wrap.
2. Press `F10` then arrow to open the File menu (or press `F10` and navigate left).
3. **Expected**: "New", "Open", "Save", "Save As", "Save As Encoding...", "Exit" — none have a `✓` prefix; column width is unchanged from pre-feature appearance.

---

## Scenario 4: No regression in menu navigation

**Goal**: Verify SC-003, SC-004.

```bash
./edit /tmp/test.txt
```

1. Press `F10` to open the menu bar.
2. Navigate with arrow keys across all 6 menus; open each dropdown.
3. Use mouse click to select a non-toggle item.
4. **Expected**: All navigation works identically to pre-feature behavior; no visible delay.

---

## Unit Test Validation

```bash
cargo test --lib menubar 2>&1 | grep -E "test .* (ok|FAILED)"
```

Expected (5 tests all `ok`):
```
test ui::menubar::tests::test_checkmark_shown_when_toggle_true ... ok
test ui::menubar::tests::test_no_checkmark_when_toggle_false ... ok
test ui::menubar::tests::test_non_toggle_menu_unaffected ... ok
test ui::menubar::tests::test_label_alignment_in_checkable_menu ... ok
test ui::menubar::tests::test_empty_toggle_states_no_regression ... ok
```

---

## Full Gate

```bash
make ci-local
```

All existing tests must still pass (no regressions); new tests must pass.
