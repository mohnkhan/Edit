# Data Model: UX completeness hardening (round 2)

Behavior/robustness changes; no persisted data changes. The "entities" are the in-memory pieces touched.

## Display-width function (`src/ui/width.rs`, new)

- **Represents**: the single source of truth for terminal column width of a grapheme / string.
- **Contract**: `display_width(grapheme) == unicode-width of the grapheme` (combining marks → 0,
  East-Asian wide → 2, normal → 1); `str_width(s) == sum of display_width over graphemes`.
- **Invariant (FR-010)**: every column computation (editor click/scroll, file browser columns &
  truncation, tab bar, dialog field caret/anchoring) uses this function — no other width helper remains.

## Status message (`App.status_message: Option<String>`)

- **Represents**: transient one-line feedback.
- **New writes (FR-005/011/012/013)**: "Saved"/"Save failed: …"; "Copied"/"Cut"/"Pasted"/"Nothing to
  paste"/"Clipboard unavailable"; "Buffer is read-only"; "Open failed: <path> — <reason>". Cleared by the
  existing one-shot mechanism.

## File open (`Buffer::open`, `BufferError`)

- **New rule (FR-004)**: if the file's size exceeds `MAX_OPEN_BYTES` (documented constant), return a
  `BufferError` "file too large" before reading; the app surfaces it (FR-013) instead of a blank buffer.

## Selection text extraction (delete/cut)

- **Rule (FR-001)**: the removed text is taken by char range (`chars().skip(lo).take(hi-lo)`),
  `lo = min(s,e).min(total)`, `hi = max(s,e).min(total)` — empty/reversed → empty, never panics.

## Recovery-prompt path (`RecoveryDialog.path`)

- **Rule (FR-002)**: truncated for display by char/width (keep a trailing window with a leading `...`),
  never by a byte slice.

## byte→char conversion (`EditorRope::byte_to_char`)

- **Rule (FR-003)**: input byte offset is clamped down to a char boundary (and to `len`) before counting;
  never panics.

## Keymap & menu

- **FR-014**: `Ctrl+W → Action::Close` added to the default map; a `File ▸ Close` item routes to it.

## Theme

- **FR-015**: each bundled theme's selected-menu style has contrasting fg/bg (light theme fixed).

## Modal precedence

- **FR-016**: Go-to-Line open is guarded by `!menu_bar.is_active()`, consistent with other modals.
