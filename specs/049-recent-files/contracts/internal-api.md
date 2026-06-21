# Internal Contract: Recent-Files List
- C-1: `RecentFiles::record(path, limit)` dedups, moves to front, caps at `limit` (0 ⇒ empty).
- C-2: `recent::load()` returns empty on absent/corrupt; `save()` is atomic.
- C-3: `resolve_menus(plugin_items, recent)` appends File-menu entries with `Action::OpenRecent(i)`;
  empty recent ⇒ menus identical to before (FR-006).
- B-1: open + save-as record the path (front, dedup) and persist (FR-001/003).
- B-2: `Action::OpenRecent(i)` opens `recent.paths[i]` via the normal validated open path; missing file
  → normal open-failure message, no crash (FR-005).
- B-3: untitled/no-path buffers are never recorded.
- B-4: limit 0 / empty list ⇒ no menu entries, no behavior change (FR-006).
- T: unit (record dedup/front/cap, limit 0), persistence round-trip, menu-injection (File menu contains
  recent + OpenRecent action), empty-list-unchanged; suite + clippy -D warnings + fmt clean.
