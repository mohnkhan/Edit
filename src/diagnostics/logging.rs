//! Logging initialisation (Task T006).
//!
//! Initialises `env_logger` and routes log output to the XDG-compliant path
//!   `$XDG_STATE_HOME/edit/logs/edit-<date>.log`
//! falling back to `~/.local/state/edit/logs/` when `XDG_STATE_HOME` is unset.
//!
//! If the environment variable `EDIT_DEBUG_RENDER=1` is set the function also
//! emits a render-debug preamble so that ncurses traces can be correlated with
//! log entries.

use std::fs;
use std::path::PathBuf;

// ── XDG path resolution ──────────────────────────────────────────────────────

/// Returns the directory that log files should be written to.
///
/// Resolution order:
/// 1. `$XDG_STATE_HOME/edit/logs/`
/// 2. `~/.local/state/edit/logs/`   (XDG default)
/// 3. `/tmp/edit/logs/`             (last resort if home dir is unavailable)
fn log_dir() -> PathBuf {
    // `dirs::state_dir()` already implements the XDG_STATE_HOME / fallback
    // logic, returning `None` only when neither the env var nor $HOME is set.
    let base: PathBuf = dirs::state_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

    base.join("edit").join("logs")
}

/// Returns the full path for today's log file.
fn log_file_path() -> PathBuf {
    let today = {
        // Use `YYYY-MM-DD` derived from the system clock without pulling in
        // `chrono` — we only need the date portion.
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Julian-day arithmetic (Gregorian calendar, no leap-second handling).
        let days = secs / 86_400;
        let (y, m, d) = days_to_ymd(days);
        format!("{y:04}-{m:02}-{d:02}")
    };

    log_dir().join(format!("edit-{today}.log"))
}

/// Converts a count of days since the Unix epoch (1970-01-01) to (year, month, day).
///
/// Uses the civil-date algorithm from Howard Hinnant's public-domain date library.
fn days_to_ymd(z: u64) -> (u32, u32, u32) {
    // The algorithm expects a signed integer starting from the epoch.
    let z = z as i64 + 719_468; // shift to 0000-03-01 epoch
    let era: i64 = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m as u32, d as u32)
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Initialises the global logger.
///
/// * Configures `env_logger` at `level` (the `RUST_LOG` env var can still
///   override this at runtime — see `env_logger` docs).
/// * Creates the XDG log directory (`0o700`) if it does not exist.
/// * Opens (or creates) a date-stamped log file and adds it as a second log
///   target alongside stderr.
/// * When `EDIT_DEBUG_RENDER=1` is set, emits a render-debug preamble and
///   records the resolved locale.
///
/// This function is idempotent: calling it more than once has no effect
/// because `env_logger::try_init` is a no-op after the first successful call.
pub fn init_logging(level: log::LevelFilter) {
    // ── 1. Prepare the log directory ─────────────────────────────────────
    let dir = log_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        // Cannot write to disk — fall back to stderr-only logging.
        eprintln!("[edit] WARNING: could not create log directory {dir:?}: {e}");
    } else {
        // Set directory permissions to 0o700 on Unix.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o700);
            let _ = fs::set_permissions(&dir, perms);
        }
    }

    // ── 2. Open the log file ─────────────────────────────────────────────
    let log_path = log_file_path();
    let log_file: Option<fs::File> = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| {
            eprintln!("[edit] WARNING: could not open log file {log_path:?}: {e}");
        })
        .ok();

    // ── 3. Build and install env_logger ──────────────────────────────────
    let mut builder = env_logger::Builder::new();
    builder.filter_level(level);
    // Allow RUST_LOG to override the compiled-in level.
    builder.parse_default_env();

    if let Some(file) = log_file {
        // Write log records to *both* stderr and the file.
        use std::io::BufWriter;
        let file = BufWriter::new(file);

        // env_logger's `target` accepts any `Write + Send + 'static`.
        builder.target(env_logger::Target::Pipe(Box::new(DualWriter::new(file))));
    }

    // `try_init` is a no-op if another logger is already installed.
    if let Err(e) = builder.try_init() {
        eprintln!("[edit] logger already initialised: {e}");
        return;
    }

    // ── 4. Log startup information ────────────────────────────────────────
    log_startup_info(&log_path);
}

/// Emits startup log lines after the logger is ready.
fn log_startup_info(log_path: &std::path::Path) {
    log::info!("edit starting — log file: {}", log_path.display());

    // Resolved locale.
    let locale = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_else(|_| "(unset)".to_owned());
    log::info!("resolved locale: {locale}");

    // Render-debug preamble.
    if std::env::var("EDIT_DEBUG_RENDER").as_deref() == Ok("1") {
        log::debug!(
            "EDIT_DEBUG_RENDER=1 — render-debug mode active; \
             ncurses trace path: {}",
            std::env::var("NCURSES_TRACE").unwrap_or_else(|_| "(not set)".to_owned())
        );
        log::debug!("TERM={}", std::env::var("TERM").unwrap_or_default());
        log::debug!(
            "COLORTERM={}",
            std::env::var("COLORTERM").unwrap_or_default()
        );
    }
}

// ── DualWriter ───────────────────────────────────────────────────────────────

/// Writes log records to a file *and* stderr simultaneously.
///
/// env_logger accepts a single `Write` target; this wrapper fans out to two.
struct DualWriter<W: std::io::Write + Send + 'static> {
    file: W,
}

impl<W: std::io::Write + Send + 'static> DualWriter<W> {
    fn new(file: W) -> Self {
        Self { file }
    }
}

impl<W: std::io::Write + Send + 'static> std::io::Write for DualWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Write to file; ignore individual write errors so the other sink
        // still receives the record.
        let _ = self.file.write_all(buf);
        // Write to stderr.
        std::io::stderr().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let _ = self.file.flush();
        std::io::stderr().flush()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_to_ymd_epoch() {
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
    }

    #[test]
    fn days_to_ymd_known_date() {
        // 2026-06-18 — day 20622 since 1970-01-01
        // 56 years (1970-2025), 14 leap years (72,76,...,2024) → 42*365+14*366 = 20454
        // Jan(31)+Feb(28)+Mar(31)+Apr(30)+May(31) = 151, Jun 1-18 offset = 168
        // 20454 + 168 = 20622; Unix timestamp = 20622 * 86400 = 1_781_740_800
        let days = 1_781_740_800u64 / 86_400; // = 20622
        let (y, m, d) = days_to_ymd(days);
        assert_eq!((y, m, d), (2026, 6, 18));
    }

    #[test]
    fn log_dir_is_under_state() {
        let dir = log_dir();
        assert!(dir.ends_with("edit/logs"));
    }
}
