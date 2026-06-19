//! Construction of the sandboxed Rhai engine (Feature 008).
//!
//! The engine enforces a wall-clock per-call deadline via `on_progress`, caps resource use,
//! and disables module imports so scripts cannot reach the filesystem except through the
//! permission-gated `read_file` host function registered in [`crate::plugin::api`].

use std::sync::{Arc, Mutex};
use std::time::Instant;

use rhai::{Dynamic, Engine};

use crate::plugin::api::{register_host_functions, HostState};

/// Per-call wall-clock time limit (FR-007). Fixed in v1 (see spec rationale).
pub const PLUGIN_CALL_TIMEOUT_MS: u64 = 50;

// Resource caps — generous enough for real highlighters, tight enough to bound abuse.
const MAX_OPERATIONS: u64 = 5_000_000;
const MAX_CALL_LEVELS: usize = 32;
const MAX_STRING_SIZE: usize = 256 * 1024;
const MAX_ARRAY_SIZE: usize = 100_000;
const MAX_MAP_SIZE: usize = 100_000;

/// Shared deadline the host updates before each call; the engine's `on_progress` aborts
/// once the wall clock passes it.
pub type Deadline = Arc<Mutex<Instant>>;

/// Build the shared, sandboxed engine. Returns the engine and the deadline handle the host
/// must set (to `Instant::now() + timeout`) immediately before each `call_fn`.
pub fn build_engine(host: HostState) -> (Engine, Deadline) {
    let mut engine = Engine::new();

    engine.set_max_operations(MAX_OPERATIONS);
    engine.set_max_call_levels(MAX_CALL_LEVELS);
    // Match the release-build default expression depth so scripts parse identically in
    // debug and release; still bounded to protect the recursive parser from deep input.
    engine.set_max_expr_depths(128, 64);
    engine.set_max_string_size(MAX_STRING_SIZE);
    engine.set_max_array_size(MAX_ARRAY_SIZE);
    engine.set_max_map_size(MAX_MAP_SIZE);
    // Default-deny: no module imports, so scripts cannot `import` host filesystem modules.
    engine.set_max_modules(0);

    // Deadline starts in the past so a stray call before the host sets it aborts immediately.
    let deadline: Deadline = Arc::new(Mutex::new(Instant::now()));
    let dl = deadline.clone();
    engine.on_progress(move |_ops| {
        let expired = dl.lock().map(|d| Instant::now() >= *d).unwrap_or(true);
        if expired {
            Some(Dynamic::UNIT)
        } else {
            None
        }
    });

    register_host_functions(&mut engine, host);

    (engine, deadline)
}

/// Arm the deadline for a single call.
pub fn arm_deadline(deadline: &Deadline) {
    if let Ok(mut d) = deadline.lock() {
        *d = Instant::now() + std::time::Duration::from_millis(PLUGIN_CALL_TIMEOUT_MS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::api::new_host_state;

    #[test]
    fn test_engine_aborts_runaway_operation_count() {
        let host = new_host_state();
        let (engine, deadline) = build_engine(host);
        arm_deadline(&deadline);
        let ast = engine
            .compile("fn spin() { let x = 0; loop { x += 1; } x }")
            .unwrap();
        let mut scope = rhai::Scope::new();
        let res = engine.call_fn::<i64>(&mut scope, &ast, "spin", ());
        // Either the operation cap or the deadline fires; both are Err.
        assert!(res.is_err());
    }

    #[test]
    fn test_deadline_resets_between_calls() {
        let host = new_host_state();
        let (engine, deadline) = build_engine(host);
        let ast = engine.compile("fn quick(n) { n * 2 }").unwrap();
        let mut scope = rhai::Scope::new();
        arm_deadline(&deadline);
        let a = engine
            .call_fn::<i64>(&mut scope, &ast, "quick", (21_i64,))
            .unwrap();
        assert_eq!(a, 42);
        // A second armed call after the first must still succeed (no stale deadline).
        arm_deadline(&deadline);
        let b = engine
            .call_fn::<i64>(&mut scope, &ast, "quick", (50_i64,))
            .unwrap();
        assert_eq!(b, 100);
    }
}
