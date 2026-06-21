# Feature Specification: Recent-Files List

**Feature Branch**: `049-recent-files` | **Created**: 2026-06-21 | **Status**: Draft
**Input**: Issue #81 — "Recent-files list (constitution baseline: recent_files_limit)."

## Overview

The constitution lists a recent-files capability (`recent_files_limit`) but it was never implemented.
This feature tracks the files you open, persists the list across sessions, and surfaces it in the **File
menu** so you can reopen a recent file in one click. It honors a configurable cap.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Reopen a recent file from the menu (Priority: P1)
As a user, the File menu shows my recently opened files (most-recent first); choosing one reopens it.

**Independent Test**: open a few files, then assert the File menu lists them most-recent-first and that
choosing one opens that path.

**Acceptance Scenarios**:
1. **Given** I have opened files A then B, **When** I view the File menu, **Then** it lists B above A.
2. **Given** the recent list, **When** I choose an entry, **Then** that file opens (as a new/!active tab).
3. **Given** I reopen A, **When** I view the list, **Then** A moves to the top (no duplicate).

### User Story 2 - Persisted and capped (Priority: P1)
As a user, the recent list survives restart and never grows beyond the configured limit.

**Acceptance Scenarios**:
1. **Given** I open files and quit, **When** I relaunch, **Then** the recent list is preserved.
2. **Given** `recent_files_limit = N`, **When** I open more than N files, **Then** only the N most recent
   are kept.

### Edge Cases
- A recent entry whose file no longer exists → opening it surfaces the normal "open failed" message
  (no crash); the stale entry may be dropped on failure.
- Unsaved/untitled buffers (no path) are never recorded.
- A corrupt/absent recent-list file loads as empty (no error).
- `recent_files_limit = 0` → feature effectively disabled (no entries shown/kept).

## Requirements *(mandatory)*
- **FR-001**: Opening a file (and Save-As to a new path) MUST record its path at the front of a
  recent-files list, de-duplicated (existing entry moves to front).
- **FR-002**: The list MUST be capped at `config.recent_files_limit` (a new config key; sensible default).
- **FR-003**: The list MUST persist across sessions (stored under the editor's state dir).
- **FR-004**: The File menu MUST show the recent files (most-recent first) and choosing one MUST open it.
- **FR-005**: Untitled/no-path buffers MUST NOT be recorded; a missing/corrupt store loads as empty;
  opening a now-missing recent file MUST NOT crash (normal open-failure handling).
- **FR-006**: Behavior MUST be unchanged when the list is empty or the limit is 0 (no menu entries; no
  errors); existing menu/dispatch behavior otherwise preserved.
- **FR-007**: Full suite + `clippy -D warnings` + `fmt` clean; 042/046 guardrails hold.

## Success Criteria *(mandatory)*
- **SC-001**: Opening files records them most-recent-first, de-duplicated, capped at the limit (test).
- **SC-002**: The list round-trips through persistence (save/load) (test).
- **SC-003**: The File menu contains the recent entries and an `OpenRecent` action resolves to the path
  (test).
- **SC-004**: Empty list / limit 0 leaves the menu and behavior unchanged (test).
- **SC-005**: Full suite + lints green.

## Assumptions
- Stored as a small list file under `$XDG_STATE_HOME/edit/` (e.g. `recent.toml`), like the session file.
- Surfaced by injecting items into the File menu via the existing dynamic-menu mechanism
  (`resolve_menus`, which already merges plugin items); a new `Action::OpenRecent(index)` opens the path.
- Default `recent_files_limit` = 10.
- Recorded on open + save-as; not on every cursor move. Display shows the file name (path on collision is
  acceptable; full path is the stored value).
