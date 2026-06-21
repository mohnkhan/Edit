# Phase 1 Data Model: Restore Scroll/Selection/Encoding
## `session::BufferEntry` (additive, all `#[serde(default)]`)
```rust
pub struct BufferEntry {
    pub path: String,
    pub cursor_line: u32, pub cursor_col: u32,
    #[serde(default)] pub soft_wrap: bool,            // 045
    #[serde(default)] pub scroll_line: u32,           // NEW: viewport top row (0-based)
    #[serde(default)] pub scroll_col: u32,            // NEW: viewport left col (0-based)
    #[serde(default)] pub selection: Option<SelectionEntry>, // NEW
    #[serde(default)] pub encoding: String,           // NEW: canonical name; "" => as-opened
}
pub struct SelectionEntry { // 1-based, mirrors cursor
    pub anchor_line: u32, pub anchor_col: u32,
    pub active_line: u32,  pub active_col: u32,
}
```
SelectionEntry derives Serialize/Deserialize/PartialEq/Clone/Debug.
## Compatibility
v2 files without the new keys → defaults (scroll 0, no selection, encoding "" → default decode) = today's
behavior. No `deny_unknown_fields`. Schema version unchanged (2).
## Application on restore (clamped, checked — no panic)
- encoding: `Buffer::open(path, if entry.encoding.is_empty() { default } else { encoding_from_str(&entry.encoding) })`
- scroll: `buf.scroll_offset = (min(scroll_line, line_count-1), scroll_col)` then existing clamp
- selection: build from SelectionEntry, clamp each endpoint to (line_count, grapheme_count_on_line);
  drop if anchor==active.
## Writer
scroll from `buf.scroll_offset`; selection from `buf.selection`; encoding via `encoding_to_str(buf.encoding)`.
## No migration (additive).
