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
