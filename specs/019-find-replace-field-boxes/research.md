# Phase 0 Research: Bordered-box Find/Replace fields

No open `NEEDS CLARIFICATION` markers from the spec. Research below records the design decisions made
by reading the existing code so the implementation reuses what feature 018 established.

## Decision 1: Where the dialog is rendered

- **Decision**: Modify the inline Find/Replace overlay block in `src/ui/mod.rs` (`Ui::render`,
  currently lines ~205-287). The `FindReplaceDialog` struct in `src/ui/dialog.rs` is pure state; it
  has no `Widget` impl, so all visuals live in `mod.rs`.
- **Rationale**: Keeps the change in one render site; matches where the file browser and other
  overlays are dispatched.
- **Alternatives considered**: Give `FindReplaceDialog` its own `Widget` impl (like
  `EncodingSelectDialog`). Rejected for this scope — it would relocate logic that reads `app.theme`
  and `app.search_state`, enlarging the diff without behavior benefit. May be revisited under #38.

## Decision 2: Reuse the file-browser input-box treatment

- **Decision**: Render each field as a label row (`Find what:` / `Replace with:`) above a 3-row
  bordered `Block` (`Borders::ALL`), with the field text drawn inside on the middle row and a caret.
  Reuse `truncate_to_width()` (already `pub` in `src/ui/file_browser.rs`) and the right-anchored
  caret pattern (keep the caret + tail visible when text is wider than the inner box).
- **Rationale**: This is exactly the affordance feature 018 shipped for the file browser
  (`compute_layout` reserves `field_label_row` + a 3-row `field_box`; the render appends a `▏` caret
  and right-anchors). Reusing it guarantees visual consistency (SC-001) and the long-text scroll
  behavior (edge case).
- **Alternatives considered**: A 1-row "box" using side characters only. Rejected — does not match
  the 018 look (full bordered box) the spec asks to mirror.

## Decision 3: Caret glyph

- **Decision**: Use the same caret glyph as the file browser, `▏` (U+258F LEFT ONE EIGHTH BLOCK),
  drawn inside the box. Today the inline Find/Replace uses `│` (U+2502), which is also the box
  vertical-border glyph and would be ambiguous inside a bordered box.
- **Rationale**: Visual consistency with 018 and avoids confusing the caret with the box border.
- **Alternatives considered**: Keep `│`. Rejected — collides visually with the border now that the
  field is boxed. Note: this changes the glyph asserted by any render snapshot; update tests
  accordingly.

## Decision 4: Focus indication between the two Replace fields

- **Decision**: The caret is rendered only in the focused field's box (unfocused box shows its text
  with no caret), preserving today's behavior (`with_caret` already gates on focus). Optionally
  emphasize the focused box's border/title; keep it minimal.
- **Rationale**: Satisfies FR-005 (focused field visually distinguishable) without introducing a
  focus ring (out of scope, #38). Matches current semantics.
- **Alternatives considered**: A full focus ring spanning fields + buttons — explicitly deferred to
  issue #38.

## Decision 5: Layout sizing & small-terminal degradation

- **Decision**: Compute dialog height from the number of stacked elements: Find mode = 1 field box
  (label + 3 rows) + options row + count/hint; Replace mode = 2 field boxes + options + hint. Clamp
  the dialog width to `size.width` and height to `size.height` (as the current code already does with
  `.min(size.width)` / `.min(size.height)`), and lay out child rects with `saturating_*` so a short
  terminal clamps rather than panics.
- **Rationale**: Preserves the feature-015 small-terminal crash fix and satisfies FR-009 / SC-003.
- **Alternatives considered**: Fixed height. Rejected — would clip on small terminals or waste space.

## Decision 6: Testing approach

- **Decision**: Add render tests using `ratatui`'s `TestBackend` that render the Find and Replace
  overlays and assert: box-drawing characters present (`┌`, `└`, `│`), the field label text present,
  the caret glyph present in the focused field, and the option/hint text still present. Mirror the
  existing file-browser render tests (`render_browser` helper pattern in `file_browser.rs`). Keep the
  existing `dialog.rs` field-editing unit tests unchanged.
- **Rationale**: Constitution V requires an automated test for the visible behavior; render snapshot
  assertions catch box/caret regressions invisible in diffs.
- **Alternatives considered**: Manual-only verification. Rejected — violates the test gate.
