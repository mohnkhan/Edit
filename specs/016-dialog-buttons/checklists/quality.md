# Quality Checklist: Focusable dialog buttons

**Purpose**: Requirements-quality gate for feature 016. Tests the requirements, not the implementation.
**Created**: 2026-06-20
**Feature**: [spec.md](../spec.md)

## Rendering & focus

- [x] CHK001 Is "boxed button" + exactly-one-focused specified, with focus visually distinct? [§FR-001/§FR-002]
- [x] CHK002 Is drawn-position == clickable-region required (shared geometry)? [§FR-005/§FR-009]
- [x] CHK003 Is graceful degradation + no-panic-on-small-terminal specified? [§FR-010 / Edge Cases]

## Keyboard

- [x] CHK004 Is Tab/Shift+Tab order with wrap specified? [§FR-003]
- [x] CHK005 Is Enter/Space activation + Esc cancel specified (with the plugin-manager Space exception)? [§FR-004 / Clarif]
- [x] CHK006 Is coexistence with existing letter shortcuts and list Up/Down required? [§FR-007]
- [x] CHK007 Is a sensible (safe) default-focused button per dialog defined? [§FR-008 / data-model]

## Mouse

- [x] CHK008 Is single-click-activates-button specified via matching hit-test? [§FR-005]
- [x] CHK009 Is inside-not-on-button inert and outside-click-cancel (where safe) specified? [§FR-006]

## Non-regression & measurability

- [x] CHK010 Is "choice by button == choice by old shortcut" stated and measurable? [§FR-004 / §SC-004]
- [x] CHK011 Is modality + no-regression to editing and file-browser/menu mouse required? [§FR-011 / §SC-005]
- [x] CHK012 Are success criteria objectively checkable (all buttons reachable by Tab in N steps; click on drawn box activates)? [§SC-002/§SC-003]

## Scope

- [x] CHK013 Is the dialog scope explicit and the deferred set (Find/Replace, file browser) recorded for follow-up? [Clarif / plan]
