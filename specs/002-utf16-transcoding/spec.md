# Feature Specification: UTF-16 Transcoding Support

**Feature Branch**: `002-utf16-transcoding`

**Created**: 2026-06-18

**Status**: Draft

**GitHub Issue**: #5

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Open a UTF-16 file without garbled display (Priority: P1)

A developer receives a UTF-16 LE text file from a Windows colleague (common for files
produced by Notepad, PowerShell `Out-File`, or many Windows build tools). They open it
with `edit` and expect to read and edit it exactly as they would any other text file,
without garbled characters or a binary-content warning.

**Why this priority**: This is the core gap. Without it, UTF-16 files are unreadable in the
editor regardless of any other feature.

**Independent Test**: Open a UTF-16 LE file with a BOM (`\xFF\xFE`) containing ASCII and
non-ASCII characters; verify text displays correctly and cursor movement is accurate.

**Acceptance Scenarios**:

1. **Given** a UTF-16 LE file with BOM, **When** the user opens it with `edit`, **Then** the
   content is displayed correctly, the status bar shows `UTF-16 LE`, and no garbled or
   replacement characters appear.
2. **Given** a UTF-16 BE file with BOM (`\xFE\xFF`), **When** the user opens it, **Then**
   the content is displayed correctly and the status bar shows `UTF-16 BE`.
3. **Given** a UTF-16 LE file with no BOM, **When** the user opens it, **Then** the editor
   either detects the encoding automatically or prompts to confirm encoding, and content is
   displayed correctly.
4. **Given** a UTF-16 file containing characters outside the Basic Multilingual Plane
   (surrogate pairs), **When** the user opens it, **Then** the supplementary characters are
   correctly decoded and displayed.

---

### User Story 2 — Save a file preserving its original encoding (Priority: P1)

After editing a UTF-16 file, the user saves it and expects the output file to remain UTF-16
(same byte-order and BOM) so it is readable by the originating Windows tool.

**Why this priority**: Opening without saving correctly is only half the feature; the file
must survive a round-trip.

**Independent Test**: Open a UTF-16 LE file, make a small edit, save, and verify with
`hexdump` that the output file has the `\xFF\xFE` BOM and UTF-16 LE encoding.

**Acceptance Scenarios**:

1. **Given** a UTF-16 LE file that was opened and edited, **When** the user saves (Ctrl+S),
   **Then** the file is written as UTF-16 LE with the BOM preserved.
2. **Given** a UTF-16 BE file that was opened and edited, **When** the user saves,
   **Then** the file is written as UTF-16 BE with the BOM preserved.
3. **Given** any UTF-16 file, **When** the user performs save-as to a new path, **Then**
   the new file uses the same encoding as the original.

---

### User Story 3 — Force UTF-16 encoding via CLI flag (Priority: P2)

A user has a UTF-16 file without a BOM (produced by some tools). They open it with
`edit --encoding utf-16-le myfile.txt` to force the correct encoding.

**Why this priority**: BOM-less UTF-16 cannot be reliably auto-detected; a CLI override is
the safe fallback.

**Independent Test**: Open a known UTF-16 LE file without BOM using `--encoding utf-16-le`
and verify correct display.

**Acceptance Scenarios**:

1. **Given** a BOM-less UTF-16 LE file, **When** opened with `--encoding utf-16-le`,
   **Then** content displays correctly with no replacement characters.
2. **Given** a BOM-less UTF-16 BE file, **When** opened with `--encoding utf-16-be`,
   **Then** content displays correctly.
3. **Given** an invalid encoding name, **When** passed to `--encoding`, **Then** the editor
   reports an error and falls back to UTF-8.

---

### User Story 4 — Convert a file to UTF-16 via save-as (Priority: P3)

A user has a UTF-8 file and needs to produce a UTF-16 LE copy for a Windows tool. They use
save-as and select UTF-16 LE encoding to create the converted copy.

**Why this priority**: Useful but not core to the initial implementation; requires UI changes
beyond the basic encoding pipeline.

**Independent Test**: Open a UTF-8 file, save-as with UTF-16 LE encoding selected, verify
with `hexdump` that the output is UTF-16 LE with BOM.

**Acceptance Scenarios**:

1. **Given** a UTF-8 file is open, **When** the user invokes save-as and selects UTF-16 LE
   encoding, **Then** the saved file is UTF-16 LE with `\xFF\xFE` BOM.
2. **Given** any file is open, **When** the user saves-as with UTF-16 BE encoding,
   **Then** the saved file is UTF-16 BE with `\xFE\xFF` BOM.

---

### Edge Cases

- What happens when a file has a UTF-16 BOM but contains malformed surrogate pairs?
  → The editor must report a decode error and offer to open in binary-safe mode or abort.
- What happens when a UTF-16 file contains a null byte that the underlying OS treats as
  a filename terminator? → The file path itself is UTF-8; this only applies to file content,
  which the rope buffer handles as bytes internally.
- What happens when UTF-16 text contains characters that cannot round-trip through the
  editor's internal UTF-8 representation? → All Unicode code points can be represented in
  UTF-8; round-trip loss is impossible for well-formed UTF-16.
- What happens when the file size is odd (broken UTF-16)? → The editor reports a
  decode error (UTF-16 requires an even number of bytes).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor MUST detect UTF-16 LE and UTF-16 BE encodings via BOM inspection
  on file open, before any content is read into the buffer.
- **FR-002**: The editor MUST transcode UTF-16 LE byte streams to the internal UTF-8
  representation, preserving all Unicode code points including supplementary characters
  (surrogate pairs).
- **FR-003**: The editor MUST transcode UTF-16 BE byte streams to the internal UTF-8
  representation with the same fidelity as UTF-16 LE.
- **FR-004**: The editor MUST write UTF-16 LE files with a `\xFF\xFE` BOM when saving
  a buffer whose encoding is UTF-16 LE.
- **FR-005**: The editor MUST write UTF-16 BE files with a `\xFE\xFF` BOM when saving
  a buffer whose encoding is UTF-16 BE.
- **FR-006**: The editor MUST accept `utf-16-le` and `utf-16-be` as valid values for the
  `--encoding` CLI flag, overriding auto-detection.
- **FR-007**: The status bar MUST display the active encoding as `UTF-16 LE` or `UTF-16 BE`
  for buffers using those encodings.
- **FR-008**: The editor MUST report a clear, human-readable error when a file claimed to be
  UTF-16 contains malformed byte sequences (odd length, invalid surrogates).
- **FR-009**: The encoding registry and `encoding_from_str()` function MUST recognise
  common aliases: `utf-16-le`, `utf16-le`, `utf16le`, `utf-16-be`, `utf16-be`, `utf16be`.
- **FR-010**: Saving a UTF-16 buffer MUST produce a byte-for-byte identical file to the
  original when no edits were made (pure open-save round-trip).

### Key Entities

- **EncodingId**: Extended enumeration that includes `Utf16Le` and `Utf16Be` variants
  alongside existing `Utf8`, `Cp437`, `Cp850`, `Iso8859_1`, `Windows1252`.
- **EncodingProfile**: Registry entry for each encoding (id, display name, aliases).
- **Buffer**: The open-file state machine that owns encoding, line-ending, and content;
  must propagate the detected or forced encoding through open → edit → save.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A UTF-16 LE file with BOM opens and displays correctly in under 1 second
  for files up to 10 MB.
- **SC-002**: A UTF-16 round-trip (open → no edit → save) produces a byte-identical output
  file 100% of the time.
- **SC-003**: All existing 196+ tests continue to pass after the change (no regressions).
- **SC-004**: At least 15 new tests cover UTF-16 LE/BE detection, decode, encode, round-trip,
  error cases (malformed, odd-length, alias recognition), and CLI flag override.
- **SC-005**: The `--encoding utf-16-le` and `--encoding utf-16-be` flags are accepted
  and correctly applied as verified by the integration test suite.

## Assumptions

- The internal buffer representation remains UTF-8; UTF-16 is only a file I/O encoding, not
  an internal format. This is consistent with the existing design and Constitution Principle II.
- BOM-less UTF-16 detection is treated as best-effort; the editor will not auto-detect
  BOM-less UTF-16 without an explicit `--encoding` flag (reliable detection is impossible for
  short files that contain only ASCII code points, since UTF-16 LE ASCII bytes look like
  null-interleaved ASCII).
- Line endings inside UTF-16 files follow the same CRLF/LF detection logic already
  implemented in `Buffer::open`, applied after decoding to UTF-8.
- The `encoding_rs` crate (already a dependency) provides UTF-16 LE/BE decoders; no new
  crate dependency is required.
- UTF-16 is primarily relevant for files originating on Windows; the feature targets
  interoperability, not primary authoring.
