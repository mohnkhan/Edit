# Tasks: Plugin API (Feature 008)

**Input**: Design documents from `specs/008-plugin-api/`

**Prerequisites**: plan.md ✅ | spec.md ✅ | research.md ✅ | data-model.md ✅ | contracts/ ✅ | quickstart.md ✅

**Engine**: Rhai (pure-Rust embedded scripting). Plugins are `plugin.toml` + `plugin.rhai`
text files — no compilation, no binary artifacts, no WASM toolchain. Fixtures are committed
as plain text.

**Organization**: Tasks grouped by user story to enable independent implementation and testing.
Per Constitution Principle V (TDD), test tasks are ordered **before** the implementation they
cover within each phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks in same phase)
- **[Story]**: Maps to user story from spec.md (US1–US5)

---

## Phase 1: Setup

- [x] T001 Add `rhai = "1"` to `[dependencies]` in `Cargo.toml`; add `[[test]] name = "plugin_api" path = "tests/integration/plugin_api.rs"`; run `cargo build` and confirm it succeeds. Do NOT add `extism` or `rmp-serde`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: All shared plugin infrastructure — types, manifest parser, consent store, Rhai
engine builder, host-function registry, in-memory registry, host integration points, and the
two "bad-plugin" text fixtures that later phases depend on.

**⚠️ CRITICAL**: No user story work begins until this phase passes `cargo test`.

- [x] T002 Create `src/plugin/mod.rs` declaring the `plugin` module and re-exporting `PluginHost`; add `pub mod plugin;` to `src/lib.rs`

- [x] T003 Create `src/plugin/types.rs` with all shared types:
  - `pub enum PluginType { Highlighter, Keybinding, Menu }` (derive `Debug, Clone, Copy, PartialEq, serde::Deserialize`)
  - `pub enum Permission { ReadPath(std::path::PathBuf), WriteDir(std::path::PathBuf) }` (derive `Debug, Clone`)
  - `pub enum TokenKind { Default, Keyword, String, Comment, Number, Operator, Type }` (derive `Debug, Clone, Copy, PartialEq`) with `pub fn from_str(s: &str) -> Option<TokenKind>` mapping lowercase script strings ("keyword"…"default") to variants (unknown → `None`)
  - `pub struct HighlightToken { pub byte_start: u32, pub byte_end: u32, pub kind: TokenKind }` (derive `Debug, Clone, PartialEq`)
  - `pub struct Plugin { pub id: String, pub name: String, pub version: semver::Version, pub host_api: semver::VersionReq, pub types: Vec<PluginType>, pub extensions: Vec<String>, pub permissions: Vec<Permission>, pub script_path: std::path::PathBuf, pub manifest_path: std::path::PathBuf }` (derive `Debug, Clone`)
  - `pub struct PluginInstance { pub plugin: Plugin, pub ast: rhai::AST, pub scope: rhai::Scope<'static>, pub disabled: bool, pub fs_violations: u8 }` (no derive; holds the compiled script AST + persistent scope)
  - `pub const HOST_PLUGIN_API_VERSION: i32 = 1;`

- [x] T004 Create `src/plugin/manifest.rs` with `pub fn parse_manifest(manifest_path: &Path) -> Result<Plugin, PluginLoadError>`:
  - Read file as bytes; validate UTF-8 (reject non-UTF-8 → `InvalidUtf8`); parse TOML via `serde`
  - Validate `id` matches `^[a-z0-9][a-z0-9-]*[a-z0-9]$`
  - Validate `version` is valid semver; validate `host_api` is a valid `semver::VersionReq`; reject if it does not match `HOST_PLUGIN_API_VERSION` → `ApiVersionMismatch`
  - Validate `name` is valid UTF-8 and ≤ 64 chars
  - Validate `types` non-empty; `extensions` non-empty when `Highlighter` ∈ `types`
  - Set `script_path` to `manifest_path.parent().unwrap().join("plugin.rhai")`
  - `pub enum PluginLoadError`: `ManifestParseError(String)`, `InvalidId(String)`, `InvalidVersion(String)`, `ApiVersionMismatch { plugin: String, host: i32 }`, `InvalidUtf8(String)`, `ScriptParseError(String)`, `ConsentDenied`
  - **Unit tests (write first)**: `test_valid_manifest_parses`, `test_invalid_id_rejected`, `test_missing_extensions_for_highlighter_rejected`, `test_name_too_long_rejected`, `test_incompatible_host_api_rejected`, `test_non_utf8_manifest_rejected`

- [x] T005 Create `src/plugin/consent.rs`:
  - `pub struct ConsentRecord { pub allowed: bool, pub consented_at: String, pub version_consented: String }` (derive `Debug, Clone, serde::Serialize, serde::Deserialize`)
  - `pub fn load_consent_records(config_dir: &Path) -> HashMap<String, ConsentRecord>` — reads `<config_dir>/plugins.toml`; empty map if absent
  - `pub fn save_consent_record(config_dir: &Path, plugin_id: &str, record: &ConsentRecord) -> std::io::Result<()>` — atomic write (`.tmp` + rename)
  - `pub fn is_allowed(records: &HashMap<String, ConsentRecord>, plugin_id: &str) -> Option<bool>`
  - **Unit tests (write first)**: `test_load_returns_empty_map_when_file_absent`, `test_round_trip_persist_and_load`, `test_is_allowed_returns_none_for_unknown_plugin`

- [x] T006 Create `src/plugin/sandbox.rs` with `pub fn build_engine() -> rhai::Engine`:
  - `engine.set_max_operations(...)`, `set_max_call_levels(...)`, `set_max_string_size(...)`, `set_max_array_size(...)`, `set_max_map_size(...)` (document chosen caps)
  - `engine.set_max_modules(0)` and an empty module resolver to disable `import` (default-deny FS)
  - `engine.on_progress(...)` closure that aborts (returns `Some(...)`) when a shared wall-clock deadline is exceeded — deadline supplied per call by `dispatch_*`
  - `pub const PLUGIN_CALL_TIMEOUT_MS: u64 = 50;`
  - **Unit test (write first)**: `test_engine_aborts_runaway_operation_count` — a script with a huge loop returns `Err(ErrorTerminated)` (or operation-limit error), not a hang

- [x] T007 Create `src/plugin/api.rs` with `pub fn register_host_functions(engine: &mut rhai::Engine, shared: HostState)`:
  - `log(level: i64, msg: &str)` — validates/uses UTF-8; writes to structured editor log
  - `status_bar(msg: &str)` — queues message (≤ 120 chars, truncate) into shared `App` status state
  - `read_file(path: &str) -> Result<String, Box<EvalAltResult>>` — resolves the current plugin's approved `ReadPath` permissions; on undeclared path returns a Rhai error, logs the violation, and increments the instance `fs_violations` counter (disable after 3); on success returns file contents as a `String`
  - `HostState` is a cloneable handle (e.g. `Rc<RefCell<…>>` / `Arc<Mutex<…>>`) to the log sink, status queue, and current-plugin permission context
  - **Unit test (write first)**: `test_read_file_denied_for_undeclared_path`

- [x] T008 Create `src/plugin/registry.rs`:
  - `pub struct PluginRegistry { pub instances: Vec<PluginInstance>, pub disabled: Vec<String> }`
  - `pub fn highlighters_for(&self, ext: &str) -> Vec<&PluginInstance>` — active highlighter instances whose `extensions` contain `ext`, in load order (caller uses first; first-wins per spec)
  - `pub fn all_keybindings(&self) -> Vec<(String, String)>` — `(key_seq, action_name)` from active keybinding manifests
  - `pub fn menu_items(&self) -> Vec<PluginMenuItem>` where `pub struct PluginMenuItem { pub menu: String, pub item: String, pub item_id: String, pub plugin_id: String, pub position: Option<u32> }`
  - `pub fn disable(&mut self, plugin_id: &str)` — sets the instance `disabled = true` and records the id in `disabled`
  - **Unit tests (write first)**: `test_highlighters_for_returns_matching_extension`, `test_highlighters_for_empty_when_no_match`, `test_disable_marks_instance_and_records_id`

- [x] T009 Implement `src/plugin/mod.rs` body with `pub struct PluginHost { engine: rhai::Engine, registry: PluginRegistry, no_plugins: bool }` and `impl PluginHost`:
  - `pub fn new(no_plugins: bool) -> Self` — builds the shared engine via `build_engine()` + `register_host_functions()`
  - `pub fn load_all(&mut self, config_dir: &Path, consent: &HashMap<String, ConsentRecord>, pending_consent: &mut Vec<Plugin>)` — if `no_plugins`, returns immediately; else scans `<config_dir>/plugins/` subdirs, `parse_manifest` each, applies consent (`Some(true)`→compile script to AST + register; `Some(false)`→skip; `None`→push to `pending_consent`); compiles `plugin.rhai` via `engine.compile_file`/`compile`; aggregates `[keybindings]` and `[[menu_items]]`
  - `pub fn dispatch_highlight(&mut self, ext: &str, line: &str) -> Vec<HighlightToken>` — **stub returning `vec![]` in this task** (real body in T014)
  - `pub fn dispatch_menu_action(&mut self, plugin_id: &str, item_id: &str, buf: &str) -> Option<String>` — **stub returning `None` in this task** (real body in T024)
  - `pub fn registry(&self) -> &PluginRegistry`

- [x] T010 [P] Add `pub no_plugins: bool` (default `false`) to `Config` in `src/config/schema.rs`; add `--no-plugins` flag (`ArgAction::SetTrue`) to the `clap` CLI in `src/main.rs`; wire it into `config.no_plugins`

- [x] T011 Add `pub plugin_host: plugin::PluginHost`, `pub pending_plugin_consent: Vec<plugin::Plugin>` to `App` in `src/app.rs`; initialize from `PluginHost::new(config.no_plugins)` in `App::new()`; call `load_all()` after config load; populate `pending_plugin_consent`

- [x] T012 [P] Commit shared bad-plugin text fixtures (depended on by Phase 3 & Phase 7):
  - `tests/fixtures/plugins/infinite-loop/plugin.toml` (id="infinite-loop", types=["highlighter"], extensions=[".lua"], host_api="^1") + `plugin.rhai` containing `fn highlight(line, ext) { loop {} }`
  - `tests/fixtures/plugins/fs-violation/plugin.toml` (id="fs-violation", types=["menu"], host_api="^1", `[[menu_items]]` menu="Tools" item="Leak" item_id="leak") + `plugin.rhai` whose `menu_action` calls `read_file("/etc/passwd")`

**Checkpoint**: `cargo test` green; `cargo clippy -- -D warnings` clean.

---

## Phase 3: User Story 1 — Syntax Highlighter Plugin (Priority: P1) 🎯 MVP

**Goal**: A highlighter plugin produces visible token colouring on file open.

**Independent Test**: Drop `tests/fixtures/plugins/lua-syntax/` into the plugin dir; open a
`.lua` file; verify `Comment` tokens are present for `--` lines.

- [x] T013 [US1] Build the `lua-syntax` reference plugin under `examples/plugins/lua-syntax/`: `plugin.toml` (id="lua-syntax", types=["highlighter"], extensions=[".lua",".luac"], host_api="^1") + `plugin.rhai` with `fn highlight(line, ext)` returning an array of `#{start,end,kind}` maps — `"comment"` for `--`-prefixed spans, `"keyword"` for Lua keywords. Copy both files to `tests/fixtures/plugins/lua-syntax/`.

- [x] T014 [US1] **Tests first** in `tests/integration/plugin_api.rs` + a unit test in `src/plugin/mod.rs`:
  - `test_highlighter_plugin_loads_and_returns_tokens` — load `lua-syntax` fixture; `dispatch_highlight(".lua", "-- comment")` returns ≥1 `Comment` token
  - unit `test_validate_tokens_discards_overlapping` — call the token-validator with a hand-built overlapping `Vec` and assert it returns empty (no fixture needed)
  - `test_highlighter_returning_overlaps_discarded` — a fixture script returning overlapping tokens yields `vec![]`, plugin NOT disabled
  - `test_highlighter_timeout_disables_plugin` — load `tests/fixtures/plugins/infinite-loop/` (from T012); `dispatch_highlight` returns `vec![]` within 200 ms and the plugin appears in `registry().disabled`

- [x] T015 [US1] Implement the real `dispatch_highlight()` in `src/plugin/mod.rs`: for the first active highlighter matching `ext`, set the `on_progress` deadline to now+50 ms, call `engine.call_fn::<rhai::Array>(&mut inst.scope, &inst.ast, "highlight", (line.to_string(), ext.to_string()))` inside `std::panic::catch_unwind`; convert each `Map` to a `HighlightToken` via a `validate_tokens()` helper (bounds, no-overlap, `TokenKind::from_str`); on `ErrorTerminated`/error/panic mark the plugin disabled and return `vec![]`; on invalid tokens return `vec![]` without disabling

- [x] T016 [US1] Extend `EditorWidget::render()` in `src/ui/editor.rs`: call `app.plugin_host.dispatch_highlight(ext, line)` per visible line and apply the `Theme` colour per `TokenKind`; add six colour fields (`syntax_keyword/string/comment/number/operator/type`) to `Theme` in `src/ui/theme.rs` with sensible DOS-palette defaults. **Precedence (spec edge case)**: when an active plugin highlighter matches the file extension, it takes precedence over any built-in highlighter for that extension; the built-in highlighter is used only as the fallback when no active plugin matches (returns empty tokens or no plugin registered for the ext)

**Checkpoint**: `cargo test --test plugin_api` green; `make ci-local` green.

---

## Phase 4: User Story 2 — Custom Keybindings Plugin (Priority: P2)

**Goal**: A keybinding plugin's `[keybindings]` manifest section adds key→action mappings.

**Independent Test**: Install a plugin with `[keybindings] "F9" = "save"`; assert the binding is
present and F9 saves.

- [x] T017 [US2] Create `examples/plugins/custom-keys/plugin.toml` (id="custom-keys", types=["keybinding"], host_api="^1", `[keybindings] "F9" = "save"`); copy to `tests/fixtures/plugins/custom-keys/plugin.toml` (manifest-only — no `.rhai`)

- [x] T018 [US2] **Tests first** in `tests/integration/plugin_api.rs`:
  - `test_keybinding_plugin_maps_f9_to_save` — load `custom-keys` fixture; `registry().all_keybindings()` contains `("F9","save")`; dispatch `Action::Save` via simulated F9 on an `App` with a temp file; assert the file was written
  - `test_keybinding_conflict_logged` — a fixture declaring `"Ctrl+S" = "quit"`; assert the conflict is logged and `Ctrl+S` still maps to `Action::Save` (safety-critical action not overridden)

- [x] T019 [US2] In `PluginHost::load_all()`, parse each active plugin's `[keybindings]` table; validate each key-seq is non-empty UTF-8 and each action resolves via `action_from_str()` in `src/input/keymap.rs`; log+skip invalid bindings

- [x] T020 [US2] In `App::new()` after `load_all()`, merge `all_keybindings()` into the `KeybindingMap` (plugin precedence; log conflicts at `warn`); safety-critical actions (`Action::Quit`, `Action::Save`) MUST NOT be overrideable — log and discard such conflicts

**Checkpoint**: `cargo test --test plugin_api` green.

---

## Phase 5: User Story 3 — Menu Item Plugin (Priority: P3)

**Goal**: A menu plugin adds "Tools > Word Count"; selecting it shows the count in the status bar.

**Independent Test**: Load word-count fixture; activate `Action::PluginMenuActivated("word-count","wc")`; assert `app.status_message` contains a number.

- [x] T021 [US3] Build the `word-count` reference plugin under `examples/plugins/word-count/`: `plugin.toml` (id="word-count", types=["menu"], host_api="^1", `[[menu_items]] menu="Tools" item="Word Count" item_id="wc"`) + `plugin.rhai` with `fn menu_action(item_id, buf_content)` returning `#{ status: "ok", message: "Word count: " + <count> }`. Copy both to `tests/fixtures/plugins/word-count/`.

- [x] T022 [US3] **Tests first** in `tests/integration/plugin_api.rs`:
  - `test_menu_plugin_registers_item` — load word-count fixture; `registry().menu_items()` contains `{menu:"Tools", item:"Word Count", item_id:"wc"}`
  - `test_menu_action_sets_status_bar` — dispatch `Action::PluginMenuActivated("word-count","wc")` on an `App` whose buffer has 5 words; assert `app.status_message` contains `"5"`

- [x] T023 [US3] In `PluginHost::load_all()`, collect `[[menu_items]]` from active manifests into `registry().menu_items()`; validate `menu`/`item`/`item_id` are non-empty UTF-8

- [x] T024 [US3] Implement the real `dispatch_menu_action()` in `src/plugin/mod.rs`: call `menu_action(item_id, buf)` via `engine.call_fn::<rhai::Map>` with the 50 ms deadline + `catch_unwind`; read `status`/`message`; return the message string; on error/timeout disable the plugin and return a warning string. Add `Action::PluginMenuActivated(String, String)` to `Action` in `src/input/keymap.rs`; add the `handle_action` arm in `src/app.rs` setting `status_message`

- [x] T025 [US3] **Resolved by feature 009** (see `specs/009-menu-bar-activation/`, issue #19). Extend `MenuBarWidget` in `src/ui/menubar.rs` to render plugin-declared top-level menus after "Options" (one per unique `menu` value from `registry().menu_items()`); activating an item dispatches `Action::PluginMenuActivated`. New top-level menus appear between Options and Help (per spec edge case). Deferred because the menu-bar item-selection event path is not yet wired for built-in menus either; belongs with a broader menu-interaction pass. The registry, sandboxed `menu_action` dispatch, consent dialog, and plugin manager (T021–T024) are all complete.

**Checkpoint**: `cargo test --test plugin_api` green.

---

## Phase 6: User Story 4 — Plugin Manager + Consent UI (Priority: P4)

**Goal**: Options > Plugins lists installed plugins with enable/disable toggle persisted to
`plugins.toml`; first-run plugins prompt for consent before loading (US4 AC-3).

**Independent Test**: Load two fixtures; open dialog; toggle one off; close; assert
`plugins.toml` records `allowed = false`. Separately: an unconsented plugin shows a consent
prompt and declining persists `allowed = false`.

- [x] T026 [US4] **Tests first** in `tests/integration/plugin_api.rs`:
  - `test_plugin_manager_toggle_disable_persists` — two fixture plugins; toggle-disable first; close; `plugins.toml` shows `allowed = false`
  - `test_plugin_manager_reenable_persists` — disabled plugin in `plugins.toml`; toggle-enable; close; `plugins.toml` shows `allowed = true`
  - `test_consent_decline_persists_denied` — unconsented fixture in `pending_plugin_consent`; simulate decline; `plugins.toml` shows `allowed = false`; plugin not loaded
  - `test_consent_accept_loads_plugin` — simulate accept; `plugins.toml` shows `allowed = true`; plugin active

- [x] T027 [US4] Add `Action::OpenPluginManager` to `Action` in `src/input/keymap.rs`; add `MenuItem { label: "Plugins…", action: Action::OpenPluginManager }` to the Options menu slice in `src/ui/menubar.rs`

- [x] T028 [US4] Create `src/ui/plugin_manager.rs` — `pub struct PluginManagerDialog<'a>` { registry, theme, cursor }; `impl Widget`: centered DOS-themed overlay, scrollable list `[✓]/[ ] name  version  type(s)`, hint line `"  [↑↓] Navigate  [Space] Toggle  [Esc] Close  "`; clamp/truncate names with `…` under 40 cols; when `--no-plugins`, show "Plugins disabled (--no-plugins)"

- [x] T029 [US4] Add `pub pending_plugin_manager: bool`, `pub plugin_manager_cursor: usize` to `App`; in `handle_action`: `OpenPluginManager`→open; `MoveUp/MoveDown`→move cursor (wrap); `Confirm` (Space)→toggle and `save_consent_record(allowed=...)`; `MenuClose` (Esc)→close

- [x] T030 [US4] Implement consent prompt: add `pub pending_consent_cursor`/state to `App`; in `Ui::render()` (`src/ui/mod.rs`) render a centered consent overlay when `pending_plugin_consent` is non-empty — list first pending plugin's name, version, publisher, and requested permissions; hint `"  [Enter] Allow  [Esc] Deny  "`; handle in `handle_action`: `Confirm`→`save_consent_record(allowed=true)` + load that plugin, pop; `MenuClose`→`save_consent_record(allowed=false)`, pop

- [x] T031 [US4] In `Ui::render()` (`src/ui/mod.rs`) add the `PluginManagerDialog` overlay branch when `app.pending_plugin_manager` (after existing dialog overlays)

**Checkpoint**: `cargo test --test plugin_api` green.

---

## Phase 7: User Story 5 — Sandboxed Execution (Priority: P5)

**Goal**: Misbehaving plugins (infinite loop, FS violation, runtime error) are contained within
200 ms; editor and buffers survive intact. (Bad-plugin fixtures already committed in T012.)

**Independent Test**: Load infinite-loop fixture; `dispatch_highlight` returns `vec![]` within
200 ms and the plugin is disabled.

- [x] T032 [US5] **Tests** in `tests/integration/plugin_api.rs`:
  - `test_infinite_loop_terminated_within_200ms` — infinite-loop fixture; `dispatch_highlight(".lua","x")` returns within `Duration::from_millis(200)`; plugin in `registry().disabled`
  - `test_runtime_error_disables_plugin` — a fixture whose `highlight` throws (e.g. indexes out of range); `dispatch_highlight` returns `vec![]`; plugin disabled; `app.active_buffer()` still accessible (editor intact)
  - `test_undeclared_fs_path_denied` — fs-violation fixture (from T012); `dispatch_menu_action("fs-violation","leak", …)` → `read_file` denied; violation logged; editor intact; after 3 violations plugin disabled

- [x] T033 [US5] Verify the `on_progress` deadline mechanism in `src/plugin/sandbox.rs`/`mod.rs` correctly resets per call and never leaves a stale deadline; add unit test `test_deadline_resets_between_calls` (a slow-but-legal call after a timed-out call still succeeds)

- [x] T034 [US5] Confirm `std::panic::catch_unwind` wraps every `call_fn` dispatch boundary in `mod.rs`; add unit test `test_panic_in_dispatch_is_contained` (simulate via a host fn that panics, or a script error) asserting the host returns cleanly and disables the plugin

**Checkpoint**: `cargo test --test plugin_api` fully green.

---

## Phase 8: Polish & Docs Gate

- [x] T035 [P] Write reference-plugin READMEs: `examples/plugins/lua-syntax/README.md` (no build step — just copy `plugin.toml` + `plugin.rhai` into the plugin dir; document the `plugin.toml` schema and the `highlight`/`menu_action` script contract); equivalent READMEs for `word-count` and `custom-keys`

- [x] T036 [P] Write smoke test `tests/smoke/plugin_highlighter.exp`: launch editor on a temp `.lua` file with the lua-syntax fixture installed (pre-consented via `plugins.toml`); assert terminal output contains at least one ANSI colour-change sequence (proxy for token colouring); exit cleanly

- [x] T037 [P] Write `tests/smoke/plugin_startup_perf.sh` (SC-003): install 10 copies of a trivial highlighter fixture (distinct ids), launch `./edit` headless, measure cold start, assert ≤ 2 s; wire into `make perf-check`

- [x] T038 [P] Update `CHANGELOG.md` — feature 008 entry: "Plugin API (Rhai embedded scripting): syntax highlighters, custom keybindings, menu items; default-deny sandbox with 50 ms per-call time limit; consent dialog; Options > Plugins manager; `--no-plugins` flag; reference plugins in `examples/plugins/`. Pure-Rust, no new C/C++ deps."

- [x] T039 [P] Update `docs/STATUS.md` — add F008-US1…US5 rows (Complete); bump dev version

- [x] T040 [P] Update `docs/CAPABILITIES.md` — add `--no-plugins` CLI flag row; "Plugins…" Options-menu item; plugin-provided menus note

- [x] T041 [P] Update `man/edit.1` — `--no-plugins` in OPTIONS; "Plugins…" in the Options-menu description

- [x] T042 [P] Update `ROADMAP.md` — change Plugin API from "Deferred" to "Complete as of 2026-06-19 (feature 008)"; note Rhai engine, default-deny sandbox, on_progress time limit

- [x] T043 Run `make ci-local` (fmt → clippy `-D warnings` → tests → smoke → perf-check) AND `make static` (verify the static binary still links with `rhai` added); fix all regressions

- [x] T044 Close GitHub issue #2 with a comment referencing the merged PR

---

## Dependencies & Execution Order

- **Phase 1** → no deps
- **Phase 2** → Phase 1; **BLOCKS all US phases**. Within: T002→T003→T004→T005→T006→T007→T008→T009 sequential (shared modules); T010 [P] (different files); T011 after T009+T010; T012 [P] (fixtures, anytime in phase)
- **Phase 3 (US1)** → Phase 2 (incl. T012 infinite-loop fixture). T013→T014→T015→T016
- **Phase 4 (US2)** → Phase 2. T017→T018→T019→T020
- **Phase 5 (US3)** → Phase 2. T021→T022→T023→T024→T025
- **Phase 6 (US4)** → Phase 2 (consent state from T011). T026→T027→T028→T029→T030→T031
- **Phase 7 (US5)** → Phase 3 (needs real `dispatch_highlight`) + T012 fixtures. T032→T033→T034
- **Phase 8** → Phases 3–7. T035–T042 all [P]; T043→T044 sequential

### Parallel opportunities
- T010, T012 parallel with the T003–T009 module chain
- US phases 3/4/5/6 can each start once Phase 2 is done (they touch mostly different files; coordinate edits to `src/plugin/mod.rs`, `src/app.rs`, `src/input/keymap.rs`, `src/ui/mod.rs`)
- All Phase 8 docs/test-authoring tasks (T035–T042) are independent

---

## Implementation Strategy

### MVP (User Story 1)
1. Phase 1 Setup (T001) → 2. Phase 2 Foundational (T002–T012, all unit tests green) → 3. Phase 3 (T013–T016) → 4. STOP & validate quickstart Scenario 1.

### Incremental delivery
Setup+Foundational → US1 highlighter → US2 keybindings → US3 menu → US4 manager+consent →
US5 sandbox tests → Polish.

---

## Notes

- TDD: every phase lists its test task(s) before the implementation they cover (Constitution V)
- `cargo test` + `cargo clippy -- -D warnings` MUST stay green after every task
- Fixtures are plain `plugin.toml` + `plugin.rhai` text — no compilation, no WASM toolchain, committed directly; they double as the `examples/plugins/` sources
- Rhai's base language has no io/fs/process/network; `import` is disabled — the only FS access is the permission-gated `read_file` host fn (Constitution VII; FR-006)
- All plugin-provided strings (manifest + script-returned) are UTF-8 validated by the host before use (Constitution II; FR-011); Rhai strings are already Rust `String`
- Buffer state is never exposed to scripts; all interaction goes through host functions
- `pending_plugin_consent`, `pending_plugin_manager` are additive to existing `pending_*` App fields
- No `extism`, no `rmp-serde`, no `wasm32-*` target, no FreeBSD `cfg` special-casing — Rhai is pure Rust on every Tier-1 target
