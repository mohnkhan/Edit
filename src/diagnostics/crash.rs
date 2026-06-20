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

use crossterm::{
    cursor::Show,
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};

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

/// Best-effort restore of the terminal to a usable state (Feature 028).
///
/// Mirrors the teardown `App::run` performs on a clean exit: leave the alternate
/// screen, disable mouse capture, show the cursor, and disable raw mode. Every
/// step ignores errors — this runs from the panic hook, must never itself panic,
/// and may run when the terminal was never put into raw mode (e.g. headless
/// tests or a panic before UI init), in which case the calls are harmless no-ops.
pub fn restore_terminal_best_effort() {
    let mut out = std::io::stdout();
    let _ = execute!(out, LeaveAlternateScreen, DisableMouseCapture, Show);
    let _ = disable_raw_mode();
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
        // ── Restore the terminal FIRST (Feature 028) ──────────────────────
        // Otherwise the process exits with raw mode + the alternate screen still
        // active, leaving the user's terminal garbled/"hung" and the report below
        // unreadable. Best-effort; never panics.
        restore_terminal_best_effort();

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

/// Installs a SIGTERM/SIGABRT handler that writes a crash report then exits.
///
/// SIGSEGV is a synchronous fault signal forbidden by `signal-hook`; the panic
/// hook already covers Rust-level memory safety violations. This handler covers
/// external termination (SIGTERM) and abort (SIGABRT) so crash reports are
/// written when the OS or another process kills the editor.
pub fn install_signal_handler() {
    for &sig in &[signal_hook::consts::SIGTERM, signal_hook::consts::SIGABRT] {
        let result = unsafe {
            signal_hook::low_level::register(sig, move || {
                let dir = crash_dir();
                let _ = fs::create_dir_all(&dir);
                let path = dir.join(format!("crash-{}-signal{}.log", unix_ts(), sig));
                if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
                    write_report(&mut f, "SIGNAL", &format!("Received signal {sig}"));
                    let _ = f.flush();
                }
                std::process::exit(128 + sig);
            })
        };
        match result {
            Ok(_) => log::debug!("Signal {} crash handler installed", sig),
            Err(e) => eprintln!("[edit] WARNING: could not install signal {sig} handler: {e}"),
        }
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

    // T009 (Feature 028): the terminal-restore path must run best-effort without
    // panicking even when no terminal/raw-mode is active (headless test runner).
    #[test]
    fn restore_terminal_best_effort_does_not_panic() {
        restore_terminal_best_effort();
        restore_terminal_best_effort(); // idempotent
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
