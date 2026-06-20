---
description: "Task list for feature 029 — UX completeness hardening (round 2)"
---

# Tasks: UX completeness hardening (round 2)

**Input**: Design documents from `specs/029-ux-completeness-hardening/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/behavior.md, quickstart.md

**Tests**: REQUIRED — Constitution Principle V (Test-Gated Merges). Tests first.

**Organization**: Setup → Foundational (shared width helper) → US1 crash → US2 save-data-loss →
US3 dialogs → US4 encoding → US5 click+width → US6 feedback → US7 reachability/theme → Polish/deferrals.

## Format: `[ID] [P?] [Story?] Description`

---

## Phase 1: Setup

- [X] T001 Confirm a clean baseline build on branch `029-ux-completeness-hardening` (`make tmpfs-setup`, `make`).
- [X] T002 Register a new integration test target `ux_round2` in `Cargo.toml` and create `tests/integration/ux_round2.rs`.

---

## Phase 2: Foundational — unified display width (Blocking for US5)

- [X] T003 [P] Unit tests in `src/ui/width.rs`: `display_width("a")==1`, combining mark (U+0301) ==0, CJK (世) ==2, emoji handled; `str_width` sums graphemes.
- [X] T004 Create `src/ui/width.rs` with `display_width(grapheme: &str) -> usize` and `str_width(s: &str) -> usize` built on `unicode_width`; export from `src/ui/mod.rs`.
- [X] T005 Replace `file_browser::grapheme_width` and `app::unicode_segmentation_width` (and the editor click width call) with `ui::width`; keep `truncate_to_width` using the shared width. Remove the now-dead custom helpers. (FR-010)

**Checkpoint**: one width function in use everywhere; `make check` green.

---

## Phase 3: US1 — no operation can crash (Priority: P1) 🎯

- [X] T006 [P] [US1] Unit test in `src/app.rs`: `delete_selection` over multibyte ("éàûü") removes the right chars, pushes correct undo text, and is a no-op on an empty/reversed range — no panic.
- [X] T007 [US1] In `src/app.rs` `delete_selection`, extract the deleted text char-safely (reuse `selection_text`/char range), not by byte slice. (FR-001)
- [X] T008 [P] [US1] Unit test in `src/ui/dialog.rs`: recovery-dialog render with a long Unicode path does not panic and shows a truncated path.
- [X] T009 [US1] In `src/ui/dialog.rs`, truncate the recovery path by char/width with a leading `...`, never by byte slice. (FR-002)
- [X] T010 [P] [US1] Unit test in `src/buffer/rope.rs`: `byte_to_char` returns a correct count for a non-char-boundary offset and for `>= len`, no panic.
- [X] T011 [US1] In `src/buffer/rope.rs`, clamp `byte_idx` to the nearest char boundary (and to len) before counting. (FR-003)
- [X] T012 [P] [US1] Unit test in `src/buffer/mod.rs`: opening a file whose size exceeds `MAX_OPEN_BYTES` returns a "file too large" `BufferError` (use a small override or a crafted check), no read.
- [X] T013 [US1] In `src/buffer/mod.rs` `Buffer::open`, stat the file and return a clear "file too large" error above `MAX_OPEN_BYTES` (documented constant) before reading. (FR-004)

**Checkpoint**: multibyte/Unicode/oversized inputs never panic; `make check` green.

---

## Phase 4: US2 — saving never silently loses data (Priority: P1)

- [X] T014 [P] [US2] Unit tests in `src/app.rs`: plain save success sets a "Saved" status; a forced save failure sets a "Save failed" status and keeps `modified = true`.
- [X] T015 [US2] In `src/app.rs` `handle_save_action`, set `status_message` on success ("Saved"/name) and on failure ("Save failed: <reason>", keep modified). (FR-005)
- [X] T016 [US2] Surface autosave/recovery write failures (src/buffer/autosave.rs → app) as a non-intrusive notice; keep logging. Add a focused test. (FR-006)

**Checkpoint**: a failed save is never mistaken for success; `make check` green.

---

## Phase 5: US3 — dialog consistency (Priority: P1)

- [X] T017 [P] [US3] Unit test in `src/app.rs`: with `pending_save_prompt`, `Action::MenuClose` cancels (prompt cleared, not saved/discarded).
- [X] T018 [US3] In `src/app.rs` SavePrompt intercept, add `MenuClose | Quit => prompt_cancel_quit()`. (FR-007)
- [X] T019 [P] [US3] Unit test: a Go-to-Line request while `menu_bar.is_active()` does not open the prompt. 
- [X] T020 [US3] Guard the Go-to-Line open path with `!menu_bar.is_active()` (consistent with other modals). (FR-016)

**Checkpoint**: `make check` green.

---

## Phase 6: US4 — Save-As encoding via browser (Priority: P1)

- [X] T021 [P] [US4] Unit test in `src/app.rs`: with `pending_save_as_encoding = Some(enc)`, completing the browser Save path applies `enc` to the buffer (and clears the pending value).
- [X] T022 [US4] In `src/app.rs` (the `apply_browse_outcome`/`do_save_as` Save path), apply `pending_save_as_encoding` before writing; clear it after. (FR-008)

**Checkpoint**: `make check` green.

---

## Phase 7: US5 — correct clicks + wide-char alignment (Priority: P2)

- [X] T023 [P] [US5] Unit test in `src/app.rs`: with `config.line_numbers = true`, a click at terminal column C maps to text column `C - gutter` (+ horizontal scroll); a click within the gutter clamps to column 0.
- [X] T024 [US5] In `src/app.rs` `handle_mouse_click`, subtract the gutter width and add `scroll_offset.1` in both the soft-wrap and non-wrap branches; use `ui::width` for grapheme widths. (FR-009)

**Checkpoint**: clicks land correctly with line numbers + scroll; `make check` green.

---

## Phase 8: US6 — action feedback / no silent no-ops (Priority: P2)

- [X] T025 [P] [US6] Unit tests in `src/app.rs`: copy/cut set "Copied"/"Cut"; paste of empty clipboard → "Nothing to paste"; an edit attempt on a read-only buffer sets "Buffer is read-only".
- [X] T026 [US6] In `src/app.rs`, add status feedback to copy/cut/paste (success + empty-clipboard + clipboard-unavailable) and a read-only message at the central edit guard. (FR-011, FR-012)
- [X] T027 [P] [US6] Unit test: a file-open failure surfaces an "Open failed: <path> — <reason>" status (and startup non-NotFound errors are surfaced rather than a silent blank buffer).
- [X] T028 [US6] In `src/app.rs` (startup open mapping ~318-340 and the Ctrl+O open path), surface open errors as a status message; preserve the new-file (NotFound) behavior. (FR-013)

**Checkpoint**: every listed action gives feedback; `make check` green.

---

## Phase 9: US7 — Close reachable + theme legible (Priority: P3)

- [X] T029 [P] [US7] Unit tests: `default_map()` binds `Ctrl+W → Action::Close`; both bundled themes render a selected menu item with fg != bg.
- [X] T030 [US7] Bind `Ctrl+W → Action::Close` in `src/input/keymap.rs`; add a `File ▸ Close` menu item in `src/ui/menubar.rs` routing to `Action::Close`. (FR-014)
- [X] T031 [US7] In `src/ui/theme.rs`, give the light theme a contrasting `menu_selected_bg` so the selected item is legible; add a headless render assertion. (FR-015)

**Checkpoint**: `make check` green.

---

## Phase 10: Polish, deferrals & docs

- [X] T032 [P] Integration tests in `tests/integration/ux_round2.rs`: end-to-end via `handle_action`/render — SavePrompt Esc, save feedback, copy/read-only feedback, Ctrl+W close, click-with-line-numbers column, width alignment.
- [X] T033 [P] Update `CHANGELOG.md` (feature 029 under `[Unreleased]`), `docs/STATUS.md`, `docs/CAPABILITIES.md` (Ctrl+W now real; feedback messages; file-size limit).
- [X] T034 File the DEFERRED enhancements as GitHub issues with the `follow-up` label and add ROADMAP.md rows referencing them: (a) in-dialog mouse text editing & list-item clicks; (b) double-click-word / triple-click-line selection; (c) right-click context menu; (d) extra DOS F-keys (F4/F6–F9/F11). (Constitution deferral rule.)
- [X] T035 Run `make ci-local`; fix findings; note the known pre-existing sandbox smoke failure (F12/Ctrl+O PTY) is not a regression.
- [X] T036 Run the `specs/029-ux-completeness-hardening/quickstart.md` manual walkthrough.

---

## Dependencies & Execution Order

- Setup → Foundational width (blocks US5's width use). US1–US7 are largely independent (different
  surfaces); land P1 stories (US1–US4) first, then P2 (US5–US6), then P3 (US7). Polish/deferrals last.

### Parallel opportunities

- All `[P]` unit-test tasks; T033/T034 docs/issues are `[P]`.

## Implementation Strategy

TDD per fix (Constitution V). No new crates (IV). No silent data loss (VII). Reuse `selection_text`, the
existing status-message channel, `unicode-width`, and the feature-016 dialog infra. Branch
`029-ux-completeness-hardening`, PR to `master`, merge via GitHub. No AI attribution in commits/PR/issues.
