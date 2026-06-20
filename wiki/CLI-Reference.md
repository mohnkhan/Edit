# CLI Reference

`edit` is driven by command-line flags built with [`clap`](https://docs.rs/clap). The general form
is:

```
edit [OPTIONS] [FILE...]
```

You can open zero, one, or many files. Opening multiple files enters multi-file editing mode (cycle
buffers with `F6` / `Shift+F6`; see [Keybindings](Keybindings.md)).

## Options

| Flag | Description |
|---|---|
| `FILE...` | One or more files to open; omit for a new empty buffer (multi-file editing) |
| `-e`, `--encoding <ENC>` | Force file encoding: `utf-8`, `cp437`, `cp850`, `iso-8859-1`, `windows-1252` |
| `--theme <NAME>` | Color theme: `classic`, `high-contrast`, `plain` |
| `-n`, `--line-numbers` | Show line numbers in the gutter |
| `--no-highlight` | Disable syntax highlighting |
| `--no-autosave` | Disable auto-save and crash recovery |
| `--no-session` | Skip the session restore prompt on startup; open a blank buffer |
| `--no-watch` | Disable external file modification watching for this session |
| `--no-plugins` | Disable all plugin loading for this session (does not change saved consent) |
| `-r`, `--readonly` | Open all files in read-only mode |
| `--locale <LOCALE>` | Override locale detection (e.g. `C.UTF-8`) |
| `-d`, `--debug` | Enable verbose diagnostic logging |
| `-V`, `--version` | Print version and exit |
| `-h`, `--help` | Print help and exit |

## Notes on selected flags

### `-e`, `--encoding <ENC>`

Forces the decode/encode codec for opened files, overriding auto-detection. Accepts the labels above
(case-insensitive). UTF-16 LE/BE files are auto-detected by their byte-order mark on read and do not
need a flag. See [Encodings](Encodings.md) for detection and round-trip details.

### Opening classic DOS (CP437) files

To open classic DOS files, pass `--encoding cp437` (or `--encoding cp850` for code page 850). The
default encoding can also be set with `default_encoding` in `config.toml`.

### `--no-watch`

Disables filesystem watching for the session — no reload prompts and no file-deletion notices. The
same effect can be made persistent with `no_watch = true` in `config.toml`.

### `--no-plugins`

Suppresses all plugin loading for the current session. It does **not** alter the per-plugin consent
decisions saved in `plugins.toml`, so re-launching without the flag restores your previous setup.
See [Plugin Development](Plugin-Development.md).

### `--locale <LOCALE>`

Overrides locale detection. The `EDIT_LOCALE` environment variable serves the same purpose. `edit`
is UTF-8 throughout; this controls the resolved runtime locale.

### `-r`, `--readonly`

Opens every file read-only — useful for inspecting files you don't want to accidentally modify.
