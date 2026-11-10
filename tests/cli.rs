//! Tests of the CLI: the process the user runs, on every platform.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// A bad header in RESTLOOP_HEADERS ends the process before the run: it says which
/// line is wrong, never the text, and the last-message file is left as it was.
#[test]
fn a_bad_header_ends_the_run_before_it_starts() {
    // The directory of the test is fresh under the target's temp dir, so a file
    // of another run cannot be taken for this one's.
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("restloop-cli-bad-header");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let last = dir.join("last.txt");
    std::fs::write(&last, "the old reply\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_restloop"))
        .arg("hi")
        .stdin(Stdio::null())
        .env("RESTLOOP_MODEL", "m")
        .env("RESTLOOP_API_KEY", "k")
        .env("RESTLOOP_BASE_URL", "http://127.0.0.1:1/v1")
        .env("RESTLOOP_HEADERS", "Bad Name: secret")
        .env("RESTLOOP_LAST_MESSAGE", &last)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("RESTLOOP_HEADERS: line 1 is not a valid header"),
        "{stderr}"
    );
    assert!(!stdout.contains("secret"), "{stdout}");
    assert!(!stderr.contains("secret"), "{stderr}");
    assert!(last.exists());
    let _ = std::fs::remove_dir_all(&dir);
}
