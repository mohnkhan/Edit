# Feature Specification: DOS-style menu mnemonic accelerators

**Feature Branch**: `013-menu-mnemonics`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "DOS-style mnemonic accelerators for all menus and menu items. Every top-level menu (File, Edit, Search, View, Options, Help) and every dropdown item must display a visible mnemonic letter (DOS convention: a highlighted/underlined accelerator letter) so users can see which key activates it. While a dropdown is open, pressing that letter activates the corresponding item. Pressing Alt highlights the menu bar mnemonics; Alt+letter opens the matching top-level menu (already partially wired for File/Edit/Search/View/Options/Help). Each item's mnemonic must be unique within its menu. Plugin-provided menu items should also receive mnemonics. This recreates the look-and-feel of MS-DOS EDIT.COM and standard menu-driven applications."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See which key activates each menu and item (Priority: P1)

A user looks at the menu bar and any open dropdown and can immediately tell, for every
top-level menu and every item, which single letter is its accelerator, because that letter
is rendered visually distinct from the rest of the label (the DOS-faithful highlighted
accelerator). The user does not have to memorize shortcuts or consult documentation — the
interface advertises them.

**Why this priority**: This is the core of the request and the most visible part of the
DOS-faithful look. Without the visible indicator the feature is invisible; with it, the
editor immediately reads as a DOS-style menu-driven app, even before any new key works.

**Independent Test**: Open the editor, observe the menu bar — each of the six built-in menu
titles shows exactly one highlighted accelerator letter. Open any dropdown and observe that
each item likewise shows exactly one highlighted accelerator letter. Fully testable by
inspecting the rendered output; delivers immediate discoverability value.

**Acceptance Scenarios**:

1. **Given** the editor is at the normal editing screen, **When** the user looks at the menu
   bar, **Then** each top-level menu title (File, Edit, Search, View, Options, Help) shows
   exactly one accelerator letter rendered visually distinct from the rest of its label.
2. **Given** a dropdown menu is open, **When** the user looks at its items, **Then** every
   item shows exactly one accelerator letter rendered visually distinct from the rest of its
   label.
3. **Given** any single menu (bar level, or one dropdown), **When** its accelerators are
   compared, **Then** no two entries in that menu share the same accelerator letter
   (case-insensitive).

---

### User Story 2 - Activate a dropdown item by its accelerator letter (Priority: P1)

With a dropdown open, the user presses the accelerator letter shown for an item and that item
is activated immediately (the same effect as highlighting it and pressing Enter), and the
menu closes. The user no longer has to arrow down to each item.

**Why this priority**: This is the functional payoff of the visible accelerators and the
behavior DOS users expect. It is independently valuable: even one menu's items becoming
letter-activatable speeds up every interaction with that menu.

**Independent Test**: Open the File menu, press `N` → a new buffer is created and the menu
closes. Open the View menu, press the Soft-Wrap accelerator → soft wrap toggles. Testable by
driving keys and asserting the resulting action/state.

**Acceptance Scenarios**:

1. **Given** the File dropdown is open, **When** the user presses the accelerator letter for
   "New" (in any letter case), **Then** a new buffer is created and the menu closes.
2. **Given** a dropdown is open, **When** the user presses a letter that is not an accelerator
   of any item in that dropdown, **Then** nothing happens and the dropdown stays open.
3. **Given** a dropdown is open, **When** the user presses an item's accelerator, **Then** the
   effect is identical to highlighting that item and pressing Enter.

---

### User Story 3 - Open a top-level menu by Alt + its accelerator (Priority: P2)

From normal editing, the user presses `Alt` together with a top-level menu's accelerator
letter and that menu's dropdown opens. The accelerator the user presses is the one shown
highlighted in the bar — the visible indicator and the working key always agree.

**Why this priority**: Alt+letter opening of the six built-in menus already works; this story
guarantees the *visible* accelerator and the *bound* key are the same letter (consistency),
and extends the guarantee to any new top-level menus (e.g. plugin menus). It builds on US1.

**Independent Test**: For each top-level menu, the highlighted bar letter matches the letter
that, with Alt, opens that menu. Testable by comparing the rendered accelerator against the
keymap / open behavior.

**Acceptance Scenarios**:

1. **Given** normal editing, **When** the user presses `Alt` + a built-in menu's shown
   accelerator, **Then** that menu's dropdown opens.
2. **Given** the menu bar is active (highlighted) but no dropdown is open, **When** the user
   presses a top-level menu's accelerator (without Alt), **Then** that menu's dropdown opens.
3. **Given** any top-level menu, **When** its shown accelerator is compared with the key that
   opens it, **Then** they are the same letter.

---

### User Story 4 - Plugin menu items get accelerators automatically (Priority: P3)

A plugin that contributes menu items (and possibly a new top-level menu) has its items shown
with accelerator letters too, assigned automatically so they are unique within their menu and
do not collide with sibling items. The plugin author does not have to specify accelerators.

**Why this priority**: Plugins are an existing capability; mnemonics should not regress or
crash on plugin menus. Lower priority because most users run without plugins, but required
for correctness/parity.

**Independent Test**: Load a plugin contributing two items into a menu; both show distinct
accelerator letters; pressing each activates the correct plugin item. Testable with a
synthetic plugin menu.

**Acceptance Scenarios**:

1. **Given** a plugin contributes items to a menu, **When** that dropdown is open, **Then**
   each plugin item shows a unique accelerator within that dropdown.
2. **Given** a plugin contributes a new top-level menu, **When** the bar is shown, **Then**
   that menu shows an accelerator that is unique among the top-level menus.
3. **Given** a plugin item with an accelerator, **When** the user presses that letter while
   the dropdown is open, **Then** the plugin item is activated.

---

### Edge Cases

- **No available unique letter**: a label whose every letter is already used by a sibling (or
  has no letters at all, e.g. a separator) — the entry is shown with no accelerator and is
  reachable only by arrow keys + Enter or mouse. The menu must still render and operate.
- **Wide / multi-byte labels**: accelerator selection and the visual indicator must work on
  UTF-8 labels (including East-Asian wide characters and combining marks) without splitting a
  character or corrupting alignment.
- **Letter collision between an item accelerator and ordinary typing**: letter-activation only
  applies while a dropdown/bar is active; in normal editing the same key still types the
  character.
- **Ambiguous case**: accelerator matching is case-insensitive (`N` and `n` both activate the
  "New" item).
- **Separators / non-actionable rows**: never receive an accelerator and never activate. (The
  current built-in menus contain no separator rows; this rule is defensive/forward-looking so that
  any future non-actionable row is handled correctly.)
- **Terminal without the chosen highlight style**: the indicator must degrade so the label
  text is still fully readable (no lost or doubled characters).

## Clarifications

### Session 2026-06-20

- Q: How should the accelerator letter be visually indicated? → A: Underline the accelerator
  character (terminal underline attribute), theme-independent; degrade so the label stays readable
  where underline is unsupported.
- Q: How should built-in menu item accelerators be chosen? → A: Hand-author each built-in item's
  accelerator to match DOS/standard conventions (e.g. New=N, Open=O, Save=S, Save As=A, Exit=X);
  auto-compute accelerators only for plugin items.
- Q: Should tapping Alt alone (no letter) activate the menu bar? → A: Yes — tapping and releasing
  Alt by itself highlights the menu bar (top-level active, no dropdown), like F10; a subsequent
  letter then opens the matching menu.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every top-level menu in the bar MUST display exactly one accelerator letter,
  rendered with the terminal underline attribute so it is visually distinct from the rest of the
  title.
- **FR-002**: Every actionable dropdown item MUST display exactly one accelerator letter,
  rendered underlined and visually distinct from the rest of the label, unless no unique letter is
  available (see FR-006).
- **FR-003**: Within the menu bar, the top-level accelerators MUST be unique (case-insensitive);
  within each dropdown, the item accelerators MUST be unique (case-insensitive). Uniqueness is
  scoped per menu — the same letter MAY recur across different dropdowns.
- **FR-004**: While a dropdown is open, pressing a letter that matches an item's accelerator
  (case-insensitive) MUST activate that item exactly as if it were highlighted and Enter were
  pressed, then close the menu.
- **FR-005**: Pressing `Alt` + a top-level menu's accelerator MUST open that menu's dropdown;
  while the bar is active without a dropdown, pressing the accelerator alone MUST open it. The
  accelerator displayed for a menu MUST be the same letter that opens it.
- **FR-005a**: Tapping `Alt` by itself (no accompanying letter) MUST activate the menu bar at the
  top level (highlighted, no dropdown), identical to `F10`. Built-in item accelerators MUST follow
  DOS/standard conventions (hand-assigned: New=N, Open=O, Save=S, Save As=A, Exit=X, …); plugin
  item accelerators MUST be auto-assigned per FR-009.
- **FR-006**: When a label has no letter that can be made unique within its menu (all candidate
  letters already taken, or the label has no letters), the entry MUST render with no accelerator
  indicator and MUST remain reachable via arrow-key navigation, Enter, and mouse.
- **FR-007**: Pressing a letter that matches no accelerator in the currently open menu MUST be a
  no-op that leaves the menu open (it MUST NOT fall through to editing the buffer).
- **FR-008**: Accelerator assignment MUST be deterministic: the same menu set always yields the
  same accelerators (so the visible letters are stable across runs and across renders).
- **FR-009**: Plugin-contributed menu items and any plugin-contributed top-level menus MUST
  receive accelerators by the same rules, assigned automatically, unique within their scope, and
  without colliding with built-in entries in the same scope.
- **FR-010**: Accelerator selection, matching, and the visual indicator MUST be correct for
  UTF-8 labels, never splitting a multi-byte character and never misaligning the dropdown.
- **FR-011**: Letter-activation and the visible indicators MUST NOT change behavior outside an
  active menu: in normal editing every key behaves exactly as before this feature.
- **FR-012**: The existing menu interactions MUST continue to work unchanged: `F10` activation,
  arrow navigation, `Enter` to activate, `Esc` to close, mouse click/double-click, and the
  existing `Ctrl`/`F`-key shortcuts shown alongside items.

### Key Entities *(include if feature involves data)*

- **Menu accelerator**: the single character within a menu title or item label that is
  highlighted and that activates the entry. Attributes: which character position in the label it
  occupies; the matching key (case-insensitive). Belongs to exactly one menu entry; unique within
  its menu scope.
- **Menu entry**: an existing top-level menu or dropdown item; gains an optional accelerator. A
  separator or letter-less entry has no accelerator.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of top-level menus and 100% of actionable items that have at least one
  assignable letter display exactly one visible accelerator.
- **SC-002**: For every visible item accelerator, pressing that letter while its dropdown is open
  activates that exact item (100% correspondence between shown letter and working key).
- **SC-003**: Within any single menu, there are zero duplicate accelerators (case-insensitive).
- **SC-004**: A user can reach and trigger any built-in menu action using only the keyboard via
  visible accelerators (Alt+letter to open, then the item letter) without using arrow keys.
- **SC-005**: With a plugin contributing menu items loaded, every plugin item shows a unique
  accelerator within its dropdown and activates correctly; no crash or duplicate.
- **SC-006**: No regression: all pre-existing menu navigation, shortcuts, and editing keys behave
  exactly as before when no menu is active.

## Assumptions

- The visible indicator is the terminal underline attribute on the accelerator character
  (clarified 2026-06-20); it degrades gracefully so the label stays fully readable where underline
  is unsupported (FR-001/FR-002).
- The six built-in top-level accelerators follow the DOS/standard convention of the menu's first
  letter: File→F, Edit→E, Search→S, View→V, Options→O, Help→H (these already back the existing
  `Alt+letter` bindings).
- Built-in dropdown item accelerators are hand-assigned to DOS/standard letters (clarified
  2026-06-20). Plugin item accelerators are auto-selected deterministically — prefer the first
  letter of the label, then the next unused letter — satisfying uniqueness (FR-003) and determinism
  (FR-008).
- Letter-activation inside a dropdown applies to printable letter keys without modifiers; it does
  not override the existing `Esc`, arrow, and `Enter` handling.
- This feature concerns only discoverability/activation of existing menu entries; it does not add,
  remove, or rename any menu or item, nor change what each action does.
- Plugin menu labels are arbitrary UTF-8 supplied at load time; accelerators for them are computed
  at resolve time, not authored by the plugin.
