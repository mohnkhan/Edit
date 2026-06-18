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
                write!(f, "one or more characters cannot be encoded in the target encoding")
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
            let (cow, _encoding_used, had_errors) =
                encoding_rs::WINDOWS_1252.decode(bytes);
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
        assert!(matches!(decode(bad, Utf8), Err(TranscodeError::InvalidUtf8)));
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
