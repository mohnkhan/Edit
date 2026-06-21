# Internal Contract: Persist Per-Tab Soft-Wrap

No external API. Serialization + behavioral guarantees.

## Serialization (FR-001/FR-003/FR-004)
- C-1: `BufferEntry` serializes a `soft_wrap` bool; `#[serde(default)]` so absence → `false`.
- C-2: `SessionData.version` is written as `2`; the loader accepts `1` and `2` and rejects anything else
  (e.g. the existing `test_unknown_version_returns_err` with a bogus version still errors).
- C-3: No `deny_unknown_fields` — forward-tolerant.

## Behavioral (FR-002/FR-005/FR-006)
- B-1: Save captures each eligible buffer's live `soft_wrap`.
- B-2: Restore sets each restored buffer's `soft_wrap` from its entry (per tab, independent).
- B-3: A legacy (v1) file restores tabs at the configured default; no error.
- B-4: New/opened tabs after restore still seed from `config.soft_wrap` (044 behavior unchanged).

## Test contract (FR / SC)
- T-1: round-trip — build SessionData from an App with mixed per-tab wrap, serialize+deserialize (or
  save+load), restore, assert each tab's wrap matches (SC-001).
- T-2: legacy — deserialize a TOML/JSON payload with no `soft_wrap` field → `false` default, loads OK
  (SC-002).
- T-3: version — a v2 file loads; a bogus version still rejected; existing 003 session tests pass
  (adjusted for version 2 where they assert the written version).
- T-4: full suite + 044 per-tab tests green; `fmt` + `clippy -D warnings` clean.
