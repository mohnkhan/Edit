# Phase 1 Data Model: Recent-Files List
## New module `recent`
```rust
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecentFiles { pub paths: Vec<String> } // most-recent first
impl RecentFiles {
    pub fn record(&mut self, path: &str, limit: usize) {
        self.paths.retain(|p| p != path);   // dedup
        self.paths.insert(0, path.into());  // front
        self.paths.truncate(limit);         // cap
        if limit == 0 { self.paths.clear(); }
    }
}
pub fn recent_path() -> PathBuf;                 // $XDG_STATE_HOME/edit/recent.toml
pub fn load() -> RecentFiles;                    // empty on absent/corrupt
pub fn save(r: &RecentFiles) -> io::Result<()>;  // atomic tmp-rename
```
## Config
`Config.recent_files_limit: usize` (`#[serde(default = "default_recent_limit")]` → 10).
## Action
`Action::OpenRecent(usize)` — index into the live recent list at dispatch time.
## App
`recent_files: RecentFiles` (loaded in `App::new`); `recent_files(&self) -> &RecentFiles` accessor.
Recorded (+ saved) in `handle_open_file` and `do_save_as`.
## Menu
`resolve_menus(plugin_items, recent_paths)` appends to the File menu, per recent path i: `ResolvedItem {
label: filename_of(path_i), action: Action::OpenRecent(i), mnemonic: None }`. Empty list ⇒ no change.
## Persistence/format: standalone `recent.toml`; corrupt/absent ⇒ empty (no error).
