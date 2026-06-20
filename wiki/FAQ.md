# FAQ

Practical questions and answers about `edit`.

### How is this different from the real MS-DOS EDIT.COM?

`edit` is a *reimplementation*, not a port. It recreates the EDIT.COM look-and-feel — blue
background, pull-down menus, F-key bindings, status bar — but it is a modern Rust program built on
`ratatui` + `crossterm`, and it is **UTF-8 / Unicode native** rather than limited to a single DOS
code page. It also adds conveniences the original lacked: regex find/replace, syntax highlighting,
session restore, external-file watching, soft-wrap, and a sandboxed plugin system. A few extensions
(e.g. soft-wrap) are explicitly marked "(ext)" because they go beyond DOS behavior.

### Does it need DOS, DPMI, or DOSBox?

**No.** There is no DOS/DPMI runtime dependency. `edit` is a native Linux binary. It also has no
X11/Wayland dependency — it runs entirely in a terminal.

### What terminals are supported?

Any terminal that works with the `crossterm` backend. For mouse features the terminal must report
mouse events in crossterm's supported protocol; if not, everything is still fully usable from the
keyboard. Integration/smoke tests run under `LC_ALL=C.UTF-8 LANG=C.UTF-8`, and a UTF-8 locale is
recommended for correct rendering.

### How do I open a CP437 (DOS) file?

Force the encoding explicitly with `--encoding`:

```sh
edit --encoding cp437 OLDFILE.TXT
edit --encoding cp850 file.txt
edit --encoding windows-1252 legacy.txt
```

UTF-16 LE/BE files are auto-detected by their BOM. See [Encodings](Encodings.md).

### How do I save a file in a different encoding?

Press **F12** (or **File › Save As Encoding…**) and pick from UTF-8, UTF-16 LE/BE, CP437, CP850,
ISO-8859-1, or Windows-1252. The chosen encoding sticks for subsequent `Ctrl+S` saves.

### How do I disable plugins?

Launch with `--no-plugins` to suppress all plugin loading for that session (it does not change your
saved consent decisions). To disable a specific plugin persistently, open **Options › Plugins** and
toggle it off. See [Plugin Development](Plugin-Development.md).

### Are plugins safe to run?

Plugins run in a **default-deny sandbox**: no filesystem, network, or process access except a
permission-gated `read_file`; a 50 ms per-call time limit; and automatic disabling of any plugin that
loops, errors, or misbehaves. Each newly installed plugin must also pass a one-time consent prompt
before it can run.

### Where are my config, recovery, session, and log files?

| What | Path |
|---|---|
| Config | `$XDG_CONFIG_HOME/edit/config.toml` (≈ `~/.config/edit/config.toml`) |
| Plugin consent | `$XDG_CONFIG_HOME/edit/plugins.toml` |
| Installed plugins | `$XDG_CONFIG_HOME/edit/plugins/<id>/` |
| Recovery snapshots | `$XDG_STATE_HOME/edit/recovery/` (≈ `~/.local/state/edit/recovery/`) |
| Session | `$XDG_STATE_HOME/edit/session.toml` |
| Logs | `$XDG_STATE_HOME/edit/logs/edit-<date>.log` |
| Crash reports | `$XDG_STATE_HOME/edit/crash-<timestamp>.log` |

See [Configuration](Configuration.md) for details.

### My terminal popped up a "reload from disk?" prompt — why?

`edit` watches open files for external changes. If another process (a build tool, `git checkout`,
another editor) rewrites a file you have open, you'll be asked **[Y] Reload / [N] Keep**. If the file
is deleted, the buffer is kept in memory and saving recreates the file. Disable watching with
`--no-watch` (or `no_watch = true` in `config.toml`).

### What languages get syntax highlighting?

C (`.c .h`), Python (`.py`), Shell (`.sh .bash`), YAML (`.yml .yaml`), Markdown (`.md`), Rust
(`.rs`), JSON (`.json`), and TOML (`.toml`). Plugins can add more — a plugin highlighter takes
precedence over the built-in one for its declared extensions.

### How do I recover work after a crash?

Auto-save writes `EDIT-RECOVERY-V1` snapshots periodically. On the next launch, if a recovery file
exists for a file you open, `edit` offers to restore or discard it. Auto-save can be disabled with
`--no-autosave`.

### Will this be part of MyOS?

**Yes** — that's the plan. `edit` is being developed as a standalone editor now, but it is intended
to ship as the **built-in text editor component of MyOS**, a Linux-based OS project. Its design goals
(single minimally dependent binary, no X11/Wayland, static musl build, small auditable plugin
surface) are chosen with that role in mind. See the [Home](Home.md) page.

### What's the minimum Rust version to build it?

Stable **Rust 1.74.0** or newer. Nightly is needed only for the static musl build. See
[Installation](Installation.md) and [Development](Development.md).
