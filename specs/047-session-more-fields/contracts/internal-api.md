# Internal Contract: Restore Scroll/Selection/Encoding
## Serialization (FR-001..003, FR-005)
- C-1: BufferEntry serializes scroll_line/scroll_col/selection/encoding; all `#[serde(default)]`.
- C-2: `encoding_to_str` ∘ `encoding_from_str` round-trips for every EncodingId.
- C-3: schema stays v2; v2 files without the fields load with defaults.
## Behavioral (FR-004, FR-006)
- B-1: restore opens each buffer in the recorded encoding (or default if absent/unknown).
- B-2: restore applies scroll + selection clamped to current content; out-of-range never panics.
- B-3: new/opened tabs and non-session users are unchanged.
## Tests (SC-001..004)
- T-1 round-trip: BufferEntry with scroll≠0, a selection, non-UTF8 encoding → serialize+load equal.
- T-2 legacy: TOML without the fields → defaults, loads OK.
- T-3 clamp: oversized saved scroll/selection on a small file → restored clamped, no panic.
- T-4 suite + clippy -D warnings + fmt clean.
