# Quickstart / Validation Guide: Session Restore (Feature 003)

**Date**: 2026-06-18

This guide describes how to validate the session restore feature end-to-end once
implementation is complete.

---

## Prerequisites

- `cargo build` succeeds with no warnings (`cargo clippy -- -D warnings` clean)
- `cargo test` (unit + integration) passes
- A terminal with at least 80×24 cells

---

## Scenario 1 — Core Restore (US1 / P1)

**Goal**: Verify that a clean exit writes a session file and a subsequent launch restores it.

```bash
# 1. Create two test files
echo "Hello from file A" > /tmp/test_a.txt
echo "Hello from file B" > /tmp/test_b.txt

# 2. Launch editor, open both files
./target/debug/edit /tmp/test_a.txt /tmp/test_b.txt
# Navigate to line 3 in file A, line 2 in file B (using arrow keys)
# Switch to split view (F6 or menu View > Split)
# Quit cleanly: Ctrl+Q (or File > Quit)

# 3. Check that session.toml was written
cat ~/.local/state/edit/session.toml
# Expected: version=1, two [[buffers]] entries with correct paths and cursor positions

# 4. Relaunch without file arguments
./target/debug/edit
# Expected: "Restore previous session? [Y/n]" dialog appears

# 5. Press Y (or Enter)
# Expected: both files reopen at their saved cursor positions in split view
```

**Pass criteria**:
- `session.toml` exists after step 2
- Restore dialog appears in step 4
- Both files open at exact saved line/column after step 5
- Total time from launch to interactive ≤ 2 s (SC-002)

---

## Scenario 2 — Missing File (US2 / P2)

**Goal**: Verify graceful skip when a recorded file is deleted before restore.

```bash
# 1. Create one file, launch, navigate, quit
echo "test content" > /tmp/test_restore.txt
./target/debug/edit /tmp/test_restore.txt
# Move cursor to line 1, col 5; quit cleanly

# 2. Delete the file
rm /tmp/test_restore.txt

# 3. Relaunch; confirm restore
./target/debug/edit
# Press Y

# Expected:
# - Status bar shows warning: "session: /tmp/test_restore.txt not found"
# - Editor opens with a blank buffer (no crash)
```

**Pass criteria**:
- No crash or hang
- Status bar warning visible
- Blank buffer opened

---

## Scenario 3 — No-Session Flag (US3 / P3)

**Goal**: Verify `--no-session` suppresses the restore prompt.

```bash
# 1. Ensure a session file exists from a prior clean exit
cat ~/.local/state/edit/session.toml   # should exist

# 2. Launch with --no-session
./target/debug/edit --no-session

# Expected: no restore dialog; editor opens blank buffer immediately
```

**Pass criteria**:
- No restore dialog appears
- Blank buffer opens immediately

---

## Scenario 4 — Explicit Files Bypass Restore (US4 / P4)

```bash
./target/debug/edit /tmp/test_a.txt
# Expected: file opens directly; no restore dialog
```

---

## Scenario 5 — Corrupt Session File

```bash
# Write garbage to the session file
echo "NOT VALID TOML {{{" > ~/.local/state/edit/session.toml

# Launch without file arguments
./target/debug/edit

# Expected: no restore dialog; blank buffer; no crash
# Check logs: ~/.local/state/edit/logs/ should show a warning about corrupt session
```

---

## Scenario 6 — Crash Does Not Write Session

```bash
# 1. Run editor and kill it with SIGKILL (simulates crash)
./target/debug/edit /tmp/test_a.txt &
EDIT_PID=$!
sleep 1
kill -9 $EDIT_PID

# 2. Check session file timestamp
ls -la ~/.local/state/edit/session.toml

# Expected: session.toml either does not exist or has a timestamp predating
# this run (was not updated by the kill)
```

---

## Automated Tests

```bash
# Unit tests (session module)
cargo test session

# Integration tests
cargo test --test session

# Smoke tests (if expect/tmux available)
make smoke
```

---

## Key Files for Review

- `src/session/mod.rs` — session data model and read/write
- `src/app.rs` — restore dialog state and `do_restore_session` logic
- `src/main.rs` — `--no-session` flag and startup session load
- `specs/003-session-restore/contracts/session-toml.md` — TOML schema contract
- `specs/003-session-restore/data-model.md` — entity definitions and state transitions
