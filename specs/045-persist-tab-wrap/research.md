# Phase 0 Research: Persist Per-Tab Soft-Wrap

Codebase-internal decisions, verified against the post-044 source.

## R1 — Where to store the persisted value

**Decision**: Add `pub soft_wrap: bool` to `session::BufferEntry` (the per-tab session record holding
`path`, `cursor_line`, `cursor_col`). Annotate with `#[serde(default)]`.

**Rationale**: `BufferEntry` is the existing per-tab persistence unit; wrap is per-tab. `serde(default)`
makes a v1 file (no field) deserialize with `soft_wrap = false`, then restore applies it (→ default).
**Alternatives**: a parallel list keyed by index — rejected (second source of truth; the bug pattern we
avoid). A top-level wrap list — rejected (couples to buffer ordering separately).

## R2 — Backward/forward compatibility & version

**Decision**: Bump the session schema version `1 → 2`. The writer emits version 2. The loader's strict
check (`session/mod.rs:145`, `if data.version != 1`) becomes "accept 1 or 2". `soft_wrap` is
`#[serde(default)]` so a v1 file restores tabs at the default; a v2 file restores the saved values.
`deny_unknown_fields` is NOT set on `BufferEntry`, so an older binary reading a v2 file silently ignores
the field (forward compatible too).

**Rationale**: FR-003/FR-004 — old files must load. The version bump documents the change while
`serde(default)` does the actual compatibility work. **Alternatives**: keep version 1 and rely solely on
`serde(default)` — works, but silently changes the schema under a version that claims "always 1";
rejected for clarity. Use `deny_unknown_fields` — rejected (would break forward compat).

**Verified**: `load`/validation at `session/mod.rs:108–150` checks `version != 1` and parses TOML; the
inline tests `test_unknown_version_returns_err` (must still reject e.g. version 99) and the round-trip
test (`version: 1` literals at 209/235) — these need version-accept updates / will exercise v2.

## R3 — Writer & reader wiring

**Decision**:
- `build_session_data` (`app/fileops.rs`): set `soft_wrap: buf.soft_wrap` in each `BufferEntry`; set
  `version: 2` on `SessionData`.
- `do_restore_session` (`app/fileops.rs`, restore loop ~158–186): after `Buffer::open` and cursor seek,
  set `buf.soft_wrap = entry.soft_wrap;` before pushing. This supersedes the feature-044 config-default
  seed *for restored tabs only*.

**Rationale**: FR-001/FR-002/FR-005 — capture the live per-tab value on save; apply it per tab on
restore. New/opened tabs after restore keep the 044 config-seed behavior (FR-006), since they go through
`new_buffer`/`handle_open_file`, not the restore loop.

## R4 — Scope guard

**Decision**: Persist only the on/off bool. Wrap cache and geometry are recomputed at runtime (the
event-loop cache gate reads `active_buffer().soft_wrap`; 043 invalidates on switch). Nothing else added
to the session file.

## Open questions

None. No `NEEDS CLARIFICATION` remain.
