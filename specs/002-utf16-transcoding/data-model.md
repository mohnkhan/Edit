# Data Model: UTF-16 Transcoding Support

## EncodingId (extended enum)

```
EncodingId
├── Utf8          (existing)
├── Cp437         (existing)
├── Cp850         (existing)
├── Iso8859_1     (existing)
├── Windows1252   (existing)
├── Utf16Le       (NEW) — UTF-16 Little-Endian; BOM = [0xFF, 0xFE]
└── Utf16Be       (NEW) — UTF-16 Big-Endian; BOM = [0xFE, 0xFF]
```

**State transitions for a UTF-16 buffer**:

```
File on disk (UTF-16 LE bytes)
  ↓  detect_encoding() → EncodingId::Utf16Le
  ↓  decode(bytes, Utf16Le) → strips [0xFF,0xFE] BOM, decodes via encoding_rs → String (UTF-8)
  ↓  Buffer { rope: UTF-8, encoding: Utf16Le, ... }
  ↓  User edits (all in UTF-8 internally)
  ↓  encode(utf8_str, Utf16Le) → prepends [0xFF,0xFE] + UTF-16 LE payload → Vec<u8>
  ↓  atomic write to disk
File on disk (UTF-16 LE bytes, round-tripped)
```

## EncodingProfile (registry entries added)

| Field      | Utf16Le value              | Utf16Be value              |
|------------|----------------------------|----------------------------|
| `name`     | `"UTF-16 LE"`              | `"UTF-16 BE"`              |
| `id`       | `EncodingId::Utf16Le`      | `EncodingId::Utf16Be`      |
| `bom`      | `Some(&[0xFF, 0xFE])`      | `Some(&[0xFE, 0xFF])`      |

## Alias table (encoding_from_str)

| Input string (case-insensitive) | Resolved EncodingId |
|---------------------------------|---------------------|
| `utf-16-le`, `utf16-le`, `utf16le`, `utf-16 le` | `Utf16Le` |
| `utf-16-be`, `utf16-be`, `utf16be`, `utf-16 be` | `Utf16Be` |
| `utf-16` (no endian specified) | `Utf16Le` (LE is the default Windows UTF-16) |

## Validation Rules

- A file with an odd byte count and UTF-16 encoding → `BufferError::DecodeError`
- A file with a UTF-16 LE BOM followed by invalid surrogate pairs → decoded with U+FFFD
  replacement (encoding_rs behavior); editor still opens the file but status bar may note
  replacement characters.
- `encode(s, Utf16Le)` always succeeds for valid UTF-8 input (all Unicode code points
  are representable in UTF-16).
