# Quickstart / Validation Guide: Persist Per-Tab Soft-Wrap

## Prerequisites
```bash
make tmpfs-setup && make
```

## Automated validation
```bash
make check     # full suite incl. round-trip + legacy-load tests + 003/044 tests
make ci-local  # fmt --check → clippy -D warnings → test → smoke → perf-check
```
**Expected**: green. Session-restore tests that assert the written schema version now expect `2`;
a legacy payload without `soft_wrap` deserializes to `false`.

## Manual validation (round-trip)
```bash
./target/debug/edit a.txt b.txt        # two tabs
# toggle soft-wrap ON for tab a only (Ctrl+W / Alt+Z), then quit (saves the session)
./target/debug/edit                    # no args → restore prompt → accept
```
**Expected**: tab `a` comes back **wrapped**, tab `b` **unwrapped** — each tab restored to the wrap
state it had at quit. Opening a brand-new file still starts at the configured default.

## Legacy file
- Restoring a `session.toml` written before this feature (no `soft_wrap` keys) loads without error;
  tabs come up at the configured default.

## References
- Schema + version handling: [research.md](./research.md), [data-model.md](./data-model.md)
- Guarantees: [contracts/internal-api.md](./contracts/internal-api.md)
