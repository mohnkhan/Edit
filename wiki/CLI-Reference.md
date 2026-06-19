# CLI Reference

`edit` is driven by command-line flags built with [`clap`](https://docs.rs/clap). The general form
is:

```
edit [OPTIONS] [FILE...]
```

You can open zero, one, or many files. Opening multiple files enters multi-file editing mode (cycle
buffers with `Ctrl+Tab`; see [Keybindings](Keybindings.md)).

## Options

| Flag | Description |
|---|---|
| `FILE...` | One or more files to open (multi-file editing) |
| `--encoding <ENC>` | Override file encoding: `utf-8`, `cp437`, `cp850`, `iso-8859-1`, `windows-1252`, `utf-16-le`, `utf-16-be`, `utf-16` |
| `--theme <NAME>` | Override theme: `classic`, `high-contrast`, `plain` |
| `--line-numbers` | Enable line numbers in the gutter |
| `--no-highlight` | Disable syntax highlighting |
| `--no-autosave` | Disable auto-save and crash recovery |
| `--no-session` | Skip the session restore prompt on startup; open a blank buffer |
| `--no-watch` | Disable external file modification watching for this session |
| `--no-plugins` | Disable all plugin loading for this session (does not change saved consent) |
| `--readonly` | Open all files in read-only mode |
| `--locale <LOC>` | Override locale detection (e.g. `C.UTF-8`) |
| `--legacy-cp437` | Enable CP437→UTF-8 transcoding on file open |
| `--debug` | Enable debug logging |
| `--version` | Print version and exit |
| `--help` | Print help and exit |

## Notes on selected flags

### `--encoding <ENC>`

Forces the decode/encode codec for opened files, overriding auto-detection. Accepts the labels above
(case-insensitive, with common aliases such as `utf16le`). `utf-16` defaults to little-endian. See
[Encodings](Encodings.md) for detection and round-trip details.

### `--legacy-cp437`

A convenience switch for opening classic DOS files: enables CP437 → UTF-8 transcoding on read. For
other code pages use `--encoding`.

### `--no-watch`

Disables filesystem watching for the session — no reload prompts and no file-deletion notices. The
same effect can be made persistent with `no_watch = true` in `config.toml`.

### `--no-plugins`

Suppresses all plugin loading for the current session. It does **not** alter the per-plugin consent
decisions saved in `plugins.toml`, so re-launching without the flag restores your previous setup.
See [Plugin Development](Plugin-Development.md).

### `--locale <LOC>`

Overrides locale detection. The `EDIT_LOCALE` environment variable serves the same purpose. `edit`
is UTF-8 throughout; this controls the resolved runtime locale.

### `--readonly`

Opens every file read-only — useful for inspecting files you don't want to accidentally modify.
