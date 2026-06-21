# Phase 1 Data Model: Persist Per-Tab Soft-Wrap

## Entity change: `session::BufferEntry`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BufferEntry {
    pub path: String,
    pub cursor_line: u32,
    pub cursor_col: u32,
    #[serde(default)]          // NEW (Feature 045): absent in v1 files → false
    pub soft_wrap: bool,
}
```

## Entity change: `session::SessionData` (version)

- `version` writer value: `1 → 2`.
- Loader accepts `version ∈ {1, 2}` (was `== 1` only). A v1 file has no `soft_wrap` per entry → each
  restored tab defaults to `false`, then the App's restore path leaves it as the (default) value.

## Serialization compatibility

| File written by | Read by this version | Result |
|---|---|---|
| pre-045 (v1, no field) | this version | loads; `soft_wrap` defaults false (config-default behavior) |
| this version (v2) | this version | each tab's saved `soft_wrap` restored |
| this version (v2) | pre-045 binary | loads; unknown `soft_wrap` field ignored (no `deny_unknown_fields`) — but pre-045 binary rejects `version == 2` (it only accepts 1). Acceptable: forward-incompat on version is expected; data not corrupted. |

## Application on restore

`do_restore_session`: for each `entry`, after opening the buffer and seeking the cursor,
`buf.soft_wrap = entry.soft_wrap`. Writer: `BufferEntry { …, soft_wrap: buf.soft_wrap }`.

## No new storage / format family — same `session.toml`.
