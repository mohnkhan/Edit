# Implementation Plan: Syntax highlighting for Rust, JSON, and TOML

**Branch**: `026-highlight-rust-json-toml` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/026-highlight-rust-json-toml/spec.md`

## Summary

Add three built-in highlighters following the established pattern (`src/highlight/languages/c.rs` is the
template): lazy-compiled `regex` patterns → collect `(start, end, style)` candidates by priority → sort →
emit non-overlapping, sorted `Span`s using `CLASSIC.highlight_*` colors. Each is a new module
implementing the `Highlighter` trait; register `.rs`/`.json`/`.toml` in `detect_highlighter`. Plugin
precedence is untouched (the plugin registry is consulted before the built-in fallback), so a plugin
highlighter for these extensions still wins.

Style mapping (reusing existing theme classes): Rust → keyword/type/string/number/comment (+ attributes
and macros mapped to keyword/operator); JSON → key→type, string→string, number→number, true/false/null→
keyword, punctuation→operator; TOML → header→type, key→keyword, string→string, number/date→number,
bool→keyword, `#` comment→comment.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74).

**Primary Dependencies**: existing `regex`, `ratatui::style`, `src/highlight` (`Highlighter`, `Span`,
`detect_highlighter`), `src/ui/theme::CLASSIC` (highlight_* colors). No new crates.

**Storage**: N/A.

**Testing**: `cargo test` — unit tests per highlighter (representative lines: keywords/types/strings/
numbers/comments/headers/keys; spans non-overlapping + sorted; no panic on malformed/UTF-8/empty/long
lines) + a `detect_highlighter` test that `.rs`/`.json`/`.toml` resolve. TDD per Constitution V.

**Target Platform**: Linux + portable; terminal TUI.

**Project Type**: Single-project Rust desktop TUI application.

**Performance Goals**: per-line regex highlighting, same cost class as the existing 5 highlighters.

**Constraints**: spans non-overlapping + sorted + byte-correct (renderer contract); no panic on any input;
line-based best-effort (no cross-line state); reuse trait/Span/theme; plugin precedence preserved; no
change to the rendering pipeline/buffer/other features.

**Scale/Scope**: 3 new modules + 3 `detect_highlighter` arms + tests.

## Constitution Check

| Principle | Assessment |
|---|---|
| **I. DOS-Faithful UI** | ✅ Highlighting is a baseline UI feature; this extends it within the existing model. |
| **II. UTF-8 First** | ✅ Spans use byte offsets from `regex` over the UTF-8 line; tests cover multi-byte content. |
| **III. Portable Build** | ✅ Pure Rust, existing deps only. |
| **IV. Minimal Footprint** | ✅ No new crates; reuses `regex` + the highlight subsystem. |
| **V. Test-Gated (NON-NEGOTIABLE)** | ✅ TDD: per-language unit tests + detect test before implementation. |
| **VI. Simplicity / YAGNI** | ✅ Three more languages, justified by this spec (Principle VI gate satisfied); same line-based approach, no regex engine swap. |
| **VII. Security Hardening** | ✅ No new input/attack surface; pure styling of in-buffer text. |

**Result**: PASS.

## Project Structure

```text
src/highlight/languages/rust.rs   # NEW — RustHighlighter
src/highlight/languages/json.rs   # NEW — JsonHighlighter
src/highlight/languages/toml.rs   # NEW — TomlHighlighter
src/highlight/languages/mod.rs    # add `pub mod rust; pub mod json; pub mod toml;`
src/highlight/mod.rs              # add `.rs`/`.json`/`.toml` arms to detect_highlighter (+ a detect test)
```

**Structure Decision**: Single-project; each language mirrors `c.rs` exactly (lazy regexes + candidate
collection + non-overlap resolution + inline `#[cfg(test)]`), so the new code is uniform and low-risk.
Plugin precedence and the render pipeline are unchanged.

## Phase 0 — Research

See [research.md](./research.md). Decisions: clone the `c.rs` highlighter pattern; per-language regex sets
+ priority order; map tokens to existing theme classes; register by extension; keep line-based best-effort
behavior; verify the existing plugin-precedence path needs no change.

## Phase 1 — Design & Contracts

- [data-model.md](./data-model.md) — the per-language token→style mapping and span rules.
- [contracts/highlighters.md](./contracts/highlighters.md) — per-language coverage + the span/no-panic
  contract the tests assert.
- [quickstart.md](./quickstart.md) — build/test + manual walkthrough on real files.

Agent context updated to point at this plan.

## Complexity Tracking

No constitution violations — table omitted.
