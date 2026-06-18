# Quickstart Validation Guide: Linux EDIT.COM Clone

**Feature**: `specs/001-linux-editcom-clone`
**Phase**: 1 — Design (validation reference)
**Date**: 2026-06-18

This guide provides runnable validation scenarios that prove each user story works
end-to-end. Use it to confirm implementation completeness before and after each phase.

---

## Prerequisites

```bash
# Build the debug binary
cargo build

# Aliases used throughout
EDIT=./target/debug/edit

# Verify binary exists and reports version
$EDIT --version   # → "edit 0.1.0" (or current version)
$EDIT --help      # → usage summary with all flags
```

For smoke tests (UI scenarios), the following must be installed:
- `expect` (apt: `expect`, brew: `expect`)
- `tmux` (apt: `tmux`, brew: `tmux`)

For performance benchmarks:
- `cargo bench` (requires nightly for criterion, or stable with criterion 0.5+)

---

## Validation: User Story 1 — Basic File Editing

### Scenario A: Open existing file, edit, save, quit

```bash
# Setup
echo "Hello world" > /tmp/test_edit.txt

# Run
$EDIT /tmp/test_edit.txt
# In the editor:
#   1. Navigate to end of "Hello world" with End key
#   2. Type ", from EDIT.COM clone"
#   3. Press Ctrl+S to save
#   4. Press Ctrl+Q to quit

# Verify
cat /tmp/test_edit.txt
# Expected: "Hello world, from EDIT.COM clone"
```

### Scenario B: New file created on save

```bash
$EDIT /tmp/brand_new_file.txt
# In the editor: type "New content", Ctrl+S, Ctrl+Q
cat /tmp/brand_new_file.txt
# Expected: "New content"
```

### Scenario C: Quit with unsaved changes prompts dialog

```bash
# In editor: make a change, then press Ctrl+Q without saving
# Expected: dialog appears asking "Save / Discard / Cancel"
# Press Escape or "c" → editor returns to editing (no quit)
```

### Scenario D: Read-only mode

```bash
$EDIT --readonly /etc/hostname
# Expected: status bar shows "[Read Only]"; Ctrl+S does nothing (no dialog)
```

### Scenario E: Large file opens within 3 seconds

```bash
# Generate a 100 MB UTF-8 file
python3 -c "print('A' * 99, end='\n')" | head -c 104857600 > /tmp/bigfile.txt

time $EDIT /tmp/bigfile.txt
# Expected: "real" time < 3s until editor is interactive
# In editor: PageDown/PageUp must be responsive (< 50ms lag)
```

---

## Validation: User Story 2 — UTF-8 and Unicode Display

### Scenario A: Japanese characters in two visual columns

```bash
echo "日本語テスト" > /tmp/jp.txt
LC_ALL=C.UTF-8 $EDIT /tmp/jp.txt
# Expected: each kanji/kana character occupies 2 visual columns;
#           cursor advances 2 columns per character
```

### Scenario B: Combining characters render as one column

```bash
printf "e\xcc\x81\n" > /tmp/combining.txt   # e + combining acute → é
LC_ALL=C.UTF-8 $EDIT /tmp/combining.txt
# Expected: "é" occupies 1 column; cursor moves past it as a unit
```

### Scenario C: Emoji occupies two columns

```bash
echo "Hello 😀 world" > /tmp/emoji.txt
LC_ALL=C.UTF-8 $EDIT /tmp/emoji.txt
# Expected: 😀 occupies 2 columns; cursor skips it as one grapheme cluster
```

### Scenario D: Non-UTF-8 locale warning

```bash
LC_ALL=C $EDIT /tmp/jp.txt
# Expected: warning message visible in status bar or dialog:
#           "Terminal locale is not UTF-8. Set LC_ALL=C.UTF-8 for full Unicode support."
#           Editor starts anyway with UTF-8 forced internally.
```

### Scenario E: CP437 encoding round-trip

```bash
# Create a CP437-encoded file (box-drawing characters)
printf '\xc9\xcd\xcd\xbb\n\xba  \xba\n\xc8\xcd\xcd\xbc\n' > /tmp/box.cp437

$EDIT --encoding=cp437 /tmp/box.cp437
# Expected: box-drawing characters render as their Unicode equivalents (╔══╗ etc.)

# Save back as CP437 and verify bytes unchanged
# (use File > Save As → select cp437 encoding)
xxd /tmp/box.cp437 | head
# Expected: original bytes 0xC9, 0xCD, 0xCD, 0xBB, ... preserved
```

---

## Validation: User Story 3 — DOS-Style Menu Navigation

```bash
# Run the menu navigation smoke test
expect tests/smoke/menu_nav.exp
# Expected: all exit codes 0; script navigates File/Edit/Search/View/Options/Help menus
```

Manual validation:
```
$EDIT /tmp/test_edit.txt
# Press F10 or Alt+F → File menu drops down
# Navigate with ↓ arrow → each item highlighted in turn
# Press Escape → menu closes, cursor returns to text
# Press Alt+H → Help menu opens
# Press F1 → built-in help screen appears
```

Verify no-color fallback:
```bash
TERM=dumb $EDIT /tmp/test_edit.txt
# Expected: menu bar uses reverse-video (not color); editor is fully functional
```

---

## Validation: User Story 4 — Search and Replace

```bash
cat > /tmp/search_test.txt << 'EOF'
The quick brown fox
jumps over the lazy dog
The fox is quick
EOF

$EDIT /tmp/search_test.txt
# Press Ctrl+F → find dialog opens
# Type "fox" → first match highlights; scroll to match
# Press F3 → advances to second match ("The fox is quick")
# Press F3 again → wraps to first match with "Search wrapped" in status bar

# Open Find & Replace (Ctrl+H or Search > Replace):
# Find: "fox", Replace: "cat"
# Replace All → "2 replacements" shown in status bar

# Save and quit
cat /tmp/search_test.txt
# Expected: both "fox" occurrences replaced with "cat"
```

```bash
# Regex mode
$EDIT /tmp/search_test.txt
# Ctrl+F → enable regex (checkbox or Ctrl+R toggle)
# Pattern: "T[hH]e" → matches "The" in lines 1 and 3
```

---

## Validation: User Story 5 — Auto-Save and Crash Recovery

```bash
$EDIT /tmp/recovery_test.txt &
EDIT_PID=$!

# In the editor: type some text, wait 35 seconds for auto-save
sleep 35

# Verify recovery file was created
ls $XDG_RUNTIME_DIR/edit/*.recovery 2>/dev/null || ls /tmp/edit-recovery/*.recovery
# Expected: one .recovery file present

# Simulate crash
kill -9 $EDIT_PID

# Reopen the file
$EDIT /tmp/recovery_test.txt
# Expected: dialog "A recovery file exists from a previous session. Recover? [Y/N]"
# Press Y → buffer loads the auto-saved content
# Press Ctrl+Q and discard → verify recovery file is deleted
```

Automated recovery test:
```bash
cargo test --test integration recovery
# Expected: all recovery tests pass
```

---

## Validation: User Story 6 — Multi-File Editing

```bash
echo "File A content" > /tmp/fileA.txt
echo "File B content" > /tmp/fileB.txt

$EDIT /tmp/fileA.txt /tmp/fileB.txt
# Expected: both files listed in View menu or window title shows "fileA.txt | fileB.txt"
# Alt+V → View menu → switch to fileB.txt
# Edit fileB.txt: append " edited"
# Ctrl+S saves only fileB.txt

cat /tmp/fileA.txt   # → "File A content" (unchanged)
cat /tmp/fileB.txt   # → "File B content edited"
```

---

## Validation: User Story 7 — Syntax Highlighting

```bash
cat > /tmp/hello.py << 'EOF'
def greet(name):
    # Say hello
    print(f"Hello, {name}!")

greet("world")
EOF

$EDIT /tmp/hello.py
# Expected:
#   "def" keyword in keyword color (yellow in classic theme)
#   "# Say hello" comment in comment color (grey/bright-black)
#   "Hello, {name}!" string in string color (green)
#   Plain identifiers in default foreground color

# Disable highlighting: Options > Syntax Off or --no-highlight flag
$EDIT --no-highlight /tmp/hello.py
# Expected: all text in uniform foreground color
```

Supported file types to validate: `.c`, `.py`, `.sh`, `.yaml`/`.yml`, `.md`

---

## Validation: User Story 8 — Configurable Keybindings and Themes

```bash
# Write a test config that remaps Ctrl+S to Ctrl+W
mkdir -p ~/.config/edit
cat > ~/.config/edit/config.toml << 'EOF'
theme = "high-contrast"
[keybindings]
"Ctrl+W" = "save"
EOF

$EDIT /tmp/test_edit.txt
# Expected: editor opens with high-contrast theme (black background, bright text)
# Make a change; press Ctrl+W → file saved
# Press Ctrl+S → no action (not mapped)

# Restore default config
rm ~/.config/edit/config.toml
```

---

## Validation: Security Requirements

```bash
# FR-022: Escape injection prevention
# Create a file with ANSI escape sequences
printf '\x1b[2J\x1b[?25l\x1b[H\x1b[31mHACKED\x1b[0m' > /tmp/escape.txt
$EDIT /tmp/escape.txt
# Expected: escape sequences rendered as literal text, NOT interpreted by the terminal
# Terminal must not be cleared or otherwise manipulated by the file content

# FR-023: Path traversal prevention
$EDIT '../../etc/passwd'
# Expected: editor opens the file relative to cwd (not navigating above project root);
#           or if path resolves to a protected file, permission error is shown cleanly

# FR-021: No privilege escalation
ls -la /root/sensitive 2>/dev/null
$EDIT /root/sensitive
# Expected: "Permission denied" error; process does NOT attempt sudo or setuid
```

---

## Validation: SC-001 — First-Use Task Completion (Manual Only)

> **SC-001**: A first-time user can open a file, make an edit, save, and quit within 60 seconds.
>
> **No automated test exists for this criterion** — it is a qualitative UX metric. Validate manually
> using the US1 Scenario A procedure above. If a new contributor cannot complete the task in under
> 60 seconds on their first attempt, consider simplifying the status bar or adding an in-editor hint.

---

## Validation: Non-Functional — Performance and Stress

```bash
# SC-003: Cold start ≤ 2 seconds
time $EDIT --help > /dev/null   # measures startup time (approximate)
hyperfine "$EDIT /tmp/test_edit.txt"   # if hyperfine installed

# SC-008: 72-hour stress test (abbreviated to 5-minute smoke)
cargo test --test integration stress_5min -- --nocapture
# Expected: no memory growth, no crashes

# SC-010: Cross-platform build
cargo build --target x86_64-unknown-linux-musl
file target/x86_64-unknown-linux-musl/debug/edit
# Expected: "ELF 64-bit ... statically linked"
```

---

## Validation: Man Page and Help

```bash
# Built-in help
$EDIT --help | grep -q "encoding"   # → exits 0

# Man page (after installation)
man edit   # → displays man page; q to quit
man edit | grep -q "SYNOPSIS"   # → exits 0
```
