<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
at specs/015-find-replace-dialog/plan.md
<!-- SPECKIT END -->

This is the always-loaded behavioral summary. Per-feature implementation detail and the rationale
behind every rule below live in `docs/ai-context-reference.md` — open it when working on a specific
subsystem.

## Project Overview

**Goal:** A Linux-compatible reimplementation of Microsoft's EDIT.COM (MS-DOS text editor), packaged
as a modern Linux application. The editor recreates the DOS-style look-and-feel as a native **Rust**
TUI and enforces UTF-8/Unicode throughout the toolchain and runtime. It is developed standalone and
is destined to ship as the built-in text editor component of **MyOS**.

**Key design constraints:**
- Build target: Linux (x86_64, aarch64); no DOS/DPMI runtime dependency
- UI: full DOS-like look-and-feel (blue background, pull-down menus, F-key bindings, status bar)
- Encoding: UTF-8 everywhere — source files, runtime locale (`LC_ALL=en_US.UTF-8`), file I/O,
  clipboard; legacy CP437/CP850 input must transcode on read
- Toolchain: **Rust** (cargo + rustc, MSRV 1.74, edition 2021); `ratatui` + `crossterm` for terminal
  rendering, `ropey` for the text buffer, `encoding_rs`/`oem_cp` for transcoding, `rhai` for plugins
- Distribution: single static (musl) or minimally-dynamic binary; no X11/Wayland dependency

## AI Context File Ecosystem

| File | Loaded by | How |
|---|---|---|
| `CLAUDE.md` | Claude Code | automatic (this file) |
| `AGENT.MD` | OpenAI Codex CLI, Aider | symlink → `CLAUDE.MD` |
| `GEMINI.MD` | Gemini CLI | symlink → `CLAUDE.MD` |

**`CLAUDE.md` is the single source of truth.** Keep `CLAUDE.MD` in sync (copy content) so the
symlinked `AGENT.MD` / `GEMINI.MD` stay accurate.

## Mandatory Rules

- **Git workflow** — Never commit directly to `master`. Every change gets its own branch
  `NNN-short-description`, created with `git checkout -b <branch> origin/master`. Open a PR
  targeting `master`; merge via GitHub. Applies to ALL changes regardless of size.
- **Docs gate** — Every feature PR must update `CHANGELOG.md` + `docs/STATUS.md` (and
  `docs/CAPABILITIES.md` if a user-visible capability — keybinding, menu item, file format,
  CLI flag — changed). Bypass for docs-only or infra-only PRs with `[no-docs]` in any commit
  message between the branch's merge-base with master and HEAD.
- **Deferrals** — Any descoped user story / FR / sub-task needs **both** a GitHub issue (problem,
  why deferred, suggested approach, pointer to decision, effort, `follow-up` label) **and** a row in
  `ROADMAP.md` referencing it, before the PR merges. PR description alone is insufficient.
- **Feature numbering** — Pick the next number from the `CHANGELOG.md` ledger (max + 1).
- **Make targets** — When adding a top-level target to `Makefile`, update all three of: the
  `.PHONY:` declaration, the file-header comment, and the `help:` body. Verify with
  `make help | grep <new-target>`.
- **AI attribution** — Never add `Co-Authored-By:` trailers or AI-generated badges/footers naming
  any AI system (Claude, Gemini, GPT, …) anywhere user-facing: commits, PR bodies, issue bodies,
  comments, or docs. Strip any "Generated with …" footer before `gh pr create`.
- **UTF-8 hygiene** — All source files must be UTF-8. Any function that reads external bytes must
  validate or transcode before passing to the editor buffer. Never construct buffer text directly
  from raw `&[u8]` — decode/transcode through the helpers in `src/encoding/` (`detect.rs` /
  `transcode.rs`) so all text enters the rope as valid UTF-8.

## Workflow Shortcuts

- **Build** — `make` (`cargo build`) builds the debug binary; `make release` builds the stripped
  release binary (LTO); `make static` builds the musl static binary; `make check` (`cargo test`)
  runs unit + integration tests; `make ci-local` runs the full gate (`cargo fmt --check` → `cargo
  clippy -D warnings` → `cargo test` → `make smoke` → `make perf-check`).
- **Run** — `./target/debug/edit [file]` (or `./edit`) launches the editor. Pass `--locale C.UTF-8`
  to override the detected locale, `--debug` to enable debug logging, and `--legacy-cp437` to enable
  CP437→UTF-8 transcoding on file open.
- **Debugging order:**
  - Rendering glitch → run with `--debug` (and `RUST_LOG=debug`); inspect the log under
    `$XDG_STATE_HOME/edit/logs/`. Rendering goes through `ratatui`/`crossterm` (`src/ui/`).
  - Encoding issue → run `file <path>` and `hexdump -C <path>` before touching the editor code.
  - Key-binding regression → check `src/input/keymap.rs` and the DOS scan-code mapping table.
  - Crash/panic → set `RUST_BACKTRACE=1` and reproduce on the debug build; check the crash report
    under `$XDG_STATE_HOME/edit/crash-<timestamp>.log`.
- **Locale enforcement** — Integration tests always launch with `LC_ALL=C.UTF-8 LANG=C.UTF-8`.
  Locale resolution must always fall back gracefully and log the resolved locale.

## Deep Reference

For per-feature detail (`ratatui` rendering pipeline, UTF-8 rope-buffer internals, CP437 transcoding
table, keybinding scan-code map, build flag matrix, integration test harness) see
`docs/ai-context-reference.md` and the `specs/NNN-*/` directories it links. Open them on demand;
they are intentionally not loaded into every turn.
