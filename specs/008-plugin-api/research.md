# Research: Plugin API (Feature 008)

**Date**: 2026-06-19 | **Plan**: [plan.md](plan.md)

---

## Decision 1 — Plugin Delivery Mechanism

**Decision**: Plugins are **Rhai** scripts — pure-Rust embedded scripting (`rhai = "1"`) — not WebAssembly/Extism and not native shared libraries.

**Rationale**:

| Criterion | Native cdylib (dlopen) | WASM / Extism | **Rhai (chosen)** |
|---|---|---|---|
| Plugin crash kills editor | **Yes — always** (same process, SIGSEGV propagates) | No — trap returns `Err` | **No** — safe Rust, cannot segfault; all errors return `Err` |
| Binary size overhead | ~negligible | **2–15 MB** (Wasmtime + C++ stdlib) | **~500 KB**, pure Rust, zero C/C++ deps |
| Static linking (`make static`) | n/a | Painful — pulls C++ stdlib | **Trivial** — pure Rust |
| FreeBSD ≥13 build (Principle III) | Toolchain-dependent ABI | **Tier-3 / CI risk** | **Builds clean, zero cfg special-casing** |
| Sandboxing | Bolt-on only; NO crash isolation | Built-in linear-memory isolation | **Default-deny by construction** (see Decision 4) |
| Author languages | Rust only | Any WASM target | Rhai only (niche) |

**Key evidence**:
- **Constitution Principle IV (minimal, static-linkable footprint; EDIT.COM heritage ~70 KB).** Rhai adds roughly 500 KB, is pure Rust with no C/C++ dependencies, and `make static` works trivially. WASM via Wasmtime adds 2–15 MB and pulls in the C++ standard library, which makes static linking painful and bloats a tool whose entire heritage is measured in kilobytes.
- **Constitution Principle III (every commit must build and pass tests on FreeBSD ≥13).** Rhai is pure Rust and builds on FreeBSD with zero `cfg` special-casing. Wasmtime is a Tier-3 target on FreeBSD — a real, recurring CI risk we are unwilling to carry on every commit.
- **Sandboxing is first-class and default-deny** (detailed in Decision 4): the base Rhai language has no file, network, or process access at all.
- **No JavaScript anywhere.** Rhai's syntax resembles JS/Rust, but it is an independent, self-contained Rust library — no V8, no Node, no embedded JS runtime.

**Trade-off (stated honestly)**: plugin authors must write Rhai specifically, a niche language, where WASM would have allowed authorship in any source language. This was judged acceptable because multi-language plugin authorship is not a stated requirement for this project, and the footprint and FreeBSD wins are decisive.

**Alternatives considered**:
- **Native cdylib / dlopen** — rejected: a crashing native plugin kills the editor (shared address space, SIGSEGV propagates), there is no crash isolation, and the Rust ABI is fragile across compiler versions.
- **WASM via Extism / Wasmtime** — rejected *for this project*: 2–15 MB binary bloat, painful static linking (C++ stdlib), and Tier-3 FreeBSD build risk all strain Principle IV and Principle III. (A perfectly reasonable choice for a larger, GUI-class application — just not this one.)
- **Lua via `mlua`** — viable and backed by a huge ecosystem, but bundles a C dependency (vendored Lua) that complicates the pure-Rust static-link story. Rhai's zero-C-dependency edge decided it.

---

## Decision 2 — Plugin Format / Manifest

**Decision**: A sidecar `plugin.toml` manifest (engine-agnostic) alongside a `plugin.rhai` source script, both in the plugin directory `$XDG_CONFIG_HOME/edit/plugins/<id>/`. There is **no compilation step and no binary artifact** — the script is parsed to a `rhai::AST` at load time.

**Why sidecar manifest + source script**:
- The host needs plugin metadata — `id`, `version`, requested `permissions` — *before* it runs any script code, because that metadata drives the consent flow (FR-010). A separate TOML file makes this possible without executing untrusted code first.
- The TOML is human-readable, so users can audit a plugin's identity and requested permissions without any tooling.
- The manifest is engine-agnostic, so the format does not leak Rhai specifics; only the `plugin.rhai` script does.
- Shipping source (not a binary) keeps plugins inspectable and removes any build toolchain from the install path.

**Manifest schema** (in `plugin.toml`):
```toml
id          = "lua-syntax"              # kebab-case; globally unique
name        = "Lua Syntax Highlighter"  # UTF-8, ≤ 64 chars
version     = "1.0.0"                    # semver
host_api    = "^1"                       # semver range the plugin requires
types       = ["highlighter"]            # highlighter | keybinding | menu
extensions  = [".lua", ".luac"]          # highlighter type only
publisher   = "example.org"              # optional
description = "Highlights Lua source"    # optional

[keybindings]
# key = "action"

[[menu_items]]
# label = "..."; action = "..."

[permissions]
read_paths = []                          # paths the script may read via read_file()
write_dirs = []                          # directories the script may write into
```

The plugin directory therefore contains exactly two files: `plugin.toml` (metadata + permissions) and `plugin.rhai` (the script). No `.wasm`, no compiled cache.

---

## Decision 3 — API Versioning Strategy

**Decision**: A single integer API version in the host plus a semver range in the manifest, checked by the host at load. The manifest is authoritative.

- `HOST_PLUGIN_API_VERSION: i32 = 1` — a compile-time constant in the host. Incremented only on breaking changes to the host-registered functions.
- `plugin.toml: host_api = "^1"` — each plugin declares the semver range it targets. The host parses this with the `semver` crate and rejects any plugin whose range does not overlap the host's version.
- Unlike the previous WASM design, **no secondary in-script version export is required** — the manifest is the single source of truth, and the script is never run if the manifest range is incompatible. Optionally, the host may inject a `HOST_API_VERSION` constant into the script scope so a script can read it for its own conditional logic.
- Within a major API version (v1.x), new host functions may be added (additive); existing functions may not be removed or have their signatures changed. Additive changes do not bump the integer.

This delivers the "stable across patch releases" guarantee from FR-004 and SC-005 without any binary ABI stamp.

---

## Decision 4 — Sandboxing and Time Limits

**Decision**: Default-deny by construction using Rhai's engine configuration — no OS-level mechanisms required — plus a wall-clock deadline enforced through `Engine::on_progress`.

This is a Rhai strength: the sandbox is the *default state*, not something bolted on.

**Filesystem — default-deny**:
- The base Rhai language has **no file, io, network, or process access of any kind**. There is nothing to lock down because nothing is exposed.
- Script `import` of other scripts is disabled via `Engine::set_max_modules(0)` and an empty module resolver — a script cannot pull in additional code.
- The **only** filesystem access is a single host-registered `read_file(path)` function, and every call is gated against the plugin's declared `ReadPath` permissions from the manifest.
- This satisfies the sandbox requirement (FR-005, FR-006) **without** WASI mounts, seccomp, or Landlock.

**Time limit** (FR-007):
- An `Engine::on_progress` callback checks a wall-clock deadline (an `Instant` captured at the start of each call; default 50 ms) and returns `Some(token)` to abort. Rhai then returns `Err(EvalAltResult::ErrorTerminated)`, which unwinds cleanly back to the host.
- `Engine::set_max_operations` serves as a coarse backstop in case a script somehow avoids progress checks.
- `PLUGIN_CALL_TIMEOUT_MS: u64 = 50` is the host constant.

**Resource caps**: `set_max_operations`, `set_max_call_levels`, `set_max_string_size`, `set_max_array_size`, and `set_max_map_size` bound CPU and memory use per call.

**Crash isolation**: Rhai scripts run as safe Rust with no guest `unsafe`, so they **cannot segfault**. Every script outcome — including the timeout abort — comes back as an `Err` the host can handle. As belt-and-suspenders, the host additionally wraps each dispatch in `std::panic::catch_unwind`. On any timeout or error, the offending plugin is marked `disabled` for the session, a status-bar warning is shown, and the editor continues.

---

## Decision 5 — Plugin Discovery and Consent

**Decision**: Directory scan at startup → manifest parse → API compatibility check → consent check → compile to AST → register.

**Load sequence**:
1. Scan `$XDG_CONFIG_HOME/edit/plugins/` for subdirectories (each containing `plugin.toml` + `plugin.rhai`).
2. Parse `plugin.toml` for each; validate the schema and check `host_api` compatibility against `HOST_PLUGIN_API_VERSION`.
3. Check the consent record in `$XDG_CONFIG_HOME/edit/plugins.toml` for this plugin's `id`:
   - `allowed = true` → load.
   - `allowed = false` → skip.
   - Missing → show a one-time consent dialog listing the plugin's identity (name, version, publisher) and its requested permissions; persist the decision.
4. Compile `plugin.rhai` to a `rhai::AST`.
5. Register the plugin in the `PluginRegistry` and connect the appropriate extension hooks.

**`plugins.toml`** (at `$XDG_CONFIG_HOME/edit/plugins.toml`):
```toml
[plugins.lua-syntax]
allowed = true
consented_at = "2026-06-19T14:00:00Z"
version_consented = "1.0.0"

[plugins.evil-plugin]
allowed = false
```

**`--no-plugins` flag**: skips the entire scan, compilation, and registration; `plugins.toml` is never read or modified.

---

## Dependencies

- **Add**: `rhai = "1"` — pure-Rust embedded scripting engine.
- **Drop**: `extism` and `rmp-serde` — Rhai `Array`/`Map` values convert directly to and from Rust types, so there is no MessagePack marshalling boundary.
- **Reuse**: existing `serde`, `toml`, and `semver` crates (manifest parsing and version-range checks).
