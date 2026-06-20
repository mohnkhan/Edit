# Quickstart / Validation: Rust / JSON / TOML highlighting

## Build & test

```sh
make tmpfs-setup
make
make check        # cargo test — per-language unit tests + detect_highlighter
make ci-local     # fmt → clippy -D warnings → test → smoke → perf-check
```

## Automated coverage (what `make check` must prove)

- **Unit** (`src/highlight/languages/{rust,json,toml}.rs`): representative lines style the expected
  tokens (keyword/type/string/number/comment/header/key); spans are sorted + non-overlapping; malformed
  input, empty/whitespace lines, multi-byte content, and a very long line don't panic.
- **detect** (`src/highlight/mod.rs`): `detect_highlighter` resolves `a.rs`→Rust, `a.json`→JSON,
  `a.toml`→TOML, and returns `None` for an unknown extension.

## Manual walkthrough

1. `./target/debug/edit src/app.rs` → keywords, types, strings, numbers, and comments are colorized.
2. `./target/debug/edit Cargo.toml` → `[package]`/`[dependencies]` headers, keys, `"…"` versions, and
   `#` comments are colorized.
3. Create and open a small `.json` config → keys vs string values are distinguishable; numbers and
   `true`/`false`/`null` are styled.
4. Confirm the existing 5 languages (`.c`, `.py`, `.sh`, `.yaml`, `.md`) still highlight unchanged.

## Expected outcome

`.rs`, `.json`, and `.toml` files highlight automatically on open (≥ 8 languages total), with valid spans
and no panic, and a plugin highlighter for those extensions still overrides the built-in (SC-001..SC-004).
