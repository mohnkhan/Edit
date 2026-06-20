# Implementation Plan: File Browser Dialogs

**Branch**: `012-file-browser-dialogs` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/012-file-browser-dialogs/spec.md`

## Summary

Replace the two bare path-text dialogs (`OpenFileDialog`, `SaveAsFileDialog`) with a single
**navigable file browser** modal that lists the current directory's folders and files, supports
descending/ascending the tree, and is driven identically by keyboard and mouse. A new
`FileBrowser` state object (mode = Open/Save, cwd, sorted entries, selection, scroll, filename
field, error) becomes the single source of truth; a `FileBrowserWidget` renders it, and a shared
hit-test maps clicks to entries — mirroring the feature-009/011 menu pattern where one model drives
both render and mouse hit-testing. The browser reuses the existing modal-intercept/overlay flow
(`pending_*` in `app.rs`), the mouse routing in `App::handle_mouse_event`, the path sanitizer in
`src/security/sanitize.rs`, and `Buffer::open` / `Buffer::save_as`.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV 1.74.0)

**Primary Dependencies**: ratatui 0.26 (widgets), crossterm (key/mouse events), `std::fs` for
directory listing, `unicode-segmentation` + existing wide-char helpers for UTF-8-correct truncation.
No new crates.

**Storage**: N/A (transient dialog state only). Reads directory entries on demand.

**Testing**: `cargo test` (unit + integration), `expect`+tmux smoke. TDD per Constitution V.

**Target Platform**: Linux x86_64/aarch64 (+ FreeBSD/macOS per constitution); headless terminals.

**Project Type**: Single-project desktop TUI application (Rust binary `edit`).

**Performance Goals**: Keystroke/click→render ≤ 50 ms (Constitution). Directory listings are read
once per navigation and are small (hundreds of entries typical); rendering is O(visible rows).

**Constraints**: UTF-8/wide-char-correct rendering and truncation of all names (Principle II); all
paths validated through `src/security/sanitize.rs` before any read/write (Principle VII); no new
dependencies (Principle IV); existing dialogs' modal precedence and Esc-cancel preserved.

**Scale/Scope**: One browser model + widget; replaces 2 dialog widgets and 2 `Option<String>`
state fields. Touch points: `src/ui/file_browser.rs` (new), `src/app.rs` (state, intercept, mouse,
action wiring, save-on-unnamed), `src/ui/mod.rs` (render overlay + module decl), `src/ui/dialog.rs`
(remove the two obsolete widgets).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Assessment |
|---|---|
| **I. Platform-Native, DOS-Faithful UI** | ✅ A blue-box file picker with keyboard + DOS-style navigation is squarely DOS-faithful and improves the native feel. Degrades with no color via existing theme path. |
| **II. UTF-8 First (NON-NEGOTIABLE)** | ✅ Directory names are OS strings rendered as UTF-8 (lossy-decoded for display); truncation is grapheme/width-aware so multi-byte names never split. No raw-byte buffer paths introduced. |
| **III. Portable Build** | ✅ Pure `std::fs` + ratatui; no platform-specific code, no new deps. |
| **IV. Minimal Footprint** | ✅ No new dependencies; static build unaffected. |
| **V. Test-Gated Merges (NON-NEGOTIABLE)** | ✅ TDD: unit tests for browser navigation/sort/hit-testing; integration + expect smoke for end-to-end open/save by browsing. |
| **VI. Simplicity / YAGNI** | ✅ One model type + one widget; reuses modal + mouse infrastructure. Hidden-file toggle, multi-select, and previews explicitly out of scope. |
| **VII. Security Hardening** | ✅ Every open/save path goes through `validate_path`; directory reads handle permission errors gracefully (no crash, surfaced notice). No new attack surface. |

**Gate result: PASS.** No violations; Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/012-file-browser-dialogs/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── file-browser-interaction.md   # keyboard + mouse + dispatch contract
├── checklists/
│   └── requirements.md  # spec quality checklist (from /speckit-specify)
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── ui/
│   ├── file_browser.rs   # NEW: FileBrowser model (mode/cwd/entries/selection/scroll/filename/
│   │                     #      error) + navigation methods + FileBrowserWidget + hit_test +
│   │                     #      grapheme-aware name truncation
│   ├── dialog.rs         # MODIFY: remove OpenFileDialog / SaveAsFileDialog (superseded)
│   └── mod.rs            # MODIFY: declare file_browser module; render the browser overlay
├── app.rs                # MODIFY: replace pending_open/pending_save_as with
│                         #         file_browser: Option<FileBrowser>; route Action::Open/SaveAs/
│                         #         Save-on-unnamed; modal intercept (keys); mouse routing in
│                         #         handle_mouse_event; call Buffer::open / do_save_as on confirm
└── security/sanitize.rs  # REUSE unchanged (validate_path)

tests/
├── integration/
│   └── file_browser.rs   # NEW: end-to-end open-by-browse + save-by-browse
└── smoke/
    └── file_browser.exp  # NEW: headless keyboard + mouse navigation
```

**Structure Decision**: Single-project layout (existing). The browser is a new UI module consumed
by the app event loop and renderer; no other subsystem changes.

## Architecture Decisions (detail)

1. **One `FileBrowser` model drives render + hit-testing.** Like `resolve_menus`/`hit_test_menu`
   (feature 009/011), the same struct the widget renders is the one the mouse handler hit-tests, so
   clicks always match what is drawn. Geometry helpers (visible row range, row→entry index) are
   shared, not duplicated.

2. **Replace the two `Option<String>` fields with `file_browser: Option<FileBrowser>`.** The Open
   and Save dialogs are the same widget parameterized by `BrowseMode`. This removes
   `pending_open` / `pending_save_as` and the `OpenFileDialog` / `SaveAsFileDialog` widgets.

3. **Modal intercept mirrors the existing `pending_open` block** in `handle_action`, placed among
   the other modal guards (same precedence: below higher-priority modals, above the menu-bar guard).
   Keys: ↑/↓ move; Enter/→ activate (enter dir / pick file / save); ←/Backspace-at-empty-field go to
   parent; Esc cancel; printable chars edit the filename/path field.

4. **Mouse handled in `App::handle_mouse_event`** (feature 011). A left-press inside the browser box
   is hit-tested: clicking an entry acts on it directly (enter folder / pick file); clicking outside
   cancels. Reuses the existing modal-precedence guard already in that method.

5. **Listing built with `std::fs::read_dir`**, sorted `..` → directories → files, each
   case-insensitive alphabetical; dot-files included. Permission errors set the browser's `error`
   notice and keep the previous directory rather than navigating.

6. **All paths validated via `validate_path`** before `Buffer::open` (Open) or `Buffer::save_as`
   (Save). Save also records `self_write_times` and clears `modified`, exactly like the current
   `do_save_as`. Note: `validate_path` canonicalises (requires existence) so it suits Open targets
   and existing directories; for a brand-new Save filename the directory is validated and the file
   name is joined to the validated directory (see research.md).

7. **`Ctrl+S` on an unnamed buffer opens the Save browser** (clarified). `handle_save_action`
   routes to the browser when `active_buffer().path` is `None`.

## Complexity Tracking

No constitution violations; section intentionally empty.
