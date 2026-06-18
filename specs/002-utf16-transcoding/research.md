# Research: UTF-16 Transcoding Support

**Feature**: 002-utf16-transcoding | **Date**: 2026-06-18

## Decision 1: Use `encoding_rs` for UTF-16 decode/encode

**Decision**: Use `encoding_rs::UTF_16_LE` and `encoding_rs::UTF_16_BE` encoding objects,
already present as a dependency (`encoding_rs = "0.8"` in `Cargo.toml`).

**Rationale**: `encoding_rs` is the WHATWG Encoding Standard implementation used by
Firefox. It provides SIMD-accelerated UTF-16 decode/encode, correct surrogate-pair
handling, and is already in the dependency tree. Adding a second crate would be unnecessary.

**API used**:
```rust
// Decode UTF-16 LE bytes → String
let (cow, _encoding, _had_errors) = encoding_rs::UTF_16_LE.decode(bytes_without_bom);
// Encode String → UTF-16 LE bytes (no BOM — we prepend manually)
let (cow, _had_unmappables) = encoding_rs::UTF_16_LE.encode(s);
```

**Alternatives considered**:
- `std::char::decode_utf16`: lower-level, requires manual u16 slice construction from bytes.
- `byteorder` crate: would need a new dependency just for byte swapping.
- Manual implementation: not justified when `encoding_rs` already covers it.

---

## Decision 2: Strip BOM on decode, preserve on encode

**Decision**: Strip the 2-byte UTF-16 BOM (`\xFF\xFE` or `\xFE\xFF`) on `decode()` and
prepend the appropriate BOM on `encode()`.

**Rationale**: The editor's internal rope stores pure UTF-8 text without any BOM markers.
The BOM is a file-level artifact, not a buffer-level one. Saving a UTF-16 buffer must
re-add the BOM so the output file is spec-compliant and readable by Windows tools.

**Alternatives considered**:
- Store BOM as part of the buffer: rejected — violates the "UTF-8 everywhere internally"
  principle (Constitution II) and complicates cursor movement.
- Only write BOM when the original file had one: accepted as behavior — since we detect
  UTF-16 by BOM, we always know a BOM was present; if `--encoding utf-16-le` was forced
  without BOM, we still write a BOM for maximum compatibility.

---

## Decision 3: BOM-less UTF-16 — no auto-detection

**Decision**: Do not attempt to auto-detect UTF-16 without a BOM. Require explicit
`--encoding utf-16-le` / `--encoding utf-16-be` flag.

**Rationale**: BOM-less UTF-16 cannot be reliably auto-detected for files that contain
only ASCII characters (common in config files or scripts), because UTF-16 LE ASCII looks
like null-interleaved bytes that could be any encoding. False positives would cause data
corruption for valid binary files or other encodings.

**Alternatives considered**:
- Heuristic detection (count null bytes, check for common null-interleaved patterns):
  rejected — too many false positives on small files and binary content.
- Use chardetng for UTF-16: chardetng does not detect UTF-16 (by design — it focuses on
  8-bit encodings).

---

## Decision 4: `detect_encoding()` returns `Utf16Le`/`Utf16Be` (not `Utf8`)

**Decision**: Change the two lines in `detect_encoding()` that currently return
`EncodingId::Utf8` for UTF-16 BOMs to return `EncodingId::Utf16Le` and `EncodingId::Utf16Be`.

**Rationale**: The current code has a comment "caller must transcode" but the caller
(`Buffer::open`) calls `decode(bytes, detected_encoding)`. Since `decode()` only handles
`Utf8` today, UTF-16 BOMs are silently mis-decoded. The correct fix is to propagate the
real encoding ID so `decode()` can handle it.

**Impact**: `Buffer::open` works correctly with no change — `detected_encoding` flows into
`decode()`, which now has UTF-16 arms.

---

## Decision 5: `encode()` for UTF-16 prepends BOM before payload

**Decision**: `encode(s, Utf16Le)` returns `[0xFF, 0xFE] ++ utf16le_bytes(s)`.
Similarly for `Utf16Be`.

**Rationale**: UTF-16 files without BOMs are non-standard and cause interoperability
problems. Since we always consume the BOM on read, we must always emit it on write to
preserve round-trip fidelity.

**Implementation**: `encoding_rs::UTF_16_LE.encode(s)` returns bytes without a BOM;
prepend `\xFF\xFE` manually before returning.

---

## Key `encoding_rs` API facts

- `encoding_rs::UTF_16_LE.decode(bytes)` → `(Cow<str>, &'static Encoding, bool)`:
  strips BOM if present, replaces lone surrogates with U+FFFD, returns had_errors flag.
- `encoding_rs::UTF_16_BE.decode(bytes)` → same, big-endian.
- `encoding_rs::UTF_16_LE.encode(s)` → `(Cow<[u8]>, &'static Encoding, bool)`:
  returns UTF-16 LE bytes WITHOUT BOM. All Unicode code points can round-trip
  (no unencodable characters in UTF-16).
- Neither encode nor decode can fail on valid UTF-8 input — all code points are encodable.

---

## Existing Code Inventory

| File | Relevant state |
|------|---------------|
| `src/encoding/detect.rs` | `detect_encoding()` detects UTF-16 BOMs but maps both to `Utf8` (lines 98-99). Fix: return `Utf16Le`/`Utf16Be`. Also `ENCODING_REGISTRY` needs 2 new entries. |
| `src/encoding/transcode.rs` | `decode()` and `encode()` have no UTF-16 arms. Add match arms. |
| `src/encoding/mod.rs` | `encoding_from_str()` has no UTF-16 aliases. Add them. |
| `src/ui/statusbar.rs` | Reads `encoding` field from `Buffer`. Needs no change if `EncodingProfile.name` is set correctly in registry. |
| `tests/integration/encoding_roundtrip.rs` | Has 15 tests for existing encodings. Add ≥ 10 UTF-16 tests. |
