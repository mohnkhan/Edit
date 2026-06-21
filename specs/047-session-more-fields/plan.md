# Implementation Plan: Restore Scroll, Selection & Encoding in Session
**Branch**: `047-session-more-fields` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md) | Issue #83

## Summary
Additive extension of the session record (045 pattern): `BufferEntry` gains scroll, selection, and
encoding as `#[serde(default)]` fields; the writer captures them; restore applies them clamped and
(re)opens each buffer in the recorded encoding. Schema stays v2; old files load with defaults.

## Technical Context
Rust 2021; serde+toml (existing). No new deps. Tests: cargo test + session integration. Behavior-
preserving for non-session users (FR-006); 042/046 guardrails hold.

## Constitution Check
I/II/III/IV PASS; V PASS (round-trip + legacy + clamp tests); VI PASS (additive, minimal); VII N/A.
All gates pass.

## Project Structure
- `src/encoding/mod.rs` — add `encoding_to_str(EncodingId) -> &'static str` (round-trips via `encoding_from_str`).
- `src/session/mod.rs` — `BufferEntry` += `#[serde(default)]` scroll_line/scroll_col, selection (Option<SelectionEntry>), encoding (String). New `SelectionEntry { anchor_line, anchor_col, active_line, active_col }`.
- `src/app/fileops.rs` — `build_session_data` writes them; `do_restore_session` opens with recorded encoding, applies clamped scroll + selection.

## Key decisions (research.md)
- Encoding as canonical string (EncodingId not Serialize). Empty string → as-opened default.
- Selection as a nested optional record (1-based, mirrors cursor); clamp both endpoints on restore.
- Scroll stored 0-based; `clamp_scroll` already bounds it after apply.
- Re-open in recorded encoding via `Buffer::open(path, enc)`; fall back to default if unknown/absent.

## Phases (one PR)
1. `encoding_to_str` + schema fields. 2. Writer + reader (open-with-encoding, apply clamped scroll/sel).
3. Tests: round-trip, legacy load, out-of-range clamp.

## Complexity Tracking
*Empty.*
