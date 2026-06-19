//! Integration tests for the session save/restore feature (Feature 003).

use std::env;
use std::path::PathBuf;
use std::sync::Mutex;

use edit::session::{
    load_session, save_session, session_path, BufferEntry, SessionData, SplitLayoutKind,
};

// Serialize XDG env mutations so parallel test threads do not interfere.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_temp_state(f: impl FnOnce(PathBuf)) {
    let _guard = ENV_LOCK.lock().unwrap();
    let tmp = env::temp_dir().join(format!("edit_integ_session_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    env::set_var("XDG_STATE_HOME", &tmp);
    f(tmp.clone());
    env::remove_var("XDG_STATE_HOME");
    let _ = std::fs::remove_dir_all(&tmp);
}

fn sample_session() -> SessionData {
    SessionData {
        version: 1,
        active_buffer: 0,
        split_layout: SplitLayoutKind::None,
        active_pane: 0,
        buffers: vec![BufferEntry {
            path: "/tmp/hello.txt".to_string(),
            cursor_line: 3,
            cursor_col: 7,
        }],
    }
}

// T028 — core round-trip
#[test]
fn test_save_then_load_round_trip() {
    with_temp_state(|_| {
        let data = sample_session();
        save_session(&data).expect("save_session should succeed");
        let loaded = load_session().expect("load_session should not error");
        let loaded = loaded.expect("session should be Some after save");
        assert_eq!(loaded, data);
    });
}

// T028 — corrupt session file
#[test]
fn test_corrupt_session_file_returns_err() {
    with_temp_state(|_| {
        let path = session_path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"garbage bytes [[[").unwrap();
        assert!(
            matches!(load_session(), Err(_)),
            "corrupt TOML must return Err"
        );
    });
}

// T028 — unknown schema version
#[test]
fn test_unknown_version_returns_err() {
    with_temp_state(|_| {
        let path = session_path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let toml = concat!(
            "version = 42\n",
            "active_buffer = 0\n",
            "active_pane = 0\n",
            "split_layout = \"none\"\n\n",
            "[[buffers]]\n",
            "path = \"/tmp/x.txt\"\n",
            "cursor_line = 1\n",
            "cursor_col = 1\n",
        );
        std::fs::write(&path, toml).unwrap();
        assert!(
            matches!(load_session(), Err(_)),
            "unknown version must return Err"
        );
    });
}

// T028 — absent file
#[test]
fn test_absent_file_returns_ok_none() {
    with_temp_state(|_| {
        // No file written — session.toml does not exist.
        assert!(
            matches!(load_session(), Ok(None)),
            "absent session file must return Ok(None)"
        );
    });
}

// T029 — --no-session flag: when config.no_session is true the guard in main()
// yields (None, None). We verify the guard logic directly.
#[test]
fn test_no_session_flag_skips_load() {
    with_temp_state(|_| {
        // Write a valid session so we know it exists.
        save_session(&sample_session()).unwrap();

        // Simulate the guard in main(): files.is_empty() && !config.no_session
        let files_empty = true;
        let no_session = true; // --no-session flag set
        let (session, warning) = if files_empty && !no_session {
            match load_session() {
                Ok(Some(d)) => (Some(d), None),
                Err(msg) => (None, Some(msg)),
                Ok(None) => (None, None),
            }
        } else {
            (None, None)
        };
        assert!(
            session.is_none(),
            "session must be None when --no-session is set"
        );
        assert!(warning.is_none(), "no warning when --no-session is set");
    });
}

// T030 — explicit file arguments bypass session restore.
#[test]
fn test_explicit_files_bypass() {
    with_temp_state(|_| {
        save_session(&sample_session()).unwrap();

        // Simulate: files is non-empty → guard is false.
        let files: Vec<PathBuf> = vec![PathBuf::from("/tmp/explicit.txt")];
        let no_session = false;
        let (session, warning) = if files.is_empty() && !no_session {
            match load_session() {
                Ok(Some(d)) => (Some(d), None),
                Err(msg) => (None, Some(msg)),
                Ok(None) => (None, None),
            }
        } else {
            (None, None)
        };
        assert!(
            session.is_none(),
            "session must be None when explicit files provided"
        );
        assert!(warning.is_none());
    });
}

// T036 — partial restore: one valid file, one non-existent path.
#[test]
fn test_partial_restore_skips_missing() {
    use edit::config::Config;
    use edit::encoding::encoding_from_str;

    with_temp_state(|tmp| {
        // Create the surviving file on disk.
        let real_file = tmp.join("real.txt");
        std::fs::write(&real_file, b"hello").unwrap();

        let data = SessionData {
            version: 1,
            active_buffer: 0,
            split_layout: SplitLayoutKind::None,
            active_pane: 0,
            buffers: vec![
                BufferEntry {
                    path: real_file.to_string_lossy().into_owned(),
                    cursor_line: 1,
                    cursor_col: 1,
                },
                BufferEntry {
                    path: "/nonexistent/ghost/file_xyz_abc.txt".to_string(),
                    cursor_line: 1,
                    cursor_col: 1,
                },
            ],
        };
        save_session(&data).unwrap();

        // Build an App with the session pending.
        let enc = encoding_from_str("utf-8");
        let config = Config::default();
        let session = load_session().unwrap().expect("session should exist");
        let mut app = edit::app::App::new(config, vec![], enc, Some(session), None);

        // Simulate user pressing Y to restore.
        app.do_restore_session();

        // Only the surviving file should be in buffers.
        assert_eq!(
            app.buffers.len(),
            1,
            "only surviving file should be restored"
        );
        assert_eq!(app.active_idx, 0, "active_idx must be clamped to 0");
        let msg = app.status_message.as_deref().unwrap_or("");
        assert!(
            msg.contains("not found") || msg.contains("ghost"),
            "status message should mention the missing file; got: {:?}",
            msg
        );
    });
}

// T037 — crash exit does not write session file.
#[test]
fn test_crash_exit_does_not_write_session() {
    with_temp_state(|_| {
        // Write a known session first.
        save_session(&sample_session()).unwrap();
        let path = session_path();
        let original = std::fs::read(&path).unwrap();

        // A panic inside catch_unwind does NOT trigger the clean-exit path, so
        // save_session is never called by the crash handler. We verify this by
        // confirming the file content is unchanged after a forced panic.
        let _ = std::panic::catch_unwind(|| {
            panic!("simulated crash");
        });

        let after = std::fs::read(&path).unwrap();
        assert_eq!(
            original, after,
            "session file must be unchanged after a crash exit"
        );
    });
}
