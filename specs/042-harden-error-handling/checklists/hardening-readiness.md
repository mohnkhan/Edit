# Checklist: Hardening Readiness & Behavior Preservation

**Purpose**: Validate that feature 042's requirements are complete, unambiguous, and consistent enough
to implement a behavior-preserving anti-crash refactor. These items test the **requirements**, not the
eventual code.
**Created**: 2026-06-21
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Is the set of conversion targets bounded and enumerated (which files, how many sites)? [Completeness, Spec §FR-001, plan]
- [ ] CHK002 - Is the absent-arm behavior for each conversion specified (must equal the prior no-op / fall-through)? [Completeness, Spec §FR-001/§Edge Cases]
- [ ] CHK003 - Are the fuzz sweep's required coverage dimensions stated — every overlay state (incl. async-stacked external-change/consent), and a set of terminal sizes incl. the 80×24 minimum and a sub-minimum? [Coverage, Spec §FR-003/§SC-002]
- [ ] CHK004 - Is the guardrail's scope defined precisely (the `app` module tree) and its propagation mechanism stated? [Completeness, Spec §FR-005, research §R2]
- [ ] CHK005 - Are the explicitly-accepted exclusions enumerated (highlight `Regex::new` literal unwrap; best-effort `let _ =` cleanup; raw `[index]` deferred)? [Completeness, Spec §FR-006/§Assumptions]
- [ ] CHK006 - Is the recovery-net invariant stated (panic hook + SIGSEGV handler unchanged)? [Completeness, Spec §FR-008]

## Requirement Clarity

- [ ] CHK007 - Is "behavior-preserving" defined testably (identical observable state for any non-panicking input; no assertion changes)? [Clarity, Spec §FR-002/§FR-007]
- [ ] CHK008 - Is "deterministic" defined concretely (fixed seed, no wall-clock/RNG) rather than vaguely? [Clarity, Spec §FR-004]
- [ ] CHK009 - Is "guarded unwrap" defined (a value proven present by an earlier check) so the target set is unambiguous? [Clarity, Spec §Key Entities]
- [ ] CHK010 - Is the no-panic assertion mechanism unambiguous (a panic fails the test; no behavioral assertion on ignored input)? [Clarity, Spec §FR-003]

## Requirement Consistency

- [ ] CHK011 - Do spec, plan, research, and tasks agree on the site counts (14/7/3/3 = 27) and the four target files? [Consistency, all artifacts]
- [ ] CHK012 - Is the guardrail scope consistent everywhere (app tree only — not crate-wide), so FR-005 and FR-006 don't conflict? [Consistency, Spec §FR-005/§FR-006]
- [ ] CHK013 - Is the US2-before-US1 ordering consistent with the claim that the fuzz test is a proof/safety net (not red-green TDD)? [Consistency, tasks §Dependencies]

## Acceptance Criteria Quality

- [ ] CHK014 - Are success criteria measurable (zero residual guarded unwraps; ≥ several-thousand events; zero panics; suite count unchanged + fuzz)? [Measurability, Spec §SC-001..003]
- [ ] CHK015 - Is the guardrail's "demonstrably active" criterion concrete (a stray unwrap makes `clippy -D warnings` fail)? [Measurability, Spec §SC-004]
- [ ] CHK016 - Is there a stated baseline (current 1262/0/11) to compare the unchanged-suite claim against? [Measurability, tasks §T001]

## Scenario & Edge-Case Coverage

- [ ] CHK017 - Are async/stacked overlay states (external-change pending under another dialog; consent queue) covered by the fuzz requirement, not just simple overlays? [Coverage, Spec §Edge Cases]
- [ ] CHK018 - Is the too_small (sub-minimum terminal) render path included in the size set? [Edge Case, Spec §FR-003]
- [ ] CHK019 - Are out-of-bounds mouse coordinates part of the fuzz input space (clicks just past the edges)? [Edge Case, data-model]
- [ ] CHK020 - Is the case "fuzz surfaces a genuine (non-guarded) panic" handled in the plan (fix it the same way, note it)? [Coverage, tasks §T010]

## Non-Functional & Constitution

- [ ] CHK021 - Is Principle V (test-gated) satisfied — suite stays green and the safety net grows? [Traceability, plan §Constitution Check]
- [ ] CHK022 - Is the no-new-dependency constraint honored (hand-rolled PRNG, no `rand`)? [Non-Functional, Spec §Assumptions, plan]
- [ ] CHK023 - Is determinism reconciled with the project's "no wall-clock/RNG in tests" convention? [Consistency, Spec §FR-004]

## Dependencies, Assumptions & Ambiguities

- [ ] CHK024 - Is the assumption that "existing suite + fuzz = sufficient proof of behavior preservation" stated? [Assumption, Spec §Assumptions]
- [ ] CHK025 - Is the raw-`[index]` hardening explicitly out of scope (separate tracked effort) so this PR's bound is clear? [Scope, Spec §Assumptions]
- [ ] CHK026 - Is there any ambiguity about whether the lint should be `deny` vs `warn`? (Resolved: `deny`, surfaced as error under `-D warnings`.) [Ambiguity, plan]

## Notes

- Load-bearing items: CHK002 (absent-arm equivalence), CHK007 (behavior-preserving definition),
  CHK012 (guardrail scope vs FR-006), CHK017 (async-stacked fuzz coverage). If these are crisp, the
  existing suite + the fuzz sweep carry the rest.
- All items are answerable "yes, specified" from the current artifacts; any "no" is a gap to close
  before `/speckit-implement`.
