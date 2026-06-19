# Requirements Quality Checklist: Plugin API — Security & Sandbox

**Purpose**: Validate that the security/sandbox requirements (the highest-risk domain per
Constitution Principle VII) are complete, clear, consistent, and measurable BEFORE implementation.
**Created**: 2026-06-19
**Feature**: [spec.md](../spec.md) | **Contract**: [contracts/plugin-rhai-api.md](../contracts/plugin-rhai-api.md)

> "Unit tests for the requirements" — each item checks the *requirement writing*, not the code.

## Sandbox Isolation Requirements

- [x] CHK001 Is the default filesystem access posture specified unambiguously (default-deny vs allow-list)? [Clarity, Spec §FR-006]
- [x] CHK002 Is it specified how a plugin obtains the content it needs without filesystem access (host-passed arguments)? [Completeness, Spec §FR-006]
- [x] CHK003 Are the conditions under which a plugin gains additional read paths defined (manifest declaration + consent)? [Completeness, Spec §FR-006, §FR-010]
- [x] CHK004 Is write access posture specified (never granted by default)? [Clarity, Spec §FR-006]
- [x] CHK005 Is it specified that a plugin cannot read or write the editor's internal buffer state directly? [Completeness, Spec §FR-005]
- [x] CHK006 Is the behavior for an undeclared filesystem path access defined (deny + log + violation count)? [Coverage, Contract error table]

## Resource & Time-Limit Requirements

- [x] CHK007 Is the per-call time limit quantified with a specific value (50 ms)? [Measurability, Spec §FR-007]
- [x] CHK008 Is the action on time-limit exceedance specified (terminate + disable for session + status warning)? [Completeness, Spec §FR-007]
- [x] CHK009 Is the maximum termination latency measurable (within 200 ms)? [Measurability, Spec §SC-002]
- [x] CHK010 Are resource caps beyond time (memory/operations) addressed in the design? [Coverage, Contract sandbox section]
- [x] CHK011 Is the time limit's configurability status explicitly decided (fixed constant in v1)? [Clarity, Spec §FR-007]

## Crash & Error Isolation Requirements

- [x] CHK012 Is the behavior when a plugin causes an unrecoverable error specified (disable for session; editor + buffers survive)? [Completeness, Spec §US5-AC3]
- [x] CHK013 Is it specified that a misbehaving plugin must not affect other plugins or built-in behavior? [Consistency, Spec §US1-AC3]
- [x] CHK014 Is the distinction between "discard output but keep enabled" (invalid tokens) and "disable" (trap/timeout) defined? [Clarity, Contract error table]

## Consent & Trust Requirements

- [x] CHK015 Is the first-run consent flow specified (one-time prompt listing identity + permissions before load)? [Completeness, Spec §FR-010]
- [x] CHK016 Is the persistence location and format of consent decisions specified (plugins.toml)? [Completeness, Spec §FR-010]
- [x] CHK017 Is the decline outcome specified (permanent disable)? [Clarity, Spec §FR-010]
- [x] CHK018 Is the consent UX cost bounded (≤ 1 additional keypress)? [Measurability, Spec §SC-006]

## Encoding-Safety Requirements (Principle II ∩ VII)

- [x] CHK019 Is UTF-8 validation of all plugin-provided strings required before rendering? [Completeness, Spec §FR-011]
- [x] CHK020 Is escape-injection prevention for plugin-provided display strings addressed? [Coverage, Constitution §VII, Spec §SC-004]

## Observability Requirements

- [x] CHK021 Are all security-relevant events required to be logged (load errors, consent, violations, timeouts)? [Completeness, Spec §FR-012]
- [x] CHK022 Is "no errors silently swallowed" stated as a requirement? [Clarity, Spec §FR-012]

## Bypass / Override Requirements

- [x] CHK023 Is a global plugin off-switch specified (`--no-plugins`) with its non-persistence semantics? [Completeness, Spec §FR-008]
- [x] CHK024 Is the interaction between `--no-plugins`, per-plugin consent, and the manager toggle defined? [Consistency, Plan §Constraints]

## Notes

All 24 items pass after the analyze-pass remediations (N1–N8). FR-006 now states default-deny;
FR-007 fixes the 50 ms limit and records the configurability decision; escape-injection is
covered by Constitution §VII + SC-004 (host sanitizes terminal control sequences from plugin
content, consistent with the existing `src/security/` rendering sanitizer).
