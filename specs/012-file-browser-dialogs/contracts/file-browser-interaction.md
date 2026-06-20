# Contract: File Browser Interaction

Defines the keyboard, mouse, and dispatch contract for the file browser modal. "Browser open" means
`App.file_browser.is_some()`.

## Modal precedence

- The browser is a modal overlay: it sits **above** the editor and menu bar, **below** higher-priority
  modals (save-prompt, session-restore, encoding-select, plugin consent/manager, external-change).
- Only one file browser is open at a time. Opening one requires no other higher modal to be active.

## Entry into the browser

| Trigger | Result |
|---|---|
| `Action::Open` (File ▸ Open, `Ctrl+O`) | Open browser in **Open** mode at the start directory. |
| `Action::SaveAs` (File ▸ Save As) | Open browser in **Save** mode at the start directory. |
| `Ctrl+S` / `handle_save_action` on a buffer with **no path** | Open browser in **Save** mode. |
| `Ctrl+S` on a buffer **with** a path | Saves directly (unchanged behaviour). |

Start directory = active buffer's parent dir if it has a path, else process CWD, canonicalized.

## Keyboard contract (while browser open; all other actions consumed)

| Key (Action) | Effect |
|---|---|
| `↑` / `↓` (MoveUp/MoveDown) | Move highlight one entry (clamped, no wrap); list scrolls to keep it visible. |
| `Enter` (InsertNewline) or `→` (MoveRight) | Activate selected entry: `..`/dir → navigate; file → Open mode opens it / Save mode loads its name into the filename field. |
| `←` (MoveLeft) | Go to parent directory (no-op at root). |
| `Backspace` | Save/Open path field non-empty → delete last char; field empty → go to parent directory. |
| printable char (InsertChar) | Append to the filename field (Save) or jump-path field (Open). |
| `Esc` (MenuClose) | Cancel: close browser, no change to editor state. |
| any other action | Consumed (no buffer mutation) while open. |

### Save confirmation

- In Save mode, pressing `Enter` when the highlighted entry is **not** a directory (or when focus is
  the filename field) confirms the save using the current `filename`:
  - empty filename → no-op (stay open);
  - otherwise validate (R1) and write via `do_save_as`; close on success; on write error keep the
    browser open and show the error notice.

## Mouse contract (left-button press; via `App::handle_mouse_event`)

| Click target | Effect |
|---|---|
| An entry row inside the box | Acts directly: `..`/dir → navigate; file → Open opens it / Save selects its name. (Single click; matches Enter.) |
| Inside the box but not on an entry (header/footer/filename line) | No navigation (filename line may receive focus; out of scope to edit by mouse). |
| Outside the box | Cancel (close browser). |

Hit-testing uses the same geometry the widget renders with (shared method on `FileBrowser`), so the
clicked row always equals the drawn row.

## Side-effect ownership

- The `FileBrowser` model only reads directories (`read_dir`, `canonicalize`) and mutates its own
  state. It returns an `Outcome`.
- The **app** performs `Buffer::open` (Open) or `do_save_as` (Save) on `Outcome::OpenFile` /
  `Outcome::SaveFile`, then sets `file_browser = None`. Path validation happens before any read/write.

## Invariants

- `cwd` is always canonical/absolute and contains no `..`.
- Cancelling at any point leaves all buffers and editor state unchanged.
- No file is read or written unless its path passed `validate_path` (directory for Save).
