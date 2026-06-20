# UX / Interaction Requirements Quality Checklist: Menu mnemonic accelerators

**Purpose**: Validate that the requirements for DOS-style menu mnemonics are complete, clear,
consistent, and measurable — covering DOS-faithful behavior, accessibility/degradation, keyboard
correctness, and UTF-8 rendering. These items test the *requirements*, not the implementation.
**Created**: 2026-06-20
**Feature**: [spec.md](../spec.md)

## DOS-Faithful Behavior

- [x] CHK001 Are the exact built-in item accelerator letters enumerated for every menu rather than
  described only as "DOS convention"? [Completeness, Spec §FR-005a / research §R4]
- [x] CHK002 Is the visual indicator (underline) specified unambiguously, including that exactly one
  glyph per label carries it? [Clarity, Spec §FR-001/§FR-002]
- [x] CHK003 Is the requirement that the displayed accelerator and the activating key are the same
  letter stated explicitly (no drift)? [Consistency, Spec §FR-005]
- [x] CHK004 Is bare-`Alt`→bar-activation behavior defined as equivalent to an existing, named entry
  point (F10) rather than a new undocumented mode? [Clarity, Spec §FR-005a]

## Keyboard Correctness

- [x] CHK005 Is letter-activation scoped precisely to "while a dropdown/bar is active" so it cannot be
  confused with normal typing? [Clarity, Spec §FR-004/§FR-011]
- [x] CHK006 Is case-insensitive matching of the accelerator key explicitly required? [Completeness,
  Spec Edge Cases]
- [x] CHK007 Is the behavior of a letter press that matches NO accelerator defined (inert no-op, menu
  stays open, no buffer mutation)? [Coverage, Spec §FR-007]
- [x] CHK008 Is "activate by letter" required to be identical in effect to "highlight + Enter" so
  there is one source of truth for an item's action? [Consistency, Spec §FR-004]
- [x] CHK009 Are the existing menu keys (F10, arrows, Enter, Esc, mouse, Ctrl/F-key shortcuts)
  explicitly required to remain unchanged? [Regression Coverage, Spec §FR-012]

## Accessibility & Graceful Degradation

- [x] CHK010 Is the degradation requirement for terminals lacking underline support stated (label
  stays fully readable, no lost/doubled characters)? [Edge Case, Spec §FR-001/§FR-002]
- [x] CHK011 Is the fallback for bare-`Alt` on terminals without keyboard-enhancement support defined
  (F10 / Alt+letter still work)? [Edge Case, Spec §FR-005a / research §R2]
- [x] CHK012 Is there a defined behavior for an entry that can receive no unique accelerator (rendered
  with none, still reachable by arrows + Enter + mouse)? [Coverage, Spec §FR-006]

## UTF-8 / Rendering Quality

- [x] CHK013 Is correct accelerator selection and underline placement on multi-byte / wide / combining
  labels required, with a prohibition on splitting a character? [Completeness, Spec §FR-010]
- [x] CHK014 Is the underline position rule unambiguous (which occurrence of the letter is underlined)
  for labels where the letter appears more than once? [Ambiguity, data-model §"Derived: underline
  index"]
- [x] CHK015 Are plugin-supplied UTF-8 labels covered by the same selection/rendering rules, including
  the no-letter-available case, without crash? [Coverage, Spec §FR-009/§FR-010]

## Uniqueness & Assignment Rules

- [x] CHK016 Is the uniqueness scope explicitly bounded (per-bar for top-level, per-dropdown for
  items; the same letter may recur across different dropdowns)? [Clarity, Spec §FR-003]
- [x] CHK017 Is deterministic assignment required so the visible letters are stable across runs and
  renders? [Measurability, Spec §FR-008]
- [x] CHK018 Is the plugin auto-assignment required to seed from existing built-in letters so it
  cannot collide with built-ins in the same scope? [Consistency, Spec §FR-009 / data-model
  §"Assignment rules"]

## Acceptance Criteria Quality

- [x] CHK019 Are the success criteria expressed as objectively checkable outcomes (e.g., "100% of
  items show exactly one accelerator", "zero duplicate accelerators per menu") rather than vague
  adjectives? [Measurability, Spec §SC-001..§SC-006]
- [x] CHK020 Is "no regression" defined as a verifiable criterion (existing menu/editor tests stay
  green; keys behave as before when no menu is active)? [Measurability, Spec §SC-006/§FR-011]

## Dependencies & Assumptions

- [x] CHK021 Is the assumption that built-in top-level accelerators equal their existing `Alt+letter`
  bindings documented and validated against the keymap? [Assumption, Spec §Assumptions / tasks
  T022a]
- [x] CHK022 Is the dependency on terminal keyboard-enhancement support for bare-`Alt` documented as a
  known limitation rather than an unconditional guarantee? [Assumption, research §R2]
