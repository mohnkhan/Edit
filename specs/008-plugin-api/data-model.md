# Data Model: Plugin API (Feature 008)

**Date**: 2026-06-19 | **Plan**: [plan.md](plan.md) | **Research**: [research.md](research.md)

---

## Entities

### Plugin

The distributable unit installed by a user. Lives as a subdirectory in
`$XDG_CONFIG_HOME/edit/plugins/<plugin-id>/`.

| Field | Type | Description |
|---|---|---|
| `id` | `String` | Unique reverse-domain or slug identifier (e.g. `lua-syntax`). Immutable. |
| `name` | `String` | Human-readable display name (e.g. "Lua Syntax Highlighter"). |
| `version` | `semver::Version` | Semver version of the plugin release. |
| `host_api` | `semver::VersionReq` | Semver range of host API the plugin requires (e.g. `^1`). |
| `types` | `Vec<PluginType>` | Extension types provided: `Highlighter`, `Keybinding`, `Menu`. |
| `extensions` | `Vec<String>` | File extensions handled (highlighter type only; e.g. `[".lua"]`). |
| `permissions` | `Vec<Permission>` | Declared extra permissions beyond the default empty sandbox. |
| `script_path` | `PathBuf` | Absolute path to the `plugin.rhai` file. |
| `manifest_path` | `PathBuf` | Absolute path to `plugin.toml`. |

**Validation rules**:
- `id` MUST match `[a-z0-9][a-z0-9-]*[a-z0-9]` (kebab-case, no leading/trailing hyphens).
- `version` MUST be valid semver.
- `host_api` MUST be a valid semver requirement parseable by the `semver` crate.
- `types` MUST be non-empty.
- `extensions` MUST be non-empty when `types` contains `Highlighter`.
- `name` MUST be valid UTF-8 and ≤ 64 characters.

---

### PluginType (enum)

```
Highlighter   — provides syntax coloring rules for one or more file extensions
Keybinding    — declares key-sequence → action mappings
Menu          — adds items to top-level menus
```

A single plugin may declare multiple types (e.g. `[highlighter, menu]`).

---

### Permission (enum)

Declared in `plugin.toml` `permissions` array. Absent = default-deny sandbox (no filesystem access, no process execution).

```
ReadPath(PathBuf)    — read access to a specific file or directory
WriteDir(PathBuf)    — write access to a specific directory (not recursively)
```

The Rhai base language exposes no filesystem or I/O at all; the only way a script can touch the filesystem is through the host-registered `read_file` function, which is permission-gated against the `ReadPath` grants above. All permissions are shown to the user at consent time and recorded in `plugins.toml`.

---

### PluginInstance

The runtime state of a loaded, compiled Rhai plugin. Held in-memory for the session.

| Field | Type | Description |
|---|---|---|
| `plugin` | `Plugin` | The owning plugin definition (from manifest). |
| `ast` | `rhai::AST` | The script compiled once at load time; reused for every call. |
| `scope` | `rhai::Scope` | Per-plugin persistent variable scope carried across calls. |
| `disabled` | `bool` | Set to `true` if a trap, timeout, or repeated FS violation occurs; persists for the session. |
| `fs_violations` | `u8` | Count of denied `read_file` attempts for undeclared paths; the plugin is disabled once this reaches 3 (per the error-handling contract). |

The shared `rhai::Engine` (configured with execution limits + host-registered functions) is **not** held per instance: it is owned once by `PluginHost` and reused across all `PluginInstance`s. Only the compiled `ast` and the persistent `scope` are per-plugin.

**State transitions**:
```
Loading → Active     (on successful instantiation + consent)
Active  → Disabled   (on trap / timeout / permission violation)
Loading → Rejected   (on manifest parse error / API version mismatch / consent denied)
```

---

### PluginRegistry

In-memory catalogue of all successfully instantiated plugins for the current session. Singleton, owned by `App`.

| Field | Type | Description |
|---|---|---|
| `instances` | `Vec<PluginInstance>` | All active (non-rejected) plugin instances, in load order. |
| `disabled` | `Vec<String>` | IDs of plugins disabled this session (trap/timeout). |

**Queries**:
- `highlighters_for(ext: &str) -> Vec<&PluginInstance>` — returns active highlighter plugins matching extension; first match wins for the session.
- `all_keybindings() -> Vec<(KeySeq, Action)>` — aggregated from all active keybinding plugins.
- `menu_items() -> Vec<(MenuPath, PluginMenuAction)>` — aggregated menu additions.

---

### ConsentRecord

Persistent per-plugin consent decision. Written to `$XDG_CONFIG_HOME/edit/plugins.toml`.

| Field | Type | Description |
|---|---|---|
| `allowed` | `bool` | Whether the user granted consent. |
| `consented_at` | `DateTime<Utc>` | ISO-8601 timestamp of the consent decision. |
| `version_consented` | `String` | Plugin version at consent time. |

**Rules**:
- A new consent prompt is shown if the installed plugin version differs from `version_consented` AND the new version declares additional permissions not in the original consent.
- Version upgrades that add no new permissions re-use the existing consent silently.

---

### PluginApiVersion

A compile-time `i32` constant in the host (`HOST_PLUGIN_API_VERSION: i32 = 1`). Plugins do **not** export a version function; instead the manifest's `host_api` semver range is authoritative and is matched against the host version at load time. The host MAY optionally inject a `HOST_API_VERSION` constant into the script scope so scripts can branch on it.

| Version | When incremented |
|---|---|
| 1 | Initial release |
| 2+ | Any breaking change to host-registered functions or the highlight token contract (name, signature, semantics) |

Additive changes (new host functions, new optional map fields) do NOT increment the version.

---

### HighlightToken

Returned by a highlighter plugin call. Represents a single coloured span in the buffer.

| Field | Type | Description |
|---|---|---|
| `byte_start` | `u32` | Start byte offset within the line (UTF-8). |
| `byte_end` | `u32` | End byte offset (exclusive). |
| `kind` | `TokenKind` | Semantic token type (Keyword, String, Comment, Number, Operator, Default). |

**Wire form**: tokens are returned by the script as a Rhai `Array` of `Map`s, each shaped `#{ start, end, kind: "<string>" }`. The host validates the array and converts it into `Vec<HighlightToken>`. There is no MessagePack / binary serialization — the values cross the boundary as native Rhai dynamic types.

**Invariants**:
- `byte_start < byte_end`.
- Both offsets are within the bounds of the input line bytes.
- Tokens MUST NOT overlap.
- The host validates these invariants after each plugin call; an invalid array is discarded in full.

---

### TokenKind (enum)

Maps to a theme colour slot in the editor's `Theme` struct. Plugins do not specify colours directly — they declare semantic kinds; the host maps them to colours.

```
Keyword    — language keyword (e.g. `if`, `function`)
String     — string literal
Comment    — line or block comment
Number     — numeric literal
Operator   — operator symbol
Type       — type name
Default    — unstyled / fallback
```

Scripts express the kind as a lowercase string — `"keyword"`, `"string"`, `"comment"`, `"number"`, `"operator"`, `"type"`, `"default"` — which the host parses into this enum. An unknown string causes the entire token array to be discarded.

---

### MenuPath

Represents the location of a plugin-provided menu item.

| Field | Type | Description |
|---|---|---|
| `menu` | `String` | Top-level menu label (e.g. `"Tools"`). Created if it doesn't exist. |
| `item` | `String` | Menu item label (e.g. `"Word Count"`). |
| `position` | `Option<u32>` | Insertion position within the menu; `None` = append. |

---

### PluginMenuAction

| Field | Type | Description |
|---|---|---|
| `path` | `MenuPath` | Where the item appears. |
| `plugin_id` | `String` | Owning plugin's ID. |
| `fn_name` | `String` | Rhai function in the plugin script to call when the item is activated. |

---

## Relationships

```
PluginRegistry
  ├── 0..N PluginInstance
  │     └── 1   Plugin (owns manifest data)
  └── 0..N disabled plugin IDs (session-only blacklist)

Plugin
  └── 0..N Permission (declared in manifest)

ConsentRecord (persisted, keyed by plugin id)
  └── 1   per-plugin consent decision

PluginApiVersion
  └── validated against each PluginInstance at load time
```

---

## Storage

| Entity | Storage | Format |
|---|---|---|
| Plugin manifest | `<plugin-dir>/plugin.toml` | TOML |
| Plugin script | `<plugin-dir>/plugin.rhai` | Rhai source (UTF-8 text) |
| Consent records | `$XDG_CONFIG_HOME/edit/plugins.toml` | TOML |
| PluginRegistry | In-memory only (session) | — |
| PluginInstance | In-memory only (session) | — |

`<plugin-dir>` = `$XDG_CONFIG_HOME/edit/plugins/<plugin-id>/`
