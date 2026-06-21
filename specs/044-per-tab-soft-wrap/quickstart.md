# Quickstart / Validation Guide: Per-Tab Soft-Wrap

## Prerequisites

```bash
cd /home/main/MyOS-2026/edit
make tmpfs-setup
make
```

## Automated validation

```bash
make check       # full suite incl. new per-tab tests + 043 cache tests + 042 fuzz, all green
make ci-local    # fmt --check → clippy -D warnings → test → smoke → perf-check
```

**Expected**: green. Existing soft-wrap tests pass with assertions retargeted to
`app.active_buffer().soft_wrap` (no behavior change); new per-tab tests pass; `clippy -D warnings` clean
(the 042 unwrap guardrail still holds).

## Manual validation (the user's scenario)

```bash
./target/debug/edit  file_with_long_lines.txt  short_file.txt   # two tabs
```

1. On tab 1 (long lines), toggle soft-wrap (Ctrl+W or View ▸ Soft Wrap) → tab 1 wraps.
2. Switch to tab 2 → it is **still unwrapped**, line-number gutter aligned, **no ghost wrap**.
3. Switch back to tab 1 → it is **still wrapped** (setting preserved).
4. Open the View menu on each tab → the "Soft Wrap" check mark matches that tab; the status-bar wrap
   indicator matches the active tab.
5. Open a third file → it starts at the configured default wrap setting.

**Expected**: each tab independently remembers its wrap; toggling one never changes another; indicators
always track the active tab.

## References

- Field move + seeding + reader retargeting: [research.md](./research.md), [data-model.md](./data-model.md)
- Behavioral guarantees: [contracts/internal-api.md](./contracts/internal-api.md)
