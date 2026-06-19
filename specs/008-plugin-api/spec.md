# Feature Specification: Plugin API

**Feature Branch**: `008-plugin-api`

**Created**: 2026-06-19

**Status**: Draft

**Input**: User description: "Feature 008: Plugin API — C FFI / WASM interface for
extensibility. Allow third-party developers to extend the editor with custom syntax
highlighters, keybindings, and menu items without modifying the editor source. The
plugin interface should be stable across patch releases. Deferred from v0.1.0 (Issue
#2) pending internal API stabilization, which is now achieved after features 001–007.
Two candidate delivery mechanisms: (a) native shared libraries via dlopen/C FFI, or
(b) WebAssembly modules via a WASM runtime. Plugins must not be able to corrupt editor
state, crash the process, or access filesystem paths outside their declared sandbox."

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Install and Activate a Syntax Highlighter Plugin (Priority: P1)

A developer publishes a syntax highlighter for the Lua language as a plugin file. A user
downloads it, places it in the editor's plugin directory (`~/.config/edit/plugins/`), and
restarts the editor. When they open a `.lua` file the correct token colours appear — keywords
are highlighted in a different colour from strings and comments — without the user modifying
the editor binary or any built-in configuration.

**Why this priority**: Syntax highlighting is the extension type users most commonly want to
add for unsupported languages. Demonstrating it proves the full plugin load-call-render cycle
works end-to-end, making it the best MVP proof.

**Independent Test**: Drop a pre-built test plugin that highlights words matching `--` in
bright red into `~/.config/edit/plugins/`; open a Lua file; verify the `--` comment tokens
are red; remove the plugin file; restart; verify they are no longer red.

**Acceptance Scenarios**:

1. **Given** a valid plugin file is present in the plugin directory, **When** the editor
   starts, **Then** the plugin is loaded and its highlighter is active for all configured
   file extensions.
2. **Given** a plugin is loaded, **When** the user opens a file whose extension matches the
   plugin's declared extension list, **Then** syntax tokens are coloured as the plugin defines.
3. **Given** a malformed or missing plugin file, **When** the editor starts, **Then** the
   editor starts normally, logs a warning, and the bad plugin is skipped; other plugins and
   built-in highlighting are unaffected.
4. **Given** a plugin that panics or crashes during highlighting, **When** the editor calls
   it, **Then** the crash is caught, the plugin is disabled for the session, a status-bar
   warning is shown, and the editor continues running normally.

---

### User Story 2 — Add Custom Keybindings via a Plugin (Priority: P2)

A developer publishes a "vim-motions" plugin that registers `jk` (insert mode) as Escape
and `dd` as delete-line. A user installs it; after restart the keybindings are active.
Pressing `jk` exits insert mode as expected. The plugin's bindings can co-exist with
built-in bindings; where they conflict the plugin's binding takes precedence and the
conflict is logged.

**Why this priority**: Custom keybindings serve power users and is the second most
requested extension type. The plugin mechanism for keybindings is simpler than rendering,
making it a good second story.

**Independent Test**: Install a test plugin that maps `F9` → `Action::Save`; open a file,
make a change, press `F9`; verify the file is saved on disk.

**Acceptance Scenarios**:

1. **Given** a keybinding plugin declares `F9 → SaveFile`, **When** the user presses F9,
   **Then** the save action executes.
2. **Given** a plugin binding conflicts with a built-in binding, **When** the editor loads,
   **Then** the conflict is logged; the plugin binding takes precedence; the user is not
   prompted unless the conflict involves a safety-critical action (Quit, Save).
3. **Given** a keybinding plugin is removed from the plugin directory, **When** the editor
   restarts, **Then** the custom binding is gone and the built-in binding (if any) is restored.

---

### User Story 3 — Add Custom Menu Items via a Plugin (Priority: P3)

A developer publishes a "word-count" plugin that adds a "Tools > Word Count" menu item.
When the user selects it, a status-bar message shows the word count for the active buffer.
The menu item appears in the correct position, styled with the editor's theme colours.

**Why this priority**: Menu items are the most visible extension surface and enable
plugin-provided commands to be discoverable without memorizing keybindings.

**Independent Test**: Install the word-count test plugin; open the "Tools" menu (new, plugin-
provided); select "Word Count"; verify the status bar shows the correct count.

**Acceptance Scenarios**:

1. **Given** a plugin declares a menu item `Tools > Word Count`, **When** the user opens
   the Tools menu, **Then** the item appears at the position declared by the plugin.
2. **Given** the user selects a plugin menu item, **When** the item's action runs, **Then**
   its result (status message, dialog, or buffer mutation) is applied without crashing the editor.
3. **Given** the plugin providing a menu item is removed, **When** the editor restarts,
   **Then** the menu item is gone and the menu renders correctly without the missing entry.

---

### User Story 4 — Plugin Manager: Discover, Enable, Disable (Priority: P4)

A user opens Options > Plugins from the menu and sees a list of all installed plugins with
their name, version, type (highlighter / keybinding / menu), and enabled/disabled status.
They toggle a plugin off; it takes effect after restart. They can re-enable it without
reinstalling.

**Why this priority**: Without a management UI, users have no visibility into what plugins
are active or why something behaves unexpectedly. The manager also provides the consent
flow required by Principle VII.

**Independent Test**: Install two plugins; open Options > Plugins; verify both appear;
disable one; restart; verify the disabled plugin's effect is absent; re-enable; restart;
verify it is active again.

**Acceptance Scenarios**:

1. **Given** one or more plugins are installed, **When** the user opens Options > Plugins,
   **Then** each plugin is listed with name, version, type, and enabled/disabled state.
2. **Given** a plugin is enabled, **When** the user disables it and restarts, **Then** the
   plugin is not loaded and has no effect.
3. **Given** a plugin is newly installed (first run), **When** the editor starts, **Then**
   the user is shown a one-time consent prompt ("Allow plugin X from publisher Y to run?")
   before the plugin is loaded; declining permanently disables the plugin.

---

### User Story 5 — Sandboxed Execution: Misbehaving Plugin Cannot Harm the Editor (Priority: P5)

A plugin attempts to read files outside its declared sandbox, consume unlimited memory, or
execute an infinite loop. The editor detects the violation, terminates the plugin's execution
context, logs the event, and continues running normally. The user's open files and unsaved
changes are unaffected.

**Why this priority**: Constitution Principle VII mandates plugin sandboxing. A plugin API
without a security model would be a blocking violation.

**Independent Test**: Run a test plugin that attempts to open `/etc/passwd`; verify the
attempt is rejected; verify the editor continues running; verify the violation is logged.

**Acceptance Scenarios**:

1. **Given** a plugin attempts to access a file path outside its declared sandbox, **When**
   the access is made, **Then** the access is denied; the plugin receives an error; the
   editor is not affected.
2. **Given** a plugin enters an infinite loop, **When** the execution time limit is exceeded,
   **Then** the plugin call is terminated; a status-bar warning is shown; the editor remains
   responsive.
3. **Given** a plugin causes an unrecoverable error (crash, OOM), **When** the error occurs,
   **Then** the plugin is disabled for the session; the editor and all open buffers survive intact.

---

### Edge Cases

- What if two plugins register the same file extension for syntax highlighting?
  → Both are loaded; the one appearing first in the plugin directory listing takes priority;
  a warning is logged; the user can resolve via the plugin manager ordering.
- What if a plugin highlighter and the built-in highlighter both handle the same extension
  (e.g. a plugin for `.py`, which has built-in Python highlighting)?
  → The plugin highlighter takes precedence for that extension while the plugin is active;
  the built-in highlighter is the fallback used only when no active plugin matches the
  extension. Disabling or removing the plugin restores built-in highlighting.
- What if a plugin declares an incompatible API version?
  → The plugin is rejected at load time with a clear error: "Plugin X requires API v2 but
  editor provides API v1." The editor starts without it.
- What if the plugin directory does not exist?
  → The editor starts normally; no plugins are loaded; no error is shown (the absence of
  the directory is the same as having zero plugins installed).
- What if a plugin is updated (file replaced) while the editor is running?
  → The running session continues using the old version. The update takes effect on next
  restart. No hot-reload in this version.
- What if a plugin registers a menu item in a menu that does not exist yet (e.g. "Tools")?
  → The editor creates the new top-level menu automatically; it appears between "Options"
  and "Help" in the menu bar.
- What if the editor is started with `--no-plugins`?
  → No plugins are loaded; the plugin directory is not scanned; Options > Plugins shows
  an empty list with a notice that plugins are disabled for this session.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor MUST scan `$XDG_CONFIG_HOME/edit/plugins/` (default
  `~/.config/edit/plugins/`) at startup for plugin files and attempt to load each one.
  Loading MUST complete before the first frame is rendered.
- **FR-002**: Each plugin MUST declare: a unique identifier, a human-readable name, a
  semantic version, the plugin API version it targets, and the extension type(s) it provides
  (highlighter, keybinding, menu).
- **FR-003**: The plugin system MUST support three extension types: (a) syntax highlighter —
  maps file extensions to token coloring rules; (b) keybinding — maps key sequences to
  editor actions; (c) menu item — adds items to existing or new top-level menus.
- **FR-004**: The plugin host MUST expose only a versioned, stable API surface to plugins.
  The host MUST reject plugins whose declared API version is newer than the host supports,
  with a human-readable error logged to the editor log file.
- **FR-005**: Plugins MUST run in an isolated execution context. A plugin MUST NOT be able
  to directly read or write the editor's internal buffer state; all interactions MUST go
  through the declared API.
- **FR-006**: Plugins MUST run in a default-deny filesystem sandbox: by default a plugin has
  **no** filesystem access of any kind (no read, no write, no directory listing). The content
  a plugin needs (the current line for highlighting, the active buffer text for menu actions)
  is provided to it by the host as call arguments — the plugin never reads the file itself.
  A plugin MAY declare additional read paths in its manifest; the user MUST be shown these at
  consent time (FR-010), and only declared paths become accessible (via the host-provided file
  read function). Write access is never granted by default.
- **FR-007**: The plugin host MUST enforce a per-call wall-clock time limit of 50 ms on plugin
  callbacks. Calls that exceed the limit MUST be terminated; the plugin MUST be disabled for the
  session; a status-bar warning MUST be shown. (The limit is a fixed constant in v1; a
  user-configurable limit is deferred per Principle VI — no user story requires tuning it.)
- **FR-008**: The editor MUST provide a `--no-plugins` CLI flag that suppresses all plugin
  loading for the session. This flag MUST NOT modify the persisted enabled/disabled state.
- **FR-009**: The editor MUST expose an Options > Plugins menu item that opens a plugin
  management dialog listing all installed plugins with name, version, type, and enabled/
  disabled state. The user MUST be able to toggle the enabled state of any plugin from this
  dialog; the change takes effect on next restart.
- **FR-010**: When a plugin is installed (present in the plugin directory for the first time),
  the editor MUST show a one-time consent dialog before loading it, listing the plugin's
  declared identity and requested sandbox permissions. The user must explicitly confirm before
  the plugin runs. Declining permanently disables the plugin (stored in
  `$XDG_CONFIG_HOME/edit/plugins.toml`).
- **FR-011**: The plugin manifest and all plugin-provided strings rendered in the UI (menu
  labels, dialog text) MUST be validated as UTF-8; non-UTF-8 bytes MUST be rejected and
  the plugin disabled, preserving Principle II compliance.
- **FR-012**: Plugin load errors, consent decisions, sandbox violations, and time-limit
  terminations MUST be written to the editor's structured log file. No errors are silently
  swallowed.
- **FR-013**: A reference example plugin (syntax highlighter for a simple language) MUST
  be included in the repository under `examples/plugins/` with step-by-step installation
  documentation (plugins are interpreted source scripts — installation is copying the
  manifest and script files into the plugin directory; no compilation or build script is
  required), to serve as the canonical starting point for plugin developers.

### Key Entities

- **Plugin**: A distributable unit of extension code with a declared manifest (id, name,
  version, API version, extension types, sandbox permissions).
- **PluginManifest**: The metadata block each plugin provides declaring its identity and
  capabilities; validated by the host before any plugin code runs.
- **PluginRegistry**: The in-memory list of loaded, active plugins maintained by the host
  for the duration of a session.
- **PluginPermissions**: The filesystem and capability grants a plugin declares; shown to
  the user at consent time and enforced at runtime.
- **ConsentRecord**: The persistent record of user consent decisions, stored in
  `plugins.toml`; prevents re-prompting for already-decided plugins.
- **PluginApiVersion**: A monotonically increasing integer (not semver) identifying the
  host ABI generation; incremented only when the API changes in a breaking way.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can write and install a working syntax-highlighter plugin
  using only the public API documentation and the reference example, without reading the
  editor's internal source code.
- **SC-002**: A plugin that misbehaves (infinite loop, FS violation) is terminated within
  200 ms of the violation, and the editor remains responsive to user input throughout.
- **SC-003**: The editor starts in under 2 seconds (existing performance baseline, Principle
  VI) with up to 10 plugins installed, measured on standard developer hardware.
- **SC-004**: 100% of plugin-provided strings rendered in the UI are validated as UTF-8
  before display; zero terminal escape-injection vulnerabilities introduced by plugin content.
- **SC-005**: The plugin API version number is stable across all patch releases within a
  given minor version; a plugin built against API v1.0 runs without modification on any
  1.x editor release.
- **SC-006**: The consent flow adds no more than one additional keypress (Enter to confirm
  or Escape to deny) to the first-run experience for each new plugin.

---

## Assumptions

- The plugin directory path follows the XDG Base Directory specification; `$XDG_CONFIG_HOME`
  defaults to `~/.config` if unset.
- Plugins are distributed as pre-built files; the editor does not compile plugin source code
  at runtime. Build instructions are the plugin developer's responsibility.
- Hot-reload (activating a plugin without restarting the editor) is out of scope for this
  version; plugins take effect on next start.
- Mouse support in plugin-provided menu items is out of scope; keyboard navigation only,
  consistent with the rest of the editor.
- Plugin developers are responsible for supplying pre-built artifacts for each target
  platform (Linux x86_64, aarch64, macOS, BSD) they wish to support.
- The `plugins.toml` consent file is stored in `$XDG_CONFIG_HOME/edit/`; it is a
  human-readable TOML file so users can inspect and manually edit consent decisions.
- Only one plugin manager dialog is in scope; a marketplace or auto-update mechanism is
  explicitly deferred.
- Plugins cannot extend the encoding pipeline (FR-011 ensures they cannot bypass UTF-8
  validation); encoding is a host-only concern.

## Dependencies

- **Features 001–007**: Internal API stabilization (buffer, UI, config, session, watcher)
  required before the plugin API surface could be defined without constant breakage. All
  complete.
- **Constitution Principle VII**: Plugin sandboxing and consent are non-negotiable
  requirements; the feature cannot ship without them.
- **Constitution Principle II**: All plugin-provided display strings must be UTF-8 validated
  by the host before rendering.
- **Downstream**: Future features that wish to be extensible (e.g. LSP client, macro
  recorder) will build on the plugin API defined here.
