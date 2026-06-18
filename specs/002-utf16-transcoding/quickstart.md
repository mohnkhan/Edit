# Quickstart: Validating UTF-16 Transcoding

## Prerequisites

- `cargo build` succeeds (debug binary at `target/debug/edit`)
- `python3` available (used to generate UTF-16 test fixtures)
- `hexdump` available

## 1. Generate test fixtures

```bash
# UTF-16 LE with BOM
python3 -c "
text = 'Hello, UTF-16 LE! こんにちは 🌍'
with open('/tmp/test_utf16le.txt', 'wb') as f:
    f.write(text.encode('utf-16-le').join([b'\xff\xfe', b'']))
# Simpler:
open('/tmp/test_utf16le.txt', 'wb').write('Hello, UTF-16 LE! こんにちは 🌍'.encode('utf-16'))
"

# UTF-16 BE with BOM
python3 -c "
open('/tmp/test_utf16be.txt', 'wb').write('Hello, UTF-16 BE! こんにちは 🌍'.encode('utf-16-be').join([b'\xfe\xff', b'']))
# Simpler:
import codecs
open('/tmp/test_utf16be.txt', 'wb').write(codecs.BOM_UTF16_BE + 'Hello, UTF-16 BE!'.encode('utf-16-be'))
"

# Verify BOMs
hexdump -C /tmp/test_utf16le.txt | head -2
# Expected: ff fe 48 00 65 00 ...
hexdump -C /tmp/test_utf16be.txt | head -2
# Expected: fe ff 00 48 00 65 ...
```

## 2. Open a UTF-16 LE file

```bash
./target/debug/edit /tmp/test_utf16le.txt
```

**Expected**: File opens, status bar shows `UTF-16 LE`, text displays correctly without
garbled characters. Press `Ctrl+Q` to quit.

## 3. Open a UTF-16 BE file

```bash
./target/debug/edit /tmp/test_utf16be.txt
```

**Expected**: File opens, status bar shows `UTF-16 BE`.

## 4. Round-trip test (open, no edit, save)

```bash
cp /tmp/test_utf16le.txt /tmp/test_utf16le_orig.txt
./target/debug/edit /tmp/test_utf16le.txt
# press Ctrl+S then Ctrl+Q
diff <(hexdump -C /tmp/test_utf16le_orig.txt) <(hexdump -C /tmp/test_utf16le.txt)
```

**Expected**: `diff` produces no output (byte-identical round-trip).

## 5. Force encoding via CLI

```bash
# Strip BOM from LE file, then force the encoding
python3 -c "
data = open('/tmp/test_utf16le.txt','rb').read()
open('/tmp/test_utf16le_nobom.bin','wb').write(data[2:])  # strip 2-byte BOM
"
./target/debug/edit --encoding utf-16-le /tmp/test_utf16le_nobom.bin
```

**Expected**: Content displays correctly.

## 6. Run automated tests

```bash
# Unit tests for encode/decode
cargo test --lib encoding 2>&1 | grep -E "test .* (ok|FAILED)"

# Integration round-trip tests
cargo test --test encoding_roundtrip 2>&1 | grep -E "test .* (ok|FAILED)"
```

**Expected**: All tests pass, no FAILED lines. New UTF-16 tests appear in the list.

## 7. Verify status bar display

```bash
cargo test --lib 2>&1 | grep utf16
```

**Expected**: Tests named `utf16_*` all show `ok`.

## 8. Verify no regressions

```bash
cargo test 2>&1 | tail -5
```

**Expected**: All previously passing tests still pass.
