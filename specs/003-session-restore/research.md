# Research: Session Restore (Feature 003)

**Date**: 2026-06-18
**Feature**: Session Restore (`specs/003-session-restore/spec.md`)

---

## Decision 1 — Session File Format

**Decision**: TOML (human-readable), schema version 1, stored at
`$XDG_STATE_HOME/edit/session.toml`.

**Rationale**: `serde` + `toml` are already Cargo dependencies. TOML is the project's
established config format (`config.toml`). Human-readable as required by FR-009. Versioned
with a top-level `version` integer so future schema changes can be detected and gracefully
rejected (FR-010).

**Alternatives considered**:
- JSON: less human-friendly for a config/state file; rejected.
- Binary (bincode): not human-readable; rejected by FR-009.
- INI: no good Rust crate with the same adoption as `toml`; rejected.

---

## Decision 2 — Atomic Write Strategy

**Decision**: Write to `<session_path>.tmp`, then `std::fs::rename` to the final path.

**Rationale**: On POSIX (Linux, macOS, FreeBSD) `rename` is atomic at the filesystem
level — a reader will see either the old complete file or the new complete file, never
a partial write. Protects against `kill -9` and power-loss scenarios. The `.tmp`
leftover after a kill is harmless (it will be overwritten on the next exit).

**Alternatives considered**:
- Direct write: not atomic; partial file causes corrupt-TOML on next startup (handled by
  FR-010 but avoidable). Rejected.
- `fsync` before rename: adds latency; not needed for session data (not financial-grade
  durability). Not used.

---

## Decision 3 — Restore Prompt Location (TUI vs. Terminal)

**Decision**: Render the restore prompt as an in-TUI dialog overlay after the editor UI
has initialized, not as a pre-startup terminal prompt (e.g. `print! / stdin::read_line`).

**Rationale**: Constitution Principle I requires the DOS-faithful TUI to be the only
user-interaction surface. A pre-TUI prompt would break the "blue background, full-screen"
experience and cause artifacts if the terminal is already in a non-clean state. The
existing save-before-quit dialog is already rendered as a TUI overlay; the session dialog
follows the same pattern.

**Alternatives considered**:
- Pre-TUI `eprintln!` + `stdin` prompt: simpler to implement but violates Principle I and
  looks wrong on a 256-color terminal already in alternate-screen mode. Rejected.

---

## Decision 4 — Cursor Coordinate Convention

**Decision**: Store 1-based (line, column) in TOML as `cursor_line` and `cursor_col`.
Convert to 0-based `CursorPos` (`line`, `grapheme_col`) on restore.

**Rationale**: FR-009 specifies 1-based coordinates for readability (matches how editors
and users describe positions). `CursorPos` is 0-based internally. Conversion is trivial:
`cursor_line - 1`, `cursor_col - 1` with a `.saturating_sub(1)` guard for corrupt values.

**Alternatives considered**:
- 0-based in TOML: technically simpler, but unusual for human-readable files. Rejected per FR-009.

---

## Decision 5 — Clean Exit Detection

**Decision**: Call `session::save_session` inside the three clean-exit paths in `app.rs`:
1. `handle_quit` — when no buffer is modified (fast path, `self.running = false`)
2. `prompt_save_and_quit` — when save succeeds
3. `prompt_discard_and_quit` — when user discards changes

Crash exits go through the panic hook (`diagnostics::crash`) which does NOT call
`session::save_session`. This guarantees FR-002.

**Alternatives considered**:
- Write session on every tick: wasteful; session should only be written at the exit
  "boundary". Rejected.
- Write in `run()` after `event_loop` returns: the loop can return from both clean and
  IO-error exits; distinguishing them is complex. Rejected in favor of targeted hooks.

---

## Decision 6 — Path Validation on Restore

**Decision**: Run each path from `session.toml` through `security::sanitize::validate_path`
before calling `Buffer::open`. Paths that fail validation are treated as missing (FR-005).

**Rationale**: A hand-edited or externally modified `session.toml` could contain traversal
sequences (`../../../etc/passwd`). The existing sanitize module already implements this
check. No new security logic needed.

---

## Decision 7 — Active Pane Derivation

**Decision**: Store `active_pane: u32` in TOML (0 = left/only pane, 1 = right pane).
Derive it from `app.active_idx`: if `split_mode == Vertical && active_idx > 0` → pane 1,
else → pane 0.

**Rationale**: `App` has no explicit "active pane" field; the active buffer index serves
that role. This mapping is lossless for the current two-pane split model.

---

## Decision 8 — Scratch Buffer Handling

**Decision**: Session write filters to only buffers where `buf.path.is_some()` and the
path exists on disk (check `buf.path.as_ref().map(|p| p.exists()).unwrap_or(false)`).
New (unsaved) buffers and anonymous buffers are silently omitted.

**Rationale**: Per the spec Assumptions section: "Scratch buffers (new files not yet saved
to disk) are not recorded in the session." An unsaved buffer has no stable path; recording
it would produce an unrestorable entry.

---

## No Open Questions

All NEEDS CLARIFICATION items from the spec have been resolved in this document. No
external research was required; all decisions are deterministic from the existing codebase
and spec requirements.
