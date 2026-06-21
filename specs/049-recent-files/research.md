# Phase 0 Research: Recent-Files List
## R1 — Storage
Mirror the session file: a TOML file `$XDG_STATE_HOME/edit/recent.toml` holding `paths: Vec<String>`
(most-recent first). `load()` → empty on absent/corrupt (no error, like a fresh list). `save()` atomic
tmp-rename. `record(path, limit)`: remove any existing equal path, push front, truncate to `limit`.
Decision: a dedicated `recent` module, not part of session (recent files persist independent of the
restore prompt).
## R2 — Config
`recent_files_limit` is named by the constitution but absent from `Config`. Add `#[serde(default = ...)]`
with default 10. Limit 0 disables (record no-ops; menu shows nothing).
## R3 — Menu surface
The menu already supports dynamic injection: `resolve_menus(plugin_items)` merges built-in + plugin
items into `ResolvedMenu`. Decision: extend it to `resolve_menus(plugin_items, recent_paths)` and append
recent entries to the File menu (after a separator-ish position), each as a `ResolvedItem` with
`Action::OpenRecent(idx)` and label = the file name. Two callers (render `ui/mod.rs`, hit-test
`app/mouse.rs`) pass `app.recent_files()`. Alternative (a dedicated modal list) rejected — more code
(new Modal variant + render + input) than reusing the menu path.
## R4 — Dispatch
New `Action::OpenRecent(usize)` (mirrors `MenuOpen(usize)`). Handler resolves the index against the live
recent list and calls `handle_open_file(path)` (which validates the path + surfaces errors + records it
again → moves to front). A now-missing file → existing open-failure message; optionally drop the stale
entry.
## R5 — Recording points
`handle_open_file` (open) and `do_save_as` (new path) call `record`. Not on session-restore opens
(those are already "recent" by nature; optional — keep simple: restore does not re-record). De-dup keeps
the list clean.
## Open questions: none.
