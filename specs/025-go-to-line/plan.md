# Implementation Plan: Go to Line

**Branch**: `025-go-to-line` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/025-go-to-line/spec.md`

## Summary

Add a Go-to-Line modal: a new `Action::GoToLine` bound to `Ctrl+G` and added to the **Search** menu opens
a small overlay that collects a 1-based line number; Enter clamps it to `[1, line_count]` and calls the
existing `set_cursor_lc(line-1, 0)` (which already scrolls the cursor into view via `clamp_scroll`); Esc
cancels; invalid/empty is a no-op. The prompt is a modal mirroring the encoding-dialog input pattern.

Pieces: (1) `Action::GoToLine` + `Ctrl+G` binding + `action_from_string` mapping + a `SEARCH_MENU` item;
(2) App state `pending_goto_line: Option<String>` + open/intercept in `handle_action` + jump on Enter;
(3) a render overlay in `src/ui/mod.rs`; (4) treat it as a modal in the mouse/wheel/scrollbar guards.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74).

**Primary Dependencies**: `src/input/keymap.rs` (Action enum, default map, `action_from_string`),
`src/ui/menubar.rs` (`SEARCH_MENU`), `src/app.rs` (`handle_action`, `set_cursor_lc`/`clamp_scroll`,
modal state + guards, `handle_mouse_event` modal-ignore), `src/ui/mod.rs` (overlay render). No new crates.

**Storage**: N/A.

**Testing**: `cargo test` — unit (clamp of a parsed number to `[1, line_count]`; digit-only field edit),
integration (Ctrl+G → type → Enter moves cursor to the line start and scrolls into view; over-range
clamps to last; `0`/below clamps to first; Esc no-op; empty/non-numeric no-op; prompt captures input so
the buffer isn't edited). TDD per Constitution V.

**Target Platform**: Linux + portable; terminal TUI.

**Project Type**: Single-project Rust desktop TUI application.

**Performance Goals**: O(1) command; the jump reuses existing cursor/scroll math.

**Constraints**: 1-based UI; land at column 1; modal captures input (buffer untouched); one modal at a
time; no panic on empty buffer / oversized input / tiny terminal; no change to editing/find-replace.

**Scale/Scope**: one action + one small modal + one jump path.

## Constitution Check

| Principle | Assessment |
|---|---|
| **I. DOS-Faithful UI** | ✅ Go to Line under the Search menu + a key shortcut is the EDIT.COM idiom. |
| **II. UTF-8 First** | ✅ Numeric field is ASCII digits; cursor set via grapheme-aware `set_cursor_lc`. |
| **III. Portable Build** | ✅ Pure Rust, no new deps. |
| **IV. Minimal Footprint** | ✅ One action + a small modal; reuses cursor/scroll machinery. |
| **V. Test-Gated (NON-NEGOTIABLE)** | ✅ TDD: clamp unit + jump/no-op integration before implementation. |
| **VI. Simplicity / YAGNI** | ✅ Reuse the encoding-dialog input pattern; no new scroll/config. |
| **VII. Security Hardening** | ✅ No new input/attack surface; navigation only. |

**Result**: PASS.

## Project Structure

```text
src/input/keymap.rs   # Action::GoToLine; bind "Ctrl+G"; action_from_string ("GoToLine"); test update
src/ui/menubar.rs     # add a "Go to Line" item to SEARCH_MENU (mnemonic 'g')
src/app.rs            # pending_goto_line: Option<String>; open on Action::GoToLine (if no other modal);
                      #   intercept block (digit/Backspace/Enter/Esc); on Enter parse→clamp→
                      #   set_cursor_lc(line-1, 0); add to modal guards in handle_mouse_event / wheel /
                      #   scrollbar_regions so the editor doesn't act under the prompt.
src/ui/mod.rs         # render a small centered "Go to line: <n>▏" modal overlay.
tests/integration/go_to_line.rs  # NEW
```

**Structure Decision**: Single-project; the modal mirrors the existing encoding-select dialog (a simple
text-entry overlay with its own `handle_action` intercept), so input routing and rendering follow an
established pattern.

## Phase 0 — Research

See [research.md](./research.md). Key decisions: reuse `set_cursor_lc`+`clamp_scroll` (already scrolls
into view); model the prompt as `Option<String>` like `pending_encoding_select`; digit-only field;
clamp to `[1, line_count]`; Search menu + `Ctrl+G`; treat as a modal in the mouse/wheel/scrollbar guards.

## Phase 1 — Design & Contracts

- [data-model.md](./data-model.md) — the prompt state + the parse/clamp/jump mapping.
- [contracts/go-to-line.md](./contracts/go-to-line.md) — open/typing/confirm/cancel/clamp behavior + the
  no-regression contract.
- [quickstart.md](./quickstart.md) — build/test + manual walkthrough.

Agent context updated to point at this plan.

## Complexity Tracking

No constitution violations — table omitted.
