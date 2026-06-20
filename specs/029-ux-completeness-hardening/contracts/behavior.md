# Contract: UX completeness hardening (round 2)

Behavioral contracts the tests assert against.

## Crash safety

- Deleting/cutting a selection over multibyte text removes the right text, records a correct undo entry,
  and never panics; an empty/reversed selection is a safe no-op.
- The crash-recovery prompt renders a long Unicode path (truncated) without panicking.
- `byte_to_char` returns a correct count for any byte offset, including non-char-boundary offsets, with
  no panic.
- Opening a file larger than `MAX_OPEN_BYTES` returns a "file too large" error (no read, no OOM, no
  panic).

## No silent data loss

- A successful plain save sets a "Saved" status; a failed save sets a "Save failed: …" status and leaves
  the buffer marked modified.
- An autosave/recovery write failure produces a visible notice.

## Dialog & correctness

| Surface | Input | Effect |
|---|---|---|
| Save-before-quit prompt | Esc | cancels (no save, no discard) — matches the `Cancel (Esc)` label |
| Save-As via file browser | confirm with a pending encoding | file written in that encoding |
| Go-to-Line request while a menu is open | — | does not open over the menu |
| Editor click, line numbers on | click at column X | cursor lands at X (gutter + h-scroll accounted) |
| Editor click on the gutter | click | does not place the cursor mid-text |

## Display width (single function)

- `display_width("a") == 1`, combining mark `== 0`, CJK (e.g. 世) `== 2`, and the same function is used by
  the editor, file browser, tab bar, and dialog fields (no divergent helper remains).

## Feedback

- Copy/Cut → "Copied"/"Cut"; Paste of empty clipboard → "Nothing to paste"; clipboard failure →
  "Clipboard unavailable"; edit on a read-only buffer → "Buffer is read-only"; file-open failure →
  "Open failed: <path> — <reason>".

## Reachability & legibility

- `Ctrl+W` closes the current buffer; a `File ▸ Close` menu item does the same.
- Every bundled theme renders the selected menu item with contrasting (legible) colors.

## No-regression

- All existing keys, mouse interactions, editing semantics, file formats, dialogs, and the crash-log file
  output are unchanged except where a listed defect is corrected. No new dependencies.
