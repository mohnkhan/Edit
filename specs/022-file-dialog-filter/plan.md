# Implementation Plan: File dialog — glob filtering + richer entry details

**Branch**: `022-file-dialog-filter` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/022-file-dialog-filter/spec.md`

## Summary

Two changes to the file browser (`src/ui/file_browser.rs`), both UI/local:

1. **Live filtering** — the field text now filters the listing live: a glob (`*`,`?`) matches entry
   names; plain text matches by case-insensitive substring; an absolute path (`/…`) keeps its existing
   jump behavior; clearing restores the full list. Directories and `..` always stay so navigation is
   never blocked.
2. **Detail columns** — each row shows a human-readable size (files) or a `<DIR>` indicator (dirs/`..`)
   plus a modified date, right-aligned, with the name truncated (width-correct) when space is tight.

**Approach (least-invasive).** Keep the displayed `entries: Vec<Entry>` as the *filtered* view (so all
existing render/nav/hit-test/activate code is unchanged) and add an `all_entries: Vec<Entry>` source of
truth. `reload()` builds `all_entries` (now reading size + mtime from `std::fs::metadata`) then calls a
new `apply_filter()` that derives `entries`. Field edits (`push_char`/`backspace`/clear) call
`apply_filter()` and re-clamp `selected`/`scroll`. No new crates — a tiny in-house glob matcher,
byte→human size formatter, and epoch→`YYYY-MM-DD HH:MM` (UTC) formatter (Constitution IV).

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74).

**Primary Dependencies**: std only (`std::fs::metadata`, `SystemTime`); existing
`src/ui/file_browser.rs` (`Entry`, `reload`, `compute_layout`, widget, `hit_test`, `activate*`),
`unicode-segmentation`/`unicode-width` (already used) for width-correct columns. No new crates.

**Storage**: N/A (reads filesystem metadata only).

**Testing**: `cargo test` — unit (glob matcher, size/date formatters, `apply_filter` keeps dirs/`..`,
selection re-clamp, detail-column truncation), integration (type `*.log`/substring filters the listing;
absolute path still jumps; Save-mode confirm unchanged; buttons/scrollbar still work). TDD per
Constitution V.

**Target Platform**: Linux + portable; terminal TUI.

**Project Type**: Single-project Rust desktop TUI application.

**Performance Goals**: filtering is O(entries) per keystroke over one directory listing — negligible;
metadata is read once per `reload`, not per frame.

**Constraints**: never hide directories/`..`; width/UTF-8-correct columns + truncation; no panic on
unreadable metadata, empty dirs, no-match filters, tiny terminals; preserve feature-012 navigation,
feature-020 buttons/focus ring, feature-021 scrollbar; security — path handling still goes through the
existing `validate_path` on open/save (Constitution VII; no `../` escape change).

**Scale/Scope**: one module + its app wiring; ~3 small std-only helpers.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Assessment |
|---|---|
| **I. DOS-Faithful UI** | ✅ A filterable file list with size/date columns is standard DOS/file-picker UX; navigation keys unchanged. |
| **II. UTF-8 First (NON-NEGOTIABLE)** | ✅ Column layout/truncation use grapheme/display-width; names from `to_string_lossy` already valid UTF-8. |
| **III. Portable Build** | ✅ std-only; `SystemTime`/`metadata` are cross-platform; no platform code. |
| **IV. Minimal Footprint** | ✅ No new crates — in-house glob/size/date helpers. |
| **V. Test-Gated (NON-NEGOTIABLE)** | ✅ TDD: matcher/formatter units + filter/detail integration before implementation. |
| **VI. Simplicity / YAGNI** | ✅ Reuse the existing entry/list pipeline; add `all_entries` + `apply_filter`; no regex engine. |
| **VII. Security Hardening** | ✅ Open/Save still validate via `validate_path`; filtering is display-only; no new path-traversal surface. |

**Result**: PASS (no violations; Complexity Tracking not required).

## Project Structure

### Documentation (this feature)

```text
specs/022-file-dialog-filter/
├── plan.md, research.md, data-model.md, quickstart.md
├── contracts/file-dialog.md
├── checklists/requirements.md
└── tasks.md   # /speckit-tasks output
```

### Source Code (repository root)

```text
src/ui/file_browser.rs   # Entry gains size/mtime; FileBrowser gains all_entries + apply_filter();
                         #   reload() reads metadata + builds all_entries then filters; push_char/
                         #   backspace re-filter; widget renders name + size + date columns (name
                         #   truncated), preserving the feature-021 scrollbar column; small std-only
                         #   helpers: glob_match (case-insensitive), human_size, format_mtime (UTC)
src/app.rs               # file-browser key intercept already routes typing/backspace through the
                         #   browser; no behavioral change beyond re-filter side-effects
tests/integration/file_dialog_filter.rs  # NEW — filter + jump + Save-mode + no-regression
```

**Structure Decision**: Single-project layout; nearly all change is contained in
`src/ui/file_browser.rs` (model + widget + helpers), with one new integration test file. `Entry` is only
constructed in `reload()` (3 literals), so adding fields is low-impact.

## Phase 0 — Research

See [research.md](./research.md). Key decisions: in-house wildcard matcher (no regex/glob crate);
filtered-view-as-`entries` with `all_entries` source; std-only size/date formatting (UTC); absolute-path
text bypasses filtering; directories/`..` always retained.

## Phase 1 — Design & Contracts

- [data-model.md](./data-model.md) — `Entry` (+ size/mtime), `FileBrowser` (+ all_entries), the
  filter-interpretation rules, and the detail-column layout.
- [contracts/file-dialog.md](./contracts/file-dialog.md) — filter behavior table, detail-rendering rules,
  and the no-regression contract the tests assert.
- [quickstart.md](./quickstart.md) — build/test + manual walkthrough.

Agent context (`CLAUDE.md`/`CLAUDE.MD` SPECKIT markers) updated to point at this plan.

## Complexity Tracking

No constitution violations — table intentionally omitted.
