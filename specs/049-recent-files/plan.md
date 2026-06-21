# Implementation Plan: Recent-Files List
**Branch**: `049-recent-files` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md) | Issue #81

## Summary
Add a persisted recent-files list, surfaced in the File menu. A small `recent` module loads/saves a
capped, de-duplicated path list under the state dir; the App records on open + save-as; the File menu
gains the entries via the existing dynamic-menu injection (`resolve_menus`), and a new
`Action::OpenRecent(index)` opens the chosen path. New config key `recent_files_limit` (default 10).

## Technical Context
Rust 2021; serde+toml (existing). No new deps. Tests: unit (recent store) + menu/dispatch + persistence.
Behavior-preserving when empty/limit 0 (FR-006). 042/046 guardrails hold.

## Constitution Check
I PASS (File-menu entries, DOS-style). II PASS (paths as strings). III/IV PASS. V PASS (tests).
VI PASS (closes a named baseline capability; minimal). VII — paths validated on open via existing
`validate_path`; no new traversal surface. All gates pass.

## Project Structure
- `src/config/schema.rs` — add `recent_files_limit: usize` (default 10; serde default).
- `src/recent/mod.rs` (NEW) — `RecentFiles { paths: Vec<String> }`; `load()/save()` (TOML at
  `$XDG_STATE_HOME/edit/recent.toml`), `record(path, limit)` (dedup → front → cap).
- `src/lib.rs`/`main.rs` — register `mod recent;`.
- `src/input/keymap.rs` — `Action::OpenRecent(usize)`.
- `src/app.rs` — field `recent_files: RecentFiles`; load in `App::new`; accessor `recent_files()`.
- `src/app/actions.rs` — record + save on `handle_open_file`; `Action::OpenRecent(i)` → open path i.
- `src/app/fileops.rs` — record on `do_save_as` (new path).
- `src/ui/menubar.rs` — `resolve_menus(plugin_items, recent)` injects recent items into the File menu
  (label = file name, action = `OpenRecent(idx)`); callers in `ui/mod.rs` + `app/mouse.rs` pass recent.

## Key decisions (research.md)
- Reuse the dynamic-menu path (`resolve_menus` already merges plugin items) → inject recent entries into
  the File menu; `Action::OpenRecent(usize)` indexes the live recent list at dispatch time.
- Persistence mirrors the session file (TOML, state dir, atomic write); corrupt/absent → empty.
- Record on open + save-as only; cap + dedup in `record`. Limit 0 ⇒ no entries.

## Phases (one PR)
1. config key + `recent` module (+ tests). 2. App field/load + record on open/save-as +
   `Action::OpenRecent` dispatch. 3. menu injection (resolve_menus signature + callers). 4. tests + docs.

## Complexity Tracking: empty.
