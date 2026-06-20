# Feature Specification: Syntax highlighting for Rust, JSON, and TOML

**Feature Branch**: `026-highlight-rust-json-toml`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "Add syntax highlighting for Rust (.rs), JSON (.json), and TOML (.toml) —
three more languages beyond the baseline 5 (C, Python, Shell, YAML, Markdown), each a new highlighter
registered by file extension. Line-based, consistent with the existing highlighters; plugin highlighters
still override built-ins. The constitution defers extra languages pending a spec — this is that spec."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Rust files are highlighted (Priority: P1)

When the user opens a `.rs` file, its syntax is colorized: keywords, types, strings, character/byte/raw
literals (best-effort), numbers, line and block comments (best-effort per line), attributes, and macro
invocations are visually distinguished. This is the headline win — the editor is itself a Rust project.

**Why this priority**: Rust is the project's own language; highlighting `.rs` files is the most valuable
and most-used of the three and proves the pattern.

**Independent Test**: Open a `.rs` file containing `fn main() { let x: u32 = 1; // note }` → `fn`/`let`
read as keywords, `u32` as a type, `1` as a number, and `// note` as a comment, each styled distinctly.

**Acceptance Scenarios**:

1. **Given** a `.rs` file is open, **When** it renders, **Then** Rust keywords, types, strings, numbers,
   comments, attributes, and macros are each styled distinctly from plain text.
2. **Given** a line with `// ...` or a `/* ... */` segment, **When** it renders, **Then** the comment
   portion is styled as a comment (best-effort, line-based).
3. **Given** a string or number literal on a line, **When** it renders, **Then** it is styled as a
   string/number and surrounding code keeps its own styling (non-overlapping, ordered spans).

### User Story 2 - JSON files are highlighted (Priority: P1)

When the user opens a `.json` file, object keys, string values, numbers, the literals `true`/`false`/
`null`, and structural punctuation are visually distinguished.

**Why this priority**: JSON configs are common (and the editor reads JSON), so highlighting them is
broadly useful and simple to get right.

**Independent Test**: Open a `.json` file with `{ "name": "edit", "n": 42, "ok": true, "x": null }` →
`"name"` reads as a key, `"edit"` as a string value, `42` as a number, `true`/`null` as literals.

**Acceptance Scenarios**:

1. **Given** a `.json` file is open, **When** it renders, **Then** keys, string values, numbers,
   booleans/null, and punctuation are each styled distinctly.
2. **Given** a key/value pair on a line, **When** it renders, **Then** the key string and the value
   string are distinguishable (a key is the string before a `:`).

### User Story 3 - TOML files are highlighted (Priority: P2)

When the user opens a `.toml` file (e.g. `Cargo.toml`), table/array-of-table headers (`[..]` / `[[..]]`),
keys, strings, numbers, booleans, dates, and `#` comments are visually distinguished.

**Why this priority**: `Cargo.toml` and config files benefit, but JSON/Rust cover more ground first.

**Independent Test**: Open `Cargo.toml` → `[package]` reads as a table header, `name = "edit"` shows
`name` as a key and `"edit"` as a string, `edition = "2021"` likewise, and `# comment` as a comment.

**Acceptance Scenarios**:

1. **Given** a `.toml` file is open, **When** it renders, **Then** table headers, keys, strings, numbers,
   booleans, dates, and comments are each styled distinctly.
2. **Given** a `# ...` comment, **When** it renders, **Then** the comment portion is styled as a comment.

### Edge Cases

- **Plugin override**: a plugin-provided highlighter for the same extension still takes precedence over
  the new built-in (existing rule, unchanged).
- **Multi-line constructs**: a block comment or string that spans lines is highlighted best-effort
  per-line (consistent with the existing C/etc. highlighters); cross-line state is not tracked.
- **Malformed / partial input**: an unterminated string, stray bracket, or odd token never panics and
  never produces overlapping/unsorted spans; the line still renders.
- **Empty lines / whitespace-only lines**: render with no spurious spans.
- **Wide / multi-byte content** (UTF-8 in strings/comments/identifiers): span offsets stay
  byte/grapheme-correct so the renderer aligns highlight regions with the text.
- **Very long lines**: highlighting completes without excessive delay or panic.
- **Unknown extensions**: behavior is unchanged (no highlighter; plain text).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Opening a `.rs` file MUST apply a Rust highlighter that distinguishes keywords, types,
  strings (incl. char/byte/raw literals best-effort), numbers, line/block comments (best-effort,
  line-based), attributes, and macro invocations.
- **FR-002**: Opening a `.json` file MUST apply a JSON highlighter that distinguishes object keys, string
  values, numbers, `true`/`false`/`null`, and structural punctuation.
- **FR-003**: Opening a `.toml` file MUST apply a TOML highlighter that distinguishes table/array-of-table
  headers, keys, strings, numbers, booleans, dates, and `#` comments.
- **FR-004**: Each highlighter MUST be selected by file extension (`.rs`, `.json`, `.toml`) through the
  existing extension-based registration.
- **FR-005**: Each highlighter MUST produce non-overlapping spans sorted by start offset (the contract the
  renderer requires), and offsets MUST be byte/width-correct for UTF-8 content.
- **FR-006**: A plugin-provided highlighter for the same extension MUST continue to take precedence over
  the new built-in highlighter (existing behavior preserved).
- **FR-007**: Highlighting MUST be line-based and MUST NOT change the rendering pipeline, the buffer, or
  any other feature; multi-line constructs are highlighted best-effort per line.
- **FR-008**: No input MUST cause a panic or produce malformed spans — including unterminated strings,
  unmatched brackets, empty lines, very long lines, and multi-byte content.

### Key Entities

- **Highlighter (per language)**: a styling rule set that maps a single line of text to an ordered,
  non-overlapping list of styled spans, identified by a human-readable name.
- **Span**: a styled region of a line (start/extent + style class), consumed by the renderer.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Opening a `.rs`, `.json`, or `.toml` file shows syntax highlighting (the language's keywords/
  keys/strings/numbers/comments are visually distinct from plain text) without any extra action.
- **SC-002**: The editor now highlights at least 8 languages (the baseline 5 plus Rust, JSON, TOML).
- **SC-003**: Highlighting produces valid spans (non-overlapping, sorted, width-correct) for every line of
  representative real files (e.g. this project's own `src/*.rs`, `Cargo.toml`, and a JSON config), with no
  panic.
- **SC-004**: A plugin highlighter registered for `.rs`/`.json`/`.toml` still overrides the built-in.

## Assumptions

- **Line-based, best-effort** highlighting consistent with the existing highlighters: multi-line block
  comments and multi-line strings are styled per line without cross-line state (matching current C/etc.
  behavior); exactness of every literal form is not required, reasonable coverage is.
- **Reuse** the existing `Highlighter` trait, `Span` type, theme styles, and plugin-precedence logic; no
  new theme keys are required (map to existing style classes — keyword/type/string/number/comment/etc.).
- **Extensions**: `.rs` → Rust; `.json` → JSON; `.toml` → TOML (lowercase; case-handling consistent with
  the existing registration).
- **Scope**: highlighting rules only — no change to the rendering pipeline, buffer, config, or other
  dialogs; no new dependencies beyond what the existing highlighters use.
- This satisfies the constitution's requirement of a spec + accepted user story before adding languages
  beyond the baseline 5 (Principle VI).
- Builds on the highlight subsystem from feature 008's plugin work and the baseline highlighters.
