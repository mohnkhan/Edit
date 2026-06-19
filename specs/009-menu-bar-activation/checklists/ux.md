# Checklist: Menu-Bar Interaction Requirements Quality (UX)

**Purpose**: Validate that the requirements for live menu-bar activation are complete, clear,
consistent, and measurable — *before* implementation. These items test the **spec**, not the code.
**Created**: 2026-06-19
**Feature**: [spec.md](../spec.md) | [contracts/menu-interaction.md](../contracts/menu-interaction.md)

## Requirement Completeness

- [x] CHK001 - Are requirements defined for every navigation key the feature uses (Up, Down, Left, Right, Enter, Esc) in each menu state? [Completeness, Spec §FR-001..FR-005]
- [x] CHK002 - Are the menu-bar **entry paths** explicitly specified for both F10 (top-level highlight) and Alt+<letter> (direct dropdown)? [Completeness, Spec §FR-015]
- [x] CHK003 - Is the placement of plugin-contributed top-level menus stated unambiguously relative to built-in menus? [Completeness, Spec §FR-007]
- [x] CHK004 - Is behavior specified for a plugin menu whose name collides with a built-in menu name? [Completeness, Spec §FR-008]
- [x] CHK005 - Are no-plugin / disabled-plugin / `--no-plugins` outcomes for the menu bar all specified? [Completeness, Spec §FR-010, Edge Cases]
- [x] CHK006 - Is the activation outcome for plugin items (which action is dispatched, where the result is shown) documented? [Completeness, Spec §FR-009]
- [x] CHK007 - Are requirements defined for the empty-dropdown (zero-item plugin menu) case? [Completeness, Spec Edge Cases]
- [x] CHK008 - Are UTF-8 / wide-character rendering requirements stated for plugin-provided labels? [Completeness, Spec §FR-014]

## Requirement Clarity

- [x] CHK009 - Is "wrap-around" precisely defined for Up/Down (within a dropdown) vs Left/Right (across the top-level ring)? [Clarity, Spec §FR-001, §FR-003]
- [x] CHK010 - Is the distinction between `TopActive` (highlight, no dropdown) and `DropDown` (open) states unambiguous, including which keys apply in each? [Clarity, Spec §FR-002, §FR-015, data-model]
- [x] CHK011 - Does "between Options and Help" pin Help as the rightmost menu without leaving ordering ambiguity for multiple plugin menus? [Clarity, Spec §FR-007]
- [x] CHK012 - Is the ordering of items within a merged/plugin menu specified (load order vs `position`)? [Clarity, research §R3]
- [x] CHK013 - Is "the editor stays responsive" for a failing plugin item expressed as an observable, testable outcome (warning shown, buffer intact, plugin disabled)? [Clarity/Measurability, Spec §FR-013]
- [x] CHK014 - Is "renders identically to today" defined against a concrete reference (existing geometry tests / column table) rather than a subjective judgement? [Measurability, Spec §FR-011]

## Requirement Consistency

- [x] CHK015 - Is the placement decision consistent across spec, plan, research, data-model, and the resolved issue text (between Options and Help, overriding "after Help")? [Consistency, Spec Clarifications vs issue #19]
- [x] CHK016 - Do the spec's acceptance scenarios, the data-model state-transition table, and the interaction contract agree on every key→transition mapping? [Consistency, Spec §US1-3 / data-model / contract §C1]
- [x] CHK017 - Is the "no new Action / no new keybinding" assumption consistent with FR-015's F10 behavior change (reuse of `Action::Menu`)? [Consistency, Spec Assumptions vs §FR-015]
- [x] CHK018 - Are modal-precedence requirements consistent with the existing dialog-handling order described in the plan/research? [Consistency, Spec §FR-012, research §R6]

## Acceptance Criteria Quality (Measurability)

- [x] CHK019 - Is each Success Criterion (SC-001..SC-006) objectively verifiable without referencing implementation internals? [Measurability, Spec §Success Criteria]
- [x] CHK020 - Is "100% of built-in menu items reachable/activatable" backed by an enumerable acceptance method? [Measurability, Spec §SC-001, §SC-005]
- [x] CHK021 - Is the ≤50 ms latency criterion tied to a stated measurement method or an existing budget rather than left abstract? [Measurability, Spec §SC-004, tasks L1 note]
- [x] CHK022 - Is "zero regressions" for no-plugin geometry expressed as a pass/fail gate (existing tests unchanged)? [Measurability, Spec §SC-003]

## Scenario & Edge-Case Coverage

- [x] CHK023 - Are alternate-entry scenarios (open via Alt+letter vs F10) both covered by acceptance scenarios? [Coverage, Spec §US1, §FR-015]
- [x] CHK024 - Are exception scenarios (plugin dispatch timeout/error during menu activation) covered as requirements, not only as a manual quickstart note? [Coverage/Exception, Spec §FR-013, contract §C6]
- [x] CHK025 - Is the modal-active-while-menu-open conflict covered (which surface owns the keys)? [Coverage, Spec §FR-012, Edge Cases]
- [x] CHK026 - Is the narrow-terminal / clipped-label case addressed for navigation reachability? [Edge Case, Spec Edge Cases]
- [x] CHK027 - Is the no-buffer / empty-buffer activation case covered for plugin actions that read buffer content? [Edge Case, Spec Edge Cases]
- [x] CHK028 - Are mid-session plugin enable/disable effects on the rendered/navigable menu set addressed? [Coverage, tasks T009/T014]
- [x] CHK029 - Is the toggle-item activation case (e.g. Soft Wrap) covered, including check-state reflection? [Coverage, Spec Edge Cases]

## Non-Functional & Constitution Alignment

- [x] CHK030 - Are DOS-faithful UI expectations (Help rightmost, arrow/Enter/Esc menu control) explicit and traceable to Constitution Principle I? [Coverage, Constitution I]
- [x] CHK031 - Is UTF-8 correctness for menu labels traceable to Constitution Principle II (no raw-byte path)? [Coverage, Constitution II, Spec §FR-014]
- [x] CHK032 - Is the plugin-sandbox/consent boundary stated as unchanged (no new attack surface) per Constitution Principle VII? [Coverage, Constitution VII, plan Constitution Check]
- [x] CHK033 - Is the TDD obligation (tests-before-impl) reflected as a requirement for every user-visible behavior? [Coverage, Constitution V, tasks]

## Dependencies, Assumptions & Conflicts

- [x] CHK034 - Is the reuse of feature-008 engine surfaces (`menu_items`, `dispatch_menu_action`, `Action::PluginMenuActivated`) documented as a validated dependency? [Dependency, plan/tasks]
- [x] CHK035 - Is the assumption "no existing automated test asserts current F10 behavior" recorded so the F10 change is known-safe? [Assumption, Spec Assumptions, analysis H1]
- [x] CHK036 - Is mouse-driven menu selection explicitly scoped out to avoid ambiguity about this feature's boundary? [Boundary/Assumption, Spec Assumptions]
- [x] CHK037 - Are there any remaining conflicts between the issue text and the resolved clarifications that a reader could misread? [Conflict, Spec Clarifications]

## Traceability

- [x] CHK038 - Does every functional requirement (FR-001..FR-015) map to at least one task and one test obligation? [Traceability, tasks / contract §C6]
- [x] CHK039 - Does every success criterion (SC-001..SC-006) map to a task or an explicit coverage note? [Traceability, tasks]

## Notes

- This checklist validates **requirement quality** for feature 009; it is not a test plan. The
  executable tests live in `tasks.md` (T002/T005/T008/T011/T013) and `contracts/menu-interaction.md` §C6.
- Items reference spec sections and the post-analysis remediations (H1, M1–M5, L1–L3).
