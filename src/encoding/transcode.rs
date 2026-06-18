//! Task T012: Encoding transcoding.
//!
//! Converts raw bytes in a known encoding to a Rust `String` (UTF-8), and
//! re-encodes a `String` back to a target encoding.

#![allow(dead_code)]

use super::detect::EncodingId;

// ---------------------------------------------------------------------------
// TranscodeError
// ---------------------------------------------------------------------------

/// Errors that can occur during transcoding.
#[derive(Debug)]
pub enum TranscodeError {
    /// The input bytes are not valid UTF-8.
    InvalidUtf8,
    /// One or more characters in the string cannot be represented in the
    /// target encoding.
    Unencodable,
    /// An I/O error occurred (used when reading/writing to streams).
    Io(std::io::Error),
}

impl std::fmt::Display for TranscodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscodeError::InvalidUtf8 => write!(f, "invalid UTF-8 sequence in input"),
            TranscodeError::Unencodable => {
                write!(
                    f,
                    "one or more characters cannot be encoded in the target encoding"
                )
            }
            TranscodeError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for TranscodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TranscodeError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TranscodeError {
    fn from(e: std::io::Error) -> Self {
        TranscodeError::Io(e)
    }
}

// ---------------------------------------------------------------------------
// UTF-8 BOM
// ---------------------------------------------------------------------------

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

// ---------------------------------------------------------------------------
// decode
// ---------------------------------------------------------------------------

/// Decode `bytes` encoded as `enc` into a UTF-8 `String`.
///
/// * [`EncodingId::Utf8`] — validated with `std::str::from_utf8`; BOM stripped.
/// * [`EncodingId::Iso8859_1`] / [`EncodingId::Windows1252`] — decoded via
///   `encoding_rs` using the WINDOWS_1252 codec (ISO-8859-1 is a strict
///   subset, so the same table covers both).
/// * [`EncodingId::Cp437`] / [`EncodingId::Cp850`] — decoded via `oem-cp`
///   using the appropriate DOS code-page table.
pub fn decode(bytes: &[u8], enc: EncodingId) -> Result<String, TranscodeError> {
    match enc {
        EncodingId::Utf8 => {
            // Strip UTF-8 BOM if present.
            let payload = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
            match std::str::from_utf8(payload) {
                Ok(s) => Ok(s.to_owned()),
                Err(_) => Err(TranscodeError::InvalidUtf8),
            }
        }

        EncodingId::Iso8859_1 | EncodingId::Windows1252 => {
            // WINDOWS_1252 is a superset of ISO-8859-1 for bytes 0x80–0x9F;
            // for our purposes the distinction rarely matters in practice.
            let (cow, _encoding_used, had_errors) = encoding_rs::WINDOWS_1252.decode(bytes);
            if had_errors {
                // encoding_rs replaces unmappable bytes with U+FFFD; we still
                // return the (lossy) string rather than an error, because
                // legacy files are inherently imperfect.
                Ok(cow.into_owned())
            } else {
                Ok(cow.into_owned())
            }
        }

        EncodingId::Cp437 => {
            let s = oem_cp::decode_string_complete_table(
                bytes,
                &oem_cp::code_table::DECODING_TABLE_CP437,
            );
            Ok(s)
        }

        EncodingId::Cp850 => {
            let s = oem_cp::decode_string_complete_table(
                bytes,
                &oem_cp::code_table::DECODING_TABLE_CP850,
            );
            Ok(s)
        }

        EncodingId::Utf16Le => {
            // `is_multiple_of(2)` was stabilised in Rust 1.86; allow the
            // manual form to stay within our MSRV of 1.74.
            #[allow(clippy::manual_is_multiple_of)]
            if bytes.len() % 2 != 0 {
                return Err(TranscodeError::InvalidUtf8);
            }
            let (cow, _, _) = encoding_rs::UTF_16LE.decode(bytes);
            Ok(cow.into_owned())
        }

        EncodingId::Utf16Be => {
            #[allow(clippy::manual_is_multiple_of)]
            if bytes.len() % 2 != 0 {
                return Err(TranscodeError::InvalidUtf8);
            }
            let (cow, _, _) = encoding_rs::UTF_16BE.decode(bytes);
            Ok(cow.into_owned())
        }
    }
}

// ---------------------------------------------------------------------------
// encode
// ---------------------------------------------------------------------------

/// Encode the UTF-8 string `s` into bytes using `enc`.
///
/// * [`EncodingId::Utf8`] — returns `s.as_bytes()` as-is.
/// * [`EncodingId::Iso8859_1`] / [`EncodingId::Windows1252`] — encoded via
///   `encoding_rs` WINDOWS_1252; unmappable characters produce
///   [`TranscodeError::Unencodable`].
/// * [`EncodingId::Cp437`] / [`EncodingId::Cp850`] — encoded via `oem-cp`;
///   unmappable characters produce [`TranscodeError::Unencodable`].
pub fn encode(s: &str, enc: EncodingId) -> Result<Vec<u8>, TranscodeError> {
    match enc {
        EncodingId::Utf8 => Ok(s.as_bytes().to_vec()),

        EncodingId::Iso8859_1 | EncodingId::Windows1252 => {
            let mut encoder = encoding_rs::WINDOWS_1252.new_encoder();
            let mut output = Vec::with_capacity(s.len());

            // encode_from_utf8 returns (CoderResult, bytes_read, bytes_written, had_unmappables).
            let mut input = s;
            loop {
                let mut buf = [0u8; 4096];
                let (result, bytes_read, bytes_written, had_unmappables) =
                    encoder.encode_from_utf8(input, &mut buf, true);
                if had_unmappables {
                    return Err(TranscodeError::Unencodable);
                }
                output.extend_from_slice(&buf[..bytes_written]);
                input = &input[bytes_read..];

                use encoding_rs::CoderResult;
                match result {
                    CoderResult::InputEmpty => break,
                    CoderResult::OutputFull => {
                        // Keep going — more buffer space available next round.
                        if bytes_read == 0 {
                            // No progress possible — unmappable character.
                            return Err(TranscodeError::Unencodable);
                        }
                    }
                }
            }
            Ok(output)
        }

        EncodingId::Cp437 => {
            // oem_cp::encode_string_checked returns Option<Vec<u8>>:
            // Some(bytes) on success, None if a char cannot be encoded.
            oem_cp::encode_string_checked(s, &oem_cp::code_table::ENCODING_TABLE_CP437)
                .ok_or(TranscodeError::Unencodable)
        }

        EncodingId::Cp850 => {
            oem_cp::encode_string_checked(s, &oem_cp::code_table::ENCODING_TABLE_CP850)
                .ok_or(TranscodeError::Unencodable)
        }

        EncodingId::Utf16Le => {
            // encoding_rs::UTF_16LE.encode() returns UTF-8 bytes unchanged for
            // ASCII, not UTF-16 bytes. Use str::encode_utf16() for correct output.
            let mut out = vec![0xFF, 0xFE];
            for unit in s.encode_utf16() {
                out.extend_from_slice(&unit.to_le_bytes());
            }
            Ok(out)
        }

        EncodingId::Utf16Be => {
            let mut out = vec![0xFE, 0xFF];
            for unit in s.encode_utf16() {
                out.extend_from_slice(&unit.to_be_bytes());
            }
            Ok(out)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use EncodingId::*;

    // --- UTF-8 round-trip ---------------------------------------------------

    #[test]
    fn utf8_roundtrip_ascii() {
        let s = "Hello, world!";
        let encoded = encode(s, Utf8).unwrap();
        let decoded = decode(&encoded, Utf8).unwrap();
        assert_eq!(decoded, s);
    }

    #[test]
    fn utf8_roundtrip_multibyte() {
        let s = "Héllo wörld – ñoño";
        let encoded = encode(s, Utf8).unwrap();
        let decoded = decode(&encoded, Utf8).unwrap();
        assert_eq!(decoded, s);
    }

    #[test]
    fn utf8_bom_stripped_on_decode() {
        let with_bom = b"\xEF\xBB\xBFHello";
        let decoded = decode(with_bom, Utf8).unwrap();
        assert_eq!(decoded, "Hello");
    }

    #[test]
    fn utf8_invalid_bytes_error() {
        let bad = b"\xFF\xFE";
        assert!(matches!(
            decode(bad, Utf8),
            Err(TranscodeError::InvalidUtf8)
        ));
    }

    // --- Windows-1252 -------------------------------------------------------

    #[test]
    fn windows1252_decode_high_bytes() {
        // 0x80 in Windows-1252 maps to U+20AC (€)
        let bytes = &[0x80u8];
        let s = decode(bytes, Windows1252).unwrap();
        assert_eq!(s, "\u{20AC}");
    }

    #[test]
    fn windows1252_encode_ascii_roundtrip() {
        let s = "Hello";
        let encoded = encode(s, Windows1252).unwrap();
        let decoded = decode(&encoded, Windows1252).unwrap();
        assert_eq!(decoded, s);
    }

    // --- CP437 --------------------------------------------------------------

    #[test]
    fn cp437_decode_ascii_passthrough() {
        // ASCII bytes 0x20–0x7E are the same in CP437.
        let bytes: Vec<u8> = (0x20u8..=0x7Eu8).collect();
        let s = decode(&bytes, Cp437).unwrap();
        assert!(s.is_ascii());
    }

    #[test]
    fn cp437_roundtrip_ascii() {
        let s = "Hello";
        let encoded = encode(s, Cp437).unwrap();
        let decoded = decode(&encoded, Cp437).unwrap();
        assert_eq!(decoded, s);
    }

    // --- CP850 --------------------------------------------------------------

    #[test]
    fn cp850_roundtrip_ascii() {
        let s = "World";
        let encoded = encode(s, Cp850).unwrap();
        let decoded = decode(&encoded, Cp850).unwrap();
        assert_eq!(decoded, s);
    }

    // --- UTF-16 LE decode ---------------------------------------------------

    #[test]
    fn utf16le_decode_with_bom() {
        // "Hi" encoded as UTF-16 LE with BOM: FF FE 48 00 69 00
        let bytes = b"\xFF\xFEH\x00i\x00";
        let s = decode(bytes, Utf16Le).unwrap();
        assert_eq!(s, "Hi");
    }

    #[test]
    fn utf16le_decode_bom_stripped() {
        let bytes = b"\xFF\xFEA\x00";
        let s = decode(bytes, Utf16Le).unwrap();
        // BOM must not appear in the result
        assert_eq!(s, "A");
        assert!(!s.contains('\u{FEFF}'));
    }

    #[test]
    fn utf16le_decode_empty() {
        let s = decode(b"", Utf16Le).unwrap();
        assert_eq!(s, "");
    }

    #[test]
    fn utf16le_decode_odd_length_error() {
        // 3 bytes is not a valid UTF-16 stream
        let bytes = b"\xFF\xFEH";
        assert!(matches!(
            decode(bytes, Utf16Le),
            Err(TranscodeError::InvalidUtf8)
        ));
    }

    #[test]
    fn utf16le_decode_japanese() {
        // "こ" = U+3053, LE encoding: 53 30
        let bytes = b"\xFF\xFE\x53\x30";
        let s = decode(bytes, Utf16Le).unwrap();
        assert_eq!(s, "こ");
    }

    #[test]
    fn utf16le_decode_surrogate_pair() {
        // U+1F30D (earth globe) in UTF-16 LE: FF FE 3C D8 0D DF
        let bytes = b"\xFF\xFE\x3C\xD8\x0D\xDF";
        let s = decode(bytes, Utf16Le).unwrap();
        assert_eq!(s, "\u{1F30D}");
    }

    // --- UTF-16 BE decode ---------------------------------------------------

    #[test]
    fn utf16be_decode_with_bom() {
        // "Hi" encoded as UTF-16 BE with BOM: FE FF 00 48 00 69
        let bytes = b"\xFE\xFF\x00H\x00i";
        let s = decode(bytes, Utf16Be).unwrap();
        assert_eq!(s, "Hi");
    }

    #[test]
    fn utf16be_decode_bom_stripped() {
        let bytes = b"\xFE\xFF\x00A";
        let s = decode(bytes, Utf16Be).unwrap();
        assert_eq!(s, "A");
        assert!(!s.contains('\u{FEFF}'));
    }

    #[test]
    fn utf16be_decode_odd_length_error() {
        let bytes = b"\xFE\xFF\x00";
        assert!(matches!(
            decode(bytes, Utf16Be),
            Err(TranscodeError::InvalidUtf8)
        ));
    }

    // --- UTF-16 LE encode ---------------------------------------------------

    #[test]
    fn utf16le_encode_ascii() {
        let bytes = encode("Hi", Utf16Le).unwrap();
        // Expect BOM + "H\x00i\x00"
        assert_eq!(&bytes[..2], &[0xFF, 0xFE]);
        assert_eq!(&bytes[2..], b"H\x00i\x00");
    }

    #[test]
    fn utf16le_encode_empty_is_bom_only() {
        let bytes = encode("", Utf16Le).unwrap();
        assert_eq!(bytes, vec![0xFF, 0xFE]);
    }

    #[test]
    fn utf16le_encode_surrogate_pair() {
        let bytes = encode("\u{1F30D}", Utf16Le).unwrap();
        // BOM + surrogate pair for U+1F30D: 3C D8 0D DF
        assert_eq!(&bytes[..2], &[0xFF, 0xFE]);
        assert_eq!(&bytes[2..], b"\x3C\xD8\x0D\xDF");
    }

    // --- UTF-16 BE encode ---------------------------------------------------

    #[test]
    fn utf16be_encode_ascii() {
        let bytes = encode("Hi", Utf16Be).unwrap();
        assert_eq!(&bytes[..2], &[0xFE, 0xFF]);
        assert_eq!(&bytes[2..], b"\x00H\x00i");
    }

    #[test]
    fn utf16be_encode_empty_is_bom_only() {
        let bytes = encode("", Utf16Be).unwrap();
        assert_eq!(bytes, vec![0xFE, 0xFF]);
    }

    // --- UTF-16 round-trip (unit level) -------------------------------------

    #[test]
    fn utf16le_roundtrip_ascii() {
        let s = "Hello, world!";
        let encoded = encode(s, Utf16Le).unwrap();
        let decoded = decode(&encoded, Utf16Le).unwrap();
        assert_eq!(decoded, s);
    }

    #[test]
    fn utf16be_roundtrip_ascii() {
        let s = "Hello, world!";
        let encoded = encode(s, Utf16Be).unwrap();
        let decoded = decode(&encoded, Utf16Be).unwrap();
        assert_eq!(decoded, s);
    }

    #[test]
    fn utf16le_roundtrip_multibyte() {
        let s = "こんにちは 🌍";
        let encoded = encode(s, Utf16Le).unwrap();
        let decoded = decode(&encoded, Utf16Le).unwrap();
        assert_eq!(decoded, s);
    }

    // --- TranscodeError Display ---------------------------------------------

    #[test]
    fn error_display_invalid_utf8() {
        let msg = format!("{}", TranscodeError::InvalidUtf8);
        assert!(msg.contains("UTF-8"));
    }

    #[test]
    fn error_display_unencodable() {
        let msg = format!("{}", TranscodeError::Unencodable);
        assert!(msg.contains("cannot be encoded"));
    }
}
