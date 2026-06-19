<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
at specs/009-menu-bar-activation/plan.md
<!-- SPECKIT END -->

This is the always-loaded behavioral summary. Per-feature implementation detail and the rationale
behind every rule below live in `docs/ai-context-reference.md` — open it when working on a specific
subsystem.

## Project Overview

**Goal:** A Linux-compatible reimplementation of Microsoft's EDIT.COM (MS-DOS text editor), packaged
as a modern Linux application. The editor wraps a DOS-style UI clone (FreeDOS EDIT lineage or a
ncurses-based equivalent) and enforces UTF-8/Unicode throughout the toolchain and runtime.

**Key design constraints:**
- Build target: Linux (x86_64, aarch64); no DOS/DPMI runtime dependency
- UI: full DOS-like look-and-feel (blue background, pull-down menus, F-key bindings, status bar)
- Encoding: UTF-8 everywhere — source files, runtime locale (`LC_ALL=en_US.UTF-8`), file I/O,
  clipboard; legacy CP437/CP850 input must transcode on read
- Toolchain: gcc/clang + make/cmake; ncurses or PDCurses for terminal rendering
- Distribution: single static or minimally-dynamic binary; no X11/Wayland dependency

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
  validate or transcode before passing to the editor buffer. Never widen `char` paths to accept raw
  bytes — use the `utf8_*` helpers in `src/encoding/`.

## Workflow Shortcuts

- **Build** — `make` builds the debug binary; `make release` builds the stripped release binary;
  `make check` runs unit tests; `make ci-local` runs the full gate (format → lint → tests →
  integration smoke).
- **Run** — `./edit [file]` launches the editor. Set `EDIT_LOCALE=C.UTF-8` to override the
  detected locale. Pass `--legacy-cp437` to enable CP437→UTF-8 transcoding on file open.
- **Debugging order:**
  - Rendering glitch → set `EDIT_DEBUG_RENDER=1` and capture the ncurses trace (`NCURSES_TRACE`).
  - Encoding issue → run `file <path>` and `hexdump -C <path>` before touching the editor code.
  - Key-binding regression → check `src/input/keymap.c` and the DOS scan-code mapping table.
  - Crash/SIGSEGV → build with `make debug-asan` (AddressSanitizer enabled) and reproduce.
- **Locale enforcement** — Integration tests always launch with `LC_ALL=C.UTF-8 LANG=C.UTF-8`.
  Never hardcode `setlocale(LC_ALL, "")` without a fallback that logs the resolved locale.

## Deep Reference

For per-feature detail (ncurses rendering pipeline, UTF-8 buffer internals, CP437 transcoding
table, keybinding scan-code map, build flag matrix, integration test harness) see
`docs/ai-context-reference.md` and the `specs/NNN-*/` directories it links. Open them on demand;
they are intentionally not loaded into every turn.
