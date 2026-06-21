# Data Model: Sandboxed fuzz covering file-I/O actions (050)

Test-only feature; no persisted or runtime production entities change. The "entities" are local test
constructs.

## Sandbox (test-local)
- A unique temp directory: `std::env::temp_dir().join(format!("edit_io_fuzz_{}", std::process::id()))`.
- Subdirs: `state/` (→ `$XDG_STATE_HOME`), `config/` (→ `$XDG_CONFIG_HOME`).
- Seed files: `seed_a.txt`, `seed_b.txt` (small UTF-8 content) — valid Open/Revert targets.
- Lifetime: created at sweep start, removed at sweep end (incl. on panic via RAII).

## EnvGuard (test-local RAII)
- Captures at construction: original `current_dir()`, prior `XDG_STATE_HOME`, prior `XDG_CONFIG_HOME`
  (each `Option<OsString>`).
- On construction: sets cwd to sandbox and the two XDG vars to the sandbox subdirs.
- On `Drop`: restores cwd; restores each XDG var (set back if previously present, else `remove_var`);
  best-effort `remove_dir_all(sandbox)`.

## Lock
- `static SWEEP_ENV_LOCK: std::sync::Mutex<()>` — process-wide; held for the whole sweep body so the
  global cwd/env mutation is serialized against any other test taking the same lock.

## Action set (test-local)
- The feature-042 action vector PLUS: `Save`, `SaveAs`, `SaveAsEncoding`, `Open`, `Revert`.
- `insert_chars` biased to path-ish characters: letters, digits, `.`, `/`, `_`, plus the multibyte
  stress chars retained from 042 (`é`, `中`, `✓`, `😀`, combining mark) so UTF-8 paths are exercised.

## Sweep parameters
- Seeds: 3 fixed `u64` constants (reuse 042's).
- Sizes: `[(80,24),(120,40),(200,60),(40,12)]` (last = sub-minimum "too small").
- Events per (seed,size): ~1500; render every Nth iteration (as in 042).

## Invariants
- No filesystem path outside the sandbox is written (structural: cwd + XDG redirected).
- After the sweep, cwd and env equal their pre-sweep values; sandbox removed.
- No panic across the full budget (the assertion).
