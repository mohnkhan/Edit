//! Encoding detection and transcoding.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use crate::encoding::{EncodingId, detect_encoding, decode, encode};
//!
//! let bytes = std::fs::read("file.txt").unwrap();
//! let enc = detect_encoding(&bytes);
//! let text = decode(&bytes, enc).unwrap();
//! let back = encode(&text, enc).unwrap();
//! ```

#![allow(dead_code, unused_imports)]

pub mod detect;
pub mod transcode;

pub use detect::{detect_encoding, EncodingId, EncodingProfile, ENCODING_REGISTRY};
pub use transcode::{decode, encode, TranscodeError};

/// Return the `EncodingProfile` for the given `EncodingId`.
pub fn profile_for(id: EncodingId) -> &'static EncodingProfile {
    ENCODING_REGISTRY
        .iter()
        .find(|p| p.id == id)
        .expect("ENCODING_REGISTRY must contain all EncodingId variants")
}

/// Parse an encoding name string (case-insensitive) into an `EncodingId`.
/// Returns `EncodingId::Utf8` for unrecognised strings.
pub fn encoding_from_str(s: &str) -> EncodingId {
    match s.to_ascii_lowercase().as_str() {
        "utf-8" | "utf8" => EncodingId::Utf8,
        "utf-16-le" | "utf16-le" | "utf16le" | "utf-16 le" | "utf16_le" => EncodingId::Utf16Le,
        "utf-16-be" | "utf16-be" | "utf16be" | "utf-16 be" | "utf16_be" => EncodingId::Utf16Be,
        "utf-16" => EncodingId::Utf16Le,
        "cp437" | "437" => EncodingId::Cp437,
        "cp850" | "850" => EncodingId::Cp850,
        "iso-8859-1" | "iso8859-1" | "latin-1" | "latin1" => EncodingId::Iso8859_1,
        "windows-1252" | "cp1252" | "win-1252" => EncodingId::Windows1252,
        other => {
            log::warn!("Unknown encoding {:?}; falling back to UTF-8", other);
            EncodingId::Utf8
        }
    }
}

/// Canonical string name for an [`EncodingId`] (Feature 047), suitable for
/// persistence. Every returned string round-trips through [`encoding_from_str`].
pub fn encoding_to_str(enc: EncodingId) -> &'static str {
    match enc {
        EncodingId::Utf8 => "utf-8",
        EncodingId::Utf16Le => "utf-16-le",
        EncodingId::Utf16Be => "utf-16-be",
        EncodingId::Cp437 => "cp437",
        EncodingId::Cp850 => "cp850",
        EncodingId::Iso8859_1 => "iso-8859-1",
        EncodingId::Windows1252 => "windows-1252",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16le_aliases() {
        assert_eq!(encoding_from_str("utf-16-le"), EncodingId::Utf16Le);
        assert_eq!(encoding_from_str("utf16-le"), EncodingId::Utf16Le);
        assert_eq!(encoding_from_str("utf16le"), EncodingId::Utf16Le);
    }

    #[test]
    fn utf16le_aliases_case_insensitive() {
        assert_eq!(encoding_from_str("UTF-16-LE"), EncodingId::Utf16Le);
        assert_eq!(encoding_from_str("UTF16LE"), EncodingId::Utf16Le);
    }

    #[test]
    fn utf16be_aliases() {
        assert_eq!(encoding_from_str("utf-16-be"), EncodingId::Utf16Be);
        assert_eq!(encoding_from_str("utf16-be"), EncodingId::Utf16Be);
        assert_eq!(encoding_from_str("utf16be"), EncodingId::Utf16Be);
    }

    #[test]
    fn utf16be_aliases_case_insensitive() {
        assert_eq!(encoding_from_str("UTF-16-BE"), EncodingId::Utf16Be);
        assert_eq!(encoding_from_str("UTF16BE"), EncodingId::Utf16Be);
    }

    #[test]
    fn utf16_no_endian_defaults_to_le() {
        assert_eq!(encoding_from_str("utf-16"), EncodingId::Utf16Le);
    }

    #[test]
    fn unknown_alias_falls_back_to_utf8() {
        assert_eq!(encoding_from_str("utf-16-xx"), EncodingId::Utf8);
        assert_eq!(encoding_from_str("not-an-encoding"), EncodingId::Utf8);
    }

    #[test]
    fn existing_aliases_still_work() {
        assert_eq!(encoding_from_str("utf-8"), EncodingId::Utf8);
        assert_eq!(encoding_from_str("cp437"), EncodingId::Cp437);
        assert_eq!(encoding_from_str("windows-1252"), EncodingId::Windows1252);
    }

    #[test]
    fn encoding_to_str_round_trips_all_variants() {
        for enc in [
            EncodingId::Utf8,
            EncodingId::Utf16Le,
            EncodingId::Utf16Be,
            EncodingId::Cp437,
            EncodingId::Cp850,
            EncodingId::Iso8859_1,
            EncodingId::Windows1252,
        ] {
            assert_eq!(encoding_from_str(encoding_to_str(enc)), enc);
        }
    }
}
