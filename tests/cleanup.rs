//! Tests: what a run leaves behind when it ends or is stopped.
//!
//! The tests run the real binary, so the groups of one test cannot reach
//! another: every test starts a restloop of its own.

#![cfg(unix)]

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::{Duration, Instant};

/// The signals the tests send.
const SIGHUP: i32 = 1;
const SIGTERM: i32 = 15;
const SIGKILL: i32 = 9;

unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

/// Sends a signal to a process.
fn signal(pid: i32, sig: i32) {
    unsafe { kill(pid, sig) };
}

/// Whether a process is still there.
fn alive(pid: i32) -> bool {
    unsafe { kill(pid, 0) == 0 }
}

/// Waits for the condition, until the deadline; says whether it came true.
fn wait_until(deadline: Instant, f: impl Fn() -> bool) -> bool {
    while Instant::now() < deadline {
        if f() {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    f()
}

/// The processes that are still there at the deadline. The deadline is the
/// same for every process: they must all be gone within it, not one by one.
fn left_alive(pids: &[i32], until: Instant) -> Vec<i32> {
    let mut left = Vec::new();
    for pid in pids {
        if !wait_until(until, || !alive(*pid)) {
            left.push(*pid);
        }
    }
    left
}

/// A directory of a test in the temp dir, made for the test alone: the path is
/// one that nobody else has, so the test never takes a file of another run.
/// The directory goes when the guard is dropped, whether the test ends well or
/// fails.
struct TempDir(std::path::PathBuf);

impl TempDir {
    /// A directory of the test in the temp dir, named after the test.
    fn new(name: &str) -> Self {
        let base = std::env::temp_dir();
        for n in 0..1000 {
            let path = base.join(format!("restloop-{name}-{}-{n}", std::process::id()));
            // The directory is made in one step: create_dir fails when the path
            // is taken, so the directory of the test is never the one of another
            // run, and nothing is taken away.
            match std::fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("cannot make the directory of the test at {path:?}: {e}"),
            }
        }
        panic!("cannot make a directory of the test in {base:?}");
    }

    /// A path in the directory of the test.
    fn path(&self, name: &str) -> std::path::PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // The directory is the one the guard made: it goes when the test ends.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The model of a test: a server that answers the requests with the replies it
/// holds, one after the other.
struct Model {
    url: String,
    /// The moments the model got its requests: the first one is the moment
    /// the model got its first request.
    requests: Receiver<Instant>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Model {
    /// A server that answers with the replies, in order.
    fn new(replies: Vec<String>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        // The listener does not wait for a request forever: the thread of the
        // server gives up after a while, so no test leaves it waiting.
        listener.set_nonblocking(true).unwrap();
        let (seen, requests) = channel();
        let thread = thread::spawn(move || {
            for reply in replies {
                let Some(mut stream) = accept(&listener) else {
                    break; // no request came: the thread ends instead of waiting.
                };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let (mut len, mut body) = (0, Vec::new());
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line.trim().is_empty() {
                        break;
                    }
                    if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
                        len = v.trim().parse().unwrap();
                    }
                }
                body.resize(len, 0);
                reader.read_exact(&mut body).unwrap();
                let _ = seen.send(Instant::now());
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    reply.len()
                );
                let _ = stream.write_all((head + &reply).as_bytes());
            }
        });
        Self {
            url,
            requests,
            thread: Some(thread),
        }
    }

    /// The moment the model got its first request, when it comes in time.
    fn first_request(&self, within: Duration) -> Option<Instant> {
        self.requests.recv_timeout(within).ok()
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        // The server waits for a request that never comes: the guard wakes it
        // when the test ends, so a failed assertion cannot leave it waiting.
        wake(&self.url);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// A connection to the listener, or None when no request comes for a while:
/// the thread of the server ends then, and waits in the accept no longer
/// than that.
fn accept(listener: &TcpListener) -> Option<std::net::TcpStream> {
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let _ = stream.set_nonblocking(false);
                return Some(stream);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= until {
                    return None;
                }
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => return None,
        }
    }
}

/// A request to the server that fails quietly: the server may be gone already,
/// and the end of a test must not fail because of it.
fn wake(url: &str) {
    let Ok(mut stream) = std::net::TcpStream::connect(url.trim_start_matches("http://")) else {
        return;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.write_all(b"POST / HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}");
    let mut reply = String::new();
    let _ = stream.read_to_string(&mut reply);
}

/// A reply with one command for the model to run.
fn call(command: &str) -> String {
    calls(&[command.to_string()])
}

/// A reply with commands for the model to run.
fn calls(commands: &[String]) -> String {
    let calls: Vec<Value> = commands
        .iter()
        .enumerate()
        .map(|(i, c)| {
            json!({"id": format!("c{i}"), "type": "function", "function": {"name": "shell",
                "arguments": json!({"command": c}).to_string()}})
        })
        .collect();
    json!({"choices": [{"message": {"role": "assistant", "content": "", "tool_calls": calls}}]})
        .to_string()
}

/// The last reply of the model: no command, the task is done.
fn say(text: &str) -> String {
    json!({"choices": [{"message": {"role": "assistant", "content": text}}]}).to_string()
}

/// The restloop of a test: it is stopped when the guard goes.
struct Run(Option<Child>);

impl Run {
    /// The process id of restloop.
    fn pid(&self) -> i32 {
        self.0.as_ref().unwrap().id() as i32
    }

    /// Whether restloop is still running.
    fn running(&mut self) -> bool {
        match self.0.as_mut() {
            Some(child) => matches!(child.try_wait(), Ok(None)),
            None => false,
        }
    }

    /// Waits for restloop to end, at most the time given; says how it ended.
    fn wait(&mut self, within: Duration) -> Option<ExitStatus> {
        wait_child(self.0.as_mut()?, within)
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        let Some(mut child) = self.0.take() else {
            return;
        };
        // The run may have ended by itself: its process id is not ours to
        // signal then.
        if !matches!(child.try_wait(), Ok(None)) {
            return;
        }
        signal(child.id() as i32, SIGTERM);
        if wait_child(&mut child, Duration::from_secs(3)).is_none() {
            signal(child.id() as i32, SIGKILL);
            wait_child(&mut child, Duration::from_secs(3));
        }
    }
}

/// Waits for a child to end, at most the time given.
fn wait_child(child: &mut Child, within: Duration) -> Option<ExitStatus> {
    let until = Instant::now() + within;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {}
            Err(_) => return None,
        }
        if Instant::now() >= until {
            return None;
        }
        thread::sleep(Duration::from_millis(20));
    }
}

/// Starts restloop with the settings of the test.
fn start(cmd: &mut Command, dir: &TempDir, url: &str) -> Run {
    cmd.env("NO_PROXY", "*")
        .env("RESTLOOP_MODEL", "test-model")
        .env("RESTLOOP_BASE_URL", url)
        .env("RESTLOOP_API_KEY", "test-key")
        .env("RESTLOOP_SESSION", dir.path("session.json"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    Run(Some(cmd.spawn().unwrap()))
}

/// The restloop of a test, started the usual way.
fn restloop(dir: &TempDir, url: &str) -> Run {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_restloop"));
    cmd.arg("go");
    start(&mut cmd, dir, url)
}

/// The results of the commands in the session file.
fn results(session: &std::path::Path) -> String {
    let text = std::fs::read_to_string(session).unwrap_or_default();
    let doc: Value = serde_json::from_str(&text).unwrap_or_default();
    doc["messages"]
        .as_array()
        .map(|m| {
            m.iter()
                .filter(|v| v["role"] == "tool")
                .map(|v| v["content"].as_str().unwrap_or("").to_string())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

/// The process id in the file a command wrote.
fn read_pid(path: &std::path::Path) -> i32 {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"));
    let pid = text.trim().to_string();
    pid.parse()
        .unwrap_or_else(|e| panic!("{pid:?} is not a process id: {e}"))
}

/// The first process id in the file the commands wrote.
fn first_pid(path: &std::path::Path) -> i32 {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"));
    let pid = text.lines().next().unwrap_or_default().trim().to_string();
    pid.parse()
        .unwrap_or_else(|e| panic!("{pid:?} is not a process id: {e}"))
}

/// At the end of a run, the processes its commands started are gone.
#[test]
fn a_run_stops_what_its_commands_started() {
    let dir = TempDir::new("cleanup-end");
    let pid = dir.path("pid");
    let model = Model::new(vec![
        call(&format!(
            "sleep 20 >/dev/null 2>&1 & echo $! > '{}'; exit 7",
            pid.display()
        )),
        say("done"),
    ]);
    let mut run = restloop(&dir, &model.url);
    let status = run
        .wait(Duration::from_secs(30))
        .expect("restloop did not end");
    assert_eq!(status.code(), Some(0));
    let out = results(&dir.path("session.json"));
    assert!(out.contains("exit: 7"), "{out}");
    // The background process is gone when the run is over.
    let pid = read_pid(&pid);
    assert!(
        wait_until(Instant::now() + Duration::from_secs(3), || !alive(pid)),
        "the process {pid} is still running"
    );
}

/// A run that is stopped leaves nothing behind.
#[test]
fn a_stopped_run_leaves_no_process_behind() {
    let dir = TempDir::new("cleanup-signal");
    let pid = dir.path("pid");
    let model = Model::new(vec![call(&format!(
        "sleep 20 >/dev/null 2>&1 & echo $! > '{}'; sleep 20",
        pid.display()
    ))]);
    let mut run = restloop(&dir, &model.url);
    // The command is running: it has written its process id.
    assert!(
        wait_until(Instant::now() + Duration::from_secs(20), || {
            std::fs::read_to_string(&pid).is_ok_and(|t| !t.trim().is_empty())
        }),
        "the command did not start"
    );
    let pid = read_pid(&pid);
    signal(run.pid(), SIGTERM);
    let status = run
        .wait(Duration::from_secs(20))
        .expect("restloop did not stop");
    assert_eq!(status.code(), Some(143));
    assert!(
        wait_until(Instant::now() + Duration::from_secs(3), || !alive(pid)),
        "the process {pid} is still running"
    );
}

/// A command that ignores the signal is killed when the deadline is spent.
#[test]
fn a_command_that_ignores_the_signal_is_killed_at_the_deadline() {
    let dir = TempDir::new("cleanup-deadline");
    let model = Model::new(vec![call("trap \"\" TERM; sleep 20"), say("done")]);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_restloop"));
    cmd.arg("go");
    cmd.env("RESTLOOP_DEADLINE", "1");
    let mut run = start(&mut cmd, &dir, &model.url);
    assert!(
        run.wait(Duration::from_secs(4)).is_some(),
        "restloop did not end"
    );
    let out = results(&dir.path("session.json"));
    assert!(out.contains("exit: timeout after"), "{out}");
    assert!(!out.contains("did not exit"), "{out}");
}

/// A process that left the group does not hold the run.
#[test]
fn an_escapee_does_not_hold_the_run() {
    let dir = TempDir::new("cleanup-escapee");
    let model = Model::new(vec![
        call("perl -MPOSIX -e 'POSIX::setsid(); sleep 5' &"),
        say("done"),
    ]);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_restloop"));
    cmd.arg("go");
    let began = Instant::now();
    let mut run = start(&mut cmd, &dir, &model.url);
    assert!(
        run.wait(Duration::from_secs(4)).is_some(),
        "restloop did not end"
    );
    assert!(began.elapsed() < Duration::from_secs(4));
}

/// A run holds at most 32 groups: the next command is refused.
#[test]
fn a_run_holds_at_most_32_groups() {
    let dir = TempDir::new("cleanup-table");
    let pid = dir.path("pid");
    let commands: Vec<String> = (0..33)
        .map(|_| format!("sleep 20 >/dev/null 2>&1 & echo $! >> '{}'", pid.display()))
        .collect();
    let model = Model::new(vec![calls(&commands), say("done")]);
    let mut run = restloop(&dir, &model.url);
    assert!(
        run.wait(Duration::from_secs(60)).is_some(),
        "restloop did not end"
    );
    let out = results(&dir.path("session.json"));
    assert!(out.contains("32 commands still have background"), "{out}");
    // The first process is gone when the run is over.
    let first = first_pid(&pid);
    assert!(
        wait_until(Instant::now() + Duration::from_secs(3), || !alive(first)),
        "the process {first} is still running"
    );
}

/// A file descriptor that was inherited is not passed on to a command.
#[test]
fn an_inherited_descriptor_is_not_passed_on() {
    let dir = TempDir::new("cleanup-fd");
    let model = Model::new(vec![call("[ -e /dev/fd/7 ]; echo fd7=$?"), say("done")]);
    let mut cmd = Command::new("sh");
    cmd.args([
        "-c",
        "exec 7</dev/null; exec \"$0\" task",
        env!("CARGO_BIN_EXE_restloop"),
    ]);
    let mut run = start(&mut cmd, &dir, &model.url);
    assert!(
        run.wait(Duration::from_secs(30)).is_some(),
        "restloop did not end"
    );
    let out = results(&dir.path("session.json"));
    assert!(out.contains("fd7=1"), "{out}");
}

/// A signal that was ignored before the run stays ignored.
#[test]
fn an_inherited_ignore_stays_ignored() {
    let dir = TempDir::new("cleanup-ignore");
    let model = Model::new(vec![call("sleep 20"), say("done")]);
    let mut cmd = Command::new("sh");
    cmd.args([
        "-c",
        "trap \"\" HUP; exec \"$0\" task",
        env!("CARGO_BIN_EXE_restloop"),
    ]);
    let mut run = start(&mut cmd, &dir, &model.url);
    // The run has started: the model has been asked.
    assert!(
        wait_until(Instant::now() + Duration::from_secs(20), || {
            model.first_request(Duration::ZERO).is_some()
        }),
        "the run did not start"
    );
    signal(run.pid(), SIGHUP);
    thread::sleep(Duration::from_millis(300));
    assert!(run.running(), "restloop stopped at the signal");
    // The guard stops the run.
}

/// A TERM during a run leaves nothing behind: the commands that started are
/// stopped, and the ones that did not start never do. The signal comes after
/// the model asked for the commands, so the window in which a command is
/// being started is not reached.
#[test]
fn term_during_a_run_leaves_nothing_behind() {
    // The whole test is over in a minute.
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut recorded = false;
    for round in 0..3 {
        // The time the test may still take.
        let left = || deadline.saturating_duration_since(Instant::now());
        let dir = TempDir::new(&format!("cleanup-term-{round}"));
        let pid = dir.path("pid");
        let command = format!("sleep 20 >/dev/null 2>&1 & echo $! >> '{}'", pid.display());
        let commands = vec![command; 5];
        let model = Model::new(vec![calls(&commands), say("done")]);
        let mut run = restloop(&dir, &model.url);
        // The moment the model got its first request: the signal comes that
        // many milliseconds after it, and not after this thread noticed it.
        let within = left().min(Duration::from_secs(20));
        let first = model.first_request(within).expect("the run did not start");
        // The signal comes at another moment in every round.
        let target = first + Duration::from_millis(round * 100);
        if let Some(rest) = target.checked_duration_since(Instant::now()) {
            thread::sleep(rest);
        }
        // A run that is over is not signalled: its process id is not ours.
        if run.running() {
            signal(run.pid(), SIGTERM);
        }
        let status = run
            .wait(Duration::from_secs(3).min(left()))
            .expect("restloop did not stop");
        assert!(
            matches!(status.code(), Some(0) | Some(143)),
            "restloop ended with {:?}",
            status.code()
        );
        // Every process the commands started is gone. A round where the
        // signal came before any command started has an empty file.
        let text = std::fs::read_to_string(&pid).unwrap_or_default();
        let pids: Vec<i32> = text
            .lines()
            .map(|line| {
                line.trim()
                    .parse()
                    .unwrap_or_else(|e| panic!("{line:?} is not a process id: {e}"))
            })
            .collect();
        recorded |= !pids.is_empty();
        // The deadline is the same for every process: they must all be gone
        // within three seconds of the end of the run, not three seconds
        // each. A process that is still there when the deadline is over is
        // a failure, however many processes there are.
        let until = Instant::now() + Duration::from_secs(3).min(left());
        let left = left_alive(&pids, until);
        assert!(left.is_empty(), "the processes {left:?} are still running");
    }
    assert!(recorded, "no round recorded a process id");
    assert!(Instant::now() < deadline, "the rounds took too long");
}
