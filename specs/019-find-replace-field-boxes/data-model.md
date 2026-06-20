# Phase 1 Data Model: Bordered-box Find/Replace fields

This feature is presentation-only. **No new persisted or runtime state is introduced**, and the
existing `FindReplaceDialog` state model is unchanged. This document records the existing entities the
render code reads, to confirm nothing new is required.

## Existing entity: `FindReplaceDialog` (`src/ui/dialog.rs`) — UNCHANGED

| Field | Type | Meaning | Used by render |
|-------|------|---------|----------------|
| `mode` | `DialogMode` (`Find` / `Replace`) | Which dialog is shown | Selects 1 vs 2 field boxes, title |
| `query` | `String` | "Find what" text | First field box content |
| `replacement` | `String` | "Replace with" text | Second field box content (Replace only) |
| `focus` | `DialogField` (`Query` / `Replacement`) | Which field has the caret | Caret drawn only in focused box |
| `caret` | `usize` (grapheme index) | Insertion point in focused field | Caret position within box |
| `case_sensitive` | `bool` | Case toggle (Alt+C) | Options row |
| `wrap` | `bool` | Wrap-around toggle (Alt+A) | Options row |
| `regex` | `bool` | Regex toggle (Alt+R) | Options row |
| `whole_word` | `bool` | Whole-word toggle (Alt+W) | Options row |

State transitions (insert/backspace/move/switch_focus) are already implemented and tested in
`src/ui/dialog.rs`; this feature does not alter them.

## Existing entity: `SearchState` (read for the match count) — UNCHANGED

The render reads `app.search_state.active_match` and `app.search_state.matches` to compose the
"X/Y" / "N matches" / "not found" indicator. No change.

## Derived (render-time only) values — NOT stored

- **Field box rect**: computed per frame from the dialog area (label row + 3-row bordered box).
- **Visible text slice**: computed per frame via right-anchored `truncate_to_width` so the caret and
  trailing text stay visible for long entries. Not stored.

## Conclusion

No data-model changes. Implementation is confined to the render layer.
