# Phase 1 Data Model: File Browser Dialogs

All types live in the new `src/ui/file_browser.rs` module unless noted.

## `BrowseMode`

```text
enum BrowseMode { Open, Save }
```

Selects behaviour on activation of a file: `Open` loads it into a buffer; `Save` writes the active
buffer to it. Also toggles the filename input line in the widget.

## `EntryKind`

```text
enum EntryKind { Parent, Dir, File }
```

`Parent` is the synthetic `..` row (present unless at filesystem root). Drives sort order and the
visual marker.

## `Entry`

| Field | Type | Notes |
|---|---|---|
| `name` | `String` | Display name (UTF-8, lossy-decoded from the OS string). For `Parent` it is `".."`. |
| `kind` | `EntryKind` | Parent / Dir / File. |

## `FileBrowser` (the session/state object — held in `App`)

| Field | Type | Notes |
|---|---|---|
| `mode` | `BrowseMode` | Open or Save. |
| `cwd` | `PathBuf` | **Canonical absolute** directory currently shown. Invariant: always canonicalized, never contains `..`. |
| `entries` | `Vec<Entry>` | Sorted: `..` → dirs → files, each case-insensitive alphabetical. Includes dot-files. |
| `selected` | `usize` | Highlighted index into `entries` (clamped to range). |
| `scroll` | `usize` | First visible entry index; keeps `selected` within the visible window. |
| `filename` | `String` | Save mode: the filename being typed. Open mode: optional typed path/jump buffer. |
| `error` | `Option<String>` | Transient notice (e.g. "Permission denied"); shown until next successful action. |

### Construction

- `FileBrowser::open(start_dir: PathBuf, mode: BrowseMode) -> Self` — canonicalizes `start_dir`
  (fallback to CWD then `/`), reads the listing, selects index 0, scroll 0.

### Methods (pure state transitions; no rendering)

| Method | Effect |
|---|---|
| `reload(&mut self)` | Re-read `cwd` via `read_dir`; rebuild + sort `entries`; clamp `selected`/`scroll`. On error: set `error`, keep prior entries. |
| `move_up(&mut self)` / `move_down(&mut self)` | Move `selected` by 1 (clamped, no wrap); adjust `scroll` to keep it visible given a `visible_rows` arg. |
| `enter_parent(&mut self)` | If `cwd.parent()` exists, set `cwd` to it (canonical) and `reload`; no-op at root. |
| `activate(&mut self, visible_rows) -> Outcome` | Act on the selected entry (see Outcome). |
| `activate_index(&mut self, idx, visible_rows) -> Outcome` | Mouse path: set `selected = idx`, then `activate`. |
| `set_filename_char` / `backspace` | Edit the `filename`/path field (Save, or Open jump-path). |
| `selected_save_path(&self) -> Result<PathBuf, String>` | Validate `cwd` + single-segment `filename`; return `cwd.join(filename)` or an error string. |

### `Outcome` (returned to the app event loop)

```text
enum Outcome {
    Navigated,            // listing changed; stay open
    OpenFile(PathBuf),    // Open mode: validated file chosen
    SaveFile(PathBuf),    // Save mode: validated destination chosen
    None,                 // nothing actionable (e.g. empty filename)
}
```

The **app** performs the side effects (`Buffer::open`, `do_save_as`) and closes the browser; the
model itself never touches buffers or the filesystem beyond `read_dir`/`canonicalize`.

## `App` changes (src/app.rs)

- Remove `pending_open: Option<String>` and `pending_save_as: Option<String>`.
- Add `file_browser: Option<FileBrowser>`.
- `Action::Open` → `file_browser = Some(FileBrowser::open(start_dir, Open))`.
- `Action::SaveAs` (and `Ctrl+S`/`handle_save_action` on an unnamed buffer) →
  `file_browser = Some(FileBrowser::open(start_dir, Save))`.

## Validation rules (from requirements)

- `cwd` invariant: canonical, absolute, no `..` (R1). All navigation maintains it.
- Open target: `validate_path(cwd.join(name))` must succeed (file exists) before `Buffer::open`.
- Save target: `validate_path(cwd)` must succeed AND `filename` is non-empty, contains no path
  separator and no `..`; destination = `cwd.join(filename)`.
- Names render as UTF-8 and truncate on grapheme/width boundaries (R3).
- Directory read errors never crash; they set `error` and preserve state (R2).
