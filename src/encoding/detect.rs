//! Task T011: Encoding detection.
//!
//! Inspects raw bytes to determine the most likely character encoding,
//! using BOM sniffing, UTF-8 validation, and chardetng heuristics.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// EncodingId
// ---------------------------------------------------------------------------

/// The set of encodings recognised by this editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingId {
    Utf8,
    Cp437,
    Cp850,
    Iso8859_1,
    Windows1252,
}

// ---------------------------------------------------------------------------
// EncodingProfile
// ---------------------------------------------------------------------------

/// A static descriptor for a single encoding.
pub struct EncodingProfile {
    pub name: &'static str,
    pub id: EncodingId,
    /// Optional byte-order mark that unambiguously identifies this encoding.
    pub bom: Option<&'static [u8]>,
}

// ---------------------------------------------------------------------------
// ENCODING_REGISTRY
// ---------------------------------------------------------------------------

/// All supported encodings in BOM-check priority order (BOM entries first).
pub static ENCODING_REGISTRY: &[EncodingProfile] = &[
    EncodingProfile {
        name: "UTF-8",
        id: EncodingId::Utf8,
        bom: Some(&[0xEF, 0xBB, 0xBF]),
    },
    EncodingProfile {
        name: "CP437",
        id: EncodingId::Cp437,
        bom: None,
    },
    EncodingProfile {
        name: "CP850",
        id: EncodingId::Cp850,
        bom: None,
    },
    EncodingProfile {
        name: "ISO-8859-1",
        id: EncodingId::Iso8859_1,
        bom: None,
    },
    EncodingProfile {
        name: "Windows-1252",
        id: EncodingId::Windows1252,
        bom: None,
    },
];

// ---------------------------------------------------------------------------
// UTF-16 BOM constants (used only for detection; content is transcoded to UTF-8)
// ---------------------------------------------------------------------------

/// UTF-16 little-endian BOM.
const UTF16_LE_BOM: &[u8] = &[0xFF, 0xFE];
/// UTF-16 big-endian BOM.
const UTF16_BE_BOM: &[u8] = &[0xFE, 0xFF];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Detect the encoding of `bytes`.
///
/// Detection order:
/// 1. UTF-8 BOM (`EF BB BF`) → [`EncodingId::Utf8`]
/// 2. UTF-16 LE/BE BOM → [`EncodingId::Utf8`] (caller must transcode)
/// 3. Valid UTF-8 (no BOM) → [`EncodingId::Utf8`]
/// 4. chardetng heuristic → mapped to the nearest [`EncodingId`]
/// 5. Fallback → [`EncodingId::Utf8`]
pub fn detect_encoding(bytes: &[u8]) -> EncodingId {
    // --- BOM sniffing -------------------------------------------------------

    // UTF-8 BOM
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return EncodingId::Utf8;
    }

    // UTF-16 BOMs — we flag these as Utf8 so the transcoding layer knows to
    // convert them (it will call the appropriate UTF-16 decoder).
    if bytes.starts_with(UTF16_LE_BOM) || bytes.starts_with(UTF16_BE_BOM) {
        return EncodingId::Utf8;
    }

    // --- Strict UTF-8 validation --------------------------------------------

    if std::str::from_utf8(bytes).is_ok() {
        return EncodingId::Utf8;
    }

    // --- chardetng heuristic ------------------------------------------------

    let mut det = chardetng::EncodingDetector::new();
    det.feed(bytes, true);
    let (encoding, _confident) = det.guess_assess(None, true);

    map_encoding_rs_name(encoding.name())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Map an `encoding_rs` encoding name to our [`EncodingId`].
///
/// chardetng returns an `encoding_rs::Encoding` whose `.name()` follows the
/// WHATWG encoding standard labels (e.g. `"windows-1252"`, `"UTF-8"`, …).
fn map_encoding_rs_name(name: &str) -> EncodingId {
    // encoding_rs uses WHATWG names; normalise to lowercase for matching.
    match name.to_ascii_lowercase().as_str() {
        "utf-8" => EncodingId::Utf8,
        // WHATWG maps both ISO-8859-1 and windows-1252 to "windows-1252"
        "windows-1252" => EncodingId::Windows1252,
        "iso-8859-1" => EncodingId::Iso8859_1,
        // chardetng does not currently distinguish CP437/CP850; treat any
        // IBM437/IBM850 label as CP437 (the more common DOS code page).
        "ibm437" | "ibm850" | "oem-437" | "oem-850" => EncodingId::Cp437,
        // Any other guess: return Utf8 as the safe default.
        _ => EncodingId::Utf8,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_bom_detected() {
        let bytes = b"\xEF\xBB\xBFHello";
        assert_eq!(detect_encoding(bytes), EncodingId::Utf8);
    }

    #[test]
    fn utf16_le_bom_detected_as_utf8() {
        let bytes = b"\xFF\xFEH\x00i\x00";
        assert_eq!(detect_encoding(bytes), EncodingId::Utf8);
    }

    #[test]
    fn utf16_be_bom_detected_as_utf8() {
        let bytes = b"\xFE\xFF\x00H\x00i";
        assert_eq!(detect_encoding(bytes), EncodingId::Utf8);
    }

    #[test]
    fn pure_ascii_is_utf8() {
        assert_eq!(detect_encoding(b"Hello, world!"), EncodingId::Utf8);
    }

    #[test]
    fn valid_utf8_multibyte_is_utf8() {
        // "café" in UTF-8
        assert_eq!(detect_encoding("café".as_bytes()), EncodingId::Utf8);
    }

    #[test]
    fn registry_has_five_entries() {
        assert_eq!(ENCODING_REGISTRY.len(), 5);
    }

    #[test]
    fn registry_utf8_has_bom() {
        let utf8_profile = ENCODING_REGISTRY
            .iter()
            .find(|p| p.id == EncodingId::Utf8)
            .expect("UTF-8 profile must exist");
        assert_eq!(utf8_profile.bom, Some(&[0xEF_u8, 0xBB, 0xBF][..]));
    }

    #[test]
    fn non_bom_profiles_have_no_bom() {
        for profile in ENCODING_REGISTRY.iter().filter(|p| p.id != EncodingId::Utf8) {
            assert!(profile.bom.is_none(), "{} should have no BOM", profile.name);
        }
    }
}
