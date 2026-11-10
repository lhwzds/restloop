//! Tests of the CLI: the process the user runs, on every platform.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// The directory of the test, removed when it goes: an assertion that fails must not
/// leave it behind for the next run.
struct Dir(PathBuf);

impl Dir {
    /// A path in the directory of the test.
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        // The directory is the one the test made: it goes when the test ends.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Waits for the child to exit, and kills it after 10 s: a CLI that never ends must
/// not hold the test forever. Returns its exit code, its stdout and its stderr.
fn wait_for(child: &mut Child) -> (Option<i32>, String, String) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            let mut stdout = String::new();
            let mut stderr = String::new();
            if let Some(mut out) = child.stdout.take() {
                out.read_to_string(&mut stdout).unwrap();
            }
            if let Some(mut err) = child.stderr.take() {
                err.read_to_string(&mut stderr).unwrap();
            }
            return (status.code(), stdout, stderr);
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("restloop did not exit within 10 s");
        }
        thread::sleep(Duration::from_millis(50));
    }
}

/// A bad header in RESTLOOP_HEADERS ends the process before the run: it says which
/// line is wrong, never the text, and the last-message file is left as it was.
#[test]
fn a_bad_header_ends_the_run_before_it_starts() {
    // The directory of the test is fresh under the target's temp dir, so a file
    // of another run cannot be taken for this one's.
    let dir = Dir(PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("restloop-cli-bad-header-{}", std::process::id())));
    let _ = std::fs::remove_dir_all(&dir.0);
    std::fs::create_dir_all(&dir.0).unwrap();
    let last = dir.path("last.txt");
    std::fs::write(&last, "the old reply\n").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_restloop"))
        .arg("hi")
        .stdin(Stdio::null())
        .env("RESTLOOP_MODEL", "m")
        .env("RESTLOOP_API_KEY", "k")
        .env("RESTLOOP_BASE_URL", "http://127.0.0.1:1/v1")
        .env("RESTLOOP_HEADERS", "Bad Name: secret")
        .env("RESTLOOP_LAST_MESSAGE", &last)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let (code, stdout, stderr) = wait_for(&mut child);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.contains("RESTLOOP_HEADERS: line 1 is not a valid header"),
        "{stderr}"
    );
    assert!(!stdout.contains("secret"), "{stdout}");
    assert!(!stderr.contains("secret"), "{stderr}");
    assert!(last.exists());
}
