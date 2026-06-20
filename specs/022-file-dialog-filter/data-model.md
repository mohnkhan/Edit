# Phase 1 Data Model: File dialog — glob filtering + richer entry details

UI/filesystem-read state only — no persistence, config, or file formats.

## Entity: `Entry` (extended)

| Field | Type | Notes |
|---|---|---|
| `name` | `String` | unchanged |
| `kind` | `EntryKind` | `Parent` / `Dir` / `File` (unchanged) |
| `size` | `Option<u64>` | **new** — file byte size; `None` for dirs/`..`/unreadable |
| `mtime` | `Option<u64>` | **new** — modified time as epoch seconds; `None` if unreadable |

Constructed only in `reload()` (size/mtime read from `std::fs::metadata`, best-effort).

## Entity: `FileBrowser` (extended)

| Field | Type | Role |
|---|---|---|
| `all_entries` | `Vec<Entry>` | **new** — full sorted listing (source of truth) |
| `entries` | `Vec<Entry>` | now the **filtered** view actually displayed (existing code unchanged) |
| `filename` | `String` | the field text — interpreted as filter or jump target |
| `selected` / `scroll` | `usize` | re-clamped to `entries` after each filter |

### `apply_filter()` (new)

Derives `entries` from `all_entries` based on `filename`:

```
filter(filename):
  if filename is empty OR filename starts with '/':   entries = all_entries        # no filter / jump
  else if filename contains '*' or '?':               keep Parent|Dir; keep File if glob_match(filename, name)
  else:                                               keep Parent|Dir; keep File if name contains filename (ci)
  then: clamp selected into [0, entries.len()); reset/clamp scroll
```

Directories and `..` are **always** retained (FR-003). Matching is case-insensitive (FR-002).

## Helpers (std-only, new)

| Helper | Signature | Behavior |
|---|---|---|
| `glob_match` | `(pattern: &str, name: &str) -> bool` | `*`/`?` wildcards, case-insensitive, two-pointer backtracking |
| `human_size` | `(bytes: u64) -> String` | `B`/`K`/`M`/`G`, ≤1 decimal for small values (e.g. `1.2K`, `3.4M`) |
| `format_mtime` | `(secs: u64) -> String` | `YYYY-MM-DD HH:MM` (UTC) via days-from-civil; empty on failure |

## Listing row layout (widget)

`[marker] [name … ]  [size]  [date]` — size right-aligned in a fixed column, date in a fixed column,
name takes remaining width and truncates with `…` (grapheme/width-correct). Dirs/`..` show `<DIR>` in
the size column. The feature-021 scrollbar column (when present) is reserved before computing the name
width. On a very narrow row, detail columns are dropped (name-only) rather than corrupting layout.

## Invariants

- `entries ⊆ all_entries`; `..` and every `Dir` in `all_entries` are present in `entries`.
- `selected` always indexes a visible entry (or 0 when empty-but-for-nav).
- Filtering changes only what is displayed — open/save/navigation semantics are unchanged.
- No panic on unreadable metadata, empty dirs, no-match filters, or tiny terminals.
