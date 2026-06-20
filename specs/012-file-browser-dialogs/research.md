# Phase 0 Research: File Browser Dialogs

## R1 — Path validation for a brand-new Save file

**Decision**: Keep the browser's `cwd` as an **already-canonical absolute `PathBuf`**, obtained by
`std::fs::canonicalize` at open and on every navigation step. Resolve `..` ourselves with
`Path::parent()` (never pass a literal `..` to the sanitizer). For **Open**, validate the chosen
file with `validate_path(cwd.join(name))` — the file exists, so canonicalization succeeds. For
**Save**, validate the *directory* (`validate_path(cwd)`, which exists) and require the filename to
be a single plain component (no `/`, no `..`, non-empty); the save target is `cwd.join(filename)`.

**Rationale**: `validate_path` (src/security/sanitize.rs:136) rejects any `..` component and calls
`std::fs::canonicalize`, which fails on non-existent paths — so it cannot validate a new file path
directly. Validating the (existing) directory plus a sanitized single-segment name preserves the
traversal protection while allowing new files. Because `cwd` is always canonical and `..` is
resolved internally, no traversal can slip through.

**Alternatives considered**: (a) call `validate_path` on the full new path — rejected, fails for
new files; (b) add a new "validate for write" helper that canonicalizes only the parent — viable but
adds surface; the join-to-validated-dir approach reuses the existing helper unchanged (Principle IV).

## R2 — Directory listing & error handling

**Decision**: Build the listing with `std::fs::read_dir(cwd)`, collecting `(name, is_dir)` per
entry (using `entry.file_type()`; fall back to `metadata()` only if needed). Sort: parent `..`
first (omitted at filesystem root), then directories, then files, each case-insensitive
alphabetical. Include dot-files (clarified). On `read_dir`/permission error, do **not** change
`cwd`; set the browser's `error: Option<String>` notice and keep the current listing.

**Rationale**: `read_dir` is portable std; per-entry `file_type()` avoids extra `stat` calls on most
platforms. Graceful error handling satisfies FR-013 / SC-005 and Principle VII.

**Alternatives considered**: recursive/tree view — rejected (YAGNI, Principle VI); the flat
per-directory listing matches DOS EDIT and every mainstream picker.

## R3 — UTF-8 / wide-character-safe truncation

**Decision**: Truncate display names by accumulating **grapheme clusters** and their display width
until the column budget is reached, appending `…` when cut — never slicing by byte/char index.
Reuse the existing display-width approach already in the codebase (the wide-char width heuristic used
for the editor/menus) rather than introducing `unicode-width`.

**Rationale**: Principle II (NON-NEGOTIABLE) requires no multi-byte corruption. The project already
renders CJK/emoji widths via a local heuristic; reuse keeps behaviour consistent and adds no deps.

**Alternatives considered**: add `unicode-width` crate — rejected (Principle IV, and a heuristic
already exists).

## R4 — Rendering a scrollable list (ratatui)

**Decision**: Render the browser as a bordered box via the existing overlay pattern
(`Clear` + `Block` + manual cell writes / `Paragraph`), exactly like the menu dropdown
(src/ui/menubar.rs) and the existing dialogs. Maintain a `scroll` offset; show a window of
`visible_rows` entries, keeping `selected` within `[scroll, scroll+visible_rows)`. Header line shows
the current directory path (truncated left if too long); a footer shows key hints; Save mode adds a
filename input line.

**Rationale**: Reuses proven rendering + the shared-geometry mouse hit-test pattern from feature 011
(`dropdown_layout`/`hit_test_menu`) so clicks map to the same rows that are drawn.

**Alternatives considered**: ratatui's stateful `List` widget — workable, but manual rendering keeps
the box layout/geometry identical to existing dialogs and makes mouse hit-testing trivial and shared.

## R5 — Mouse hit-testing integration

**Decision**: Extend `App::handle_mouse_event` (src/app.rs, feature 011): when `file_browser` is
open, a left **press** is hit-tested against the browser box geometry via a `FileBrowser` method
that maps a terminal `(col,row)` to either an entry index or "outside". Click on an entry acts
directly (enter dir / pick file); click outside cancels. The existing modal-precedence guard in that
method is updated so the browser participates.

**Rationale**: Consistent with the menu mouse model; one model, shared geometry, no drift (FR-010).

## R6 — Starting directory

**Decision**: Open at the active buffer's parent directory when it has a path; else the process CWD
(`std::env::current_dir()`), canonicalized. If that fails, fall back to `/`.

**Rationale**: Matches user expectation (start where the current file lives); robust fallback.

All NEEDS CLARIFICATION resolved (the spec's open UX choices were settled in `/speckit-clarify`).
