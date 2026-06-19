# Feature Specification: Menu Check-State Indicator

**Feature Branch**: `006-menu-check-state-indicator`

**Created**: 2026-06-19

**Status**: Draft

**Input**: User description: "Feature 006: Menu Check-State Indicator. Add a visual checked/unchecked
state indicator (✓ prefix) to toggleable View menu items in the DOS-style pull-down menu bar. The
immediate trigger is the deferred FR-001 from feature 005: the 'Soft Wrap (ext)' View menu item
currently shows no checked/unchecked visual feedback when soft-wrap is active. The fix requires a
menu-bar-wide refactor: add an optional checked state to the MenuItem struct in src/ui/menubar.rs
and render a ✓ prefix in the dropdown for items where checked is Some(true). App must pass the
current toggle state when building menu event data. This is tracked as issue #13 in ROADMAP.md
with label follow-up."

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Soft Wrap Check-State Visible in Menu (Priority: P1)

A user opens the View pull-down menu while soft-wrap is active. They can immediately see that
"Soft Wrap (ext)" is checked — without having to look at the status bar or toggle the mode and
observe the `[WRAP]` indicator change — because a `✓` prefix appears next to the menu item label.

**Why this priority**: This is the primary motivating defect (FR-001 deferred from feature 005,
issue #13). The `[WRAP]` status-bar workaround is not discoverable for users who rely on the menu
for state feedback.

**Independent Test**: Launch the editor, toggle soft-wrap on (Alt+Z or View menu), then open the
View menu. Verify `✓ Soft Wrap (ext)` appears; close and toggle off; reopen menu and verify the
`✓` is gone.

**Acceptance Scenarios**:

1. **Given** soft-wrap is OFF, **When** the user opens the View pull-down menu, **Then** "Soft Wrap (ext)" appears without any prefix.
2. **Given** soft-wrap is ON, **When** the user opens the View pull-down menu, **Then** "Soft Wrap (ext)" appears prefixed with `✓ ` (checkmark + space).
3. **Given** soft-wrap is ON and the user selects "Soft Wrap (ext)" from the menu, **When** the menu closes, **Then** soft-wrap toggles OFF and the next menu open shows the item without the `✓` prefix.

---

### User Story 2 — All Future Toggleable Menu Items Benefit (Priority: P2)

Any new toggleable feature added to the menu bar in a future feature can use the same
check-state mechanism without further refactoring. The checked state support is general to
all `MenuItem` instances, not hard-coded for soft-wrap only.

**Why this priority**: Without a general mechanism, each future toggleable item would require
its own bespoke rendering path. Implementing it once, correctly, prevents the pattern from
re-appearing as technical debt.

**Independent Test**: Add a second mock toggle to the View menu and observe that the `✓` prefix
renders correctly when its state is active — same code path as "Soft Wrap (ext)".

**Acceptance Scenarios**:

1. **Given** a menu item is created with a checked state of `true`, **When** the dropdown is opened, **Then** that item is rendered with a `✓ ` prefix.
2. **Given** a menu item is created with a checked state of `false`, **When** the dropdown is opened, **Then** that item is rendered without any prefix, and the label is aligned with checked items (consistent column width).
3. **Given** a menu item has no associated toggle state (non-toggleable items, e.g. "Save", "Open"), **When** the dropdown is opened, **Then** no prefix column is rendered and label alignment is unchanged.

---

### User Story 3 — Check-State Survives Config-Persisted Restart (Priority: P3)

A user enables soft-wrap, closes the editor (config is persisted), and reopens it. On re-open,
soft-wrap is restored from config and the View menu correctly shows "Soft Wrap (ext)" as checked
immediately, without the user having to toggle it again.

**Why this priority**: Config persistence for soft-wrap is already implemented (feature 005).
The check-state indicator must correctly reflect the persisted state at startup, not just
after in-session toggles.

**Independent Test**: Set `soft_wrap = true` in `config.toml`, launch the editor, open the View
menu, and confirm `✓ Soft Wrap (ext)` is shown immediately on first open.

**Acceptance Scenarios**:

1. **Given** `soft_wrap = true` in the persisted config, **When** the editor starts, **Then** opening the View menu shows `✓ Soft Wrap (ext)`.
2. **Given** `soft_wrap = false` in the persisted config (or no config file), **When** the editor starts, **Then** opening the View menu shows "Soft Wrap (ext)" without a `✓` prefix.

---

### Edge Cases

- What happens when a menu dropdown contains a mix of toggleable and non-toggleable items? When the dropdown is checkable-aware (≥1 item has a toggle state), ALL items get a 2-char prefix slot (`✓ ` for checked, `  ` for unchecked/absent) to maintain label alignment. Non-checkable-aware dropdowns (no items have toggle states) render with no prefix column at all.
- What happens if the terminal is too narrow to display both the `✓ ` prefix and the full menu item label? The prefix takes priority; the label is truncated at the right edge consistent with how existing labels are truncated. Each cell write is individually guarded by `cx < area.right()` so no buffer overrun occurs.
- What happens if the terminal is so narrow that the prefix character itself cannot be written (e.g., the dropdown start column equals `area.right()`)? The existing `cx >= area.right() → break` guard silently skips the write. No visible character is rendered; the menu gracefully degrades to empty. No minimum terminal width is enforced by the spec — extreme narrowness is handled by the guard, not by a hard error.
- What happens when a dropdown has zero items? The `has_checkable` flag is always `false` for an empty item list (no items means no matching actions). The dropdown renders with `content_width = 0 + 4 = 4` and no item rows — identical to pre-feature behavior.
- What happens when the checked state changes while the menu is open? The menu reflects the state at the moment it was opened; live updates inside an open menu are out of scope.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The menu system MUST support an optional binary **check state** (`true` / `false` / absent) on any individual menu item. ("Check state" is the canonical term; "toggle state" and "checked/unchecked" are synonymous as used in this spec.)
- **FR-002**: When a menu item's check state is `true`, the dropdown MUST render a `✓ ` prefix (U+2713 + U+0020) immediately before the item label, using the same foreground and background colors as the rest of that item row (including the inverted-video style when the item is highlighted/selected). *(Non-DOS extension convention: the `✓` prefix is a non-DOS visual addition, analogous to the "(ext)" label on "Soft Wrap (ext)". No DOS-faithful menu item is altered by this feature — constitution Principle I is preserved.)*
- **FR-003**: When a menu item's check state is `false` or the item's action has no entry in the check-state mapping (absent), the dropdown MUST NOT render any visible checkmark; instead a 2-space filler MUST be rendered in the prefix column so that label text aligns with checked peers. (This applies only within a checkable-aware dropdown — one that contains at least one item with a check-state entry.)
- **FR-004**: The "Soft Wrap (ext)" View menu item MUST reflect the current soft-wrap toggle state (checked when ON, unchecked when OFF) every time the View dropdown is opened.
- **FR-005**: The check-state rendering MUST be driven by the application's authoritative toggle state, not by any cached or stale copy.
- **FR-006**: Non-toggleable menu items (e.g. "Save", "Open", "Find") MUST NOT receive or display any check-state prefix.
- **FR-007**: The check-state mechanism MUST be general: any menu item in any menu (File, Edit, Search, View, Options) can opt into it by having its action appear in the check-state mapping. If the same `Action` variant appears in multiple distinct menus simultaneously, each matching dropdown independently renders the `✓` prefix — no cross-menu exclusivity constraint applies.
- **FR-008**: Label text alignment across all items in a dropdown MUST be consistent — items with a checked state and items without MUST visually align at the label start column when the menu contains at least one toggleable item.

### Key Entities

- **MenuItem**: A single entry in a pull-down menu, identified by its label and associated action. The `MenuItem` definitions themselves are **not changed** — they are static constants that cannot hold runtime state.
- **MenuBarWidget toggle states**: A runtime mapping of action → boolean that the menu bar widget consults at render time. Each entry associates a toggleable action (e.g. "Soft Wrap") with its current on/off state. This mapping is constructed fresh every render frame from the application's authoritative state, so it is never stale.
- **Checkable-aware dropdown**: A rendered dropdown that contains at least one `MenuItem` whose action appears in the toggle-state mapping. Such a dropdown reserves a 2-character prefix column (`✓ ` or `  `) for every item to maintain label alignment. Dropdowns with no matching actions are **not** checkable-aware and render identically to pre-feature behavior.
- **MenuDropdown**: A rendered list of `MenuItem`s. When the widget's toggle-state mapping contains at least one action that matches an item in the dropdown (i.e., the dropdown is *checkable-aware*), the dropdown reserves a prefix column (`✓ ` or `  `) for all items to maintain alignment.
- **App toggle state**: The authoritative runtime boolean (e.g. the soft-wrap enabled flag) that is read at widget render time and fed into the toggle-states mapping. It is not cached or copied — each render frame reads the current value directly.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every time a user opens the View menu while soft-wrap is active, the `✓` prefix is visible next to "Soft Wrap (ext)" — 100% of menu opens reflect the correct state.
- **SC-002**: Toggling soft-wrap on/off and immediately re-opening the View menu always shows the updated check-state — zero stale-state occurrences during a session.
- **SC-003**: No regression in existing menu navigation behavior: keyboard up/down, mouse click, and Escape all work identically to pre-feature behavior.
- **SC-004**: The check-state feature adds no perceptible input latency: the menu opens in the same time as before (state lookup is a single scan of at most 8 items — no dedicated performance test is required).
- **SC-005**: The `✓` prefix renders correctly on all terminal emulators in the CI matrix without character corruption or layout shift.

---

## Assumptions

- The `✓` character (U+2713 CHECK MARK) is renderable by the terminals in the CI matrix (xterm-256color, VT220). The project's UTF-8-first constitution mandates UTF-8-capable terminals; U+2713 is therefore a hard requirement with no ASCII fallback. U+2713 has Unicode east-asian-width classification "Neutral" — display width is exactly **1 terminal cell** — so it occupies the same space as the `' '` it replaces when unchecked.
- Only one toggleable menu item exists today ("Soft Wrap (ext)" in the View menu). The implementation is designed to be general but only one item exercises it at the time of this spec. *(Verify before merge: if a second toggleable item was added after branch creation, update T013b and the call-site in `src/ui/mod.rs` to include the new pair.)*
- The `MenuItem` static definitions are **not changed** by this feature. Backward compatibility is automatic: when the toggle-states mapping is empty, the menu bar widget renders identically to pre-feature behavior with no prefix column.
- The checked state is derived from `App` state at widget render time; live updates while the menu is open are out of scope and not required by any existing user story.
- Menu width calculations already account for label length; extending them to include a fixed 2-character prefix column (`✓ ` or `  `) is a localized change within the menu rendering path.
- This feature closes issue #13 and removes "Menu Item Checked-State Indicator" from the Deferred section of `ROADMAP.md`.
