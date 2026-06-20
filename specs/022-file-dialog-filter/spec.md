# Feature Specification: File dialog — glob filtering + richer entry details

**Feature Branch**: `022-file-dialog-filter`

**Created**: 2026-06-20

**Status**: Draft

**Input**: User description: "File dialog glob filtering + richer entry details (feature 022). Typing a
glob like `*.log` does nothing useful today (the field only acts on an absolute path, so a glob is
silently ignored and Enter just opens the highlighted entry — appears to 'just close'); and the listing
shows only the name with no detail. Add live glob/substring filtering of the listing and per-entry
size + modified-date columns. Decisions: live-as-you-type; case-insensitive; plain text also filters
(substring); absolute paths keep their jump behavior."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Filter the listing by typing a pattern (Priority: P1)

In the file dialog (Open or Save), the user types into the field to narrow the listing to matching
entries. A pattern with wildcards (`*.log`, `*.rs`, `te?t.txt`) glob-matches entry names; plain text
(`rep`) matches any entry whose name contains it. Matching is case-insensitive. Directories and the `..`
parent entry always remain visible so the user can still navigate while a filter is active. Clearing the
field restores the full listing. An absolute path (starting with `/`) keeps today's behavior — it is a
jump target, not a filter.

**Why this priority**: Typing a glob today silently does nothing and Enter "just closes" the dialog —
the core complaint. Filtering is the headline fix and the main value.

**Independent Test**: Open a directory containing `a.log`, `b.txt`, and a sub-folder; type `*.log` → only
`a.log` (plus directories and `..`) remain; type `b` → only `b.txt` (plus dirs/`..`); clear the field →
all entries return.

**Acceptance Scenarios**:

1. **Given** a directory with mixed files, **When** the user types a wildcard pattern, **Then** the
   listing shows only files whose names match the glob (case-insensitive), with all directories and `..`
   still shown.
2. **Given** a filter is typed, **When** the user types plain text with no wildcard, **Then** the listing
   shows entries whose names contain that text (case-insensitive), plus directories and `..`.
3. **Given** a filter is active, **When** the user clears the field, **Then** the full listing is restored.
4. **Given** the user types an absolute path, **When** they press Enter, **Then** the existing jump-to-path
   / open behavior runs (the text is treated as a path, not a filter).
5. **Given** a filter hides the previously-selected entry, **When** the filter updates, **Then** the
   selection moves to a valid visible entry (no selection of a hidden row).
6. **Given** a filter that matches no files, **When** it is applied, **Then** the listing still shows the
   directories and `..` (the user can always navigate out) and indicates no matching files.

### User Story 2 - See file/folder details in the listing (Priority: P1)

Each entry in the listing shows useful detail next to its name: a human-readable size for files (e.g.
`1.2K`, `3.4M`) and a modified date; directories show a directory indicator instead of a size. Columns
are aligned, and the name is truncated (not the detail columns) when the row is too narrow.

**Why this priority**: "Not enough details about the file or folder displayed" was the second half of the
report; size/date are the standard, expected columns in a file picker.

**Independent Test**: Open a directory with a small and a large file and a sub-folder → each file row
shows a size and a date, the folder row shows a directory indicator (no size), and columns line up.

**Acceptance Scenarios**:

1. **Given** the listing is shown, **When** an entry is a file, **Then** its row shows a human-readable
   size and a modified date aligned in their columns.
2. **Given** the listing is shown, **When** an entry is a directory (or `..`), **Then** its row shows a
   directory indicator instead of a size.
3. **Given** a narrow dialog, **When** a name is too long to fit beside the detail columns, **Then** the
   name is truncated (with an ellipsis) while the size/date columns stay readable and aligned.
4. **Given** entries with multi-byte/wide names, **When** rendered, **Then** column alignment and
   truncation remain visually correct.

### Edge Cases

- **Filter + scrollbar**: when a filter shrinks the list below the visible rows, no scrollbar is shown;
  when results still overflow, the scrollbar reflects the filtered count.
- **Filter + buttons/focus ring**: the feature-020 Open/Save/Cancel buttons and focus ring keep working
  while a filter is active; Tab still reaches them.
- **Save mode**: typing a filename also filters the listing (substring) as a preview of existing matches;
  pressing the confirm action still saves the typed filename (Save semantics unchanged).
- **Unreadable entry metadata**: if size/date can't be read for an entry, the row still renders (the
  missing detail is shown blank or as a placeholder) without failing the listing.
- **Very long / huge files**: size formatting handles bytes through gigabytes without overflow or
  mis-rounding.
- **Empty directory / no matches**: the listing still shows `..` (and any sub-dirs) so the user can leave.
- **Resize while filtered**: columns re-flow and the name truncation recomputes for the new width.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The file dialog MUST filter the listing live as the field text changes: a pattern containing
  wildcards (`*`, `?`) glob-matches entry names; other (non-absolute-path) text matches names by
  case-insensitive substring.
- **FR-002**: Filter matching MUST be case-insensitive.
- **FR-003**: Directories and the `..` parent entry MUST always remain in the listing regardless of the
  active filter, so navigation is never blocked.
- **FR-004**: Clearing the field MUST restore the full, unfiltered listing.
- **FR-005**: An absolute path typed in the field MUST retain its existing jump-to-path / open-on-confirm
  behavior and MUST NOT be treated as a filter pattern.
- **FR-006**: When the active filter would hide the current selection, the selection MUST move to a valid
  visible entry (never point at a hidden row).
- **FR-007**: Each file entry MUST display a human-readable size and a modified date; each directory (and
  `..`) MUST display a directory indicator instead of a size.
- **FR-008**: Detail columns MUST be aligned across rows; when a row is too narrow, the entry **name**
  MUST be truncated (with an ellipsis) rather than the detail columns, and truncation MUST be
  grapheme/width-correct for multi-byte names.
- **FR-009**: The feature MUST apply to both Open and Save modes; Save-mode confirm (saving the typed
  filename) and Open-mode confirm (jump/open) semantics MUST be unchanged.
- **FR-010**: All existing file-browser navigation (arrow keys, parent/enter, mouse click/double-click),
  the feature-020 buttons + focus ring, and the feature-021 scrollbar MUST continue to work with
  filtering and detail columns present.
- **FR-011**: The listing (filtering + detail rendering) MUST NOT panic on unreadable metadata, empty
  directories, no-match filters, or tiny terminals.

### Key Entities

- **Listing filter**: the current field text interpreted as either an absolute path (jump target), a glob
  pattern, or a substring — producing the set of visible entries (always including directories and `..`).
- **Entry detail**: per-entry metadata shown in the listing — kind (file/dir/parent), human-readable
  size (files only), and modified date.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Typing `*.log` (or any glob) in a directory narrows the listing to matching files plus
  directories/`..`, live, without pressing Enter.
- **SC-002**: Typing plain text narrows the listing to name-substring matches (case-insensitive); clearing
  the field restores all entries.
- **SC-003**: Every file row shows a size and a modified date, and every directory row shows a directory
  indicator, with columns aligned.
- **SC-004**: 100% of existing file-dialog navigation, buttons, focus-ring, and scrollbar behaviors still
  work with filtering and details active (zero regression), verified by tests.
- **SC-005**: The listing never panics or corrupts its layout across terminal sizes, multi-byte names,
  unreadable metadata, and empty/no-match results.

## Assumptions

- **Live filtering** (per user decision): the listing updates on every keystroke, not only on Enter.
- **Case-insensitive matching** (per user decision).
- **Plain text filters by substring** (per user decision); a string containing `*`/`?` is treated as a
  glob; a string starting with `/` is treated as an absolute path (jump), not a filter.
- **Size formatting** uses human-readable units (B/K/M/G) with a small number of significant digits;
  exact byte counts are not required in the listing.
- **Date format** is a compact, locale-independent representation (e.g. `YYYY-MM-DD HH:MM`); exact
  timezone handling is best-effort from filesystem metadata.
- **Scope** is the file browser only (`src/ui/file_browser.rs` and its app wiring); no other dialogs or
  the editor are affected.
- Builds on feature 012 (file browser), feature 020 (dialog buttons/focus ring), and feature 021
  (scrollbar) — all of which must keep working.
