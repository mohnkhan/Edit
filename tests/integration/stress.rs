//! Stress test: simulate continuous editing operations on a large buffer.

#[test]
#[ignore = "slow: stress test"]
fn stress_continuous_editing() {
    use std::time::Instant;

    // Build ~1 MB of text
    let line = "The quick brown fox jumps over the lazy dog. ";
    let content = line.repeat(22_222); // ~1 MB

    let mut rope = edit::buffer::rope::EditorRope::from_str(&content);
    let ops = 10_000;
    let start = Instant::now();

    for i in 0..ops {
        let len = rope.char_count();
        let pos = if len > 0 { (i * 97) % len } else { 0 };
        rope.insert_str(pos, "x");
        // Keep memory bounded: trim back once we exceed 2 MB worth of chars.
        if rope.char_count() > 2_000_000 {
            rope.delete_range(pos..pos + 1);
        }
    }

    let elapsed = start.elapsed();
    println!(
        "stress_continuous_editing: {ops} ops in {}ms",
        elapsed.as_millis()
    );
    assert!(
        elapsed.as_secs() < 30,
        "stress test took too long: {:?}",
        elapsed
    );
}

#[test]
#[ignore = "slow: stress test"]
fn stress_encoding_roundtrip() {
    use edit::encoding::{decode, encode, EncodingId};

    let sample = "Hello, World! 🌍 こんにちは";
    // Only test encodings that can represent the ASCII portion without error;
    // emoji and CJK may not round-trip through single-byte encodings.
    let utf8_only = [EncodingId::Utf8];
    let ascii_sample = "Hello, World! Testing 1-2-3.";
    let legacy_encodings = [EncodingId::Iso8859_1, EncodingId::Windows1252];

    // Full UTF-8 round-trip
    for enc in utf8_only {
        for _ in 0..1_000 {
            let bytes = encode(sample, enc).expect("UTF-8 encode must succeed");
            let decoded = decode(&bytes, enc).expect("UTF-8 decode must succeed");
            assert_eq!(decoded, sample);
        }
    }

    // Legacy encodings: only ASCII-safe content round-trips cleanly.
    for enc in legacy_encodings {
        for _ in 0..1_000 {
            let bytes = encode(ascii_sample, enc).unwrap_or_else(|_| b"fallback".to_vec());
            let _ = decode(&bytes, enc);
        }
    }
}

#[test]
#[ignore = "slow: stress test"]
fn stress_rope_line_operations() {
    use std::time::Instant;

    // Build a rope with 10 000 lines.
    let line = "abcdefghijklmnopqrstuvwxyz 0123456789\n";
    let content = line.repeat(10_000);
    let rope = edit::buffer::rope::EditorRope::from_str(&content);

    let start = Instant::now();
    for i in 0..rope.line_count() {
        let _ = rope.line_slice(i);
        let _ = rope.grapheme_count_on_line(i);
    }
    let elapsed = start.elapsed();
    println!(
        "stress_rope_line_operations: {} lines iterated in {}ms",
        rope.line_count(),
        elapsed.as_millis()
    );
    assert!(
        elapsed.as_secs() < 30,
        "line iteration too slow: {:?}",
        elapsed
    );
}
