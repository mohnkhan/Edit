# Implementation Plan: Interactive Find and Replace dialogs

**Branch**: `015-find-replace-dialog` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/015-find-replace-dialog/spec.md`

## Summary

Turn the stubbed Find/Replace menu items into working interactive dialogs. The search engine
(`SearchEngine::find_all`, plain + regex), `SearchState`, match navigation (`find_next`/`find_prev`/
`scroll_to_match`), `replace_all`, and highlight styles (`collect_match_spans`) already exist. This
feature adds: (1) **dialog state** on `App` (`pending_find` / `pending_replace` with editable field
text, focus, cursor, and option toggles), (2) **input routing** so keystrokes edit the dialog fields
while open, (3) **wiring** so `Ctrl+F`/Find and `Ctrl+H`/Find Replace open the dialogs and Enter runs
the search / replace, (4) **match-highlight rendering** in the editor for the active result set, and
(5) **whole-word** matching added to the engine (the one option not yet supported). UTF-8-correct field
editing reuses the grapheme-aware patterns already used by the file-browser filename field.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74).

**Primary Dependencies**: existing `src/search/` (engine, state, highlight), `src/app.rs` (find/replace
handlers, modal dialog patterns), `src/ui/dialog.rs` (FindDialog/ReplaceDialog skeletons),
`src/ui/editor.rs` (render), `src/ui/mod.rs` (overlay dispatch), `src/input/keymap.rs`. `regex` crate
already a dependency (engine). No new crates.

**Storage**: N/A.

**Testing**: `cargo test` — unit (`src/search/` incl. new whole-word; dialog field editing) +
integration (`tests/integration/find_replace.rs`). TDD per Constitution Principle V.

**Target Platform**: Linux + portable; terminal TUI.

**Performance Goals**: search/highlight over typical files is sub-frame; `find_all` is already linear.
Highlight rendering adds an O(matches-on-visible-lines) overlay per frame.

**Constraints**: UTF-8/grapheme-correct field input and match offsets (FR-011). Match offsets recomputed
after edits/replace so highlights never apply to stale positions (FR-012). No regression outside an open
dialog (FR-013). Ctrl+A means Replace-All only while the Replace dialog is open.

**Scale/Scope**: Touch points — `src/search/mod.rs` (whole-word), `src/app.rs` (dialog state + intercepts
+ wiring find/replace/replace-all + recompute), `src/ui/dialog.rs` (interactive render w/ caret + toggle
states), `src/ui/editor.rs` + `src/ui/mod.rs` (match-highlight overlay), `src/input/keymap.rs` (ensure
F2/F3/Ctrl+F/Ctrl+H present — already mapped).

## Constitution Check

| Principle | Assessment |
|---|---|
| **I. DOS-Faithful UI** | ✅ Find/Replace dialogs with highlighted matches are core EDIT.COM features. Esc cancels (consistent). |
| **II. UTF-8 First** | ✅ Field editing is grapheme-aware; match offsets are char indices via the existing UTF-8-safe engine; highlight spans align to grapheme cells (FR-011). |
| **III. Portable Build** | ✅ Pure Rust; `regex` already used; no new deps/platform code. |
| **IV. Minimal Footprint** | ✅ No new dependencies. |
| **V. Test-Gated (NON-NEGOTIABLE)** | ✅ TDD: engine whole-word units, dialog field-edit units, integration for find/next/prev/replace/replace-all/options. |
| **VI. Simplicity / YAGNI** | ✅ Reuses the entire existing search core; adds only UI/state/wiring + one engine option. |
| **VII. Security Hardening** | ✅ No external input/attack surface; regex already bounded by the engine's invalid-pattern handling. |

**Gate result: PASS.**

## Project Structure

```text
specs/015-find-replace-dialog/
├── plan.md  research.md  data-model.md  quickstart.md
├── contracts/find-replace-interaction.md
├── checklists/requirements.md  checklists/quality.md
├── spec.md  tasks.md

src/
├── search/mod.rs     # whole_word option + word-boundary filtering in find_all
├── app.rs            # pending_find/pending_replace state; dialog intercepts; open/run wiring;
│                     #   recompute-after-edit; replace current/all; highlight feed to renderer
├── ui/dialog.rs      # interactive FindDialog/ReplaceDialog (caret, focus, toggle indicators)
├── ui/editor.rs      # match-highlight overlay during render
├── ui/mod.rs         # overlay dispatch for the dialogs + pass match spans to EditorWidget
└── input/keymap.rs   # confirm F2/F3/Ctrl+F/Ctrl+H mapping (already present)

tests/
└── integration/find_replace.rs   # end-to-end find / next / prev / replace / replace-all / options
```

## Phase 0: Research

See [research.md](./research.md). Resolved: dialog state shape + focus model; input routing/modal
precedence; whole-word implementation in the engine; match-highlight rendering integration; recompute
strategy after edits; in-dialog toggle keys.

## Phase 1: Design & Contracts

- **Data model**: [data-model.md](./data-model.md) — `FindReplaceDialog` state, `SearchState.whole_word`,
  recompute rules.
- **Contract**: [contracts/find-replace-interaction.md](./contracts/find-replace-interaction.md) — keys,
  modality, highlight, replace semantics.
- **Quickstart**: [quickstart.md](./quickstart.md).
- **Agent context**: point the `<!-- SPECKIT START -->` block in `CLAUDE.md` at this plan.

## Complexity Tracking

No constitution violations; no entries required.
