# Implementation Plan: Word-wise navigation, selection, and deletion

**Branch**: `032-word-wise-editing` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/032-word-wise-editing/spec.md`

## Summary

Add word-wise editing: Ctrl+Left/Right move by word, Ctrl+Shift+Left/Right select by word, and
Ctrl+Backspace/Delete delete by word. One new pure function `next_word_pos(dir)` computes the word-target
`(line, gcol)` using the existing `grapheme_class` (feature 030) so boundaries match double-click. Movement
reuses `move_cursor`/`move_cursor_selecting` paths; deletion reuses the selection + single-undo-step
`delete_selection`. New keybindings only; no new dependencies.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74)

**Primary Dependencies**: `ropey`, `unicode-segmentation`, `crossterm`. No new dependencies (Principle IV).

**Storage**: N/A.

**Testing**: `cargo test` — inline unit tests + integration tests under `tests/integration/`.

**Target Platform**: Linux TUI; headless VT100-compatible.

**Project Type**: Single-project Rust terminal application.

**Performance Goals**: Within `make perf-check` (a word step scans at most a line's graphemes).

**Constraints**: grapheme/word-boundary correctness (multibyte-safe); no panic at buffer ends; DOS-faithful;
existing keys/behaviors unchanged; read-only respected for deletion.

**Scale/Scope**: `src/input/keymap.rs` (6 actions + 6 bindings), `src/app.rs` (`next_word_pos`,
`move_word`, `move_word_selecting`, `delete_word`, action dispatch). Boundary logic reuses
`grapheme_class`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. DOS-Faithful UI**: PASS — word-wise editing keys are standard; no UI/render change.
- **II. UTF-8 First**: PASS — operates on grapheme clusters and the shared `grapheme_class`; no byte
  slicing of buffer text (deletion goes through the existing char-safe `delete_selection`).
- **III. Portable Build**: PASS.
- **IV. No New Crates**: PASS.
- **V. Test-Gated (NON-NEGOTIABLE)**: PASS — `next_word_pos` and each operation are unit-tested
  (incl. multibyte, line-crossing, buffer ends, read-only); integration tests drive the actions.
- **VI. Simplicity/YAGNI**: PASS — one boundary helper + thin wrappers over existing move/delete paths.
- **VII. Security Hardening**: PASS — editor input only; deletion respects read-only.

**Result**: No violations. Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/032-word-wise-editing/
├── plan.md · research.md · data-model.md · quickstart.md
├── contracts/behavior.md
└── tasks.md  (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── input/keymap.rs   # 6 new Action variants + action_from_str arms + default_map bindings
│                     #   (Ctrl+Left/Right, Ctrl+Shift+Left/Right, Ctrl+Backspace, Ctrl+Delete)
└── app.rs            # next_word_pos(dir) (word target via grapheme_class); move_word / move_word_selecting
                      #   (reuse clear-selection + set_cursor_lc + update_selection_to_cursor); delete_word
                      #   (select range → delete_selection = one undo step; read-only guard); dispatch arms
tests/
└── integration/
    └── word_editing.rs   # end-to-end action-driven tests (registered in Cargo.toml)
```

**Structure Decision**: Single-project Rust; no new modules. The only new logic is `next_word_pos` and
three thin operation wrappers, all in `src/app.rs`, plus keymap entries.

## Word-boundary algorithm (shared with feature 030 `grapheme_class`)

Classes: word (alphanumeric or `_`), whitespace, other. A "word start" step:

- **Right**: from the cursor, consume the run of the class at the cursor, then consume any following
  whitespace run; stop (start of the next token). At end of line → `(line+1, 0)`; at buffer end → no-op.
- **Left**: step back one; consume a preceding whitespace run, then consume the preceding token run; stop
  (start of that token). At column 0 → end of previous line; at buffer start → no-op.

## Phasing & independence

US1 (move) is the MVP and provides `next_word_pos`. US2 (select) and US3 (delete) reuse it and the
existing selection/delete paths; each is independently testable.

## Complexity Tracking

> No Constitution violations — section intentionally empty.
