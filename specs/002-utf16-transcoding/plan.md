# Implementation Plan: UTF-16 Transcoding Support

**Branch**: `002-utf16-transcoding` | **Date**: 2026-06-18 | **Spec**: [spec.md](spec.md)

## Summary

Add `Utf16Le` and `Utf16Be` variants to `EncodingId`, wire up `encoding_rs`'s UTF-16
decoders and encoders in `src/encoding/`, fix BOM detection to return the correct variant,
update the registry and `encoding_from_str`, and expose the encoding names in the status bar.
No new dependencies required — `encoding_rs` (already in `Cargo.toml`) provides UTF-16 LE/BE.

## Technical Context

**Language/Version**: Rust stable edition 2021, MSRV 1.74.0

**Primary Dependencies**: `encoding_rs 0.8` (already present — provides `UTF_16_LE` and
`UTF_16_BE` encoding objects); no new crate dependencies needed.

**Storage**: File I/O only; internal buffer remains UTF-8 rope.

**Testing**: `cargo test` (unit + integration); `make smoke` (headless expect scripts).

**Target Platform**: Linux x86_64 / aarch64; FreeBSD; macOS.

**Project Type**: CLI text editor (TUI).

**Performance Goals**: Open a 10 MB UTF-16 file in ≤ 1 second (transcode is O(n) in
file size; `encoding_rs` SIMD-accelerated).

**Constraints**: No new `Cargo.toml` dependencies; internal buffer stays UTF-8.

**Scale/Scope**: Touches ~5 source files; ~200 new lines of code + ~200 new test lines.

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. DOS-Faithful UI | ✅ Pass | Status bar updated to show UTF-16 LE / UTF-16 BE |
| II. UTF-8 First (NON-NEGOTIABLE) | ✅ Pass | Internal buffer stays UTF-8; UTF-16 is file I/O only |
| III. Portable Build | ✅ Pass | `encoding_rs` is cross-platform |
| IV. Minimal Footprint | ✅ Pass | No new dependencies; `encoding_rs` already present |
| V. Test-Gated Merges | ✅ Pass | TDD: tests written before implementation; ≥ 15 new tests required |
| VI. Simplicity / YAGNI | ✅ Pass | Spec filed, issue #5 exists, in ROADMAP.md |
| VII. Security Hardening | ✅ Pass | No privilege escalation; path sanitization unchanged |

**Self-certification**: This PR touches file I/O and encoding. Path traversal protections in
`src/security/sanitize.rs` are unchanged. No terminal escape sequences are emitted from
decoded UTF-16 content (all content goes through the existing sanitizer before rendering).

## Project Structure

### Documentation (this feature)

```text
specs/002-utf16-transcoding/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── encoding-api.md
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code Changes

```text
src/encoding/
├── detect.rs            # Add Utf16Le, Utf16Be to EncodingId; add BOM profiles to
│                        #   ENCODING_REGISTRY; fix detect_encoding() to return the
│                        #   correct variant instead of aliasing to Utf8
├── transcode.rs         # Add decode() and encode() arms for Utf16Le and Utf16Be
│                        #   using encoding_rs::UTF_16_LE / UTF_16_BE
└── mod.rs               # Add utf-16-le, utf16le, utf-16-be, utf16be aliases to
                         #   encoding_from_str()

src/ui/
└── statusbar.rs         # "UTF-16 LE" / "UTF-16 BE" display names (already
                         #   driven by profile.name; no change needed if registry
                         #   names are set correctly)

tests/
├── integration/
│   └── encoding_roundtrip.rs   # Add UTF-16 round-trip tests (≥ 10 new)
├── fixtures/
│   ├── utf16le_bom.bin          # New: UTF-16 LE with BOM
│   ├── utf16be_bom.bin          # New: UTF-16 BE with BOM
│   ├── utf16le_nobom.bin        # New: UTF-16 LE without BOM
│   └── utf16le_surrogate.bin    # New: file with supplementary chars (surrogates)
└── unit/ (via #[cfg(test)] in encode/decode modules)
```

## Complexity Tracking

No constitution violations. No complexity justification needed.
