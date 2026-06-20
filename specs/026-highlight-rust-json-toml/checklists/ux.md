# Quality Checklist: Rust / JSON / TOML highlighting

**Purpose**: Validate that the *requirements* for feature 026 are complete, clear, consistent, and
measurable before implementation.
**Created**: 2026-06-20
**Audience**: Implementer / PR reviewer
**Feature**: [spec.md](../spec.md) · [contracts/highlighters.md](../contracts/highlighters.md)

## Coverage per language

- [x] CHK001 Is Rust token coverage enumerated (keywords, types, strings/char/byte/raw, numbers, comments, attributes, macros)? [Completeness, Spec §FR-001]
- [x] CHK002 Is JSON token coverage enumerated (keys, string values, numbers, true/false/null, punctuation)? [Completeness, Spec §FR-002]
- [x] CHK003 Is TOML token coverage enumerated (headers, keys, strings, numbers, dates, booleans, comments)? [Completeness, Spec §FR-003]
- [x] CHK004 Is the key-vs-value-string distinction for JSON/TOML specified (key → distinct style)? [Clarity, data-model mapping]
- [x] CHK005 Is each token mapped to a concrete (existing) style class? [Clarity, data-model mapping]

## Selection & precedence

- [x] CHK006 Is extension-based selection (`.rs`/`.json`/`.toml`) specified? [Completeness, Spec §FR-004]
- [x] CHK007 Is plugin-over-built-in precedence specified as preserved? [Consistency, Spec §FR-006]

## Span contract & resilience

- [x] CHK008 Is the "sorted, non-overlapping spans" output contract stated? [Clarity, Spec §FR-005]
- [x] CHK009 Are byte/width-correct offsets required for UTF-8 content? [Coverage, Spec §FR-005 / Constitution II]
- [x] CHK010 Is no-panic required for malformed/unterminated/empty/very-long/multi-byte input? [Edge Case, Spec §FR-008]
- [x] CHK011 Is line-based best-effort (no cross-line state) specified, matching existing highlighters? [Consistency, Spec §FR-007]

## Scope & traceability

- [x] CHK012 Is scope bounded to highlighting rules (no pipeline/buffer/other-feature change)? [Clarity, Spec §FR-007 / Assumptions]
- [x] CHK013 Is the no-new-dependency constraint stated (reuse regex + highlight subsystem)? [Assumption, plan Constitution IV]
- [x] CHK014 Is the Principle-VI gate (spec + user story for languages beyond baseline 5) acknowledged? [Traceability, Spec Assumptions]
- [x] CHK015 Is each success criterion (SC-001..SC-004) traceable to a requirement and a task? [Traceability, analyze coverage]
- [x] CHK016 Is the existing 5-language highlighting specified as unchanged? [Consistency, quickstart]

## Notes

- Validates requirement quality, not implementation. Behavioral verification lives in the per-language
  TDD unit tests and `quickstart.md`.
