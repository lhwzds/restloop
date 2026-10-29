//! restflow: a minimal agent loop. One model, one shell tool.

use serde_json::{Value, json};
use std::io::{BufRead, Read};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use std::{env, thread};

const MAX_STEPS: u64 = 100;
static START: OnceLock<Instant> = OnceLock::new();
const MAX_OUTPUT: usize = 10_000;
const DEFAULT_TIMEOUT: u64 = 120;

fn main() {
    START.get_or_init(Instant::now);
    let task = env::args().skip(1).collect::<Vec<_>>().join(" ");
    if task == "-h" || task == "--help" {
        println!(
            "usage: restflow [task]   (no task: read tasks from stdin)\n\nenv: RESTFLOW_MODEL (required), RESTFLOW_API_KEY, RESTFLOW_BASE_URL (default https://api.openai.com/v1),\n     RESTFLOW_TIMEOUT (seconds per command, default 120), RESTFLOW_MAX_STEPS (default 100),\n     RESTFLOW_LOG_USAGE=1 (token usage per request on stderr),\n     RESTFLOW_DEADLINE (seconds for the whole task; the model is told the time used),\n     RESTFLOW_LAST_MESSAGE (file that receives the final reply),\n     RESTFLOW_TRIM_THINKING_AT (prompt tokens; past it, keep reasoning of the last 2 replies only)"
        );
        return;
    }
    let Ok(model) = env::var("RESTFLOW_MODEL") else {
        eprintln!("restflow: set RESTFLOW_MODEL (see --help)");
        std::process::exit(2);
    };
    let mut messages = vec![json!({"role": "system", "content": system_prompt()})];
    let ask = |text: &str, messages: &mut Vec<Value>| {
        messages.push(json!({"role": "user", "content": text}));
        if let Err(e) = run(&model, messages) {
            eprintln!("restflow: {e}");
            return false;
        }
        true
    };
    if !task.is_empty() {
        let ok = ask(&task, &mut messages);
        std::process::exit(if ok { 0 } else { 1 });
    }
    let stdin = std::io::stdin();
    loop {
        eprint!("> ");
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        if !line.trim().is_empty() {
            ask(line.trim(), &mut messages);
        }
    }
}

fn system_prompt() -> String {
    let cwd = env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    format!(
        "You are a coding agent. Use the shell tool to inspect files, edit them and run commands until the task is done, then reply with a short summary and no tool call.\nOS: {}. Shell: {}. Working directory: {cwd}.\nCommands run non-interactively with no stdin and a {}s timeout; long output is truncated.{}",
        env::consts::OS,
        match shell_path() {
            Some(_) if cfg!(windows) => "bash (Git Bash)",
            Some(_) => "sh",
            None => "PowerShell",
        },
        timeout(),
        deadline().map_or(String::new(), |d| format!(
            "\nYou have {d}s in total for this task; each tool result ends with the time used so far."
        ))
    )
}

/// The whole task's time budget in seconds (RESTFLOW_DEADLINE), if set.
fn deadline() -> Option<u64> {
    env::var("RESTFLOW_DEADLINE")
        .ok()
        .and_then(|v| v.parse().ok())
}

/// Seconds since restflow started.
fn elapsed() -> u64 {
    START.get_or_init(Instant::now).elapsed().as_secs()
}

/// Seconds a command may run.
fn timeout() -> u64 {
    env_num("RESTFLOW_TIMEOUT", DEFAULT_TIMEOUT)
}

/// A number from the environment, or the default when unset or invalid.
fn env_num(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn run(model: &str, messages: &mut Vec<Value>) -> Result<(), String> {
    let tools = json!([{"type": "function", "function": {
        "name": "shell",
        "description": "Run a shell command and return its exit code and output.",
        "parameters": {"type": "object", "properties": {"command": {"type": "string"}}, "required": ["command"]}
    }}]);
    let max_steps = env_num("RESTFLOW_MAX_STEPS", MAX_STEPS);
    let log_usage = env::var("RESTFLOW_LOG_USAGE").is_ok_and(|v| v == "1");
    let trim_at = env_num("RESTFLOW_TRIM_THINKING_AT", 0);
    // RESTFLOW_LAST_MESSAGE: the final reply goes to this file. Remove an old one first
    // so a run that stops early cannot leave the previous run's answer behind.
    let last_message = env::var("RESTFLOW_LAST_MESSAGE").ok();
    if let Some(path) = &last_message {
        let _ = std::fs::remove_file(path);
    }
    for step in 1..=max_steps {
        let body = json!({"model": model, "messages": messages, "tools": tools});
        let reply = complete(&body)?;
        if log_usage {
            let u = &reply["usage"];
            eprintln!(
                "usage: step={step} prompt_tokens={} completion_tokens={} cached_tokens={} elapsed={}s",
                u["prompt_tokens"],
                u["completion_tokens"],
                u["prompt_tokens_details"]["cached_tokens"],
                elapsed()
            );
        }
        let msg = &reply["choices"][0]["message"];
        let text = msg["content"].as_str().unwrap_or("");
        if !text.trim().is_empty() {
            println!("{}", text.trim());
        }
        let calls = msg["tool_calls"].as_array().cloned().unwrap_or_default();
        let mut keep = json!({"role": "assistant", "content": text});
        if let Some(r) = msg.get("reasoning_content").filter(|r| !r.is_null()) {
            keep["reasoning_content"] = r.clone();
        }
        if !calls.is_empty() {
            keep["tool_calls"] = json!(calls);
        }
        messages.push(keep);
        // Old reasoning is most of the context growth. Past the threshold, keep it only
        // for the last two replies; trimming in bursts keeps the cached prefix stable.
        if trim_at > 0 && reply["usage"]["prompt_tokens"].as_u64().unwrap_or(0) > trim_at {
            let n = trim_thinking(messages, 2);
            if n > 0 {
                eprintln!("restflow: dropped the reasoning of {n} older replies");
            }
        }
        if calls.is_empty() {
            if let Some(path) = &last_message {
                // Some models put their reasoning in <think> tags inside the reply.
                let answer = text.rsplit_once("</think>").map_or(text, |(_, a)| a).trim();
                if let Err(e) = std::fs::write(path, format!("{answer}\n")) {
                    eprintln!("restflow: cannot write {path}: {e}");
                }
            }
            return Ok(());
        }
        for call in &calls {
            let args = call["function"]["arguments"].as_str().unwrap_or("{}");
            let a: Value = serde_json::from_str(args).unwrap_or(Value::Null);
            let output = match a["command"].as_str() {
                Some(cmd) => {
                    eprintln!("\x1b[2m$ {cmd}\x1b[0m");
                    // A command may not outlive the task's time budget.
                    let left = deadline().map_or(u64::MAX, |d| d.saturating_sub(elapsed()).max(1));
                    let t = Instant::now();
                    let mut out = shell(cmd, timeout().min(left));
                    eprintln!(
                        "\x1b[2m[{:.1}s] {}\x1b[0m",
                        t.elapsed().as_secs_f64(),
                        out.lines().take(10).collect::<Vec<_>>().join("\n")
                    );
                    if let Some(d) = deadline() {
                        out.push_str(&format!("\n[time used: {}s of {d}s]", elapsed()));
                    }
                    out
                }
                None => {
                    format!("error: arguments must be JSON with a \"command\" string, got: {args}")
                }
            };
            messages.push(json!({"role": "tool", "tool_call_id": call["id"], "content": output}));
        }
    }
    Err(format!("stopped after {max_steps} steps"))
}

/// Removes the reasoning from all assistant messages but the last `keep`; returns how many changed.
fn trim_thinking(messages: &mut [Value], keep: usize) -> usize {
    let mut changed = 0;
    for m in messages
        .iter_mut()
        .rev()
        .filter(|m| m["role"] == "assistant")
        .skip(keep)
    {
        let had = m
            .as_object_mut()
            .and_then(|o| o.remove("reasoning_content"));
        let stripped = m["content"]
            .as_str()
            .and_then(|t| t.rsplit_once("</think>"))
            .map(|(_, a)| a.trim().to_string());
        if let Some(s) = &stripped {
            m["content"] = json!(s);
        }
        changed += usize::from(had.is_some() || stripped.is_some());
    }
    changed
}

fn complete(body: &Value) -> Result<Value, String> {
    let base = env::var("RESTFLOW_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into());
    let url = format!("{}/chat/completions", base.trim_end_matches('/'));
    let key = env::var("RESTFLOW_API_KEY").unwrap_or_default();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(600)))
        .build()
        .into();
    for attempt in 0..5 {
        let resp = agent
            .post(&url)
            .header("Authorization", &format!("Bearer {key}"))
            .send_json(body);
        let (status, text) = match resp {
            Ok(mut r) => (
                r.status().as_u16(),
                r.body_mut().read_to_string().unwrap_or_default(),
            ),
            Err(e) => (0, e.to_string()),
        };
        if status == 200 {
            return serde_json::from_str(&text).map_err(|e| format!("bad response: {e}: {text}"));
        }
        let what = if status == 0 {
            "request failed".to_string()
        } else {
            format!("HTTP {status}")
        };
        if !(status == 0 || status == 429 || status >= 500) || attempt == 4 {
            return Err(format!("{what}: {text}"));
        }
        eprintln!("restflow: {what}, retrying");
        thread::sleep(Duration::from_secs(2 << attempt));
    }
    unreachable!()
}

/// The shell commands run in: sh, or on Windows Git Bash when installed (None: PowerShell).
fn shell_path() -> Option<String> {
    if !cfg!(windows) {
        return Some("sh".into());
    }
    ["ProgramFiles", "ProgramFiles(x86)"]
        .iter()
        .filter_map(|v| env::var(v).ok())
        .map(|dir| format!("{dir}\\Git\\bin\\bash.exe"))
        .find(|p| std::path::Path::new(p).exists())
}

fn shell(cmd: &str, secs: u64) -> String {
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
            // The shell's own kill: slim containers often have no /bin/kill. Killing
            // the child as well means wait() cannot hang even if that fails.
            #[cfg(unix)]
            let _ = Command::new("sh")
                .args(["-c", &format!("kill -9 -{}", child.id())])
                .output();
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

fn truncate(s: &str) -> String {
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
