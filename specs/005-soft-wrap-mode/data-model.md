# Data Model: Soft-Wrap Mode (Feature 005)

**Date**: 2026-06-19

---

## Entities

### WrapCache *(new — `src/ui/wrap.rs`)*

Computed cache mapping logical lines to visual-line start positions. Stored in `App`. Invalidated on text edit, terminal resize, or soft-wrap toggle.

| Field | Type | Description |
|---|---|---|
| `viewport_width` | `u16` | Column width used to compute this cache. |
| `text_version` | `u64` | Buffer generation counter at time of computation. |
| `visual_starts` | `Vec<Vec<u32>>` | Outer index = logical line; inner `Vec` = byte offsets within that line's string where each visual sub-line starts (always includes `0`). |

**State transitions**:
- `Empty → Populated`: on first render with soft-wrap enabled.
- `Populated → Stale → Repopulated`: when `text_version` or `viewport_width` changes.
- `Populated → Empty`: when soft-wrap is disabled.

**Derived view** (flat list built at render time):

```
visual_line_map: Vec<(logical_line: u32, start_byte: u32)>
```

Used for O(1) visual-row → logical-position lookup during mouse click handling.

---

### Config *(modified — `src/config/schema.rs`)*

New field added to the existing `Config` struct:

| Field | Type | Default | Persisted | Description |
|---|---|---|---|---|
| `soft_wrap` | `bool` | `false` | Yes (TOML) | Global soft-wrap toggle. |

Serde attribute: `#[serde(default)]` — existing configs without the key parse correctly.

---

### App *(modified — `src/app.rs`)*

New field added to the existing `App` struct:

| Field | Type | Description |
|---|---|---|
| `soft_wrap` | `bool` | Runtime mirror of `Config.soft_wrap`. Updated on toggle; written back to config. |
| `wrap_cache` | `Option<WrapCache>` | `Some` when soft-wrap is active and cache is populated; `None` otherwise. |

---

### Action *(modified — `src/input/keymap.rs`)*

New variant added to the existing `Action` enum:

| Variant | Trigger | Description |
|---|---|---|
| `ToggleSoftWrap` | Alt+Z / View menu | Toggle soft-wrap on/off. |

---

### MenuItem *(modified — `src/ui/menubar.rs`)*

New item added to `VIEW_MENU`:

| Label | Action | Annotation |
|---|---|---|
| `"Soft Wrap (ext)"` | `Action::ToggleSoftWrap` | Labeled `(ext)` to mark as non-DOS extension. |

---

### StatusBar *(modified — `src/ui/statusbar.rs`)*

New field added to `StatusBar`:

| Field | Type | Description |
|---|---|---|
| `soft_wrap` | `bool` | When `true`, appends `[WRAP]` to the left flags section. |

---

### EditorWidget *(modified — `src/ui/editor.rs`)*

New fields and behaviour added to `EditorWidget`:

| Field | Type | Description |
|---|---|---|
| `soft_wrap` | `bool` | When `true`, activates wrap rendering path. |
| `wrap_starts` | `&'a [Vec<u32>]` | Slice of `WrapCache.visual_starts` for the active buffer's lines. |

Rendering branches:
- **`soft_wrap == false`**: existing horizontal-scroll path unchanged.
- **`soft_wrap == true`**: new wrap rendering path — see contracts for algorithm.

---

## Validation Rules

- `viewport_width >= 10`: soft-wrap auto-disables below this threshold.
- `visual_starts[L][0] == 0`: invariant — every logical line's first visual sub-line starts at byte offset 0.
- `visual_starts[L]` is sorted ascending: wrap break byte offsets within a line are monotonically increasing.
- Wrap break points always land on grapheme-cluster boundaries (never mid-sequence).
- Double-width characters are never split: if a 2-column character would straddle the wrap point, the break is moved one grapheme earlier.
