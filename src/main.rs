//! restflow: a minimal agent loop. One model, one shell tool.

use serde_json::{Value, json};
use std::io::{BufRead, Read};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};

const MAX_STEPS: usize = 100;
const MAX_OUTPUT: usize = 10_000;
const CMD_TIMEOUT: Duration = Duration::from_secs(300);

fn main() {
    let task = env::args().skip(1).collect::<Vec<_>>().join(" ");
    if task == "-h" || task == "--help" {
        println!(
            "usage: restflow [task]   (no task: read tasks from stdin)\n\nenv: RESTFLOW_MODEL (required), RESTFLOW_API_KEY, RESTFLOW_BASE_URL (default https://api.openai.com/v1)"
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
        "You are a coding agent. Use the shell tool to inspect files, edit them and run commands until the task is done, then reply with a short summary and no tool call.\nOS: {}. Shell: {}. Working directory: {cwd}.\nCommands run non-interactively with no stdin and a {}s timeout; long output is truncated.",
        env::consts::OS,
        if cfg!(windows) { "PowerShell" } else { "sh" },
        CMD_TIMEOUT.as_secs()
    )
}

fn run(model: &str, messages: &mut Vec<Value>) -> Result<(), String> {
    let tools = json!([{"type": "function", "function": {
        "name": "shell",
        "description": "Run a shell command and return its exit code and output.",
        "parameters": {"type": "object", "properties": {"command": {"type": "string"}}, "required": ["command"]}
    }}]);
    for _ in 0..MAX_STEPS {
        let body = json!({"model": model, "messages": messages, "tools": tools});
        let reply = complete(&body)?;
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
        if calls.is_empty() {
            return Ok(());
        }
        for call in &calls {
            let args = call["function"]["arguments"].as_str().unwrap_or("{}");
            let output = match serde_json::from_str::<Value>(args)
                .ok()
                .and_then(|a| a["command"].as_str().map(String::from))
            {
                Some(cmd) => {
                    eprintln!("\x1b[2m$ {cmd}\x1b[0m");
                    let out = shell(&cmd);
                    eprintln!(
                        "\x1b[2m{}\x1b[0m",
                        out.lines().take(10).collect::<Vec<_>>().join("\n")
                    );
                    out
                }
                None => {
                    format!("error: arguments must be JSON with a \"command\" string, got: {args}")
                }
            };
            messages.push(json!({"role": "tool", "tool_call_id": call["id"], "content": output}));
        }
    }
    Err(format!("stopped after {MAX_STEPS} steps"))
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

fn shell(cmd: &str) -> String {
    let mut c = if cfg!(windows) {
        let mut c = Command::new("powershell");
        c.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("[Console]::OutputEncoding = [Text.Encoding]::UTF8; {cmd}"),
        ]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", cmd]);
        c
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
    // open get 2s after the shell exits, then the whole process group is killed.
    let mut deadline = Instant::now() + CMD_TIMEOUT;
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
            let pid = child.id().to_string();
            let _ = if cfg!(windows) {
                Command::new("taskkill")
                    .args(["/T", "/F", "/PID", &pid])
                    .output()
            } else {
                Command::new("kill")
                    .args(["-9", "--", &format!("-{pid}")])
                    .output()
            };
            let _ = child.wait();
            status.get_or_insert(format!("timeout after {}s", CMD_TIMEOUT.as_secs()));
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
