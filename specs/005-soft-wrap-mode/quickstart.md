# Quickstart: Validating Soft-Wrap Mode (Feature 005)

**Date**: 2026-06-19
**Branch**: `005-soft-wrap-mode`

This guide describes how to validate that the soft-wrap feature works correctly end-to-end.

---

## Prerequisites

```bash
# Build the debug binary
make build

# Confirm the binary exists
./edit --version
```

---

## Scenario 1: Toggle Soft-Wrap On and See Visual Reflow

```bash
# Create a test file with a very long line
python3 -c "
print('Short line.')
print('A' * 200 + ' this is a long word-wrapped line with many words scattered across it.')
print('Another short line.')
" > /tmp/test-long.txt

# Open in the editor (terminal must be ≥ 80 columns wide)
./edit /tmp/test-long.txt
```

**Expected**:
1. The 200-char line scrolls horizontally (no wrap) by default.
2. Press `Alt+Z` → the long line reflows into multiple visual rows; `»` markers appear on continuation rows.
3. The status bar shows `[WRAP]`.
4. Press `Ctrl+S` → file saves. `hexdump -C /tmp/test-long.txt` shows no extra `0x0a` (newlines) inserted at wrap points.
5. Press `Alt+Z` again → horizontal scroll returns; `[WRAP]` disappears.

---

## Scenario 2: Cursor Navigation Through a Wrapped Line

```bash
./edit /tmp/test-long.txt
# Enable soft-wrap: Alt+Z
# Arrow to the long line
# Press End
```

**Expected**: Cursor jumps to column 270+ (end of the logical line), not column 79 (end of visual segment).

```bash
# Press ↓ while on the long line
```

**Expected**: Cursor moves to the next **logical** line (the "Another short line"), skipping all visual continuation rows of the long line.

---

## Scenario 3: Persistence Across Sessions

```bash
./edit /tmp/test-long.txt
# Enable soft-wrap: Alt+Z
# Quit: Ctrl+Q
# Relaunch:
./edit /tmp/test-long.txt
```

**Expected**: The long line is already visually wrapped on first frame; status bar shows `[WRAP]` without user action.

**Verify config written**:
```bash
grep soft_wrap ~/.config/edit/config.toml
# Expected output: soft_wrap = true
```

---

## Scenario 4: Wide Character Wrap Boundary

```bash
python3 -c "
# CJK double-width chars: each '字' is 2 columns wide
line = '字' * 50 + ' separator ' + '字' * 50
print(line)
" > /tmp/test-cjk.txt

./edit /tmp/test-cjk.txt
# Alt+Z to enable soft-wrap
```

**Expected**: Wrap points fall at grapheme cluster boundaries. No half-rendered double-width cell appears at a wrap boundary (no visual garbling of CJK characters).

---

## Scenario 5: Narrow Terminal Guard

```bash
# Resize terminal to < 10 columns, then press Alt+Z
./edit /tmp/test-long.txt
```

**Expected**: Status bar shows "Terminal too narrow for soft wrap (min 10 columns)"; soft-wrap remains disabled.

---

## Automated Test Suite

```bash
# Run all unit tests (includes WrapCache, Config, Action enum)
cargo test

# Run integration tests
cargo test --test '*'

# Run CI gate
make ci-local
```

**All tests must pass before the feature is considered complete.**

---

## Key Files to Inspect

| File | What to check |
|---|---|
| `src/ui/wrap.rs` | `WrapCache::compute()` and `visual_to_logical()` |
| `src/ui/editor.rs` | Soft-wrap rendering branch in `Widget::render` |
| `src/ui/statusbar.rs` | `[WRAP]` indicator in `flags()` |
| `src/ui/menubar.rs` | "Soft Wrap (ext)" in `VIEW_MENU` |
| `src/input/keymap.rs` | `Action::ToggleSoftWrap`, `Alt+Z` binding |
| `src/config/schema.rs` | `soft_wrap: bool` field with `#[serde(default)]` |
| `src/app.rs` | `ToggleSoftWrap` dispatch, cache invalidation |
| `tests/integration/soft_wrap.rs` | Integration test suite |
