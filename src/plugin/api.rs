//! Host functions exposed to plugin scripts, plus the shared host state they mutate
//! (Feature 008). These are the *only* capabilities a script has beyond pure computation —
//! the Rhai base language has no filesystem, process, or network access.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use rhai::{Engine, EvalAltResult};

use crate::plugin::types::Permission;

/// Mutable state shared between the host and the registered functions. The host sets
/// `current_*` fields immediately before each plugin call so `read_file` can enforce the
/// active plugin's permissions and bump its violation counter.
#[derive(Default)]
pub struct HostShared {
    /// Messages queued by `status_bar`, drained into `App.status_message` after a call.
    pub status_queue: Vec<String>,
    /// The active plugin's declared permissions (set per call).
    pub current_perms: Vec<Permission>,
    /// The active plugin's id (for log context).
    pub current_plugin_id: String,
    /// The active plugin's shared FS-violation counter (set per call).
    pub fs_violation: Option<Arc<AtomicU32>>,
}

pub type HostState = Arc<Mutex<HostShared>>;

pub fn new_host_state() -> HostState {
    Arc::new(Mutex::new(HostShared::default()))
}

/// Truncate to 120 UTF-8 chars and strip terminal escape sequences (Principle VII / FR-011).
fn sanitize_status(msg: &str) -> String {
    let stripped = crate::security::sanitize::strip_escape_sequences(msg);
    stripped.chars().take(120).collect()
}

fn path_allowed(perms: &[Permission], path: &std::path::Path) -> bool {
    perms.iter().any(|p| match p {
        Permission::ReadPath(allowed) => path == allowed || path.starts_with(allowed),
        Permission::WriteDir(_) => false,
    })
}

/// Register `log`, `status_bar`, and `read_file` into the engine.
pub fn register_host_functions(engine: &mut Engine, host: HostState) {
    engine.register_fn("log", move |level: i64, msg: &str| {
        let msg = crate::security::sanitize::strip_escape_sequences(msg);
        match level {
            0 => log::debug!("[plugin] {msg}"),
            1 => log::info!("[plugin] {msg}"),
            2 => log::warn!("[plugin] {msg}"),
            _ => log::error!("[plugin] {msg}"),
        }
    });

    let status_host = host.clone();
    engine.register_fn("status_bar", move |msg: &str| {
        if let Ok(mut h) = status_host.lock() {
            let m = sanitize_status(msg);
            h.status_queue.push(m);
        }
    });

    let read_host = host.clone();
    engine.register_fn(
        "read_file",
        move |path: &str| -> Result<String, Box<EvalAltResult>> {
            let h = read_host
                .lock()
                .map_err(|_| "host state poisoned".to_string())?;
            let p = std::path::Path::new(path);
            if path_allowed(&h.current_perms, p) {
                std::fs::read_to_string(p)
                    .map_err(|e| Box::<EvalAltResult>::from(format!("read_file io error: {e}")))
            } else {
                log::warn!(
                    "[plugin {}] denied read_file outside sandbox: {path}",
                    h.current_plugin_id
                );
                if let Some(counter) = &h.fs_violation {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
                drop(h);
                Err(Box::<EvalAltResult>::from(format!(
                    "permission denied: {path} is not in the plugin's declared read paths"
                )))
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_allowed_prefix_and_exact() {
        let perms = vec![Permission::ReadPath(PathBuf::from("/tmp/edit-plugin"))];
        assert!(path_allowed(
            &perms,
            std::path::Path::new("/tmp/edit-plugin")
        ));
        assert!(path_allowed(
            &perms,
            std::path::Path::new("/tmp/edit-plugin/data.txt")
        ));
        assert!(!path_allowed(&perms, std::path::Path::new("/etc/passwd")));
    }

    #[test]
    fn test_read_file_denied_for_undeclared_path() {
        let host = new_host_state();
        {
            let mut h = host.lock().unwrap();
            h.current_plugin_id = "t".into();
            h.current_perms = vec![];
            h.fs_violation = Some(Arc::new(AtomicU32::new(0)));
        }
        let mut engine = Engine::new();
        register_host_functions(&mut engine, host.clone());
        let ast = engine
            .compile(r#"fn leak() { read_file("/etc/passwd") }"#)
            .unwrap();
        let mut scope = rhai::Scope::new();
        let res = engine.call_fn::<String>(&mut scope, &ast, "leak", ());
        assert!(res.is_err());
        let count = host
            .lock()
            .unwrap()
            .fs_violation
            .as_ref()
            .unwrap()
            .load(Ordering::Relaxed);
        assert_eq!(count, 1);
    }
}
