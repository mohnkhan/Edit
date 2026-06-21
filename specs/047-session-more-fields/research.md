# Phase 0 Research: Restore Scroll/Selection/Encoding
## R1 — Encoding serialization
EncodingId derives only Debug/Clone/Copy/Eq (not Serialize). Decision: add `encoding_to_str` returning
canonical strings ("utf-8","utf-16-le","utf-16-be","cp437","cp850","iso-8859-1","windows-1252") that all
parse back via the existing `encoding_from_str`. Store the string; empty/absent → default decode.
Alternative (derive Serialize) rejected — keeps the on-disk format human-readable + parser-aligned.
## R2 — Schema additivity (045 pattern)
`#[serde(default)]` on every new `BufferEntry` field → v2 files without them load with defaults; no
version bump needed (already v2). Selection as `Option<SelectionEntry>` (None default). Scroll as two
u32 (0 default). Encoding as String ("" default → as-opened).
## R3 — Restore application & clamping (FR-004, no panic)
Reuse the existing cursor-clamp idioms: clamp scroll line to line_count, scroll col left to render; clamp
each selection endpoint (line→line_count-1, gcol→grapheme_count_on_line). Open via `Buffer::open(path,
encoding_from_str(entry.encoding))` when encoding present, else `self.default_encoding`. All checked
access (046) — out-of-range never panics.
## R4 — Selection value
#83 explicitly lists selection. Restored as-is (clamped). Degenerate (anchor==active) → store/restore as
no selection. Low UX risk; documented.
## Open questions: none.
