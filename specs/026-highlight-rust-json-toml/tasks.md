---
description: "Task list for feature 026 — syntax highlighting for Rust, JSON, TOML"
---

# Tasks: Syntax highlighting for Rust, JSON, and TOML

**Input**: Design documents from `specs/026-highlight-rust-json-toml/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/highlighters.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V (Test-Gated Merges, NON-NEGOTIABLE). Tests first.

**Organization**: Setup → US1 (Rust) → US2 (JSON) → US3 (TOML) → Polish. Each language is an independent
module + a `detect_highlighter` arm and can ship on its own (the MVP is Rust).

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files / independent)
- **[Story]**: US1 (Rust) / US2 (JSON) / US3 (TOML)

## Path Conventions

Single-project Rust: `src/highlight/languages/*.rs`, `src/highlight/languages/mod.rs`,
`src/highlight/mod.rs`. Unit tests inline per module.

---

## Phase 1: Setup

- [x] T001 Confirm a clean baseline build on branch `026-highlight-rust-json-toml` (`make tmpfs-setup` then `make`).
- [x] T002 Re-read `src/highlight/languages/c.rs` (the highlighter template: lazy `OnceLock<Regex>` + candidate/sort/non-overlap loop), the `Highlighter`/`Span` types, `detect_highlighter`, and the `CLASSIC.highlight_*` theme classes. Confirm the plugin-precedence path consults plugins before the built-in fallback (no change needed). No code change.

---

## Phase 2: User Story 1 — Rust (Priority: P1) 🎯 MVP

**Goal**: `.rs` files highlight keywords, types, strings/char/byte/raw, numbers, `//` + single-line
`/* */` comments, `#[…]` attributes, and `name!` macros.

**Independent Test**: `fn main() { let x: u32 = 1; // c }` → `fn`/`let` keyword, `u32` type, `1` number,
`// c` comment; spans sorted + non-overlapping.

### Tests for US1 (write first, must fail)

- [x] T003 [P] [US1] Inline unit tests in `src/highlight/languages/rust.rs`: keywords/types/strings/numbers/line+block comments/attributes/macros styled on representative lines; spans sorted + non-overlapping; no panic on an unterminated string, an empty line, a multi-byte string, and a very long line.

### Implementation for US1

- [x] T004 [US1] Create `src/highlight/languages/rust.rs` with `RustHighlighter` (lazy regexes per token class; candidate/sort/non-overlap loop from `c.rs`; map to keyword/type/string/number/comment/operator per `data-model.md`); `name() == "Rust"`.
- [x] T005 [US1] Register the module (`pub mod rust;` in `src/highlight/languages/mod.rs`) and add `Some("rs") => RustHighlighter` to `detect_highlighter` in `src/highlight/mod.rs`.

**Checkpoint**: `.rs` highlights end-to-end; `make check` green. MVP demoable.

---

## Phase 3: User Story 2 — JSON (Priority: P1)

**Goal**: `.json` files highlight keys (string-before-`:`), string values, numbers, `true`/`false`/`null`,
and punctuation.

**Independent Test**: `{ "n": 42, "ok": true }` → `"n"` key (type class), `42` number, `true` keyword.

### Tests for US2 (write first, must fail)

- [x] T006 [P] [US2] Inline unit tests in `src/highlight/languages/json.rs`: a key (string before `:`) is styled differently from a value string; numbers and `true`/`false`/`null` styled; spans sorted + non-overlapping; no panic on malformed/empty/multi-byte/long lines.

### Implementation for US2

- [x] T007 [US2] Create `src/highlight/languages/json.rs` with `JsonHighlighter` (key vs value-string via a `"…"\s*:` lookahead-equivalent; numbers; literals; punctuation→operator); `name() == "JSON"`.
- [x] T008 [US2] Register the module and add `Some("json") => JsonHighlighter` to `detect_highlighter`.

**Checkpoint**: `.json` highlights; `make check` green.

---

## Phase 4: User Story 3 — TOML (Priority: P2)

**Goal**: `.toml` files highlight `[..]`/`[[..]]` headers, bare keys, strings, numbers/dates, booleans,
and `#` comments.

**Independent Test**: `name = "edit" # x` → `name` key, `"edit"` string, `# x` comment; `[package]` header.

### Tests for US3 (write first, must fail)

- [x] T009 [P] [US3] Inline unit tests in `src/highlight/languages/toml.rs`: table/array-of-table headers, bare keys, strings, numbers, dates, booleans, and `#` comments styled; spans sorted + non-overlapping; no panic on malformed/empty/multi-byte/long lines.

### Implementation for US3

- [x] T010 [US3] Create `src/highlight/languages/toml.rs` with `TomlHighlighter` (header regex; `#` comment priority; bare-key-before-`=`; strings; date + number; booleans); `name() == "TOML"`.
- [x] T011 [US3] Register the module and add `Some("toml") => TomlHighlighter` to `detect_highlighter`.

**Checkpoint**: `.toml` (incl. `Cargo.toml`) highlights; `make check` green.

---

## Phase 5: Polish & Cross-Cutting

- [x] T012 [P] Add a `detect_highlighter` test in `src/highlight/mod.rs`: `a.rs`→"Rust", `a.json`→"JSON", `a.toml`→"TOML", unknown→`None`; and a smoke assertion that highlighting every line of a real sample of each language (e.g. snippets) yields sorted, non-overlapping spans with no panic. Plugin precedence (FR-006/SC-004/L1): confirm by inspection that the highlighter-selection path consults the plugin registry before the built-in `detect_highlighter` fallback (the existing feature-008 `plugin_api` tests already guard this); no path change is made here, so no new plugin-host test is added.
- [x] T013 [P] Update `CHANGELOG.md` (feature 026 under `[Unreleased]`), `docs/STATUS.md`, and `docs/CAPABILITIES.md` (Rust/JSON/TOML are now highlighted; ≥ 8 languages). Note in `ROADMAP.md` that the constitution's "languages beyond baseline 5" deferral is satisfied by this spec.
- [x] T014 Run `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check) and fix findings.
- [x] T015 Run the `specs/026-highlight-rust-json-toml/quickstart.md` manual walkthrough (open `src/app.rs`, `Cargo.toml`, a `.json`; confirm the existing 5 still highlight).

---

## Dependencies & Execution Order

- **Setup (P1)** → none.
- **US1/US2/US3** each depend only on Setup; they touch separate new files + independent `detect_highlighter`
  arms, so they are mutually independent and `[P]`-friendly. Recommended order: Rust (MVP) → JSON → TOML.
- **Polish (P5)** → after the languages intended for the release.

### Within each language

Write the inline unit test first (must fail) → implement the module → register → green.

### Parallel opportunities

- T003/T006/T009 (per-language tests) and T004/T007/T010 (per-language modules) are `[P]` across languages.
- T012/T013 polish are `[P]`.

---

## Implementation Strategy

### MVP

Setup → US1 Rust (T003–T005): the editor's own language highlights. Then JSON, then TOML.

### Notes

- TDD mandatory (Constitution V). No new crates — reuse `regex` + the highlight subsystem (Constitution IV).
- This spec is the Principle-VI gate for adding languages beyond the baseline 5.
- Keep AI attribution out of commits/PR/issues. Branch `026-highlight-rust-json-toml`, PR to `master`,
  merge via GitHub.
