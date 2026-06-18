# Contract: Encoding API — UTF-16 Extensions

## decode(bytes, enc) — UTF-16 arms

```
Input:  bytes: &[u8]  (raw file bytes including BOM)
        enc: EncodingId::Utf16Le | EncodingId::Utf16Be
Output: Ok(String)   — valid UTF-8 string, BOM stripped, surrogate pairs decoded
        Err(TranscodeError::InvalidUtf8)  — if bytes have odd length (broken UTF-16)
```

**Guarantees**:
- BOM (`\xFF\xFE` or `\xFE\xFF`) is consumed and not included in the returned String.
- Lone/unpaired surrogates are replaced with U+FFFD (lossy but safe; matches browser behavior).
- All valid UTF-16 code points round-trip without loss.
- Empty input (`bytes == []`) → `Ok("")`.

## encode(s, enc) — UTF-16 arms

```
Input:  s: &str      (valid UTF-8 string)
        enc: EncodingId::Utf16Le | EncodingId::Utf16Be
Output: Ok(Vec<u8>)  — BOM prepended + UTF-16 payload; never returns Err for valid UTF-8
```

**Guarantees**:
- Output always begins with `[0xFF, 0xFE]` for LE or `[0xFE, 0xFF]` for BE.
- All Unicode scalar values (U+0000–U+D7FF, U+E000–U+10FFFF) are encoded correctly,
  including supplementary characters via surrogate pairs.
- `encode("", Utf16Le)` → `Ok(vec![0xFF, 0xFE])` (BOM only).
- `encode(s, enc)` followed by `decode(result, enc)` returns `s` unchanged.

## detect_encoding(bytes) — updated BOM detection

```
bytes starts_with [0xFF, 0xFE]  →  EncodingId::Utf16Le
bytes starts_with [0xFE, 0xFF]  →  EncodingId::Utf16Be
(all other behavior unchanged)
```

## encoding_from_str(s) — new aliases

```
"utf-16-le" | "utf16-le" | "utf16le" | "utf-16 le"  →  EncodingId::Utf16Le
"utf-16-be" | "utf16-be" | "utf16be" | "utf-16 be"  →  EncodingId::Utf16Be
"utf-16"                                              →  EncodingId::Utf16Le  (LE default)
(all comparisons case-insensitive)
```

## CLI contract

```
edit --encoding utf-16-le  <file>   # force UTF-16 LE on open
edit --encoding utf-16-be  <file>   # force UTF-16 BE on open
edit --encoding utf-16     <file>   # defaults to LE
```

## Status bar display

| EncodingId  | Status bar text |
|-------------|----------------|
| Utf16Le     | `UTF-16 LE`    |
| Utf16Be     | `UTF-16 BE`    |
