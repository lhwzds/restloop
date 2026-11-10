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

/// The child of the test, stopped and reaped when it goes: a panic anywhere in the
/// test must not leave restloop running.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        // Both results are ignored: the child may already be gone.
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Reads a pipe to its end on its own thread and gives the bytes back: the child can
/// write more than a pipe holds, so the test must not read it while waiting.
fn drain(pipe: Option<impl Read + Send + 'static>) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        bytes
    })
}

/// Waits for the child to exit, and kills it after 10 s: a CLI that never ends must
/// not hold the test forever. Returns its exit code, its stdout and its stderr.
fn wait_for(child: &mut Child) -> (Option<i32>, String, String) {
    // The pipes are taken first and read apart from the waiting: a child that writes
    // more than a pipe holds cannot block on the way out.
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let deadline = Instant::now() + Duration::from_secs(10);
    let code = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status.code();
        }
        if Instant::now() >= deadline {
            // The child is stopped and reaped first, so its pipes close and the
            // threads of the reads end; only then are they joined.
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout.join();
            let _ = stderr.join();
            panic!("restloop did not exit within 10 s");
        }
        thread::sleep(Duration::from_millis(50));
    };
    (
        code,
        String::from_utf8_lossy(&stdout.join().unwrap()).into_owned(),
        String::from_utf8_lossy(&stderr.join().unwrap()).into_owned(),
    )
}

/// A bad header in RESTLOOP_HEADERS ends the process before the run: it says which
/// line is wrong, never the text, and the last-message file is left as it was.
#[test]
fn a_bad_header_ends_the_run_before_it_starts() {
    // The directory of the test is fresh under the target's temp dir, so a file
    // of another run cannot be taken for this one's. It is made in one step and
    // nothing is taken away first: a directory that is already there is another
    // run's, and the test must not use it.
    let dir = Dir(PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "restloop-cli-a_bad_header_ends_the_run_before_it_starts-{}",
        std::process::id()
    )));
    std::fs::create_dir(&dir.0)
        .unwrap_or_else(|e| panic!("cannot make the directory of the test at {:?}: {e}", dir.0));
    let last = dir.path("last.txt");
    std::fs::write(&last, "the old reply\n").unwrap();
    let mut child = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_restloop"))
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
            .unwrap(),
    );
    let (code, stdout, stderr) = wait_for(&mut child.0);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.contains("RESTLOOP_HEADERS: line 1 is not a valid header"),
        "{stderr}"
    );
    assert!(!stdout.contains("secret"), "{stdout}");
    assert!(!stderr.contains("secret"), "{stderr}");
    assert!(last.exists());
}
