# Development

A guide for contributors. `edit` follows a spec-driven workflow with strict branch, PR, and
documentation conventions.

## Repository layout

```
edit/
├── src/                 # Rust source (see Architecture.md for the module map)
├── tests/
│   ├── integration/     # integration test suites (registered in Cargo.toml)
│   └── smoke/           # expect-based smoke tests
├── benches/             # criterion benchmarks (startup, large_file, keystroke)
├── examples/plugins/    # reference & fixture plugins
├── specs/               # Spec Kit feature specs (NNN-*)
├── docs/                # STATUS.md, CAPABILITIES.md, ai-context-reference.md
├── man/edit.1           # man page
├── packaging/           # rpm spec, etc.
├── Cargo.toml
├── Makefile
├── CHANGELOG.md
└── ROADMAP.md
```

## Toolchain & MSRV

| | |
|---|---|
| Edition | 2021 |
| MSRV | stable **1.74.0** (required for `ratatui` 0.26 and `clap` 4) |
| Nightly | used **only** for the `release-static` musl profile; not needed for normal development |

## Spec Kit workflow

Features are developed under `specs/NNN-short-name/` using the Spec Kit pipeline (spec → plan →
tasks → analyze → implement). Features 001–035 are the existing record (v0.4.0):

| # | Feature |
|---|---|
| 001 | Linux EDIT.COM clone (foundation) |
| 002 | UTF-16 transcoding |
| 003 | Session restore |
| 004 | Save-As encoding UI |
| 005 | Soft-wrap mode |
| 006 | Menu check-state indicator |
| 007 | External file watch |
| 008 | Plugin API (Rhai) |
| 009 | Menu-bar activation |
| 010 | Working Escape key and File ▸ Open |
| 011 | Mouse-operable menus + working menu actions |
| 012 | Navigable file browser for Open / Save |
| 013 | DOS-style menu mnemonic accelerators |
| 014 | Undo-to-clean state + Revert to saved |
| 015 | Interactive Find and Replace dialogs |
| 016 | Focusable dialog buttons (borders, tab order, mouse) |
| 017 | Visible text selection (highlight, Shift-select, drag) |
| 018 | Editable-field affordance + Help redesign |
| 019 | Bordered-box styling for Find/Replace fields |
| 020 | Boxed buttons + focus ring for interactive dialogs |
| 021 | Scroll affordances + dialog button polish |
| 022 | File dialog — glob filtering + richer entry details |
| 023 | Mouse-wheel scrolling (app-wide) |
| 024 | Interactive (clickable + draggable) scrollbars |
| 025 | Go to Line |
| 026 | Syntax highlighting for Rust, JSON, and TOML |
| 027 | Buffer tab bar |
| 028 | UX crash-safety + keyboard-navigation hardening |
| 029 | UX completeness hardening (round 2) |
| 030 | Interaction completeness |
| 031 | Caret-on-click in dialog text fields |
| 032 | Word-wise navigation, selection, and deletion |
| 033 | Fix menu dropdown hidden behind the tab bar |
| 034 | Crash-safe line access + crash diagnostics |
| 035 | Animated demo GIF + README revamp |

Pick the next feature number from the `CHANGELOG.md` ledger (max + 1).

## Branching & PRs

- **Never commit directly to `master`.** Every change gets its own branch named
  `NNN-short-description`, created from `origin/master`:

  ```sh
  git checkout -b 010-my-feature origin/master
  ```

- Open a PR targeting `master`; merge via GitHub.
- This applies to **all** changes regardless of size.
- No AI-attribution trailers, badges, or "Generated with…" footers anywhere user-facing.

## Docs gate

Every feature PR must update:

- `CHANGELOG.md`
- `docs/STATUS.md`
- `docs/CAPABILITIES.md` **if** a user-visible capability changed (keybinding, menu item, file
  format, CLI flag)

Docs-only or infra-only PRs may bypass the gate with `[no-docs]` in a commit message. Verify
presence locally with `make docs-gate`.

## Deferrals

Any descoped user story, FR, or sub-task needs **both** a GitHub issue (problem, why deferred,
suggested approach, effort, `follow-up` label) **and** a row in `ROADMAP.md` referencing it before
the PR merges.

## Build, test & benchmark targets

```sh
make build        # debug build (cargo build) → target/debug/edit
make debug-run    # run debug binary with full backtraces + debug logging (FILE=path optional)
make release      # release build (LTO, stripped) → target/release/edit
make static       # musl static binary for x86_64 (nightly + musl target)
make check        # cargo test (unit + integration)
make smoke        # expect-based smoke tests (requires expect + tmux)
make perf-check   # criterion benchmarks
make stress-test  # 5-minute stress test (EDIT_STRESS_DURATION_SECS=300)
make demo-gif     # regenerate the README demo GIF (assets/demo.gif; needs agg)
make docs-gate    # verify CHANGELOG.md + docs/STATUS.md present
make package-deb  # .deb via cargo-deb
make package-rpm  # .rpm via rpmbuild
make ci-local     # full gate: fmt → clippy → test → smoke → perf-check
make help         # list targets
```

Opt-in developer ergonomics (see `docs/dev-tmpfs.md`) keep build artifacts off the SSD:

```sh
make tmpfs-setup     # redirect target/ into /tmp/edit/<hash>/ (tmpfs)
make tmpfs-status    # show whether target/ is tmpfs-symlinked + disk usage
make tmpfs-teardown  # remove the symlink (WIPE=1 also rm -rf the tmpfs subdir)
```

> When adding a top-level Make target, update **all three** of: the `.PHONY:` line, the file-header
> comment, and the `help:` body. Verify with `make help | grep <new-target>`.

## Test suites

| Suite | Command | Description |
|---|---|---|
| Unit | `cargo test` | All `#[cfg(test)]` modules in `src/` |
| Integration | `cargo test --test '*'` | 30+ suites under `tests/integration/` registered in `Cargo.toml` — encoding round-trip, file I/O, recovery, stress, session, soft wrap, file watch, plugin API, menu activation/mnemonics, find/replace, selection, dialog buttons, file browser, mouse wheel, scrollbar interaction, go-to-line, buffer tab bar, word editing, UX hardening, and more |
| Smoke | `make smoke` | `expect`-based scripts in `tests/smoke/` |
| Stress | `cargo test --test stress -- --ignored` | Continuous-editing / encoding stress (slow, opt-in) |
| Benchmarks | `make perf-check` | Criterion benches in `benches/` (startup, large_file, keystroke) |

## CI gate

`make ci-local` runs, in order:

1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. `make smoke`
5. `make perf-check`

## CI target matrix

| Target | Toolchain | Profile |
|---|---|---|
| `x86_64-unknown-linux-gnu` | stable 1.74.0+ | debug, release |
| `aarch64-unknown-linux-gnu` | stable 1.74.0+ | debug, release (cross) |
| `x86_64-unknown-linux-musl` | nightly | release-static |

## UTF-8 hygiene rule

All source files must be UTF-8, and any function reading external bytes must validate or transcode
*before* the bytes reach the editor buffer — use the helpers in `src/encoding/`. Never widen a `char`
path to accept raw bytes. See [Encodings](Encodings.md) and [Architecture](Architecture.md).
