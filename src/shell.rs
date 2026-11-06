//! The default command runner: a command in a fresh shell, with a timeout.

use std::ffi::OsStr;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use std::{env, thread};

/// The characters of a command's output that are kept.
pub(crate) const MAX_OUTPUT: usize = 10_000;

/// The most a command's output may take: a writer that runs away cannot fill
/// the memory. What is over it is dropped.
const MAX_BUFFER: usize = 1024 * 1024;

/// ECHILD: the errno of "no child process". The child is gone and its status
/// is not to be had.
const ECHILD: i32 = 10;

/// A year in seconds: the timeout of a command is clamped to it, so that the
/// deadline of the command can always be built.
const MAX_SECS: u64 = 365 * 24 * 3600;

/// The moment the task's deadline runs out: no command starts after it.
static LIMIT: OnceLock<Instant> = OnceLock::new();

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

/// CLI setup, once, before the first command: stop signals, SIGCHLD, inherited
/// fds, panic hook. `limit` is the moment the task's deadline runs out.
pub fn install(limit: Option<Instant>) {
    if let Some(limit) = limit {
        let _ = LIMIT.set(limit);
    }
    os::install();
    // A panic must not leave the commands' processes running.
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default(info);
        stop_all();
    }));
}

/// Stops every remembered group: TERM, up to the grace, KILL. Windows: closes
/// the jobs.
pub fn stop_all() {
    os::stop_all();
}

/// The grace of a stop: a second, or what is left of the deadline.
fn grace() -> Duration {
    let cap = Duration::from_secs(1);
    let Some(limit) = LIMIT.get() else {
        return cap;
    };
    cap.min(limit.saturating_duration_since(Instant::now()))
}

/// Whether the child has exited: its status is there, or the child is gone and
/// its status is not to be had.
fn exited(child: &mut Child) -> bool {
    match child.try_wait() {
        Ok(Some(_)) => true,
        Err(e) => e.raw_os_error() == Some(ECHILD),
        _ => false,
    }
}

/// Runs a command in a shell and returns its exit code and output.
///
/// A signal defers to the commands being started, and a caller that leaves the
/// process on its own while another thread is starting a command can leave
/// that command running.
pub fn shell(cmd: &str, secs: u64) -> String {
    // The task's time is up: the command does not start at all.
    if let Some(limit) = LIMIT.get()
        && Instant::now() >= *limit
    {
        return format!("exit: timeout after {secs}s\n");
    }
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
    let (mut child, group) = match os::spawn(&mut c) {
        Ok(v) => v,
        Err(e) => return e,
    };
    // stdout and stderr go into one buffer as they arrive, so output read so far
    // survives even if a background child keeps a pipe open.
    let buf = Arc::new(Mutex::new(Vec::new()));
    let done = Arc::new(AtomicBool::new(false));
    let pipe = |mut p: Box<dyn Read + Send>| {
        let buf = buf.clone();
        let done = done.clone();
        thread::spawn(move || {
            let mut chunk = [0u8; 8192];
            while let Ok(n @ 1..) = p.read(&mut chunk) {
                let mut b = buf.lock().unwrap();
                // What is over the buffer is dropped, but the pipe keeps being
                // read: a writer must not block on a full pipe.
                if !done.load(SeqCst) && b.len() < MAX_BUFFER {
                    let room = MAX_BUFFER - b.len();
                    b.extend_from_slice(&chunk[..n.min(room)]);
                }
            }
        })
    };
    let out = pipe(Box::new(child.stdout.take().unwrap()));
    let err = pipe(Box::new(child.stderr.take().unwrap()));
    // The timeout of the command: what is left of the deadline, if there is one.
    let mut deadline = Instant::now() + Duration::from_secs(secs.min(MAX_SECS));
    if let Some(limit) = LIMIT.get() {
        deadline = deadline.min(*limit);
    }
    // Wait for the shell and its pipes. Background children that keep the pipes
    // open get 2s after the shell exits, then the whole process group (job) is killed.
    let mut status = None;
    loop {
        if status.is_none() && exited(&mut child) {
            // The code of the child, or a note that it is gone without one.
            status = Some(match child.try_wait() {
                Ok(Some(s)) => s.code().map_or("killed".into(), |c| c.to_string()),
                _ => "exited, status unknown".into(),
            });
            deadline = deadline.min(Instant::now() + Duration::from_secs(2));
        }
        if status.is_some() && out.is_finished() && err.is_finished() {
            break;
        }
        let now = Instant::now();
        if now > deadline {
            let stopped = os::stop_command(&mut child, &group, grace());
            let mut text = status.unwrap_or_else(|| format!("timeout after {secs}s"));
            if !stopped {
                text.push_str(" (child did not exit after SIGKILL)");
            }
            status = Some(text);
            break;
        }
        // Sleep until the deadline, or for a moment, whichever comes first.
        let wait = deadline.saturating_duration_since(now);
        thread::sleep(wait.min(Duration::from_millis(50)));
    }
    // A group that is gone is forgotten now: its id could be reused before the next command.
    #[cfg(unix)]
    os::sweep();
    let text = {
        let b = buf.lock().unwrap();
        // The text is taken: what comes after it is of no use.
        done.store(true, SeqCst);
        String::from_utf8_lossy(&b).into_owned()
    };
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

/// The groups of the commands on Unix: a slot holds the process group id of a
/// command that is still running.
#[cfg(unix)]
mod os {
    use std::process::{Child, Command};
    use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering::SeqCst};
    use std::time::{Duration, Instant};

    /// The slots of the groups: 0 is free, -1 is taken by a command that is
    /// starting, and a bigger number is the process group id. A slot is never
    /// overwritten.
    static SLOTS: [AtomicI32; 32] = [const { AtomicI32::new(0) }; 32];
    /// The commands between "slot reserved" and "pgid stored".
    static SPAWNING: AtomicUsize = AtomicUsize::new(0);
    /// A stop signal that arrived while a command was starting.
    static PENDING: AtomicI32 = AtomicI32::new(0);

    /// A command's group: the process group id of the command.
    pub struct Group {
        pgid: i32,
    }

    /// Whether the group is gone: the kernel says there is no such process, or
    /// on macOS that it may not be signalled.
    ///
    /// The limit: on macOS a group with only setuid members frees its slot with
    /// no report, for the kernel cannot tell it from an empty group.
    fn gone(pgid: i32) -> bool {
        if unsafe { libc::kill(-pgid, 0) } == 0 {
            return false;
        }
        let errno = std::io::Error::last_os_error().raw_os_error();
        errno == Some(libc::ESRCH) || (cfg!(target_os = "macos") && errno == Some(libc::EPERM))
    }

    /// The group's id, or 0 when it is gone; a group that is gone frees its slot.
    pub fn alive(slot: &AtomicI32) -> i32 {
        let pgid = slot.load(SeqCst);
        if pgid <= 0 {
            return 0;
        }
        if gone(pgid) {
            // Another command may have taken the slot in the meantime.
            let _ = slot.compare_exchange(pgid, 0, SeqCst, SeqCst);
            return 0;
        }
        pgid
    }

    /// Signals the group. A SIGKILL that fails on a group that is not gone is
    /// reported, directly: stderr's lock may be held.
    fn signal_group(pgid: i32, sig: i32) {
        if unsafe { libc::kill(-pgid, sig) } != 0 && sig == libc::SIGKILL && !gone(pgid) {
            let line = b"restloop: cannot stop a process group\n";
            unsafe { libc::write(2, line.as_ptr() as *const libc::c_void, line.len()) };
        }
    }

    /// Stops the groups: SIGTERM, the grace, SIGKILL.
    fn stop_slots(slots: &[AtomicI32], grace: Duration, mut leader: Option<&mut Child>) {
        for slot in slots {
            let pgid = alive(slot);
            if pgid != 0 {
                signal_group(pgid, libc::SIGTERM);
            }
        }
        let until = Instant::now() + grace;
        loop {
            // The leader is reaped here: a zombie leader keeps its group.
            if leader.as_mut().is_some_and(|l| super::exited(l)) {
                leader = None;
            }
            // Nothing is left to stop: the work is done.
            if slots.iter().all(|s| alive(s) == 0) {
                return;
            }
            let now = Instant::now();
            if now >= until {
                break;
            }
            // poll, not thread::sleep: the signal handler stops the groups too.
            let rest = (until - now).min(Duration::from_millis(50));
            unsafe { libc::poll(std::ptr::null_mut(), 0, rest.as_millis() as i32) };
        }
        for slot in slots {
            let pgid = alive(slot);
            if pgid != 0 {
                signal_group(pgid, libc::SIGKILL);
            }
        }
    }

    /// Stops the group of a command: the grace first, then SIGKILL. Says
    /// whether the command was seen to exit.
    pub fn stop_command(child: &mut Child, group: &Group, grace: Duration) -> bool {
        // The command stops only its own group: a slot that another command
        // has taken in the meantime is never signalled.
        let own = AtomicI32::new(group.pgid);
        stop_slots(std::slice::from_ref(&own), grace, Some(child));
        // A killed child is reaped even when the deadline is spent: the wait
        // is at least a moment long.
        let until = Instant::now() + super::grace().max(Duration::from_millis(100));
        loop {
            if super::exited(child) {
                return true;
            }
            if Instant::now() >= until {
                return false;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// The groups that are gone free their slots.
    pub fn sweep() {
        for slot in SLOTS.iter() {
            alive(slot);
        }
    }

    /// Starts the command in a group of its own and remembers the group.
    pub fn spawn(c: &mut Command) -> Result<(Child, Group), String> {
        use std::os::unix::process::CommandExt;
        c.process_group(0);
        SPAWNING.fetch_add(1, SeqCst);
        if PENDING.load(SeqCst) != 0 {
            done_spawning();
            return Err("error: restloop is stopping".into());
        }
        sweep();
        let slot = SLOTS
            .iter()
            .find(|s| s.compare_exchange(0, -1, SeqCst, SeqCst).is_ok());
        let Some(slot) = slot else {
            done_spawning();
            return Err(
                "error: 32 commands still have background processes running; stop some before starting more"
                    .into(),
            );
        };
        let (child, pgid) = match c.spawn() {
            Ok(child) => {
                let pgid = child.id() as i32;
                slot.store(pgid, SeqCst);
                (child, pgid)
            }
            Err(e) => {
                slot.store(0, SeqCst);
                done_spawning();
                return Err(format!("error: {e}"));
            }
        };
        done_spawning();
        Ok((child, Group { pgid }))
    }

    /// A command has started: the number of the commands that are starting
    /// goes down. Only the last one out handles a stop signal that came while
    /// the commands were starting: an earlier one would leave the groups of
    /// the commands that are still starting behind.
    fn done_spawning() {
        if SPAWNING.fetch_sub(1, SeqCst) == 1 {
            let pending = PENDING.load(SeqCst);
            if pending != 0 {
                stop_and_exit(pending);
            }
        }
    }

    /// Stops every group and leaves.
    fn stop_and_exit(sig: i32) -> ! {
        stop_slots(&SLOTS, super::grace(), None);
        unsafe { libc::_exit(128 + sig) }
    }

    /// The handler of the stop signals.
    extern "C" fn on_signal(sig: i32) {
        PENDING.store(sig, SeqCst);
        // A forked child that gets the signal before the exec sees SPAWNING
        // above 0 in its copy: it only defers, it never walks the table or leaves.
        if SPAWNING.load(SeqCst) == 0 {
            stop_and_exit(sig);
        }
    }

    /// The setup of the CLI: the stop signals, SIGCHLD and the file
    /// descriptors that were inherited.
    pub fn install() {
        // SIGCHLD: the default disposition, so that a wait can see a status.
        unsafe { libc::signal(libc::SIGCHLD, libc::SIG_DFL) };
        let handler: extern "C" fn(i32) = on_signal;
        for sig in [libc::SIGHUP, libc::SIGINT, libc::SIGTERM] {
            // An inherited ignore (nohup) stays: the run was asked to ignore it.
            if unsafe { libc::signal(sig, libc::SIG_IGN) } != libc::SIG_IGN {
                unsafe { libc::signal(sig, handler as libc::sighandler_t) };
            }
        }
        // No signal is masked: the handler stops the groups and leaves at
        // once, so there is nothing that a mask would have to unblock.
        close_inherited_fds();
    }

    /// Closes the file descriptors that were inherited: a command must not get
    /// them. When the list cannot be read, only the descriptors up to 1023 are
    /// closed.
    fn close_inherited_fds() {
        let fds: Vec<i32> = match std::fs::read_dir("/dev/fd").ok() {
            Some(entries) => entries
                .flatten()
                .filter_map(|e| e.file_name().to_str()?.parse().ok())
                .collect(),
            None => (3..1024).collect(),
        };
        for fd in fds.into_iter().filter(|fd| *fd >= 3) {
            unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) };
        }
    }

    /// Stops every remembered group.
    pub fn stop_all() {
        stop_slots(&SLOTS, super::grace(), None);
    }
}

/// Windows has no process groups: a job object holds the command and everything it
/// starts (Git Bash children included), so a timeout can kill all of them.
#[cfg(windows)]
mod os {
    use std::ffi::c_void;
    use std::os::windows::io::AsRawHandle;
    use std::process::{Child, Command};
    use std::sync::{Mutex, PoisonError};
    use std::time::Duration;

    type Handle = *mut c_void;

    /// A command's group: the job object that holds it.
    pub struct Group {
        job: Handle,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attributes: Handle, name: *const u16) -> Handle;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn TerminateJobObject(job: Handle, exit_code: u32) -> i32;
        fn SetInformationJobObject(job: Handle, class: i32, info: *const c_void, len: u32) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
        fn TerminateProcess(process: Handle, code: u32) -> i32;
        fn WaitForSingleObject(handle: Handle, millis: u32) -> u32;
    }

    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtResumeProcess(process: Handle) -> i32;
    }

    #[cfg(not(target_pointer_width = "64"))]
    compile_error!("32-bit Windows is not supported");

    /// 64-bit JOBOBJECT_EXTENDED_LIMIT_INFORMATION: the flags follow the two i64 time
    /// limits, so they sit at byte 16 of the 144-byte struct.
    #[repr(C)]
    struct ExtendedLimit {
        time_limits: [i64; 2],
        flags: u32,
        rest: [u64; 15],
    }

    const _: () = assert!(std::mem::size_of::<ExtendedLimit>() == 144);

    /// The jobs of the commands: closing one kills the processes in it.
    static JOBS: Mutex<Vec<usize>> = Mutex::new(Vec::new());

    /// Starts the command in a job of its own and remembers the job.
    pub fn spawn(c: &mut Command) -> Result<(Child, Group), String> {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x4); // CREATE_SUSPENDED
        let child = c.spawn().map_err(|e| format!("error: {e}"))?;
        unsafe {
            let process = child.as_raw_handle();
            let job = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
            let mut info: ExtendedLimit = std::mem::zeroed();
            info.flags = 0x2000; // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            let len = std::mem::size_of::<ExtendedLimit>() as u32;
            if job.is_null()
                || SetInformationJobObject(job, 9, &info as *const _ as *const c_void, len) == 0
                || AssignProcessToJobObject(job, process) == 0
                || NtResumeProcess(process) < 0
            {
                // The suspended process must not survive the failed setup.
                let stopped =
                    TerminateProcess(process, 1) != 0 && wait_for(process, super::grace());
                if !job.is_null() {
                    CloseHandle(job);
                }
                let note = if stopped {
                    ""
                } else {
                    ", and stopping it failed"
                };
                return Err(format!(
                    "error: could not put the command in a job object{note}"
                ));
            }
            JOBS.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(job as usize);
            Ok((child, Group { job }))
        }
    }

    /// Waits for the handle, at most the time given; says whether the process
    /// exited. Only a wait that came back with the object signaled counts: a
    /// failed wait says nothing, and a timeout says the process still runs.
    fn wait_for(handle: Handle, within: Duration) -> bool {
        let ms = within.as_millis().min(u128::from(u32::MAX)) as u32;
        // 0 is WAIT_OBJECT_0.
        unsafe { WaitForSingleObject(handle, ms) == 0 }
    }

    /// Stops the group of a command: the job object takes down every process in
    /// it. There is no graceful phase on Windows. Says whether the command was
    /// seen to exit.
    pub fn stop_command(child: &mut Child, group: &Group, _grace: Duration) -> bool {
        let process = child.as_raw_handle();
        if unsafe { TerminateJobObject(group.job, 1) } == 0 {
            // Only the leader can still be stopped; the rest of the job may live on.
            unsafe { TerminateProcess(process, 1) };
            return false;
        }
        wait_for(process, super::grace().max(Duration::from_millis(100)))
    }

    /// Closes the jobs, which kills the processes in them.
    pub fn stop_all() {
        let jobs: Vec<usize> =
            std::mem::take(&mut *JOBS.lock().unwrap_or_else(PoisonError::into_inner));
        for job in jobs {
            unsafe { CloseHandle(job as Handle) };
        }
    }

    /// Windows closes the job handles when the process ends, so the groups die
    /// with it: there is nothing to set up here.
    pub fn install() {}
}
