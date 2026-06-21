# Tasks: Per-Tab Soft-Wrap

**Feature**: `044-per-tab-soft-wrap` | **Branch**: `044-per-tab-soft-wrap`
**Input**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/internal-api.md](./contracts/internal-api.md)

**Overriding constraint**: Behavior-preserving for the single-tab / untouched-default case (FR-007). No
existing test's *behavior* changes — soft-wrap test assertions only move from `app.soft_wrap` to
`app.active_buffer().soft_wrap`. No panic/corruption in any tab/pane/wrap combo (FR-008); the 042
`clippy::unwrap_used` guardrail and the 043 cache-invalidation must keep holding.

**Story → phase map**: US1 = per-tab independence (P1), US2 = indicators/toggle track active tab (P1).
The field move + reader retargeting serve both.

---

## Phase 1: Setup

- [ ] T001 Baseline: `make tmpfs-setup` then `make check`; note the passing count (1268/0/11) for the
  unchanged-suite comparison.

---

## Phase 2: Foundational (field move)

- [ ] T002 Add `pub soft_wrap: bool` to `Buffer` (`src/buffer/mod.rs`); default `false` in
  `new_empty()` and `open()`.
- [ ] T003 Remove `App::soft_wrap` (field + constructor init in `src/app.rs`). Build → the compiler now
  enumerates every reader (drives T004–T008).

**Checkpoint**: type error list = the exact reader set to migrate.

---

## Phase 3: User Story 1 — Each tab keeps its own wrap setting (Priority: P1)

**Goal**: per-buffer storage + readers + seeding so toggling one tab never affects another.

- [ ] T004 [US1] Seed `soft_wrap` from `config.soft_wrap` at every buffer-creation site: initial
  buffer(s) in `App::new` (`src/app.rs`), `new_buffer` + `handle_open_file` (`src/app/actions.rs`),
  `do_restore_session` (`src/app/fileops.rs`), and the replace-with-empty path in `close_buffer_at`.
- [ ] T005 [US1] Retarget active-buffer readers to `self.active_buffer().soft_wrap`: geometry/scrollbar
  helpers + event-loop cache gate in `src/app.rs`; `content_width` and the toggle's guard in
  `src/app/softwrap.rs`; the `wrap_cache` gate in `src/app/editing.rs`; wheel/hit-test wrap checks in
  `src/app/mouse.rs`; the session cache gate in `src/app/fileops.rs`.
- [ ] T006 [US1] Retarget `Ui::render` (`src/ui/mod.rs`): single view → `app.active_buffer().soft_wrap`;
  split view → each pane uses its own buffer's flag (`buffers[0]` left, `buffers[right_idx]` right) for
  `editor_panes` layout and the `EditorWidget` `soft_wrap` argument; pass the active buffer's
  `wrap_cache` only to the pane whose buffer index == `active_idx` (the other pane gets
  `wrap_starts = None` and renders best-effort — `EditorWidget` handles `soft_wrap` + no cache).

**Checkpoint (US1)**: build green; toggling/​rendering uses per-buffer flags.

---

## Phase 4: User Story 2 — Indicators & toggle track the active tab (Priority: P1)

- [ ] T007 [US2] Toggle: `handle_toggle_soft_wrap` (`src/app/softwrap.rs`) flips
  `active_buffer().soft_wrap` and invalidates the wrap cache; remove the `self.config.soft_wrap = …`
  write (config is now a default seed only).
- [ ] T008 [US2] Indicators: feed `StatusBar.soft_wrap` (`src/ui/mod.rs` → `src/ui/statusbar.rs`) from
  the active buffer; set the `View ▸ Soft Wrap` menu toggle-state (`src/ui/mod.rs` + `src/app/mouse.rs`
  `toggle_states`) from `app.active_buffer().soft_wrap`.

**Checkpoint (US2)**: menu check + status indicator track the active tab; toggle flips only it.

---

## Phase 5: Tests

- [ ] T009 [US1] Update existing soft-wrap tests in `src/app/tests.rs` to read
  `app.active_buffer().soft_wrap` (assertions unchanged in meaning).
- [ ] T010 [US1] Add per-tab independence test: two buffers; toggle wrap on the active one; assert the
  other's flag unchanged; switch and switch back; assert each retains its own setting (SC-001).
- [ ] T011 [US2] Add indicator/toggle test: with two buffers in different wrap states, assert the toggle
  flips only the active buffer and that the value the status bar/menu would show equals the active
  buffer's flag (SC-003). Add a render check that a switched-to tab renders for its own setting (SC-002).

---

## Phase 6: Polish & Ship

- [ ] T012 `make ci-local` (fmt → clippy -D warnings → test → smoke → perf-check). Confirm count ==
  baseline + new tests; the pre-existing env `encoding_select` smoke failure (F12) is unrelated.
- [ ] T013 Confirm FR-008 via the existing 042 fuzz sweep (it toggles soft-wrap + switches buffers) —
  still green with per-tab state.
- [ ] T014 Docs gate: `CHANGELOG.md` + `docs/STATUS.md`, AND `docs/CAPABILITIES.md` — update the
  soft-wrap toggle line (currently "Toggle soft-wrap mode (non-DOS extension)", ~line 131) to reflect
  that wrap is now **per tab** (user-visible scope change to an existing capability).
- [ ] T015 PR targeting `master` (`feat(044): per-tab soft-wrap`), strip AI-attribution, green, merge.

---

## Dependencies & Order

Setup (T001) → field move (T002–T003) → US1 readers/seeding (T004–T006) → US2 toggle/indicators
(T007–T008) → tests (T009–T011) → polish/ship (T012–T015). T003 (remove field) is the forcing function:
everything after fixes the resulting compile errors. Low parallelism (one field, many call sites in
shared files).

## MVP

US1 (per-buffer storage + readers + seeding) is the core; US2 (indicators/toggle) completes the UX. Both
ship in one PR.

## Implementation Strategy

Remove `App::soft_wrap` and let the compiler list every reader; fix each to the relevant buffer's flag;
seed at creation; retarget toggle + indicators; add per-tab tests. Keep the suite green throughout.
