# Research: Soft-Wrap Mode (Feature 005)

**Date**: 2026-06-19
**Status**: Complete — all decisions resolved

---

## Decision 1: Wrap-Point Computation Algorithm

**Decision**: Walk grapheme clusters using `unicode_segmentation::UnicodeSegmentation::graphemes(s, true)` and measure each cluster's display width with `unicode_width::UnicodeWidthStr::width(cluster)`. Break the visual line at the last cluster boundary where the running width total would exceed the viewport.

**Rationale**:
- `UnicodeWidthChar` (per-codepoint) produces wrong widths for ZWJ emoji sequences, Arabic Lam+Alef ligatures, and several script ligatures — the sum of codepoint widths can overcount. `UnicodeWidthStr` is correct for clusters.
- Both crates are already present in `Cargo.toml` (used in `src/ui/editor.rs`), so no new dependencies are added.
- The editor's existing rendering loop already uses exactly this pattern (`editor.rs:163–226`) for horizontal-scroll mode. The soft-wrap renderer will reuse the same idiom.

**Alternatives considered**:
- Per-codepoint width sum: rejected — documented to be incorrect for composite sequences.
- OS/terminal width detection via `wcwidth()` FFI: rejected — unnecessary complexity; the `unicode-width` crate correctly implements Unicode TR#11.

**Wrap-opportunity rule**: prefer breaking at the last whitespace character (space U+0020, tab U+0009) that fits. If no whitespace fits in the viewport, hard-break at the column boundary (grapheme-cluster boundary). This is the "word-wrap with hard-wrap fallback" model used by virtually all terminal editors.

---

## Decision 2: Wrap Cache Structure

**Decision**: Store computed wrap points in a `WrapCache` struct kept in `App`. It maps each logical line index to a `Vec<u32>` of byte offsets within that line's string where visual lines start (always includes offset 0 as the first entry).

```
WrapCache {
    viewport_width: u16,          // width used to compute this cache
    text_version: u64,            // matches buffer generation counter
    visual_starts: Vec<Vec<u32>>, // outer = logical line, inner = byte offsets
}
```

**Invalidation triggers**:
1. Text edit in any buffer (buffer generation counter increments)
2. Terminal resize (viewport_width changes)
3. Soft-wrap toggled off (cache dropped; not needed)

**Rationale**:
- Per-frame recomputation of wrap points is O(N·chars) for an N-line file — prohibitive for large files at 60+ Hz render.
- Using `u32` byte offsets halves memory vs. `usize` on 64-bit targets; 4 GB per line is ample.
- Keyed per logical line so a single-line edit only invalidates one entry (amortized O(1) invalidation).

**Alternatives considered**:
- Recompute on every frame: rejected — unacceptable for large files.
- Full `Vec<VisualLine>` flat structure: acceptable, but the per-line `Vec<Vec<u32>>` is simpler to invalidate partially and has better cache locality for single-line edits.

---

## Decision 3: Visual → Logical Position Mapping (Mouse Clicks)

**Decision**: Build a flat `Vec<(logical_line, start_grapheme_idx)>` from the wrap cache at render time, stored in `App` alongside the cache. On mouse click:

1. Look up the clicked visual row in the flat list to get `(logical_line, start_grapheme_idx)`.
2. Walk grapheme clusters from `start_grapheme_idx`, accumulating widths, until reaching the clicked column. That grapheme index is the new cursor `grapheme_col`.
3. Clamp to the last grapheme of the visual line segment if the click is past end-of-content.

**Rationale**: This is the approach recommended by the Xi-editor Rope Science series and confirmed by Helix's `char_idx_at_visual_offset` implementation. A flat `Vec` indexed by visual row gives O(1) lookup with a trivial binary search fallback for large screens.

**Alternatives considered**:
- Full DocFormatter pattern (Helix): overkill for a single-viewport editor without multiple cursors.

---

## Decision 4: Scope — Global Toggle, Not Per-Buffer

**Decision**: Soft-wrap is a single global toggle affecting all buffers simultaneously. It is stored in `Config.soft_wrap: bool` (persisted to TOML) and mirrored in `App.soft_wrap: bool` at runtime.

**Rationale**: A per-buffer toggle adds `O(buffers)` UI state with minimal user benefit in a DOS-style editor. The existing analogous features (`line_numbers`, `highlight`) are also global toggles. Keeping the pattern consistent simplifies the implementation.

**Alternatives considered**:
- Per-buffer soft_wrap flag in `Buffer` struct: rejected for v1 — can be added as a follow-up if user demand warrants it.

---

## Decision 5: Minimum Viewport Width Guard

**Decision**: If the terminal content width (excluding gutter) falls below 10 display columns, soft-wrap is automatically disabled and a status-bar warning is shown. The minimum is checked on toggle and on resize.

**Rationale**: A viewport narrower than ~10 columns makes wrap points degenerate (single-character segments) and produces unusable output. A hard floor prevents pathological behavior without complex logic.

---

## Decision 6: Config Key and Default

**Decision**: Key `soft_wrap` in `$XDG_CONFIG_HOME/edit/config.toml`, defaulting to `false`. Uses `#[serde(default)]` so existing config files without the key parse correctly and default to `false` (opt-in feature).

**Rationale**: Matches the `line_numbers` and `highlight` field patterns in `src/config/schema.rs`. Write-on-toggle using existing config save infrastructure.

---

## Decision 7: Continuation Marker

**Decision**: Use U+00BB `»` (RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK) displayed in the leftmost gutter column of each continuation visual line. Terminal fallback: `>` (U+003E) when the terminal cannot render U+00BB (detected by checking `TERM`/`LANG` or falling back on display failure).

**Rationale**: Matches the user spec. `»` is widely supported in modern UTF-8 terminals. The fallback ensures correctness on legacy `TERM=linux` console.

---

## Decision 8: Status Bar Indicator

**Decision**: Append `[WRAP]` to the left section of the status bar when soft-wrap is active. It appears after the existing `[Modified]`/`[Read Only]` flags. Uses the same style as those flags (no special color).

**Rationale**: Consistent with the established pattern. Easily visible without occupying the right-aligned position section.

---

## No NEEDS CLARIFICATION Items Remain

All unknowns resolved by code inspection and research. No external dependencies added.
