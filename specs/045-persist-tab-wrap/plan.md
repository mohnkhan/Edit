# Implementation Plan: Persist Each Tab's Soft-Wrap Across Restart

**Branch**: `045-persist-tab-wrap` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/045-persist-tab-wrap/spec.md`

## Summary

Additive extension of session-restore (feature 003): record each persisted tab's `soft_wrap` in the
session file and apply it on restore, so per-tab wrap (feature 044) survives restart. Backward
compatible: a new `#[serde(default)] soft_wrap` field on `BufferEntry`, the schema version bumped
1 → 2 with the loader accepting both, so older session files load and default each tab to
`config.soft_wrap` exactly as today.

## Technical Context

**Language/Version**: Rust 2021, MSRV 1.74
**Primary Dependencies**: `serde` + `toml` (already used for the session file). No new deps.
**Storage**: `$XDG_STATE_HOME/edit/session.toml` (existing).
**Testing**: `cargo test` (session unit tests + app restore integration), `make smoke`.
**Target Platform**: Linux TUI.
**Project Type**: Single-project desktop TUI app.
**Performance**: N/A (one bool per buffer in the session file).
**Constraints**: Backward compatible (FR-003/FR-004) — old files must still load; behavior unchanged for
non-session users (FR-006); the 042 `clippy::unwrap_used` guardrail holds.
**Scale/Scope**: One serde field + writer + reader + version-accept change + tests.

## Constitution Check

- **I. DOS-Faithful UI** — PASS. No UI change.
- **II. UTF-8** — PASS / N/A.
- **III. Portable Build** — PASS. Pure Rust.
- **IV. Minimal Footprint** — PASS. No new dependency.
- **V. Test-Gated (NON-NEGOTIABLE)** — PASS. Round-trip + legacy-load tests added; existing 003/044
  tests stay green.
- **VI. Simplicity / YAGNI** — PASS. Smallest additive change; only on/off persisted.
- **VII. Security** — PASS / N/A (no new external input path; same TOML parse, which already validates).

**Result**: All gates pass. Complexity Tracking empty.

## Project Structure

```text
src/
├── session/mod.rs       # BufferEntry += `#[serde(default)] pub soft_wrap: bool`; bump SCHEMA version
│                        #   to 2; loader accepts version 1 OR 2 (1 = no wrap field → default false).
└── app/fileops.rs       # build_session_data: write `soft_wrap: buf.soft_wrap` per entry (+ version 2).
                         # do_restore_session: set restored `buf.soft_wrap = entry.soft_wrap`.
```

**Structure Decision**: Unchanged layout; change is localized to the session schema + its writer/reader.

## Key design decisions (detail in research.md)

- `#[serde(default)]` on the new field → missing (v1 files) deserialize to `false`, then restore applies
  it. No `deny_unknown_fields` is set, so a v2 file is also tolerated by an older binary (it ignores the
  field). Fully forward/backward compatible.
- Version bump 1 → 2 documents the schema change; the loader's `version != 1` reject becomes
  `version != 1 && version != 2`. The writer emits version 2.
- Restore sets `buf.soft_wrap` from the entry value (superseding the 044 config-default seed for
  restored tabs). New/opened tabs after restore still seed from config (044, unchanged).

## Phased Approach (one PR, ordered commits)

1. Schema: add the field + version accept logic in `src/session/mod.rs`.
2. Writer + reader: `build_session_data` writes it; `do_restore_session` applies it (`fileops.rs`).
3. Tests: round-trip (mixed wrap) + legacy-file (no field → default) + version acceptance.

## Complexity Tracking

*No violations. Table intentionally empty.*
