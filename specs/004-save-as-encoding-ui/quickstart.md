# Quickstart Validation Guide: Save-As Encoding Selection UI

**Feature**: 004-save-as-encoding-ui | **Date**: 2026-06-19

---

## Prerequisites

```bash
# Build the debug binary
make

# Verify the binary exists
./target/debug/edit --version
```

---

## Scenario 1 — Save an Existing File as UTF-16 LE (US1, P1)

**Setup**:
```bash
echo "Hello, World!" > /tmp/test-encoding.txt
./target/debug/edit /tmp/test-encoding.txt
```

**Steps**:
1. Press `F12` (or open File menu → "Save As Encoding...")
2. The encoding dialog opens with UTF-8 pre-selected (current buffer encoding)
3. Press `↓` once to move to "UTF-16 LE"
4. Press `Enter`

**Expected outcome**:
- Status bar shows: `Saved as UTF-16 LE`
- Verify on disk:
  ```bash
  hexdump -C /tmp/test-encoding.txt | head -3
  # First two bytes must be FF FE (UTF-16 LE BOM)
  ```

---

## Scenario 2 — Cancel Without Saving (US2)

**Setup**:
```bash
echo "Original content" > /tmp/cancel-test.txt
md5sum /tmp/cancel-test.txt > /tmp/cancel-test-checksum.txt
./target/debug/edit /tmp/cancel-test.txt
```

**Steps**:
1. Press `F12` to open the dialog
2. Press `↓` to move to "UTF-16 BE"
3. Press `Esc`

**Expected outcome**:
- Dialog closes; no save occurs
- Status bar does NOT show any encoding-change message
- File is byte-identical to before:
  ```bash
  md5sum -c /tmp/cancel-test-checksum.txt
  # Must print: /tmp/cancel-test.txt: OK
  ```

---

## Scenario 3 — Encoding Persists on Subsequent Ctrl+S (US3)

**Setup** (continue from Scenario 1 or start fresh):
```bash
echo "Persist test" > /tmp/persist-test.txt
./target/debug/edit /tmp/persist-test.txt
```

**Steps**:
1. Press `F12`, select "UTF-16 BE", press `Enter` — file saved as UTF-16 BE
2. Edit the file (type a character)
3. Press `Ctrl+S`

**Expected outcome**:
- File is still UTF-16 BE after the Ctrl+S save:
  ```bash
  hexdump -C /tmp/persist-test.txt | head -2
  # First two bytes: FE FF (UTF-16 BE BOM)
  ```

---

## Scenario 4 — New Buffer Triggers Filename Prompt First (US4)

**Setup**:
```bash
./target/debug/edit   # no file argument — opens a blank buffer
```

**Steps**:
1. Type some text
2. Press `F12` to open encoding dialog
3. Select "CP437", press `Enter`
4. The filename-input dialog appears
5. Type `/tmp/new-cp437-file.txt`, press `Enter`

**Expected outcome**:
- File `/tmp/new-cp437-file.txt` is created and encoded as CP437:
  ```bash
  file /tmp/new-cp437-file.txt
  # Reports: ISO-8859 text (CP437 is byte-compatible with ISO-8859)
  ```

---

## Scenario 5 — Dialog Pre-Selects Current Encoding (US3 acceptance scenario 2)

**Setup**:
```bash
echo -e "\xFF\xFE" > /tmp/utf16le-preselect.txt  # start with UTF-16 LE BOM
# Or: use a file saved as UTF-16 LE from Scenario 1
./target/debug/edit --encoding=utf-16-le /tmp/test-encoding.txt
```

**Steps**:
1. Press `F12` to open encoding dialog

**Expected outcome**:
- The dialog opens with "UTF-16 LE" already highlighted (not UTF-8)

---

## Scenario 6 — Read-Only File Error

**Setup**:
```bash
echo "Readonly" > /tmp/readonly-test.txt
chmod 444 /tmp/readonly-test.txt
./target/debug/edit /tmp/readonly-test.txt
```

**Steps**:
1. Press `F12`, select any encoding, press `Enter`

**Expected outcome**:
- Dialog closes
- Status bar shows `Save failed: <permission denied message>`
- File is unchanged (still readable, still 444 permissions)

---

## Integration Test Command

```bash
cargo test --test encoding_select
```

All tests must pass before the PR merges.

---

## Full CI Gate

```bash
make ci-local
```

Must be green. Covers: format check → clippy → unit tests → integration tests → smoke.
