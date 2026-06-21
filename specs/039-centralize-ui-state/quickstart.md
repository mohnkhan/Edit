# Quickstart / Validation Guide: Centralize Editor UI State

This is a behavior-preserving refactor. Validation = prove nothing changed for the user, plus the new
structural invariant holds.

## Prerequisites

```bash
cd /home/main/MyOS-2026/edit
make tmpfs-setup          # keep build writes off the SSD (idempotent)
```

## Build

```bash
make                      # debug build (cargo build)
```

## Automated validation (the gate)

```bash
make check                # cargo test: 87 inline app.rs tests + 33 integration files + unit
make smoke                # 9 .exp headless TTY tests (real binary, real keystrokes/mouse)
make ci-local             # fmt --check → clippy -D warnings → test → smoke → perf-check
```

**Expected**: all green, with **no changes to any test's expected behavior**. The only permitted test
edits are mechanical field→accessor renames (e.g. `app.pending_find_replace.is_some()` →
`app.find_replace().is_some()`). If any assertion's expected value had to change, the refactor altered
behavior — treat as a defect (FR-009/SC-001).

## New invariant test (added by this feature)

A generic layer-dispatch test (Phase 2) asserting: for every active layer, a press inside that layer's
drawn rect routes to it and never to a lower layer. This subsumes the two prior point regressions
(`repro_menu_click_over_tabs`, `first_dropdown_item_clickable_with_tab_bar_open`). Runs under
`make check`.

## Manual smoke (non-default terminal size — guards bugs 014/033/038)

Resize your terminal to something other than 80×24, then:

```bash
./target/debug/edit specs/039-centralize-ui-state/spec.md
```

1. Open and close each overlay; confirm identical look/flow to before:
   - Search ▸ Find (Ctrl+F), Search ▸ Go to Line (Ctrl+G), File ▸ Open (Ctrl+O), Help (F1),
     Options ▸ Plugins, Save prompt (edit then Ctrl+Q), right-click context menu.
2. Open a second buffer so the **tab bar** shows; open a **menu dropdown** over it; click the **first
   dropdown item** → the menu action fires (NOT a tab switch). ← bug 038 regression check.
3. Click inside the Go-to-Line and Find/Replace input fields → caret lands where clicked, dialog stays
   open (NOT dismissed). ← bug 014 regression check.
4. Click every region rapidly across the top rows → no panic, no fall-through. ← bug 033/034 check.

**Expected**: behavior identical to the prior release in every case.

## Performance sanity

```bash
make perf-check           # startup ≤ 2s, 100MB open ≤ 3s — must not regress
```

## References

- Modal enum + Layer model: [data-model.md](./data-model.md)
- Accessor surface + behavioral-equivalence gate: [contracts/internal-api.md](./contracts/internal-api.md)
- Why each field maps where: [research.md](./research.md)
