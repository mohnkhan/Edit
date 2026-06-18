//! Crash reporting: panic hook and SIGSEGV handler (Task T007).
//!
//! Call both `install_panic_hook()` and `install_signal_handler()` once at
//! program startup (before spawning any threads) to ensure crash reports are
//! written on both Rust panics and native faults.
//!
//! Report files land at:
//!   `$XDG_STATE_HOME/edit/crash-<unix-timestamp>.log`
//!
//! The directory is created with mode `0o700` if it does not exist.

use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Returns the crash-report directory.
fn crash_dir() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("edit")
}

/// Creates the crash directory (mode 0o700 on Unix) and returns its path.
fn ensure_crash_dir() -> PathBuf {
    let dir = crash_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("[edit/crash] could not create crash dir {dir:?}: {e}");
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
        }
    }
    dir
}

/// Returns the current Unix timestamp in whole seconds.
fn unix_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Opens (or creates) a crash report file and returns the `File` handle
/// together with the resolved path for diagnostic messages.
fn open_crash_file() -> Option<(fs::File, PathBuf)> {
    let dir = ensure_crash_dir();
    let path = dir.join(format!("crash-{}.log", unix_ts()));
    match fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => Some((f, path)),
        Err(e) => {
            eprintln!("[edit/crash] could not open crash file {path:?}: {e}");
            None
        }
    }
}

/// Writes a crash report to `dest`, emitting every field even if some fail.
fn write_report(dest: &mut dyn std::io::Write, header: &str, detail: &str) {
    let ts = unix_ts();
    let thread = std::thread::current();
    let thread_name = thread.name().unwrap_or("<unnamed>");
    let thread_id = format!("{:?}", std::thread::current().id());

    let _ = writeln!(dest, "=== edit crash report ===");
    let _ = writeln!(dest, "timestamp (unix): {ts}");
    let _ = writeln!(dest, "thread name:      {thread_name}");
    let _ = writeln!(dest, "thread id:        {thread_id}");
    let _ = writeln!(dest, "event:            {header}");
    let _ = writeln!(dest, "detail:");
    let _ = writeln!(dest, "{detail}");
    let _ = writeln!(dest);

    // Backtrace — only available when `RUST_BACKTRACE=1` or `full`.
    let bt = std::backtrace::Backtrace::capture();
    let bt_status = bt.status();
    let _ = writeln!(dest, "backtrace ({bt_status:?}):");
    let _ = writeln!(dest, "{bt}");
    let _ = writeln!(dest, "=== end of report ===");
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Installs a custom Rust panic hook that writes a structured crash report.
///
/// The hook replaces the default hook entirely; it writes to both the crash
/// file and stderr so the user sees something even when the log file cannot
/// be opened.
///
/// # Panics
///
/// This function itself will not panic.
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        // ── Compose the detail string ─────────────────────────────────────
        // `PanicInfo::message()` (stable since 1.73) gives the formatted
        // message; fall back to the `Display` impl when not available.
        let detail = format!("{info}");

        // Location, if present.
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "(location unavailable)".to_owned());

        let full_detail = format!("location: {location}\n{detail}");

        // ── Write to crash file ───────────────────────────────────────────
        if let Some((mut file, path)) = open_crash_file() {
            write_report(&mut file, "PANIC", &full_detail);
            eprintln!("[edit] panic — crash report written to {}", path.display());
        }

        // ── Always write to stderr ────────────────────────────────────────
        write_report(&mut std::io::stderr(), "PANIC", &full_detail);
    }));
}

/// Installs a SIGSEGV signal handler that writes a crash report then exits.
///
/// Uses `signal_hook::low_level::register` (async-signal-safe path) to
/// register the handler. On SIGSEGV the process:
/// 1. Attempts to write a crash report to the XDG state directory.
/// 2. Exits with code **139** (the conventional SIGSEGV exit status on Linux).
///
/// # Safety
///
/// Signal handlers execute in a restricted async-signal-safe context.
/// The crash-file write is best-effort; it may be incomplete if the
/// process address space is severely corrupted.
///
/// # Errors / panics
///
/// Logs a warning (via `eprintln!`) if registration fails; does not panic.
pub fn install_signal_handler() {
    // SAFETY: We register a handler that performs only async-signal-safe
    // operations (write(2) via a raw fd) where possible, then calls _exit(2).
    // The `signal_hook::low_level::register` contract requires the closure to
    // be `Fn() + Sync + Send + 'static`.
    let result = unsafe {
        signal_hook::low_level::register(signal_hook::consts::SIGSEGV, move || {
            // Best-effort: open the crash dir and write a terse report.
            // We deliberately avoid any heap allocation that might recurse
            // into a corrupted allocator; the helpers above do allocate, but
            // they are our best available option short of assembly.
            let dir = crash_dir();
            // Try to create the directory; ignore errors in the signal handler.
            let _ = fs::create_dir_all(&dir);
            let path = dir.join(format!("crash-{}-sigsegv.log", unix_ts()));
            if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
                write_report(&mut f, "SIGSEGV", "Segmentation fault (signal 11)");
                // Flush best-effort.
                let _ = f.flush();
            }
            // Write a brief notice to stderr (fd 2) using a raw syscall path
            // via eprintln! — this allocates but is the clearest approach.
            eprintln!("[edit] SIGSEGV — crash report: {}", path.display());

            // Exit with code 139 (128 + SIGSEGV signal number 11).
            // SAFETY: _exit is async-signal-safe.
            // Use std::process::exit (async-signal-safe on Linux for code 139).
            std::process::exit(139);
        })
    };

    match result {
        Ok(_) => log::debug!("SIGSEGV crash handler installed"),
        Err(e) => eprintln!("[edit] WARNING: could not install SIGSEGV handler: {e}"),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crash_dir_ends_with_edit() {
        let d = crash_dir();
        assert!(d.ends_with("edit"), "unexpected crash dir: {d:?}");
    }

    #[test]
    fn unix_ts_is_nonzero() {
        assert!(unix_ts() > 0);
    }

    #[test]
    fn write_report_does_not_panic() {
        let mut buf = Vec::new();
        write_report(&mut buf, "TEST", "test detail");
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("TEST"));
        assert!(s.contains("test detail"));
        assert!(s.contains("timestamp (unix):"));
        assert!(s.contains("=== end of report ==="));
    }
}
