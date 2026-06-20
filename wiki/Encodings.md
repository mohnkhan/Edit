# Encodings

`edit` is **UTF-8 / Unicode native**, but it reads and writes a range of legacy and wide encodings so
it can open the DOS-era files its ancestor EDIT.COM produced — and modern files of every flavor.

## The UTF-8 hygiene philosophy

The internal buffer is always **valid UTF-8**. Any bytes that enter the editor from the outside —
file reads, paste, plugin output — are validated or transcoded *before* they reach the buffer; raw
byte paths are never widened to carry arbitrary input. This is what lets the cursor, search,
selection, and rendering all reason about text in terms of Unicode scalar values and grapheme
clusters rather than bytes. When you save, the buffer is encoded back out to whatever target encoding
the file uses, round-tripping byte-faithfully where the encoding allows.

## Default: UTF-8

New files and files without a BOM or override are treated as UTF-8. A UTF-8 BOM, if present, is
stripped on read.

## Legacy code pages

`edit` transcodes the following single-byte legacy encodings on read and write:

| Encoding | Description |
|---|---|
| **CP437** | DOS code page 437 (the original IBM PC character set) |
| **CP850** | DOS code page 850 (Western European) |
| **ISO-8859-1** | Latin-1 |
| **Windows-1252** | Windows Western European |

Select one of these with `--encoding`:

```sh
edit --encoding cp437 README.DOS
edit --encoding windows-1252 legacy.txt
```

For the common DOS case, pass `--encoding cp437`, which transcodes CP437 → UTF-8 on open:

```sh
edit --encoding cp437 OLDFILE.TXT
```

Transcoding is powered by `encoding_rs` and `oem_cp` under the hood.

## UTF-16 (LE / BE) with BOM auto-detection

UTF-16 files are auto-detected by their byte-order mark:

| Encoding | BOM | Behavior |
|---|---|---|
| UTF-16 LE | `FF FE` | Auto-detected on read; BOM written on encode |
| UTF-16 BE | `FE FF` | Auto-detected on read; BOM written on encode |

Round-trips are byte-identical, and surrogate pairs (e.g. emoji and other Supplementary Plane
characters) survive intact. UTF-16 is detected from the BOM on read; the `--encoding` flag itself
accepts only the single-byte code pages (`utf-8`, `cp437`, `cp850`, `iso-8859-1`, `windows-1252`).

## Encoding support matrix

| Encoding | Read | Write | Notes |
|---|---|---|---|
| UTF-8 | Yes | Yes | Default; BOM stripped on read |
| UTF-16 LE | Yes | Yes | Auto-detected by BOM (`FF FE`); BOM written on encode |
| UTF-16 BE | Yes | Yes | Auto-detected by BOM (`FE FF`); BOM written on encode |
| CP437 | Yes | Yes | DOS code page 437; `--encoding cp437` |
| CP850 | Yes | Yes | DOS code page 850 |
| ISO-8859-1 | Yes | Yes | Latin-1 |
| Windows-1252 | Yes | Yes | Windows Western European |

## Line endings

| Convention | Read | Write |
|---|---|---|
| LF (`\n`) | Yes | Yes (default on Linux) |
| CRLF (`\r\n`) | Yes (auto-detected) | Yes (preserved from original) |

`edit` detects the line-ending style on read and **preserves it** on write, so editing a CRLF file
keeps it CRLF. The current style is shown in the status bar (`LF` / `CRLF`).

## Save As Encoding (F12)

To change a file's output encoding interactively, press **F12** (or use **File › Save As
Encoding…**). A modal list lets you pick from:

- UTF-8
- UTF-16 LE
- UTF-16 BE
- CP437
- CP850
- ISO-8859-1
- Windows-1252

The dialog pre-selects the buffer's current encoding and wraps at the list boundaries. The file is
written atomically (write-to-temp then rename), the status bar confirms (e.g. `Saved as UTF-16 LE`),
and the chosen encoding **persists for all subsequent `Ctrl+S` saves**. If the write fails, the
encoding reverts to its previous value and an error is shown.
