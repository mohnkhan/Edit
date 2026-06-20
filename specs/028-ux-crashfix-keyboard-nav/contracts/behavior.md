# Contract: UX crash-safety and keyboard navigation hardening

Behavioral contracts the tests assert against.

## Crash safety

- **Renderer never panics**: for any buffer content and any `wrap_cache` state (including a cache whose
  cached segment offsets exceed the current line length, and empty lines), rendering the editor
  produces output without panicking. Out-of-range slices render truncated/blank, not a crash.
- **Wrap cache tracks the active buffer**: after session restore, next/previous buffer, opening a file,
  or closing a buffer, the next render uses a wrap layout matching the now-active buffer (the cache is
  invalidated). Concretely, `wrap_text_gen` differs before vs. after each such operation.
- **Panic restores the terminal**: on panic, the terminal is returned to cooked mode on the primary
  screen with a visible cursor before the report is printed; the crash-log file is still written; the
  hook does not itself panic.
- **Hardened slices/arith**: copy/cut on an empty or reversed selection yields empty text (no panic);
  file-browser scroll arithmetic never underflows.

## Keyboard navigation

| Surface | Key | Effect |
|---|---|---|
| Interactive dialog (open) | (on open) | focus starts on the primary control (field/list), not a button |
| Save browser | printable char | appended to the filename field; caret shown |
| Confirm dialog (016) | Right / Down | focus next button (wraps) |
| Confirm dialog (016) | Left / Up | focus previous button (wraps) |
| Interactive dialog (020), button focused | arrows | cycle the button ring (wraps), consistent with Tab |
| Help / About | Up / Down | scroll by one line (clamped) |
| Help / About | PageUp / PageDown | scroll by one page (clamped) |
| Help / About | Home / End | scroll to top / bottom |
| Help / About | Esc / Enter | dismiss |
| Editor | Home / End | cursor to line start / line end |
| File browser / encoding / plugin list | PageUp / PageDown | move selection by ~one page (clamped) |

## No-regression

- All existing dialog keys (Tab/Shift+Tab, Enter/Space, Esc, list nav, option toggles, match nav),
  mouse interactions, editing semantics, file formats, and the crash-log file output are unchanged.
- Single-button dialogs ignore arrow keys (no movement, no panic).
- Empty/single-item lists ignore PageUp/PageDown (no movement, no underflow).
