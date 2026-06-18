# Contract: Recovery File Format

**Feature**: Linux EDIT.COM Clone
**Version**: 1.0.0

## File Location

```
$XDG_RUNTIME_DIR/edit/<sha256-of-absolute-path>.recovery
$XDG_RUNTIME_DIR/edit/<sha256-of-absolute-path>.lock
```

Fallback (when `XDG_RUNTIME_DIR` unset):
```
$TMPDIR/edit-recovery/<sha256-of-absolute-path>.recovery
$TMPDIR/edit-recovery/<sha256-of-absolute-path>.lock
```

The directory is created with mode `0700` on first use.

## Recovery File Format

The recovery file is a UTF-8 text file with a header block followed by the buffer content.

```
EDIT-RECOVERY-V1
path: <absolute file path, UTF-8>
encoding: <encoding name, e.g. "utf-8" or "cp437">
timestamp: <Unix epoch seconds, decimal>
content_len: <byte length of content block, decimal>
---
<buffer content as UTF-8 text>
```

**Version compatibility**: The first line is a version magic. A reader MUST verify it before
parsing. For v1.x the only recognised value is `EDIT-RECOVERY-V1`. If a future
`EDIT-RECOVERY-V2` is introduced, readers MUST: (a) still parse `V1` files (the v1 fields above
are a guaranteed subset of any future version), and (b) on encountering an unrecognised
higher version, decline recovery with a logged warning rather than misparsing — the on-disk
file is then left untouched so a newer build can recover it. Writers always emit the current
version. This forward/backward rule means an abandoned `V1` recovery file is never silently lost.

### Field definitions

| Field | Format | Description |
|-------|--------|-------------|
| `path` | UTF-8 string | Absolute path of the original file |
| `encoding` | ASCII string | Encoding name matching `EncodingProfile.name` |
| `timestamp` | decimal integer | Unix epoch seconds of the auto-save write |
| `content_len` | decimal integer | Byte count of the content block (after `---\n`) |

The `---` separator is a literal line containing only three hyphens and a newline.
Everything after the separator line (up to `content_len` bytes) is the buffer content.

## Lock File Format

The lock file is a plain text file containing the PID of the owning `edit` process:

```
<pid>\n
```

**Lock semantics**:
- Created with mode `0600` when the buffer is opened (skipped entirely in `--readonly` mode).
- Deleted on clean exit (any exit path that reaches the shutdown handler).
- If present on startup: check if `pid` is still a running process.
  - If running: another session has the file open → warn user, offer read-only open.
  - If not running: previous session crashed → offer recovery.

**Stale-process detection mechanism**: liveness of the recorded PID MUST be checked with
`kill(pid, 0)` (POSIX signal 0 — no signal sent, only permission/existence checked):
- returns `Ok` → process exists (treat as active session);
- `Err(ESRCH)` → no such process (stale → offer recovery);
- `Err(EPERM)` → process exists but owned by another user (treat as active session).
`/proc`-based checks MUST NOT be used (not portable to FreeBSD/macOS, which are supported
targets per constitution Principle III).

**Autosave precondition**: a recovery file MUST be written ONLY when `buffer.modified == true`
at the moment the autosave timer fires. An unmodified buffer never produces a recovery file,
so an unmodified clean session leaves no `.recovery` artifact to detect on next open.

## Recovery Lifecycle

```
Buffer opened
  → create .lock (pid)
  → start autosave timer (interval_secs)

Every interval_secs with unsaved changes:
  → write .recovery (atomic: write to .recovery.tmp, rename)

Clean exit (save or discard):
  → delete .recovery (if exists)
  → delete .lock

Abnormal exit (crash, SIGKILL):
  → .lock and .recovery remain on disk

Next open of same file:
  → .lock present, pid dead → offer recovery dialog
  → user accepts → load .recovery content into buffer
  → user declines → delete .recovery; load from disk
  → after decision → recreate .lock with new pid
```

## Atomic Write

Recovery files are written atomically: content is first written to
`<recovery_path>.tmp`, then renamed to `<recovery_path>`. This prevents a partial
recovery file from being read after a crash mid-write.
