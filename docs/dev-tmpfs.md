# Tmpfs build redirection — protect your SSD

> **TL;DR**: `make tmpfs-setup` → `target/` becomes a symlink into a per-checkout subdirectory of
> `/tmp/edit/`, so Cargo's incremental builds hit RAM instead of the SSD. Reversible with
> `make tmpfs-teardown`. Opt-in. No-op on CI.

This is a developer ergonomic ported from the parent **MyOS** project's "Save your SSD" knob,
adapted for this Cargo workspace (which has a single large build-output tree, `target/`).

## Why this exists

A normal `edit` dev cycle hammers the SSD:

| Path | Size | Write pattern |
|---|---|---|
| `target/` | hundreds of MB–GBs | Cargo rewrites large chunks on every incremental build, test run, and `cargo bench` |

Over weeks of `make` / `cargo test` / `cargo clippy` cycles, that adds up to a lot of write volume.
Modern SSDs handle it, but if you're on a laptop or care about flash wear, redirecting `target/` to a
tmpfs (RAM-backed) filesystem is a meaningful win.

## What it does

```text
Before                        After  `make tmpfs-setup`
──────                        ───────────────────────────
target/   (real, on SSD)      target/  → /tmp/edit/<hash>/target/
```

The `<hash>` is a 12-char SHA-256 prefix of the absolute repo root path, so two checkouts of `edit`
get separate tmpfs subdirectories and never fight over each other's build artifacts.

Existing build, test, and CI invocations work unchanged — they reference `target/` by relative path,
and the symlink is transparent to all standard tools (Cargo, rustc, criterion).

## What it does NOT touch

- **anything tracked by git** — only the fully-gitignored `target/` tree moves
- **the shared `.gitignore`** — the symlink is hidden from `git status` via the local-only
  `.git/info/exclude`, not the committed ignore file
- **CI runners** — `tmpfs-setup` short-circuits when `$CI=true`

## Three commands

```sh
make tmpfs-setup       # one-time per checkout: create the tmpfs subdir, migrate any
                       # existing target/ contents in, replace with a symlink. Idempotent.
make tmpfs-status      # show current state: whether target/ is linked, where, how much
                       # tmpfs is used, free /tmp space
make tmpfs-teardown    # remove the symlink; recreate an empty real directory
                       # (build artifacts in tmpfs are kept for fast re-setup)
make tmpfs-teardown WIPE=1   # same, plus rm -rf the tmpfs subdirectory
```

## Trade-offs to know about

### `/tmp` is tmpfs → wiped on reboot

After a reboot, the next build starts from scratch (a cold `cargo build`). The symlink is still
there, but the directory it points to is gone. Re-running `make tmpfs-setup` after a reboot is a
no-op for the symlink and re-creates the tmpfs directory (`[revive]`); the next `cargo build` just
rebuilds.

If you want it to re-create automatically, add to your shell startup:
```sh
alias edit-tmpfs-restore='cd /path/to/edit && make tmpfs-setup'
```

### `/tmp` size is RAM-bounded

`tmpfs` typically gets about half your physical RAM by default. A full debug + release + bench
`target/` for this project is on the order of a few hundred MB to low GBs. Check `df -h /tmp` before
adopting this on a low-RAM machine (< 8 GB).

### `cargo clean` and the symlink

`cargo clean` empties `target/` through the symlink. If your Cargo version removes the symlink
itself rather than its contents, just re-run `make tmpfs-setup` (idempotent) to restore the link.
`make tmpfs-teardown` is the clean way to return to a plain on-SSD `target/`.

### Multiple checkouts

Two checkouts of `edit` each get their own subdirectory because the path is hashed. No collision
risk, no extra setup. The cost is that each checkout uses its own tmpfs space.

### CI runners

`tmpfs-setup` checks `$CI` and short-circuits if it's `true`. CI `/tmp` is disk-backed and
ephemeral; the tmpfs trick doesn't help there. Keep this opt-in for dev only.

## Verifying the win

```text
$ make tmpfs-status
[tmpfs-status] tmpfs root: /tmp/edit/<hash>

  [link]  target                 → /tmp/edit/<hash>/target  (612M)

  Filesystem      Size  Used Avail Use% Mounted on
  tmpfs            16G  1.2G   15G   8% /tmp
```

To confirm writes are actually going to RAM (not crossing the SSD), run `iostat` on your SSD device
while running a `make build` and watch that the write rate stays low.

## When to NOT use this

- **Limited-RAM machines** (< 8 GB) where tmpfs can't afford the build cache
- **Workflows that need build artifacts to survive reboots** (rare; a rebuild is cheap)
- **CI / batch automation** — already covered by the `$CI` check

## Where this lives in the tree

```
Makefile                       # tmpfs-setup, tmpfs-status, tmpfs-teardown targets + tmpfs root vars
scripts/tmpfs-setup.sh         # idempotent migration + symlink creation
scripts/tmpfs-status.sh        # read-only inspection
scripts/tmpfs-teardown.sh      # remove symlink; optional WIPE
.git/info/exclude              # local-only; the symlink is added so `git status` stays clean
                               # (the shared .gitignore is intentionally NOT modified)
```
