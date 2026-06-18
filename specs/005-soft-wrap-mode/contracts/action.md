# Contract: Action::ToggleSoftWrap

**File**: `src/input/keymap.rs`

## Enum Variant

```rust
// Added to Action enum
ToggleSoftWrap,
```

## Default Key Binding

| Key Chord | Action |
|---|---|
| `Alt+Z` | `Action::ToggleSoftWrap` |

Added to `KeybindingMap::default()`:
```rust
map.insert("Alt+Z".to_string(), Action::ToggleSoftWrap);
```

## action_from_str Mapping

```rust
"ToggleSoftWrap" => Some(Action::ToggleSoftWrap),
```

## App Event Handler Contract

When `Action::ToggleSoftWrap` is dispatched:

1. Toggle `app.soft_wrap` (`true → false` or `false → true`).
2. Sync to `app.config.soft_wrap`.
3. If enabling and `content_width < 10`: set `app.soft_wrap = false`, set status message "Terminal too narrow for soft wrap (min 10 columns)".
4. If enabling: invalidate/rebuild `app.wrap_cache` for current viewport width.
5. If disabling: drop `app.wrap_cache` (set to `None`); reset `buffer.scroll_offset.1` (horizontal scroll) to 0 for all buffers.
6. Persist `app.config` to disk via atomic config-save path (tmp-rename pattern per FR-011):
   - On success: no user notification.
   - On failure (disk full, read-only): log at warn level; set status message
     "Config save failed: [io_error]"; do NOT revert the in-memory toggle.
7. No file I/O on the active buffer.
