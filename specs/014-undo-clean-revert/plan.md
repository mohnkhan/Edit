# Implementation Plan: Undo-to-clean state and Revert to saved

**Branch**: `014-undo-clean-revert` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/014-undo-clean-revert/spec.md`

## Summary

Make the `[Modified]` indicator track *content vs. the saved baseline* instead of "any edit happened",
and add a menu-only **File ▸ Revert**. The mechanism is a **saved-point marker in the undo history**:
`UndoStack` gains `saved: Option<usize>` recording the cursor position at the last save/open. The buffer
is clean exactly when the undo cursor sits on that marker (`is_at_saved()`). The marker is invalidated
when a divergent edit discards the redo branch that contained it, which prevents a false-clean after
"undo then retype". `Buffer::open` and a successful save call `mark_saved()`; undo/redo derive
`modified = !is_at_saved()`. Revert reuses the existing feature-007 `reload_from_disk` path, guarded by
a confirmation when the buffer has unsaved changes and a no-op for path-less buffers.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74).

**Primary Dependencies**: existing `src/buffer/undo.rs` (UndoStack), `src/buffer/mod.rs` (Buffer,
open/save), `src/app.rs` (edit/undo/redo handlers, modal dialogs, `reload_from_disk`), `src/ui/menubar.rs`
(File menu), `src/input/keymap.rs` (Action). No new crates.

**Storage**: N/A (no persistence change). Operates on in-memory undo history + on-disk reload.

**Testing**: `cargo test` unit (`src/buffer/undo.rs`, `src/app.rs`) + integration
(`tests/integration/`). TDD per Constitution Principle V.

**Target Platform**: Linux + portable per constitution; terminal TUI.

**Performance Goals**: O(1) clean/dirty check (integer compare). No content hashing or rope diff.

**Constraints**: No false-clean ever (FR-004, data-loss safety). No regression to undo/redo/save/autosave
(FR-011). Revert reload goes through the existing encoding/line-ending-aware open path.

**Scale/Scope**: Touch points — `src/buffer/undo.rs` (saved marker), `src/buffer/mod.rs`
(`mark_saved` on open; helper), `src/app.rs` (mark_saved on save; derive modified on undo/redo; Revert
handler + confirm modal), `src/ui/menubar.rs` (File ▸ Revert item), `src/input/keymap.rs`
(`Action::Revert`). No keybinding (menu-only).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Assessment |
|---|---|
| **I. DOS-Faithful UI** | ✅ Undo-to-clean and a Revert command are standard DOS EDIT / editor behavior. Confirmation reuses the existing modal style. |
| **II. UTF-8 First** | ✅ Revert reload reuses `Buffer::open`, which already transcodes/validates; clean-tracking is pure integer state, no byte handling. |
| **III. Portable Build** | ✅ Pure Rust, no new deps, no platform code. |
| **IV. Minimal Footprint** | ✅ One `Option<usize>` field + small methods. No deps. |
| **V. Test-Gated (NON-NEGOTIABLE)** | ✅ TDD: undo-stack saved-marker unit tests (incl. divergent-edit no-false-clean), app-level modified-flag tests, Revert integration tests. |
| **VI. Simplicity / YAGNI** | ✅ Marker-in-history is simpler and more correct than snapshotting; Revert reuses `reload_from_disk`. No speculative config. |
| **VII. Security Hardening** | ✅ No new input/attack surface. Revert reads only the buffer's own path through the existing sanitized open path; destructive revert is gated by confirmation. |

**Gate result: PASS.**

## Project Structure

```text
specs/014-undo-clean-revert/
├── plan.md  research.md  data-model.md  quickstart.md
├── contracts/clean-state-and-revert.md
├── checklists/requirements.md
├── spec.md  tasks.md

src/
├── buffer/undo.rs   # UndoStack.saved: Option<usize>; mark_saved/is_at_saved; invalidate in push()
├── buffer/mod.rs    # Buffer::open + new_empty mark_saved; refresh_modified() helper
├── app.rs           # mark_saved after save; derive modified on undo/redo; Revert handler + confirm modal
├── input/keymap.rs  # Action::Revert (no key binding)
└── ui/menubar.rs    # File ▸ Revert item (mnemonic 'r')

tests/
└── integration/undo_clean_revert.rs   # end-to-end modified-flag + Revert
```

## Phase 0: Research

See [research.md](./research.md). Resolved: saved-marker invalidation rule (no false-clean); deriving
`modified` from history vs. per-edit flag; reusing `reload_from_disk` for Revert; confirmation-modal
approach for destructive revert.

## Phase 1: Design & Contracts

- **Data model**: [data-model.md](./data-model.md) — `UndoStack.saved` semantics, invalidation, and the
  `modified = !is_at_saved()` derivation.
- **Contract**: [contracts/clean-state-and-revert.md](./contracts/clean-state-and-revert.md) — clean
  state rules, Revert behavior and confirmation, error/no-op cases.
- **Quickstart**: [quickstart.md](./quickstart.md).
- **Agent context**: point the `<!-- SPECKIT START -->` block in `CLAUDE.md` at this plan.

## Complexity Tracking

No constitution violations; no entries required.
