# Phase 1 Data Model: Harden Raw Slice/Index Access

No persisted/schema data. The "model" is the conversion unit and the fuzz input.

## Entity: Input-influenced index site (conversion unit)
A raw `[i]` / `[a..b]` whose index/offset derives from runtime input (cursor, scroll, mouse, dialog
selection, buffer number) and can be momentarily out of range.

| Form | Before | After |
|---|---|---|
| Buffer index | `self.buffers[buf_idx].field` | `if let Some(b) = self.buffers.get(buf_idx) { … }` (no-op else) |
| List from selection | `ENCODING_OPTIONS[sel]` / `ITEMS[idx]` | `OPTIONS.get(sel)` / `ITEMS.get(idx)` → existing empty/no-op |
| String byte-slice | `&s[a..caret]` | char-boundary-clamped slice (helper) — never splits a grapheme |
| Rope line | `rope.line_to_char(line)` | clamp `line` to `[0, len_lines-1]` first (mirror `line_slice`) |

Out of scope (left raw, FR-008): `buffers[0]`, compile-time constants, test-only indexing.

## Entity: Content-bearing fuzz buffer (test-only)
The 042 sweep seeded empty buffers; this extends it so each buffer starts with multibyte text (ASCII +
combining marks + CJK + emoji over several lines), making line/grapheme/byte indexing fire. Same
deterministic xorshift seeds; same exclusion of file-I/O actions; same no-panic assertion.

## Invariants
- INV-1: For any in-range index, converted access == prior raw result (behavior-preserving).
- INV-2: For any out-of-range index, converted access yields the surrounding no-op/empty/clamp (no panic).
- INV-3: No string is sliced on a non-char-boundary offset.

## No serialization / migration.
