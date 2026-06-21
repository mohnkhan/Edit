# Tasks: Recent-Files List (#81)
**Branch**: 049-recent-files. Behavior-preserving when empty / limit 0 (FR-006).
## Setup
- [x] T001 Baseline `make check` (note 1286/0/11).
## Foundational
- [x] T002 Config: add `recent_files_limit: usize` to `src/config/schema.rs` (serde default 10); test default + round-trip.
- [x] T003 New `src/recent/mod.rs`: `RecentFiles{paths}` + `record(path,limit)` (dedup/front/cap; 0⇒empty) + `recent_path()`/`load()`/`save()` (TOML, state dir, atomic, empty-on-corrupt). Register `mod recent;` in lib.rs. Unit tests for record + round-trip.
## US1/US2 (P1)
- [x] T004 `Action::OpenRecent(usize)` in `src/input/keymap.rs`.
- [x] T005 `App`: `recent_files: RecentFiles` loaded in `App::new`; `recent_files()` accessor (`src/app.rs`).
- [x] T006 Record + save on open (`handle_open_file`, `src/app/actions.rs`) and save-as (`do_save_as`, `src/app/fileops.rs`), using `config.recent_files_limit`.
- [x] T007 Dispatch `Action::OpenRecent(i)` (`src/app/dispatch.rs` or actions): resolve `recent_files.paths.get(i)` → `handle_open_file(path)`; missing → normal failure (no crash).
- [x] T008 Menu: `resolve_menus(plugin_items, recent)` injects File-menu entries (label=file name, action=OpenRecent(idx)); update callers in `src/ui/mod.rs` and `src/app/mouse.rs` to pass `app.recent_files()`. Empty ⇒ unchanged.
## Tests
- [x] T009 Unit: record dedup/front/cap + limit 0; persistence round-trip (T003 covers).
- [x] T010 Integration: open files via App → recent list ordered/deduped/capped; OpenRecent(i) opens path; File menu (resolve_menus) contains the entries; empty list ⇒ File menu identical to before.
## Ship
- [x] T011 `make ci-local`; count == baseline + new tests.
- [x] T012 Docs: CHANGELOG + STATUS + CAPABILITIES (File menu gains recent files). PR `feat(049): recent-files list`, Closes #81, merge.
