# Phase 0 Research: File dialog — glob filtering + richer entry details

## Existing machinery (from code survey)

- `FileBrowser::reload` (`src/ui/file_browser.rs:116`) reads the dir, splits into dirs/files, sorts
  case-insensitively, prepends `..` (unless root), and sets `self.entries`. `Entry { name, kind }` is
  only constructed here (3 literals).
- The widget render loop iterates `entries[scroll + vis]`, draws a marker + truncated name (feature 021
  reserves the rightmost column for a scrollbar when overflowing). `hit_test`, `move_up/down`,
  `activate_open/activate_save` all index `self.entries` / `self.selected`.
- The field is `self.filename` (Open = jump path / now filter; Save = filename). `push_char`/`backspace`
  edit it; `activate_open` treats an **absolute** path specially and otherwise opens the selected entry.
- No glob/regex/humansize/date crates are present.

## Decision 1 — Filtered view as `entries`, full list as `all_entries`

**Decision**: Add `all_entries: Vec<Entry>` (full, sorted, with metadata). `reload()` populates it and
then calls `apply_filter()`, which sets `self.entries` to the filtered subset. All existing code keeps
using `self.entries`/`self.selected` unchanged.

**Rationale**: Minimal blast radius — render, navigation, hit-test, and activation already operate on
`entries`; treating `entries` as "currently displayed" means only `reload` + field edits change.

**Alternatives**: a `visible: Vec<usize>` index layer over a single list (more pervasive edits to every
indexing site) — rejected.

## Decision 2 — In-house wildcard matcher (no crate)

**Decision**: Implement `glob_match(pattern, name)` supporting `*` (any run) and `?` (one char),
case-insensitive, via the classic linear two-pointer backtracking algorithm. A pattern with no `*`/`?`
is matched by case-insensitive **substring** (`name.to_lowercase().contains(pat.to_lowercase())`).

**Rationale**: Constitution IV (no speculative deps). The matcher is ~20 lines, well-understood, and
needs no `[` ranges or full regex for this UX.

**Alternatives**: `globset`/`glob`/`regex` crates — rejected (footprint); `[...]` character classes —
out of scope (YAGNI).

## Decision 3 — Filter vs jump interpretation of the field

**Decision**: Interpret `self.filename` as:
- empty → no filter (full listing);
- starts with `/` (absolute path) → **jump target**, listing **not** filtered (existing behavior);
- contains `*` or `?` → **glob** filter on file names;
- otherwise → case-insensitive **substring** filter on file names.
Directories and `..` are **always** kept regardless of the filter (FR-003).

**Rationale**: Matches the three clarified decisions (live, case-insensitive, plain-text substring) and
preserves the absolute-path jump so the field stays dual-purpose without ambiguity.

## Decision 4 — std-only size & date formatting

**Decision**: `human_size(bytes)` → `B`/`K`/`M`/`G` with ≤1 decimal for sub-10 values (e.g. `1.2K`,
`15K`, `3.4M`). `format_mtime(SystemTime)` → `YYYY-MM-DD HH:MM` in **UTC**, via epoch-seconds and the
days-from-civil algorithm (no `chrono`/`time`). Read size/mtime from `std::fs::metadata` (best-effort:
`Option` fields; missing metadata renders blank, never fails).

**Rationale**: Constitution IV. UTC avoids a timezone dependency; the format is compact and stable for
tests. Directories show `<DIR>` instead of a size (FR-007).

**Alternatives**: local-time formatting (needs a tz crate) — rejected; exact byte counts — noisy.

## Decision 5 — Detail-column layout (name truncates, details don't)

**Decision**: Render fixed-width right-aligned size and date columns; the name column gets the remaining
width and is truncated with `…` (reusing `truncate_to_width`/`grapheme_width`). When the row is too
narrow for even minimal columns, drop the detail columns gracefully (name-only) rather than corrupt
layout. The feature-021 scrollbar column reservation still applies.

**Rationale**: FR-008 — details stay readable/aligned; truncation is width-correct and already
implemented for names.

## Testing strategy (Constitution V — TDD)

- **Unit**: `glob_match` (`*.log`, `te?t`, anchoring, case-insensitive), substring fallback,
  `human_size` boundaries (0, 999, 1024, large), `format_mtime` (known epoch → expected UTC string),
  `apply_filter` keeps dirs/`..` and re-clamps selection, detail-row truncation width.
- **Integration**: typing `*.log` / substring narrows the listing; clearing restores; absolute path
  still jumps/opens; Save-mode confirm still saves the typed name; buttons/focus-ring + scrollbar still
  function with a filter active.

## No open clarifications

The three product decisions are fixed; remaining choices have documented defaults. No `NEEDS
CLARIFICATION` remains.
