//! The default command runner: a command in a fresh shell, with a timeout.

use std::ffi::OsStr;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};

/// The characters of a command's output that are kept.
pub(crate) const MAX_OUTPUT: usize = 10_000;

/// Is this environment name one of restloop's own settings? Windows matches
/// names without regard to case, so a key spelled `Restloop_Api_Key` reaches
/// restloop there and must be caught too; elsewhere names are exact.
pub(crate) fn restloop_var(name: &OsStr) -> bool {
    let name = name.as_encoded_bytes();
    let prefix = b"RESTLOOP_";
    if name.len() < prefix.len() {
        return false;
    }
    #[cfg(windows)]
    return name[..prefix.len()].eq_ignore_ascii_case(prefix);
    #[cfg(not(windows))]
    return name.starts_with(prefix);
}

/// The shell commands run in: sh, or on Windows Git Bash when installed (None: PowerShell).
pub fn shell_path() -> Option<String> {
    if !cfg!(windows) {
        return Some("sh".into());
    }
    ["ProgramFiles", "ProgramFiles(x86)"]
        .iter()
        .filter_map(|v| env::var(v).ok())
        .map(|dir| format!("{dir}\\Git\\bin\\bash.exe"))
        .find(|p| std::path::Path::new(p).exists())
}

/// Runs a command in a shell and returns its exit code and output.
pub fn shell(cmd: &str, secs: u64) -> String {
    let mut c = match shell_path() {
        Some(sh) => {
            let mut c = Command::new(sh);
            c.args(["-c", cmd]);
            c
        }
        None => {
            let mut c = Command::new("powershell");
            c.args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("[Console]::OutputEncoding = [Text.Encoding]::UTF8; {cmd}"),
            ]);
            c
        }
    };
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut c, 0);
    c.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("PYTHONUTF8", "1");
    // The model's commands get none of restloop's own settings (the key, its file, the URL).
    for (k, _) in env::vars_os() {
        if restloop_var(&k) {
            c.env_remove(k);
        }
    }
    let mut child = match c.spawn() {
        Ok(child) => child,
        Err(e) => return format!("error: {e}"),
    };
    #[cfg(windows)]
    let job = win::Job::new(&child);
    // stdout and stderr go into one buffer as they arrive, so output read so far
    // survives even if a background child keeps a pipe open.
    let buf = Arc::new(Mutex::new(Vec::new()));
    let pipe = |mut p: Box<dyn Read + Send>| {
        let buf = buf.clone();
        thread::spawn(move || {
            let mut chunk = [0u8; 8192];
            while let Ok(n @ 1..) = p.read(&mut chunk) {
                buf.lock().unwrap().extend_from_slice(&chunk[..n]);
            }
        })
    };
    let out = pipe(Box::new(child.stdout.take().unwrap()));
    let err = pipe(Box::new(child.stderr.take().unwrap()));
    // Wait for the shell and its pipes. Background children that keep the pipes
    // open get 2s after the shell exits, then the whole process group (job) is killed.
    let mut deadline = Instant::now() + Duration::from_secs(secs);
    let mut status = None;
    loop {
        if status.is_none()
            && let Ok(Some(s)) = child.try_wait()
        {
            status = Some(s.code().map_or("killed".into(), |c| c.to_string()));
            deadline = deadline.min(Instant::now() + Duration::from_secs(2));
        }
        if status.is_some() && out.is_finished() && err.is_finished() {
            break;
        }
        if Instant::now() > deadline {
            #[cfg(windows)]
            job.kill();
            // The process group is signalled directly: no shell is started.
            // Killing the child as well means wait() cannot hang even if that fails.
            #[cfg(unix)]
            unsafe {
                kill(-(child.id() as i32), 9)
            };
            let _ = child.kill();
            let _ = child.wait();
            status.get_or_insert(format!("timeout after {secs}s"));
            thread::sleep(Duration::from_millis(200));
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    let text = String::from_utf8_lossy(&buf.lock().unwrap()).into_owned();
    format!("exit: {}\n{}", status.unwrap_or_default(), truncate(&text))
}

pub(crate) fn truncate(s: &str) -> String {
    let n = s.chars().count();
    if n <= MAX_OUTPUT {
        return s.to_string();
    }
    let head: String = s.chars().take(MAX_OUTPUT / 2).collect();
    let tail: String = s.chars().skip(n - MAX_OUTPUT / 2).collect();
    format!(
        "{head}\n... [{} chars truncated] ...\n{tail}",
        n - MAX_OUTPUT
    )
}

/// Windows has no process groups: a job object holds the command and everything it
/// starts (Git Bash children included), so a timeout can kill all of them.
#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    use std::os::windows::io::AsRawHandle;

    unsafe extern "system" {
        fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> *mut c_void;
        fn AssignProcessToJobObject(job: *mut c_void, process: *mut c_void) -> i32;
        fn TerminateJobObject(job: *mut c_void, exit_code: u32) -> i32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }

    pub struct Job(*mut c_void);

    impl Job {
        pub fn new(child: &std::process::Child) -> Job {
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                AssignProcessToJobObject(job, child.as_raw_handle());
                Job(job)
            }
        }

        pub fn kill(&self) {
            unsafe { TerminateJobObject(self.0, 1) };
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }
}

// The timeout path kills the process group itself; no libc dependency.
#[cfg(unix)]
unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}
