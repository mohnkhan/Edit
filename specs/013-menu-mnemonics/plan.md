# Implementation Plan: DOS-style menu mnemonic accelerators

**Branch**: `013-menu-mnemonics` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/013-menu-mnemonics/spec.md`

## Summary

Give every top-level menu and every dropdown item a single **accelerator letter**, rendered
**underlined** in the menu bar and dropdowns, and make that letter active: while a dropdown is open,
pressing the letter activates the matching item (same as highlight + Enter); while the bar is active,
pressing a top-level letter opens that menu. Built-in item accelerators are **hand-authored** to the
DOS/standard convention (New=N, Open=O, Save=S, Save As=A, Exit=X, …); plugin items and any plugin
top-level menus get accelerators **auto-assigned deterministically** and uniquely within their scope.
Tapping `Alt` alone activates the bar like `F10` (best-effort, terminal-permitting).

The technical core extends the existing **resolved menu model** (feature 009): `ResolvedMenu` and
`ResolvedItem` gain a canonical `mnemonic: Option<char>`. The static built-in `MenuItem` table gains
an authored `mnemonic`. `resolve_menus` copies built-in mnemonics and auto-assigns plugin ones,
seeding the per-scope "used letters" set so plugin items never collide with built-ins. `MenuBarWidget`
underlines the accelerator glyph; `MenuBarState` gains mnemonic-lookup methods used by the `App`
keyboard intercept. No new crate. With no plugins, the resolved model and existing geometry are
unchanged except for the added underline attribute on one cell per label.

## Technical Context

**Language/Version**: Rust (edition 2021, MSRV stable 1.74.0)

**Primary Dependencies**: ratatui 0.26 (TUI widgets — `Modifier::UNDERLINED`), crossterm 0.27
(key events; `KeyCode::Modifier`, `supports_keyboard_enhancement`, `PushKeyboardEnhancementFlags`),
existing `src/input` keymap and `src/ui/menubar.rs`. No new crates.

**Storage**: N/A (no persistence change).

**Testing**: `cargo test` (unit in `src/ui/menubar.rs` + `src/input`, integration under
`tests/integration/`), `expect`+tmux smoke under `tests/smoke/`. TDD per Constitution Principle V.

**Target Platform**: Linux x86_64/aarch64 (+ FreeBSD/macOS per constitution); headless VT100
terminals via crossterm.

**Performance Goals**: Keystroke→render latency ≤ 50 ms (Constitution). Mnemonic assignment is an
in-memory pass over a handful of menus at resolve time — negligible.

**Constraints**: Existing menu geometry, navigation, shortcuts, and all current menu/geometry tests
MUST stay green (FR-012). Accelerator selection / underline must be UTF-8 / wide-char correct
(FR-010, Principle II). No new CLI flags or config keys. Bare-`Alt` activation degrades gracefully on
terminals lacking keyboard-enhancement support (F10 always works).

**Scale/Scope**: 6 built-in menus, ~30 built-in items; realistically 0–5 plugin menus. Touch points:
`src/ui/menubar.rs` (model + assignment + nav-by-letter + underline render), `src/app.rs` (keyboard
intercept for letter activation; terminal-init enhancement flags + lone-Alt), `src/input/mod.rs`
(map lone `Alt` key → `Action::Menu`). No change to `src/plugin/*`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Assessment |
|---|---|
| **I. Platform-Native, DOS-Faithful UI** | ✅ Squarely DOS EDIT.COM behavior: underlined hotkeys + letter activation. Underline degrades gracefully where unsupported (FR-001/002). Bare-Alt is best-effort with F10 fallback. |
| **II. UTF-8 First** | ✅ Accelerator selection, matching, and underline operate on `char` boundaries via the existing wide-char-aware renderer; never splits a multi-byte char (FR-010). Plugin labels already validated UTF-8 (feat 008). |
| **III. Portable Build** | ✅ Pure Rust, no platform-specific code, no new deps. `supports_keyboard_enhancement` is crossterm-portable and gated. |
| **IV. Minimal Footprint** | ✅ No new dependencies; static build unaffected. |
| **V. Test-Gated Merges (NON-NEGOTIABLE)** | ✅ TDD: unit tests for mnemonic assignment/uniqueness, underline placement, and letter-lookup; integration tests for end-to-end letter activation; smoke for keyboard-only menu drive. |
| **VI. Simplicity / YAGNI** | ✅ One `Option<char>` field threaded through the existing model + small lookup methods. No new abstraction. Bare-Alt scoped to a guarded best-effort path, not a config surface. |
| **VII. Security Hardening** | ✅ No new external input or attack surface. Plugin labels are rendered through the existing escape-safe menu renderer; mnemonic assignment is pure string processing. |

**Gate result: PASS.** No violations; Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/013-menu-mnemonics/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── menu-mnemonics.md
├── checklists/
│   └── requirements.md
├── spec.md
└── tasks.md             # /speckit-tasks output (not created here)
```

### Source Code (repository root)

```text
src/
├── ui/
│   └── menubar.rs       # MenuItem.mnemonic (authored); ResolvedMenu/ResolvedItem.mnemonic;
│                        #   resolve_menus assignment; underline_index helper; render underline;
│                        #   MenuBarState::open_menu_by_mnemonic / select_item_by_mnemonic
├── input/
│   └── mod.rs           # dispatch_key: KeyCode::Modifier(LeftAlt|RightAlt) → Action::Menu
└── app.rs               # menu intercept: InsertChar(c) → mnemonic activation;
                         #   terminal init/teardown: gated keyboard-enhancement flags

tests/
├── integration/
│   └── menu_mnemonics.rs   # end-to-end: Alt+F then 'n' → new buffer; letter activation per menu
└── smoke/
    └── menu_mnemonics.exp  # tmux: open a menu, press a letter, verify no crash + clean exit
```

## Phase 0: Research

See [research.md](./research.md). Resolved unknowns: underline rendering in ratatui; lone-`Alt`
detection via crossterm keyboard-enhancement (gated by `supports_keyboard_enhancement`); deterministic
auto-assignment algorithm and collision handling; DOS-faithful built-in accelerator letters.

## Phase 1: Design & Contracts

- **Data model**: [data-model.md](./data-model.md) — `mnemonic` field on the static and resolved
  menu types; canonical lowercase char; underline-index derivation; assignment rules and scope.
- **Contract**: [contracts/menu-mnemonics.md](./contracts/menu-mnemonics.md) — rendering contract
  (one underlined glyph per label), keyboard contract (letter activation while bar/dropdown active,
  bare-Alt), and assignment contract (built-in authored, plugin auto, per-scope uniqueness).
- **Quickstart**: [quickstart.md](./quickstart.md) — manual + automated validation scenarios.
- **Agent context**: update the `<!-- SPECKIT START -->` block in `CLAUDE.md` to point at this plan.

## Complexity Tracking

No constitution violations; no entries required.
