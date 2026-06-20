# Phase 0 Research: Rust / JSON / TOML highlighters

## Existing machinery (from code survey)

- `Highlighter` trait (`src/highlight/mod.rs`): `highlight(&self, line: &str) -> Vec<Span>` + `name()`.
- `Span { start, end, style }` — byte offsets, `ratatui::style::Style`.
- `detect_highlighter(path)` matches by extension → `Box<dyn Highlighter>` (built-in fallback).
- Template `src/highlight/languages/c.rs`: lazy `OnceLock<Regex>` patterns; in `highlight()` collect
  `(start, end, style)` candidates in priority order (comments → strings → numbers → keywords), sort by
  `(start, longest)`, then keep a candidate only if `start >= last_end` (non-overlap), producing sorted
  non-overlapping spans.
- Theme classes (`src/ui/theme::CLASSIC`): `highlight_keyword`, `highlight_string`, `highlight_comment`,
  `highlight_number`, `highlight_operator`, `highlight_type`.
- Plugin precedence (feature 008): a plugin-registered highlighter is consulted before the built-in
  fallback — so adding built-ins for new extensions preserves precedence with no change there.

## Decision 1 — Clone the `c.rs` pattern per language

**Decision**: Each new highlighter is a struct + lazy regex set + the same candidate/sort/non-overlap loop
as `c.rs`. Comments/strings get priority over keywords/numbers so an identifier inside a comment/string
isn't re-styled.

**Rationale**: Uniform, proven, satisfies the renderer's sorted/non-overlap contract for free; minimal
risk; no new abstraction.

## Decision 2 — Per-language coverage + token→style map

- **Rust**: keywords (`fn let mut pub struct enum impl trait match if else for while loop use mod
  return self Self as where async await dyn move ref const static unsafe crate super` …) → keyword;
  primitive/types (`u8..u128 i8..i128 usize isize f32 f64 bool char str String Vec Option Result` +
  CamelCase identifiers) → type; strings (`"..."` with escapes; raw `r#"..."#`/`r"..."`; char/byte
  `'a'`/`b'a'`) → string; numbers (dec/hex/oct/bin/float with `_` and suffixes) → number; line `//` and
  single-line `/* ... */` → comment; attributes `#[...]`/`#![...]` → operator; macros `name!` → keyword.
- **JSON**: key = a `"..."` string immediately followed by optional spaces + `:` → type; other `"..."` →
  string; numbers (int/float/exp) → number; `true`/`false`/`null` → keyword; `{} [] : ,` → operator.
- **TOML**: table/array-of-table headers `^\s*\[\[?...\]\]?` → type; comment `#...` → comment; strings
  (`"..."`, `'...'`) → string; booleans `true`/`false` → keyword; dates (RFC-3339-ish
  `\d{4}-\d{2}-\d{2}([T ]\d{2}:\d{2}:\d{2}...)?`) → number; numbers → number; bare keys before `=` →
  keyword.

**Rationale**: Reuses the six existing theme classes; "key→type, value-string→string" makes JSON/TOML
keys visually distinct from string values (a common, useful distinction).

## Decision 3 — Line-based, best-effort (no cross-line state)

**Decision**: Match the existing highlighters — block comments/multi-line strings are styled per line via
single-line regexes; no cross-line state is tracked.

**Rationale**: Consistency with the current behavior and the trait signature (`highlight(line)`); keeps
scope contained (FR-007).

## Decision 4 — Register by extension; precedence unchanged

**Decision**: Add `Some("rs")`, `Some("json")`, `Some("toml")` arms to `detect_highlighter`. Do not touch
the plugin-precedence path (verified it consults plugins before the built-in fallback).

**Rationale**: FR-004/FR-006 — selection by extension; plugin override preserved automatically.

## Testing strategy (Constitution V — TDD)

- **Unit** per highlighter: representative lines assert that the expected token ranges are styled
  (keyword/type/string/number/comment/header/key), that spans are sorted + non-overlapping, and that
  malformed input (unterminated string, stray bracket), empty lines, multi-byte content, and a very long
  line don't panic.
- **detect**: `detect_highlighter` returns the right `name()` for `a.rs`/`a.json`/`a.toml` and `None` for
  an unknown extension.

## No open clarifications

Coverage and mapping are fixed by the spec; line-based best-effort matches existing behavior. No NEEDS
CLARIFICATION remains.
