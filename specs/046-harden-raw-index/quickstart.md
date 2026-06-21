# Quickstart / Validation Guide: Harden Raw Slice/Index Access

## Prerequisites
```bash
make tmpfs-setup && make
```
## Automated validation (the gate)
```bash
make check     # full suite incl. the content-bearing no-panic fuzz
make ci-local  # fmt --check → clippy -D warnings → test → smoke → perf-check
```
**Expected**: green, deterministically. The extended fuzz drives random keyboard+mouse events on
multibyte-content buffers across overlays + terminal sizes with zero panics. No existing assertion
changes (behavior-preserving for in-range input).

## Run just the fuzz
```bash
cargo test --lib no_panic   # the content-bearing sweep(s)
```
Runs twice → identical result (fixed seed). If it surfaces a panic, that's a real raw-index bug to fix.

## Manual sanity
```bash
./target/debug/edit some_utf8_file_with_emoji_and_cjk.txt
```
Type/select/navigate across multibyte text, open dialogs, click around, resize — no crash; identical
behavior to before.

## References
- Categories + conversion idioms: [research.md](./research.md), [data-model.md](./data-model.md)
- Guarantees: [contracts/internal-api.md](./contracts/internal-api.md)
