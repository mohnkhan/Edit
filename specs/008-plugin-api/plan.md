# Implementation Plan: Plugin API

**Branch**: `008-plugin-api` | **Date**: 2026-06-19 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/008-plugin-api/spec.md`

## Summary

Introduce a **Rhai-based** plugin system (pure-Rust embedded scripting) that lets third-party
developers add syntax highlighters, custom keybindings, and menu items to the editor without
modifying its source. A plugin is a directory in `$XDG_CONFIG_HOME/edit/plugins/<id>/` containing
a `plugin.toml` manifest and a `plugin.rhai` source script (parsed to an AST at load — no
compilation, no binary artifacts). A one-time consent dialog gates each new plugin; the plugin
manager (Options > Plugins) lets users enable/disable plugins. All plugin execution runs in a
default-deny Rhai sandbox with a per-call wall-clock time limit (50 ms default) and explicit,
permission-gated host functions, satisfying Constitution Principle VII.

## Technical Context

**Language/Version**: Rust stable, edition 2021; MSRV 1.74.0 (unchanged)

**Primary Dependencies** (new):
- `rhai = "1"` — pure-Rust embedded scripting engine; no C/C++ dependencies; statically links
  trivially; builds on every target including FreeBSD. Provides the sandbox primitives
  (`on_progress` deadline, `set_max_*` resource caps, default-deny base language).

Dropped vs the earlier WASM exploration: `extism` and `rmp-serde` are NOT used. Rhai
`Array`/`Map` values convert directly to/from Rust types, so there is no serialization boundary.
Existing `serde`, `toml`, `semver` are reused with no version bumps.

**Storage**:
- Plugin files: `$XDG_CONFIG_HOME/edit/plugins/<id>/plugin.{toml,rhai}` (user-managed)
- Consent records: `$XDG_CONFIG_HOME/edit/plugins.toml` (TOML, human-readable)

**Testing**:
- Unit: `cargo test` — manifest parsing, consent logic, registry queries, token validation
- Integration: `cargo test --test plugin_api` — full load/call/timeout/isolation cycle using
  `.rhai` + `.toml` text fixtures (no compile step)
- Smoke: `tests/smoke/plugin_highlighter.exp` — expect script for the highlight + menu flows
- Perf: `tests/smoke/plugin_startup_perf.sh` — startup ≤ 2 s with 10 plugins installed (SC-003)
- Reference plugins: `examples/plugins/{lua-syntax,custom-keys,word-count,infinite-loop,fs-violation}/`

**Target Platform**: Linux x86_64/aarch64, macOS, FreeBSD ≥ 13 — **all Tier 1**. Because Rhai is
pure Rust, the plugin subsystem compiles and runs on every target with no `cfg` special-casing
(a decisive advantage over a WASM runtime, which is Tier 3 on FreeBSD).

**Project Type**: CLI terminal application (existing); plugin subsystem is a new internal module
`src/plugin/`.

**Performance Goals**:
- Plugin load (all plugins): < 500 ms total at startup; < 50 ms per plugin parse+instantiate
- Per-line highlight call: ≤ 50 ms enforced by `on_progress` deadline; target < 2 ms for
  well-written scripts
- Menu action call: ≤ 50 ms enforced; no visible lag beyond 1-frame latency
- Startup ≤ 2 s with 10 plugins installed (SC-003, Constitution Principle VI baseline)

**Constraints**:
- One new Cargo dependency only (`rhai`); no transitive C/C++ libraries
- Binary size increase ≤ 1 MB (Rhai is ~500 KB); `make static` MUST continue to link cleanly
- `plugins.toml` consent file is never written without explicit user action
- Plugin callbacks that produce buffer mutations MUST go through host API functions, never
  directly into buffer state (Rhai scripts have no access to host memory regardless)
- The constitution's anticipated `plugins_enabled` config key is satisfied by the combination
  of per-plugin consent in `plugins.toml` (persistent, per-plugin allow/deny) and the
  `--no-plugins` session flag. A separate global `plugins_enabled` boolean is intentionally
  not added in v1 (it would be redundant with disabling every plugin via the manager);
  this decision is recorded here per Principle VI.

**Scale/Scope**: Up to 20 plugins loaded per session (practical limit; no artificial cap).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Status | Evidence |
|---|---|---|
| I — DOS-faithful UI | PASS | Plugin Manager dialog & consent dialog styled with theme colours; plugin menu items use existing menu infrastructure; keyboard-only navigation |
| II — UTF-8 First | PASS | `plugin.rhai`/`plugin.toml` validated UTF-8 on read; Rhai strings are Rust `String` (already UTF-8); all plugin-provided display strings validated before rendering (FR-011); non-UTF-8 → plugin disabled |
| III — Portable Build | PASS | Rhai is pure Rust — builds & passes tests on Linux, macOS, **and FreeBSD ≥ 13** with no platform `cfg`. No Tier-3 gap. |
| IV — Minimal Footprint | PASS | `rhai` only (~500 KB, zero C/C++ deps); `make static` links cleanly. The embedded scripting engine is statically compiled into the binary — not a separately-installed runtime — and is authorized by this feature's spec per Principle VI (see note below). |
| V — Test-Gated (TDD) | PASS | Test tasks ordered before implementation within each phase; integration suite covers timeout/crash/UTF-8/FS-violation paths; text fixtures need no toolchain |
| VI — Simplicity/YAGNI | PASS | No hot-reload, no marketplace, no LSP. The plugin API itself is a previously-deferred item (FR-015) now formally spec'd, which is exactly the gate Principle VI requires before adding an embedded scripting engine. |
| VII — Security Hardening | PASS | Default-deny sandbox (Rhai base language has no io/fs/process/network); `import` disabled; only permission-gated `read_file` host fn; `on_progress` 50 ms time limit; resource caps; `catch_unwind` dispatch boundary; consent dialog; all violations logged |

**Principle IV note (embedded scripting engine)**: Principle IV prohibits a *separately-installed
runtime* dependency (JVM, Python interpreter, etc.). Rhai is not that — it is a ~500 KB pure-Rust
library statically linked into the single `edit` binary, analogous to embedding an iconv or regex
library. Principle VI explicitly permits a plugin/scripting capability once it has a filed spec
and accepted user story; Feature 008 is that spec. No constitution amendment is required.

**Constitution Check post-design**: All gates PASS. The Rhai choice strengthens Principles III
and IV relative to the WASM alternative considered in research.md.

## Project Structure

### Documentation (this feature)

```
specs/008-plugin-api/
├── plan.md              # This file
├── research.md          # Rhai vs WASM vs dlopen; manifest; versioning; sandboxing; discovery
├── data-model.md        # Plugin, PluginInstance, ConsentRecord, HighlightToken entities
├── quickstart.md        # End-to-end validation scenarios
├── contracts/
│   └── plugin-rhai-api.md   # Host↔plugin Rhai API contract
└── tasks.md             # Generated by /speckit-tasks
```

### Source Code (additions to repository root)

```
src/plugin/
├── mod.rs               # PluginHost: load_all(), dispatch_highlight(), dispatch_menu_action()
├── manifest.rs          # parse_manifest() -> Plugin; validates id, semver, host_api, UTF-8 strings
├── registry.rs          # PluginRegistry: highlighters_for(), all_keybindings(), menu_items()
├── consent.rs           # load_consent_records(), save_consent_record(), ConsentRecord
├── sandbox.rs           # build_engine(): rhai::Engine with on_progress deadline, resource caps
├── api.rs               # register_host_functions(): log, read_file, status_bar
└── types.rs             # HighlightToken, TokenKind, PluginType, Permission, Plugin, PluginInstance

examples/plugins/
├── lua-syntax/          # Reference highlighter plugin (plugin.toml + plugin.rhai)
├── custom-keys/         # Reference keybinding plugin (manifest-only; no .rhai)
├── word-count/          # Reference menu plugin (plugin.toml + plugin.rhai)
├── infinite-loop/       # Test-only bad plugin: fn highlight(l,e){ loop{} }
└── fs-violation/        # Test-only bad plugin: calls read_file("/etc/passwd")

tests/integration/
└── plugin_api.rs        # Integration tests using text fixtures

tests/smoke/
├── plugin_highlighter.exp
└── plugin_startup_perf.sh

tests/fixtures/plugins/  # Plain-text fixtures (committed; no compilation)
├── lua-syntax/{plugin.toml,plugin.rhai}
├── custom-keys/plugin.toml
├── word-count/{plugin.toml,plugin.rhai}
├── infinite-loop/{plugin.toml,plugin.rhai}
└── fs-violation/{plugin.toml,plugin.rhai}

src/ui/plugin_manager.rs # PluginManagerDialog widget
src/app.rs               # Modified: plugin_host field, --no-plugins, consent + manager state
src/input/keymap.rs      # Modified: plugin keybindings merged; new Action variants
src/ui/mod.rs            # Modified: consent + PluginManagerDialog overlays
src/ui/menubar.rs        # Modified: Options>Plugins item; plugin-declared top-level menus
src/ui/editor.rs         # Modified: apply HighlightToken colours
src/ui/theme.rs          # Modified: six syntax-colour fields
src/config/schema.rs     # Modified: no_plugins: bool
src/main.rs              # Modified: --no-plugins CLI flag
Cargo.toml               # Modified: rhai; [[test]] plugin_api
CHANGELOG.md
docs/STATUS.md
docs/CAPABILITIES.md
man/edit.1
ROADMAP.md
```

## Implementation Phases

### Phase 1: Setup
1. Add `rhai = "1"` to `Cargo.toml`; add `[[test]] plugin_api`; confirm `cargo build` succeeds.

### Phase 2: Foundational (plugin infrastructure, no UI)
1. `src/plugin/types.rs` — all shared types incl. `PluginInstance`
2. `src/plugin/manifest.rs` — `parse_manifest()` with full validation
3. `src/plugin/consent.rs` — load/save `ConsentRecord`
4. `src/plugin/sandbox.rs` — `build_engine()` with `on_progress` deadline + resource caps
5. `src/plugin/api.rs` — register `log`, `status_bar`, `read_file` host functions
6. `src/plugin/registry.rs` — `PluginRegistry` with query methods
7. `src/plugin/mod.rs` — `PluginHost::load_all()` full load sequence (dispatch_* stubbed)
8. `--no-plugins` CLI flag and `no_plugins: bool` config field
9. Wire `PluginHost` into `App`; commit the shared infinite-loop & fs-violation text fixtures
   early (multiple later phases depend on them)
10. Unit tests for manifest parsing, consent persistence, registry queries, token validation

### Phase 3: US1 — Syntax Highlighter
1. Tests first: integration + token-validator unit tests
2. Implement `dispatch_highlight()` (Rhai `call_fn` + validation + `catch_unwind` + time limit)
3. Integrate token output into `EditorWidget` rendering; add `Theme` syntax colours
4. Build `examples/plugins/lua-syntax/` reference plugin (.toml + .rhai); mirror to fixtures

### Phase 4: US2 — Custom Keybindings
1. Tests first
2. Aggregate `[keybindings]` from active manifests; merge into keymap (plugin precedence; log
   conflicts; safety-critical actions non-overrideable)
3. Build `examples/plugins/custom-keys/` reference plugin (manifest-only)

### Phase 5: US3 — Menu Items
1. Tests first
2. Collect `[[menu_items]]`; render plugin-declared top-level menus; dispatch menu actions
3. Build `examples/plugins/word-count/` reference plugin (.toml + .rhai)

### Phase 6: US4 — Plugin Manager + Consent UI
1. Tests first
2. `src/ui/plugin_manager.rs` dialog; Options>Plugins; persist enable/disable to `plugins.toml`
3. Consent dialog rendering + handling (required by US4 AC-3 — implemented here, not deferred)

### Phase 7: US5 — Sandboxed Execution Tests
1. Timeout within 200 ms; editor survives (infinite-loop fixture)
2. Undeclared FS path denied + logged (fs-violation fixture)
3. Script runtime error → plugin disabled; editor continues
4. `on_progress` deadline correctness (resets per call; no stale deadline)

### Phase 8: Polish & Docs Gate
1. Smoke test + startup-perf test (SC-003)
2. Reference-plugin READMEs
3. Update `CHANGELOG.md`, `docs/STATUS.md`, `docs/CAPABILITIES.md`, `man/edit.1`, `ROADMAP.md`
4. Close GitHub issue #2; run `make ci-local` (incl. `make static` link check)

## Complexity Tracking

| Concern | Mitigation |
|---|---|
| Embedded scripting engine vs Principle IV | Pure-Rust ~500 KB, statically linked, spec-authorized per Principle VI; documented in Constitution Check |
| Plugin authors limited to Rhai | Accepted trade-off (multi-language authorship not a requirement); reference plugins + READMEs lower the barrier |
| Time-limit precision | `on_progress` wall-clock deadline + `set_max_operations` backstop; `catch_unwind` at dispatch boundary |
| ABI evolution | `HOST_PLUGIN_API_VERSION = 1`; manifest `host_api` semver range; additive-only host functions within v1.x |
| Static link regression | `make static` verified in CI gate (T044) |
