# Quickstart: External File Modification Detection (Feature 007)

## Prerequisites

- `cargo build` (debug binary at `target/debug/edit`)
- A terminal with two panes/tabs (or two separate terminal sessions)
- `bash` for shell commands

---

## Scenario 1: External Modification Detected and Reloaded (US1 + US2)

**Goal**: Verify the reload prompt appears when a file is overwritten externally, and that choosing Reload correctly replaces the buffer.

**Steps**:

```bash
# Terminal 1 — set up test file
echo "original content" > /tmp/watch_test.txt

# Terminal 1 — launch editor
./target/debug/edit /tmp/watch_test.txt
```

```bash
# Terminal 2 — overwrite the file after the editor has started (wait ~2 seconds)
echo "new content from external process" > /tmp/watch_test.txt
```

**Expected outcome**: Within 5 seconds, the editor in Terminal 1 displays a modal dialog: *"File changed on disk. Reload? [Y/n]"*.

**Verify Reload**:
- Press `Y` or `Enter` in the editor.
- The buffer now shows `new content from external process`.
- The status bar shows no "modified" indicator (the buffer is in sync with disk).

---

## Scenario 2: Decline Reload — Buffer Kept as Modified (US1 / US2)

**Continuation of Scenario 1 setup.**

```bash
# Terminal 2 — overwrite again
echo "another external write" > /tmp/watch_test.txt
```

**Expected outcome**: Reload dialog appears.

**Verify Keep**:
- Press `N` or `Esc` in the editor.
- Buffer content remains unchanged from before the external write.
- The status bar shows a "modified" indicator (the buffer is treated as having unsaved changes).
- Pressing `Ctrl+Q` triggers the "save changes?" prompt — confirming the unsaved-changes state.

---

## Scenario 3: Unsaved Changes Warning (US2)

**Goal**: Verify the enhanced warning appears when the buffer has unsaved edits.

```bash
echo "baseline" > /tmp/watch_test2.txt
./target/debug/edit /tmp/watch_test2.txt
```

1. In the editor, type a few characters (make a visible edit without saving).
2. In Terminal 2: `echo "external" > /tmp/watch_test2.txt`

**Expected outcome**: The reload dialog includes the extra line: *"You have unsaved changes. Reload and discard edits? [Y/n]"*.

---

## Scenario 4: File Deleted While Open (US3)

```bash
echo "deleteme" > /tmp/watch_del.txt
./target/debug/edit /tmp/watch_del.txt
```

```bash
# Terminal 2
rm /tmp/watch_del.txt
```

**Expected outcome**: Within 5 seconds, the status bar shows a non-modal notice: *"[watch_del.txt] File deleted on disk — buffer kept in memory"*. No modal dialog appears. The buffer remains fully editable.

**Verify Save-Recreates**:
- Press `Ctrl+S` in the editor.
- Check `ls /tmp/watch_del.txt` — the file is recreated.

---

## Scenario 5: --no-watch Disables Detection (US4)

```bash
echo "nowatchtest" > /tmp/nowatch.txt
./target/debug/edit --no-watch /tmp/nowatch.txt
```

```bash
# Terminal 2 — overwrite while editor is running
echo "external write" > /tmp/nowatch.txt
```

**Expected outcome**: No reload dialog, no notification. The editor is completely unaffected.

---

## Scenario 6: Atomic Rename Detection (Edge Case)

Many tools (cargo, rustfmt, sed -i on some systems) write to a temp file first and then rename it over the original.

```bash
echo "original" > /tmp/atomic_test.txt
./target/debug/edit /tmp/atomic_test.txt
```

```bash
# Terminal 2 — simulate atomic write
echo "new content" > /tmp/atomic_test.txt.tmp && mv /tmp/atomic_test.txt.tmp /tmp/atomic_test.txt
```

**Expected outcome**: The editor detects the atomic rename and shows the reload dialog within 5 seconds — same as Scenario 1.

---

## Automated Verification (`cargo test`)

Run the integration test suite:

```bash
cargo test --test file_watch -- --nocapture
```

Key tests:
- `test_external_write_triggers_event` — writes to a temp file, asserts `WatchEvent::Modified` received within 3s
- `test_self_write_suppressed` — editor save, asserts no `WatchEvent` produced within grace window
- `test_debounce_coalesces` — 10 rapid writes, asserts exactly 1 event produced
- `test_no_watch_flag` — `--no-watch`, asserts `file_watcher` is `None`
- `test_delete_produces_notice_not_dialog` — `rm` on watched file, asserts `watcher_notice` set, `pending_external_change` is `None`
- `test_reload_replaces_buffer_content` — reload confirmed, buffer content verified against disk
- `test_decline_marks_buffer_dirty` — decline confirmed, buffer dirty flag set
