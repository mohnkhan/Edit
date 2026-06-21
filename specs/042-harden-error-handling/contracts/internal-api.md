# Internal Contract: Harden Error Handling

No external/public API. The "contract" is the behavioral and build-gate guarantees.

## Conversion contract (FR-001/FR-002)

- C-1: Each converted site is a pattern match; the absent-arm performs exactly the prior no-op /
  fall-through. For any input that did not previously panic, observable state (buffer text, cursor,
  selection, open overlay, status message, `running`) is **identical** before vs after.
- C-2: No existing helper signatures change. Accessors used (`find_replace_mut`, `file_browser_mut`,
  `context_menu`, etc., and the `pending_external_change`/`scrollbar_drag` fields) are read via `if let`.
- C-3: No existing test assertion is modified (FR-007).

## Fuzz contract (FR-003/FR-004)

- F-1: `cargo test` includes a sweep that applies ≥ several-thousand combined keyboard+mouse events
  across all overlay states and ≥3 terminal sizes (incl. 80×24 minimum) with zero panics.
- F-2: The sweep is deterministic: same seeds ⇒ same sequence ⇒ same result on every run/host. No
  `Date::now`, no external RNG.
- F-3: The sweep calls both `handle_action` and `handle_mouse_event`, interleaved with `render`, so it
  exercises dispatch, hit-testing, the pre-render cursor clamp, and paint.

## Guardrail contract (FR-005/FR-006)

- G-1: `cargo clippy --all-targets -- -D warnings` is clean after conversion.
- G-2: Adding an `unwrap()`/`expect()` on a fallible value anywhere in `src/app.rs` or `src/app/*.rs`
  *production* code makes that command fail (demonstrable; SC-004).
- G-3: The lint does NOT fire on `highlight/languages/*` `Regex::new("<literal>").unwrap()` or
  best-effort `let _ =` cleanup (they are outside the `app` module tree).

## Recovery-net contract (FR-008)

- R-1: The panic hook (terminal restore + crash log) and SIGSEGV handler in `diagnostics/crash.rs` are
  unchanged. This feature reduces how often they're needed; it does not remove them.
