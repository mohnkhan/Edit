# Contract: Rust / JSON / TOML highlighters

Behavioral contract the tests assert against.

## Selection

- `detect_highlighter("x.rs")` → Rust highlighter (`name() == "Rust"`).
- `detect_highlighter("x.json")` → JSON highlighter (`name() == "JSON"`).
- `detect_highlighter("x.toml")` → TOML highlighter (`name() == "TOML"`).
- Unknown extension → `None`. A plugin highlighter for these extensions still overrides the built-in.

## Rust (`name() == "Rust"`)

- `fn`,`let`,`pub`,`struct`,`impl`,`match`,`use`,`return`,… → keyword.
- primitives (`u32`,`usize`,`f64`,`bool`,`str`,`String`,…) and CamelCase idents → type.
- `"…"`, raw `r#"…"#`, char `'a'`, byte `b'a'` → string.
- decimal/hex/oct/bin/float numbers (with `_`, type suffixes) → number.
- `// …` and single-line `/* … */` → comment.
- `#[…]` / `#![…]` → operator; `name!` macro → keyword.
- Example: `fn main() { let x: u32 = 1; // c }` → `fn`/`let` keyword, `u32` type, `1` number, `// c` comment.

## JSON (`name() == "JSON"`)

- a `"…"` immediately before `:` → key (type class); other `"…"` → string.
- int/float/exp numbers → number; `true`/`false`/`null` → keyword; `{} [] : ,` → operator.
- Example: `{ "n": 42, "ok": true }` → `"n"` key, `42` number, `true` keyword.

## TOML (`name() == "TOML"`)

- `^[…]` / `^[[…]]` headers → type; bare key before `=` → keyword; `"…"`/`'…'` → string;
  numbers and RFC-3339-ish dates → number; `true`/`false` → keyword; `# …` → comment.
- Example: `name = "edit" # x` → `name` keyword, `"edit"` string, `# x` comment; `[package]` → type.

## Span / resilience (all three)

- Output spans are **sorted by start** and **non-overlapping** (renderer contract).
- Byte offsets are correct for UTF-8 (multi-byte content in strings/comments/identifiers).
- No panic on: unterminated string, unmatched bracket, empty line, whitespace-only line, very long line.
- Line-based best-effort: multi-line block comments / strings are styled per line (no cross-line state),
  consistent with the existing highlighters.

## No-regression

- The render pipeline, buffer, plugin precedence, and the existing 5 highlighters are unchanged; only new
  modules + three `detect_highlighter` arms are added.
