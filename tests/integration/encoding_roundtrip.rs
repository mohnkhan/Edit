// Integration tests T109: encoding round-trip verification.
//
// For each supported non-UTF-8 encoding (CP437, CP850, ISO-8859-1,
// Windows-1252), decode a known byte sequence to UTF-8 and re-encode it,
// then assert the resulting bytes match the original.
//
// Run with:
//   cargo test --test encoding_roundtrip
//
// All fixture data is embedded inline as byte literals so the tests are
// self-contained and do not depend on files being present on disk.

use edit::encoding::{decode, encode, encoding_from_str, EncodingId};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Perform a full round-trip: decode `bytes` with `enc`, then re-encode the
/// resulting UTF-8 string with the same encoding, and assert byte equality.
fn roundtrip(bytes: &[u8], enc: EncodingId) {
    let utf8 = decode(bytes, enc).expect("decode failed");
    let encoded = encode(&utf8, enc).expect("encode failed");
    assert_eq!(
        encoded, bytes,
        "round-trip mismatch for {:?}: {:02X?} -> {:?} -> {:02X?}",
        enc, bytes, utf8, encoded
    );
}

// ── ASCII subset (0x20–0x7E): identical in all legacy encodings ───────────────

/// ASCII printable bytes 0x20–0x7E are the same in CP437, CP850,
/// ISO-8859-1, and Windows-1252.  Round-tripping them through any of these
/// encodings must produce byte-for-byte identical output.
const ASCII_PRINTABLE: &[u8] = &[
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F,
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
    0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F,
    0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E,
];

#[test]
fn cp437_ascii_roundtrip() {
    roundtrip(ASCII_PRINTABLE, EncodingId::Cp437);
}

#[test]
fn cp850_ascii_roundtrip() {
    roundtrip(ASCII_PRINTABLE, EncodingId::Cp850);
}

#[test]
fn iso8859_1_ascii_roundtrip() {
    roundtrip(ASCII_PRINTABLE, EncodingId::Iso8859_1);
}

#[test]
fn windows1252_ascii_roundtrip() {
    roundtrip(ASCII_PRINTABLE, EncodingId::Windows1252);
}

// ── CP437: box-drawing characters ─────────────────────────────────────────────

/// CP437 box-drawing bytes from tests/fixtures/cp437_box.bin:
/// 0xC9 = ╔, 0xCD = ═, 0xBB = ╗ (double-line box top)
#[test]
fn cp437_box_drawing_roundtrip() {
    let cp437_box: &[u8] = &[0xC9, 0xCD, 0xCD, 0xBB];
    roundtrip(cp437_box, EncodingId::Cp437);
}

/// CP437 single-line box chars: 0xDA = ┌, 0xC4 = ─, 0xBF = ┐
#[test]
fn cp437_single_box_roundtrip() {
    let bytes: &[u8] = &[0xDA, 0xC4, 0xC4, 0xBF];
    roundtrip(bytes, EncodingId::Cp437);
}

/// CP437: decode known high-byte to expected Unicode code point.
/// 0xC9 should decode to U+2554 (╔)
#[test]
fn cp437_box_decode_known_value() {
    let bytes: &[u8] = &[0xC9];
    let s = decode(bytes, EncodingId::Cp437).expect("decode failed");
    assert_eq!(s, "\u{2554}", "0xC9 in CP437 should be ╔ (U+2554)");
}

// ── CP850: accented Latin characters ─────────────────────────────────────────

/// CP850 high bytes covering accented Latin characters commonly used in
/// Western European locales.
/// 0x80 = Ç, 0x81 = ü, 0x82 = é, 0x83 = â, 0x84 = ä
#[test]
fn cp850_western_european_roundtrip() {
    let bytes: &[u8] = &[0x80, 0x81, 0x82, 0x83, 0x84];
    roundtrip(bytes, EncodingId::Cp850);
}

/// CP850: 0x82 should decode to U+00E9 (é)
#[test]
fn cp850_decode_known_value() {
    let bytes: &[u8] = &[0x82];
    let s = decode(bytes, EncodingId::Cp850).expect("decode failed");
    assert_eq!(s, "\u{00E9}", "0x82 in CP850 should be é (U+00E9)");
}

// ── ISO-8859-1: high bytes 0xA0–0xFF ─────────────────────────────────────────

/// ISO-8859-1 bytes 0xA0–0xFF map 1:1 to Unicode code points U+00A0–U+00FF.
/// A round-trip through Windows-1252 (which covers ISO-8859-1 for these bytes)
/// must be byte-for-byte identical.
#[test]
fn iso8859_1_high_byte_roundtrip() {
    // A representative subset: 0xA0 (NBSP), 0xC0 (À), 0xE0 (à), 0xFF (ÿ)
    let bytes: &[u8] = &[0xA0, 0xC0, 0xE0, 0xFF];
    roundtrip(bytes, EncodingId::Iso8859_1);
}

/// ISO-8859-1: 0xE9 should decode to U+00E9 (é)
#[test]
fn iso8859_1_decode_known_value() {
    let bytes: &[u8] = &[0xE9];
    let s = decode(bytes, EncodingId::Iso8859_1).expect("decode failed");
    assert_eq!(s, "\u{00E9}", "0xE9 in ISO-8859-1 should be é (U+00E9)");
}

// ── Windows-1252: high bytes ──────────────────────────────────────────────────

/// Windows-1252 bytes 0x80–0x9F include printable characters not in ISO-8859-1.
/// 0x80 = € (U+20AC), 0x82 = ‚ (U+201A), 0x92 = ' (U+2019)
#[test]
fn windows1252_high_byte_roundtrip() {
    // Bytes 0xA0–0xFF are shared with ISO-8859-1 and must round-trip correctly.
    let bytes: &[u8] = &[0xA0, 0xC9, 0xE9, 0xFF];
    roundtrip(bytes, EncodingId::Windows1252);
}

/// Windows-1252: 0x80 should decode to U+20AC (€)
#[test]
fn windows1252_euro_sign() {
    let bytes: &[u8] = &[0x80];
    let s = decode(bytes, EncodingId::Windows1252).expect("decode failed");
    assert_eq!(s, "\u{20AC}", "0x80 in Windows-1252 should be € (U+20AC)");
}

/// Windows-1252: simple ASCII string round-trip via encoding_from_str helper.
#[test]
fn windows1252_via_encoding_from_str() {
    let enc = encoding_from_str("windows-1252");
    assert_eq!(enc, EncodingId::Windows1252);
    let bytes: &[u8] = b"Hello";
    roundtrip(bytes, enc);
}

// ── UTF-16 LE: file-based round-trips ────────────────────────────────────────

/// Open the UTF-16 LE with BOM fixture, decode to UTF-8, re-encode, and
/// assert the output is byte-identical to the original.
#[test]
fn utf16le_bom_file_roundtrip() {
    let bytes = include_bytes!("../fixtures/utf16le_bom.bin");
    roundtrip(bytes, EncodingId::Utf16Le);
}

/// Open the UTF-16 BE with BOM fixture and verify byte-identical round-trip.
#[test]
fn utf16be_bom_file_roundtrip() {
    let bytes = include_bytes!("../fixtures/utf16be_bom.bin");
    roundtrip(bytes, EncodingId::Utf16Be);
}

/// UTF-16 LE file containing a surrogate pair (earth globe emoji U+1F30D)
/// must round-trip without corruption.
#[test]
fn utf16le_surrogate_pair_file_roundtrip() {
    let bytes = include_bytes!("../fixtures/utf16le_surrogate.bin");
    roundtrip(bytes, EncodingId::Utf16Le);
}

/// BOM-less UTF-16 LE file decoded with forced encoding must yield correct
/// UTF-8 text. No round-trip check here — encoding always writes a BOM.
#[test]
fn utf16le_nobom_decode_via_forced_encoding() {
    let bytes = include_bytes!("../fixtures/utf16le_nobom.bin");
    let enc = encoding_from_str("utf-16-le");
    assert_eq!(enc, EncodingId::Utf16Le);
    let text = edit::encoding::decode(bytes, enc).expect("decode must succeed");
    assert_eq!(text, "BOM-less LE file");
}

/// Verify detect_encoding correctly identifies the UTF-16 LE BOM fixture.
#[test]
fn utf16le_bom_auto_detected() {
    let bytes = include_bytes!("../fixtures/utf16le_bom.bin");
    let enc = edit::encoding::detect_encoding(bytes);
    assert_eq!(enc, EncodingId::Utf16Le);
}

/// Verify detect_encoding correctly identifies the UTF-16 BE BOM fixture.
#[test]
fn utf16be_bom_auto_detected() {
    let bytes = include_bytes!("../fixtures/utf16be_bom.bin");
    let enc = edit::encoding::detect_encoding(bytes);
    assert_eq!(enc, EncodingId::Utf16Be);
}

/// Verify encoding_from_str aliases resolve correctly for integration use.
#[test]
fn utf16_aliases_via_encoding_from_str() {
    assert_eq!(encoding_from_str("utf-16-le"), EncodingId::Utf16Le);
    assert_eq!(encoding_from_str("utf-16-be"), EncodingId::Utf16Be);
    assert_eq!(encoding_from_str("utf-16"), EncodingId::Utf16Le);
    assert_eq!(encoding_from_str("UTF16BE"), EncodingId::Utf16Be);
}

// ── encoding_from_str: parsing ────────────────────────────────────────────────

#[test]
fn encoding_from_str_cp437() {
    assert_eq!(encoding_from_str("cp437"), EncodingId::Cp437);
    assert_eq!(encoding_from_str("CP437"), EncodingId::Cp437);
    assert_eq!(encoding_from_str("437"), EncodingId::Cp437);
}

#[test]
fn encoding_from_str_cp850() {
    assert_eq!(encoding_from_str("cp850"), EncodingId::Cp850);
    assert_eq!(encoding_from_str("850"), EncodingId::Cp850);
}

#[test]
fn encoding_from_str_iso8859_1() {
    assert_eq!(encoding_from_str("iso-8859-1"), EncodingId::Iso8859_1);
    assert_eq!(encoding_from_str("latin-1"), EncodingId::Iso8859_1);
    assert_eq!(encoding_from_str("latin1"), EncodingId::Iso8859_1);
}

#[test]
fn encoding_from_str_windows1252() {
    assert_eq!(encoding_from_str("windows-1252"), EncodingId::Windows1252);
    assert_eq!(encoding_from_str("cp1252"), EncodingId::Windows1252);
}

#[test]
fn encoding_from_str_unknown_falls_back_to_utf8() {
    assert_eq!(encoding_from_str("not-an-encoding"), EncodingId::Utf8);
}
