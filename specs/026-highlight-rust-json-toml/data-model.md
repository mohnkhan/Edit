# Phase 1 Data Model: Rust / JSON / TOML highlighters

No persisted data. The "model" is the per-language token→style mapping and the span contract.

## Reused types

- `Highlighter` trait — `highlight(&self, &str) -> Vec<Span>`, `name()`.
- `Span { start: usize, end: usize, style: Style }` — byte offsets, half-open `[start, end)`.
- Theme classes: `highlight_keyword`, `highlight_string`, `highlight_comment`, `highlight_number`,
  `highlight_operator`, `highlight_type`.

## Token → style mapping

| Language | Token | Style class |
|---|---|---|
| Rust | keyword (`fn`,`let`,`impl`,`match`,…) | keyword |
| Rust | type (primitives + CamelCase) | type |
| Rust | string / char / byte / raw string | string |
| Rust | number (dec/hex/oct/bin/float, `_`, suffixes) | number |
| Rust | `//` line + single-line `/* */` | comment |
| Rust | attribute `#[…]` / `#![…]` | operator |
| Rust | macro `name!` | keyword |
| JSON | key (`"…"` before `:`) | type |
| JSON | string value | string |
| JSON | number | number |
| JSON | `true` / `false` / `null` | keyword |
| JSON | `{ } [ ] : ,` | operator |
| TOML | table / array-of-table header `[…]` / `[[…]]` | type |
| TOML | bare key (before `=`) | keyword |
| TOML | string (`"…"`, `'…'`) | string |
| TOML | number / date | number |
| TOML | `true` / `false` | keyword |
| TOML | `#` comment | comment |

## Span contract (enforced by the shared candidate/non-overlap loop)

- Candidates collected in priority order (comments & strings before keywords/numbers), sorted by
  `(start, longest-first)`, then emitted only when `start >= last_end` → output spans are **sorted** and
  **non-overlapping**.
- `start`/`end` are byte offsets from `regex` over the UTF-8 line → width/byte-correct.

## Invariants

- `highlight()` never panics for any line (malformed tokens, empty, very long, multi-byte).
- Output spans are sorted ascending by `start` and non-overlapping.
- Built-in selection is by extension; a plugin highlighter for the same extension still takes precedence
  (existing logic, unchanged).
