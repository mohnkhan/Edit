# Implementation Plan: Editable-field affordance + Help redesign

**Branch**: `018-field-affordance-help` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

## Summary

Two UX fixes. (A) **File-dialog input box**: render the file browser's editable field as a bordered,
labeled box with an always-visible caret, and **show it in Open mode too** (the jump-path field was never
drawn). (B) **Help redesign**: replace the dense, fixed-width, truncating cheat-sheet with a grouped,
aligned two-column **Key | Action** table that fits/wraps and **scrolls** when taller than the screen.

## Technical Context

**Language**: Rust 2021, MSRV 1.74. **Deps**: ratatui (Block/Borders, Paragraph), existing
`src/ui/file_browser.rs` (layout + render), `src/ui/mod.rs` (`render_help_overlay`), `src/app.rs`
(`HelpScreen`, help intercept for scroll). No new crates. **Testing**: `cargo test` unit (file-browser
render shows a boxed field in both modes; help table builds + scrolls) + existing integration. TDD per
Principle V. **Constraints**: UTF-8/grapheme-correct field text + caret; no panic on small terminals;
no regression to browse/select or other dialogs.

## Constitution Check

| Principle | Assessment |
|---|---|
| I. DOS-Faithful UI | ✅ Boxed input fields + a readable key table are the DOS dialog/help idiom. |
| II. UTF-8 First | ✅ Field text/caret + table use grapheme/width-correct helpers. |
| III/IV Portable/Minimal | ✅ Pure Rust, no deps. |
| V. Test-Gated | ✅ Unit + integration, TDD. |
| VI. Simplicity | ✅ Reuse box-drawing + the existing field state; Help is a data-driven table. |
| VII. Security | ✅ No new input/attack surface. |

**Gate: PASS.** (Find/Replace box-styling deferred → follow-up issue + ROADMAP.)

## Project Structure

```text
specs/018-field-affordance-help/ → plan/research/data-model/quickstart + contracts + checklists
src/ui/file_browser.rs → compute_layout reserves a label+box region (both modes); render a bordered
                         input box with label + caret; Open mode shows the path field.
src/ui/mod.rs          → render_help_overlay rebuilt as a grouped Key|Action table with scroll offset.
src/app.rs             → HelpScreen help-scroll offset + Up/Down/PageUp/Down handling in the help
                         intercept; Esc still closes.
tests/integration/...  → help_overlay scroll/build; file-browser field-render unit tests.
```

## Phases

- Phase 0 [research.md]: bordered single-line field geometry + caret; Open-mode field placement vs the
  list; Help table model (sections of (key,action)), column widths, scroll model + "more" cue.
- Phase 1 [data-model.md / contracts/field-and-help.md / quickstart.md]: input-box render contract,
  Help table/scroll contract.
- Agent context: point `CLAUDE.md` SPECKIT block at this plan.

## Complexity Tracking

Deferred: Find/Replace bordered-box field styling → follow-up issue + ROADMAP row.
