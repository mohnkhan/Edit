# Implementation Plan: Centralize Editor UI State

**Branch**: `039-centralize-ui-state` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/039-centralize-ui-state/spec.md`

## Summary

Behavior-preserving refactor of the editor's central state object (`src/app.rs`). Three of the
codebase's recurring bug classes trace to one root: "which overlay is open" is spread across ~14
independent `Option`/`bool` fields, and three orderings (key dispatch, mouse dispatch, paint) that
must agree are maintained by hand. The fix: (1) collapse the flags into one `Modal` enum so two
overlays open at once is unrepresentable and all three orderings derive from one `match`; (2) declare
layer stacking precedence once and consume it from both paint (bottom-up) and hit-test (top-down),
deleting the tab-bar/dropdown special-case; (3) route the last two ad-hoc geometry computations
(Go-to-Line, Find/Replace fields) through shared rect helpers and standardize active-buffer access.
Correctness is proven by the existing test suite (87 inline + 33 integration + 9 `.exp` smoke) passing
unchanged plus a new generic layer-dispatch invariant test. Delivered as one branch/PR.

## Technical Context

**Language/Version**: Rust, edition 2021, MSRV 1.74

**Primary Dependencies**: `ratatui` + `crossterm` (TUI render/input), `ropey` (text buffer). No new
dependencies introduced by this feature.

**Storage**: N/A (in-memory editor state; files via existing I/O paths, unchanged)

**Testing**: `cargo test` (unit + inline + integration); `expect`/tmux headless smoke (`make smoke`);
`criterion` perf (`make perf-check`)

**Target Platform**: Linux x86_64 / aarch64 terminal (VT100+); behavior identical across supported
terminals

**Project Type**: Single-project desktop TUI application

**Performance Goals**: No regression. Cold start ≤ 2 s, 100 MB file open ≤ 3 s, keystroke latency
≤ 50 ms (constitution baselines). The refactor is representation-only; a `match` over a small enum is
no costlier than the current `if`-chain.

**Constraints**: Pure internal refactor — zero user-visible behavior change (FR-010). Existing test
assertions must not change except mechanical field→accessor renames (FR-009). UTF-8 hygiene unaffected
(no new byte-decoding paths).

**Scale/Scope**: One feature, primarily `src/app.rs` (~7k lines) + `src/ui/mod.rs`; mechanical
field→accessor churn across `tests/integration/*.rs` and a few `src/ui/*` readers.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. DOS-Faithful UI** — PASS. No UI change; identical menus, dialogs, F-keys, status bar (FR-010).
- **II. UTF-8 First** — PASS / N/A. No new code path widens raw bytes; encoding/transcode untouched.
- **III. Portable Build** — PASS. No platform-specific code added; pure Rust state refactor.
  (Note: the constitution text predates the move to `ratatui`/`crossterm` and still references
  ncurses; the live stack per `CLAUDE.md` is `ratatui`/`crossterm` and this plan follows `CLAUDE.md`.
  No new constraint introduced, so this is not a deviation requiring Complexity Tracking.)
- **IV. Minimal Footprint** — PASS. No new dependency; binary footprint unchanged.
- **V. Test-Gated Merges (NON-NEGOTIABLE)** — PASS. Entire existing suite must stay green (SC-001/002);
  one new invariant test added for the shared-precedence behavior (SC-005). TDD note: behavior is
  pre-specified by existing tests; the new test is written before the Phase-2 precedence change.
- **VI. Simplicity / YAGNI** — PASS, net simplification. Removes ~14 fields + a dead flag, collapses
  3 hand-synced orderings to 1. No speculative abstraction: layer precedence is a concrete enum/list,
  not a framework. Full retained-widget tree explicitly deferred.
- **VII. Security Hardening** — PASS / N/A. No change to file I/O, path handling, escape sanitization,
  or plugin sandbox. Plugin consent/manager overlays move into the `Modal` enum but their consent
  gating logic is preserved verbatim.

**Result**: All gates pass. No entries required in Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/039-centralize-ui-state/
├── plan.md              # This file
├── spec.md              # Feature spec
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (the Modal enum + Layer model)
├── quickstart.md        # Phase 1 output (validation guide)
├── contracts/
│   └── internal-api.md  # Phase 1 output (accessor surface replacing the flags)
├── checklists/
│   └── requirements.md  # Spec quality checklist (from /speckit-specify)
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── app.rs               # PRIMARY: add Modal enum + accessors; rewrite handle_action,
│                        #   handle_mouse_event, handle_mouse_click; add layer precedence;
│                        #   goto_line_rect helper; active_buffer()/_mut() standardization;
│                        #   delete dead menu_active. Inline tests updated to accessors.
├── ui/
│   ├── mod.rs           # Ui::render overlay cascade → match on app.modal()/layer iteration;
│   │                    #   route goto-line + find-replace field rects through one source.
│   ├── menubar.rs       # MenuState participates in layer precedence (read-mostly).
│   ├── tabbar.rs        # tab_hit_regions reused (read-mostly).
│   ├── contextmenu.rs   # ContextMenu becomes a Modal variant payload (read-mostly).
│   ├── dialog.rs        # find/replace rect helper shared with hit-test (read-mostly).
│   └── file_browser.rs  # FileBrowser becomes a Modal variant payload (read-mostly).
├── session/mod.rs       # SessionData read via Modal::SessionRestore (read-mostly).
├── watcher/mod.rs       # ExternalChange read via Modal::ExternalChange (read-mostly).
└── plugin/mod.rs        # PluginMeta consent/manager read via Modal variants (read-mostly).

tests/
└── integration/*.rs     # Mechanical: app.pending_X.is_some() → accessor / matches!(app.modal(), …)
```

**Structure Decision**: Single-project layout (unchanged). All work lands in `src/` with the bulk in
`src/app.rs` and `src/ui/mod.rs`. No files added or moved (splitting `app.rs` is out of scope and
recorded as a deferral).

## Phased Approach (delivery in one PR, ordered commits)

- **Phase 1 — Modal enum.** Introduce `enum Modal` + accessors; fold the ~14 flags and their
  sub-state; delete dead `menu_active`; rewrite the three orderings to `match self.modal`. Behavior
  identical; existing tests fenced.
- **Phase 2 — Layer precedence.** One ordered layer list; render iterates bottom-up, mouse top-down;
  remove `!dropdown_open` special-case; convert the two patched regressions into a generic invariant
  test.
- **Phase 3 — Geometry hygiene.** Shared `goto_line_rect()` and find-replace field rects for both
  paint and hit-test; standardize `active_buffer()`/`active_buffer_mut()` where the active buffer is
  meant.

## Complexity Tracking

*No constitution violations. Table intentionally empty.*
