//! Tests: the helpers, the shell runner on this platform, and whole runs against a
//! small local server standing in for the model.

use crate::shell::*;
use crate::*;
use serde_json::{Value, json};
use std::env;
use std::ffi::OsStr;
use std::io::{BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

/// A restloop setting spelled in mixed case: the same name as RESTLOOP_API_KEY on
/// Windows (which reads names without regard to case), a different one elsewhere.
const MIXED_CASE: &str = "Restloop_Mixed_Case";

/// The CLI's defaults, mirrored here: the core has none of its own and takes the
/// steps and the timeout from the caller.
const MAX_STEPS: u64 = 100;
const DEFAULT_TIMEOUT: u64 = 120;

/// The clock the deadline counts from: the tests share one, as the CLI does.
static START: OnceLock<Instant> = OnceLock::new();

/// Seconds since the tests started.
fn elapsed() -> u64 {
    START.get_or_init(Instant::now).elapsed().as_secs()
}

/// The settings the tests use: the model, the clock the deadline counts from and the
/// defaults of the CLI. A test changes what it needs.
fn test_config() -> Config {
    Config {
        model: "m".into(),
        base_url: "http://127.0.0.1:1/v1".into(),
        key: Box::new(|| Ok("k".into())),
        responses: false,
        reasoning_effort: None,
        max_steps: MAX_STEPS,
        timeout: DEFAULT_TIMEOUT,
        deadline: None,
        start: *START.get_or_init(Instant::now),
        log_usage: false,
        last_message: None,
        session: None,
    }
}

/// The messages a saved session holds.
fn saved_session(path: &std::path::Path) -> Vec<Value> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let doc: Value = serde_json::from_str(&text).unwrap_or_default();
    doc["messages"].as_array().cloned().unwrap_or_default()
}

/// A run with the settings the test gives and the shell as the tool.
fn run_loop(cfg: &Config, messages: &mut Vec<Value>) -> Result<(), String> {
    run(cfg, messages, &mut |cmd, secs| shell(cmd, secs))
}

#[test]
fn answer_strips_only_a_leading_think_block() {
    assert_eq!(answer("<think>a</think>b"), "b");
    assert_eq!(answer("  <think>a</think>\nb"), "\nb");
    assert_eq!(answer("  <think>a</think>\nb").trim(), "b");
    // A reply that is only reasoning has no answer.
    assert_eq!(answer("<think>unclosed"), "");
    // A reply that quotes the tag keeps everything before it.
    let quoted = "VERDICT: PASS\nthe code calls rsplit_once(\"</think>\")";
    assert_eq!(answer(quoted), quoted);
    // Only the first think block goes.
    assert_eq!(
        answer("<think>x</think>ok, see </think> in code"),
        "ok, see </think> in code"
    );
}

#[test]
fn too_long_matches_only_context_overflow_errors() {
    for e in [
        // Measured on 2026-10-28 (DeepSeek, MiniMax), and the OpenAI and Anthropic wording.
        "HTTP 400: {\"error\":{\"message\":\"This model's maximum context length is 1048576 tokens. However, you requested 1300036 tokens\"}}",
        "HTTP 400: {\"error\":{\"message\":\"invalid params, context window exceeds limit (2013)\"}}",
        "HTTP 400: {\"error\":{\"code\":\"context_length_exceeded\"}}",
        "HTTP 413: prompt is too long: 250000 tokens > 200000 maximum",
    ] {
        assert!(too_long(e), "{e}");
    }
    for e in [
        "HTTP 400: invalid params, context window must be a positive integer",
        "HTTP 400: tool_call_id is too long",
        "HTTP 500: maximum context length",
        "request failed: maximum context length",
    ] {
        assert!(!too_long(e), "{e}");
    }
}

#[test]
fn drop_old_output_halves_the_outputs_until_none_are_left() {
    let mut m: Vec<Value> = (0..4)
        .map(|i| json!({"role": "tool", "tool_call_id": i, "content": format!("exit: {i}\nlong output")}))
        .collect();
    assert_eq!(drop_old_output(&mut m), 2);
    assert_eq!(
        m[0]["content"],
        "exit: 0\n[output removed to fit the context window]"
    );
    assert_eq!(m[2]["content"], "exit: 2\nlong output");
    assert_eq!(drop_old_output(&mut m), 1);
    assert_eq!(drop_old_output(&mut m), 1);
    assert_eq!(drop_old_output(&mut m), 0);
}

#[test]
fn trim_thinking_keeps_the_last_two_replies() {
    let mut m: Vec<Value> = (0..4)
        .map(|i| json!({"role": "assistant", "content": format!("<think>t{i}</think>a{i}"), "reasoning_content": "r"}))
        .collect();
    assert_eq!(trim_thinking(&mut m, 2), 2);
    assert_eq!(m[0]["content"], "a0");
    // Emptied, not removed (DeepSeek needs the field on every reply of a tool turn).
    assert_eq!(m[1]["reasoning_content"], "");
    assert_eq!(m[3]["content"], "<think>t3</think>a3");
    assert_eq!(m[3]["reasoning_content"], "r");
    assert_eq!(trim_thinking(&mut m, 2), 0);
    // A content that only quotes the tag is left as it is.
    let mut m = vec![json!({"role": "assistant", "content": "plain </think> text"})];
    assert_eq!(trim_thinking(&mut m, 0), 0);
    assert_eq!(m[0]["content"], "plain </think> text");
    // Of the Responses API's output items only the reasoning is dropped.
    let items = json!([{"type": "reasoning"}, {"type": "message", "phase": "commentary"}]);
    let mut m = vec![json!({"role": "assistant", "content": "", "output_items": items})];
    assert_eq!(trim_thinking(&mut m, 0), 1);
    assert_eq!(m[0]["output_items"], json!([items[1]]));
    assert_eq!(trim_thinking(&mut m, 0), 0);
}

#[test]
fn an_empty_reply_has_no_text_and_no_tool_call() {
    fn say(content: Value) -> Value {
        json!({"choices": [{"message": {"role": "assistant", "content": content}}]})
    }
    let call = json!({"choices": [{"message": {"role": "assistant", "content": "",
        "tool_calls": [{"id": "a", "type": "function",
            "function": {"name": "shell", "arguments": "{\"command\":\"echo hi\"}"}}]}}]});
    // A tool call is an answer, whatever the text says.
    assert!(!empty(&call));
    assert!(!empty(&say(json!("done"))));
    assert!(empty(&say(json!(""))));
    assert!(empty(&say(Value::Null)));
    let missing = json!({"choices": [{"message": {"role": "assistant"}}]});
    assert!(empty(&missing));
    assert!(empty(&say(json!("<think>a</think>\n"))));
    // A reply that quotes the tag is an answer, not an empty one.
    assert!(!empty(&say(json!("VERDICT: PASS uses </think>"))));
    // The reasoning is not an answer.
    let thinking = json!({"choices": [{"message": {"role": "assistant", "content": "",
        "reasoning_content": "thinking"}}]});
    assert!(empty(&thinking));
}

#[test]
fn truncate_keeps_head_and_tail() {
    let s = "a".repeat(MAX_OUTPUT) + &"b".repeat(100);
    let t = truncate(&s);
    assert!(t.starts_with("aaa") && t.ends_with("bbb") && t.contains("[100 chars truncated]"));
    assert_eq!(truncate("short"), "short");
}

#[test]
fn shell_returns_exit_code_and_both_streams() {
    let out = shell("echo out; echo err >&2; exit 3", 30);
    assert!(out.starts_with("exit: 3\n"), "{out}");
    assert!(out.contains("out") && out.contains("err"), "{out}");
}

#[test]
fn shell_kills_a_command_at_its_timeout() {
    let t = Instant::now();
    let out = shell("echo started; sleep 30", 3);
    assert!(out.starts_with("exit: timeout after 3s\nstarted"), "{out}");
    assert!(t.elapsed() < Duration::from_secs(15));
}

#[test]
fn shell_does_not_wait_for_a_background_child_holding_the_output() {
    let t = Instant::now();
    let out = shell("sleep 30 &\necho done", 60);
    assert!(out.starts_with("exit: 0\ndone"), "{out}");
    assert!(t.elapsed() < Duration::from_secs(10));
}

/// Answers requests on a local port with `replies` in order (a JSON string is sent as
/// it is, as an event stream) and keeps each request body, with its Authorization
/// header under "auth".
fn serve(replies: Vec<(u16, Value)>) -> (String, Arc<Mutex<Vec<Value>>>) {
    let (url, bodies, _ended) = serve_until(replies);
    (url, bodies)
}

/// The server of `serve`, and the channel that says when its thread is done: a test
/// that waits for a request that never comes can tell that the thread ends anyway.
fn serve_until(replies: Vec<(u16, Value)>) -> (String, Arc<Mutex<Vec<Value>>>, mpsc::Receiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    // The listener does not wait for a request forever: the thread of the server
    // gives up after a while, so no test leaves it waiting in the accept.
    listener.set_nonblocking(true).unwrap();
    let bodies = Arc::new(Mutex::new(Vec::new()));
    let seen = bodies.clone();
    let (ended, waiting) = mpsc::channel();
    thread::spawn(move || {
        for (status, reply) in replies {
            let Some(mut stream) = accept(&listener) else {
                break; // no request came: the thread ends instead of waiting.
            };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let (mut len, mut auth) = (0, String::new());
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
                    len = v.trim().parse().unwrap();
                }
                if let Some(v) = line.to_lowercase().strip_prefix("authorization:") {
                    auth = v.trim().to_string();
                }
            }
            let mut body = vec![0; len];
            reader.read_exact(&mut body).unwrap();
            let mut body: Value = serde_json::from_slice(&body).unwrap();
            body["auth"] = json!(auth);
            seen.lock().unwrap().push(body);
            // Status 0 stalls before headers; status 1 stalls after 200 headers.
            if status == 0 || status == 1 {
                if status == 1 {
                    stream
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\n",
                        )
                        .unwrap();
                }
                thread::sleep(Duration::from_secs(10));
                continue;
            }
            let (kind, text) = match reply {
                Value::String(s) => ("text/event-stream", s),
                r => ("application/json", r.to_string()),
            };
            let head = format!(
                "HTTP/1.1 {status} X\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                text.len()
            );
            stream.write_all((head + &text).as_bytes()).unwrap();
        }
        // The thread is done: the replies are answered, or the server was woken.
        let _ = ended.send(());
    });
    (url, bodies, waiting)
}

/// A connection to the listener, or None when no request comes for a while: the
/// thread of the server ends then, and waits in the accept no longer than that.
fn accept(listener: &TcpListener) -> Option<std::net::TcpStream> {
    let until = Instant::now() + Duration::from_secs(60);
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                // The stream of a listener that does not block does not block either
                // (on macOS): it must wait for the request like any other.
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

/// A request to the server, so that its thread leaves the accept and ends: a test
/// that sends no request at all would leave it waiting for one.
fn wake(url: &str) {
    wake_quietly(url).expect("the server cannot be woken");
}

/// A request to the server that fails quietly: the server may be gone already, and
/// the end of a test must not fail because of it.
fn wake_quietly(url: &str) -> std::io::Result<()> {
    let mut stream = std::net::TcpStream::connect(url.trim_start_matches("http://"))?;
    // The wait is short: a server that does not answer must not hold the end of a
    // test.
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.write_all(b"POST / HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}")?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply)?;
    Ok(())
}

/// The server of a test, woken when the test ends: the thread that waits for a
/// request is woken too, so an assertion that fails before the wake cannot leave it
/// waiting for a request that never comes.
struct WakeOnEnd {
    url: String,
}

impl WakeOnEnd {
    fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

impl Drop for WakeOnEnd {
    fn drop(&mut self) {
        // The server may be gone already: a failure here must not hide the failure
        // of the test itself.
        let _ = wake_quietly(&self.url);
    }
}

/// A server that waits for one request and ends when it comes: the channel says
/// when its thread is done, so a test can tell that no thread is left waiting in
/// the accept.
fn serve_one() -> (String, mpsc::Receiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (ended, waiting) = mpsc::channel();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        // The whole request is read, so that the answer does not reset the connection.
        let mut len = 0;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                break;
            }
            if line.trim().is_empty() {
                break;
            }
            if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
                len = v.trim().parse().unwrap_or(0);
            }
        }
        let mut body = vec![0; len];
        let _ = reader.read_exact(&mut body);
        let _ = stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}");
        drop(stream);
        let _ = ended.send(());
    });
    (url, waiting)
}

/// A server that is woken ends its thread: no thread is left waiting for a
/// request that never comes.
#[test]
fn a_woken_server_leaves_no_thread_waiting() {
    let (url, ended) = serve_one();
    wake(&url);
    assert!(
        ended.recv_timeout(Duration::from_secs(5)).is_ok(),
        "the server thread is still waiting for a request"
    );
}

/// A test that fails leaves no thread waiting for a request either: the server of
/// the test is woken when the test ends, however it ends.
#[test]
fn a_failed_test_leaves_no_thread_waiting_for_a_request() {
    let done = json!({"choices": [{"message": {"role": "assistant", "content": "all done"}}]});
    let (url, _bodies, ended) = serve_until(vec![(200, done)]);
    let failed = std::panic::catch_unwind(|| {
        // The guard goes out of scope when the panic unwinds the stack: it wakes
        // the server, so the thread of the server ends.
        let _wake = WakeOnEnd::new(&url);
        panic!("the assertion of the test failed");
    });
    assert!(failed.is_err());
    assert!(
        ended.recv_timeout(Duration::from_secs(5)).is_ok(),
        "the server thread is still waiting for a request"
    );
}

// The only test that touches RESTLOOP_* variables; no other test reads them.
#[test]
fn a_run_survives_a_full_context_and_resumes_from_its_session() {
    let call = json!({"choices": [{"message": {"role": "assistant", "content": "", "reasoning_content": "thinking",
        "tool_calls": [{"id": "a", "type": "function", "function": {"name": "shell", "arguments": "{\"command\":\"echo hello\"}"}}]}}]});
    let full = json!({"error": {"message": "This model's maximum context length is 10 tokens."}});
    let done =
        |text: &str| json!({"choices": [{"message": {"role": "assistant", "content": text}}]});
    let (url, bodies) = serve(vec![
        (200, call),
        (400, full),
        (200, done("first")),
        (200, done("second")),
    ]);
    let dir = env::temp_dir().join(format!("restloop-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let session = dir.join("session.json");
    // The proxy settings of the shell running the tests must not reach this run: its
    // requests would miss the local server.
    unsafe {
        for (name, _) in env::vars_os() {
            if name
                .to_string_lossy()
                .to_ascii_uppercase()
                .ends_with("_PROXY")
            {
                env::remove_var(name);
            }
        }
    }
    let mut cfg = test_config();
    cfg.base_url = url.clone();
    cfg.session = Some(session.to_string_lossy().into_owned());

    let mut messages =
        vec![json!({"role": "system", "content": system_prompt(&cfg, "sh", "linux", "/tmp")})];
    messages.push(json!({"role": "user", "content": "one"}));
    run_loop(&cfg, &mut messages).unwrap();
    let sent = bodies.lock().unwrap().clone();
    assert_eq!(sent.len(), 3);
    let lengths: Vec<_> = sent
        .iter()
        .map(|b| b["messages"].as_array().unwrap().len())
        .collect();
    assert_eq!(lengths, [2, 4, 4]);
    assert!(
        sent[1]["messages"][3]["content"]
            .as_str()
            .unwrap()
            .contains("hello")
    );
    // After the 400 the command output was cut down to its exit line and the request sent again.
    assert_eq!(
        sent[2]["messages"][3]["content"],
        "exit: 0\n[output removed to fit the context window]"
    );

    // Resuming loads exactly what was saved: the last request plus the final reply.
    let mut resumed = saved_session(&session);
    assert_eq!(resumed, messages);
    resumed.push(json!({"role": "user", "content": "two"}));
    run_loop(&cfg, &mut resumed).unwrap();
    let last = bodies.lock().unwrap()[3]["messages"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(last[..4], sent[2]["messages"].as_array().unwrap()[..]);
    assert_eq!(last[4], json!({"role": "assistant", "content": "first"}));
    // Only the session file is left: the temporary file was renamed over it.
    let files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(files, [std::ffi::OsString::from("session.json")]);
    std::fs::remove_dir_all(&dir).unwrap();

    // A 200 without choices (MiniMax reports some errors that way) is retried, not taken
    // as an empty final reply.
    let (url, bodies) = serve(vec![
        (200, json!({"base_resp": {"status_code": 1002}})),
        (200, done("third")),
    ]);
    cfg.base_url = url;
    cfg.session = None;
    let mut m = vec![json!({"role": "user", "content": "three"})];
    run_loop(&cfg, &mut m).unwrap();
    assert_eq!(bodies.lock().unwrap().len(), 2);
    assert_eq!(m[1]["content"], "third");

    // The Responses API: a shell call with encrypted reasoning, then the final reply.
    // The output goes back as it came, in order, without the ids of messages and calls;
    // the key is read from its file.
    let event = |e: Value| format!("event: x\ndata: {e}\n\n");
    let item = |n: u64, i: Value| {
        event(json!({"type": "response.output_item.done", "output_index": n, "item": i}))
    };
    // The items finish out of order; output_index puts them back.
    let call = [
        item(1, json!({"type": "message", "id": "msg_1", "role": "assistant", "phase": "commentary",
            "content": [{"type": "output_text", "text": "looking"}]})),
        item(0, json!({"type": "reasoning", "id": "rs_1", "encrypted_content": "e1", "summary": []})),
        item(3, json!({"type": "function_call", "id": "fc_1", "call_id": "c1", "name": "shell",
            "arguments": "{\"command\":\"echo hi\"}"})),
        item(2, json!({"type": "reasoning", "id": "rs_2", "encrypted_content": "e2", "summary": []})),
        event(json!({"type": "response.completed", "response": {"output": [], "usage":
            {"input_tokens": 10, "output_tokens": 5, "input_tokens_details": {"cached_tokens": 4}}}})),
    ]
    .concat();
    let done = [
        item(
            0,
            json!({"type": "message", "role": "assistant",
            "content": [{"type": "refusal", "refusal": "no"}]}),
        ),
        event(json!({"type": "response.completed", "response": {"output": []}})),
        "data: [DONE]\n\n".to_string(),
    ]
    .concat();
    let failed = |code: &str| {
        json!(event(
            json!({"type": "response.failed", "response": {"error": {"code": code}}})
        ))
    };
    // A plan that is briefly unavailable is tried again; the top-level error event
    // carries its code at the top.
    let unavailable = json!(event(
        json!({"type": "error", "code": "subscription_sharing_user_unavailable"})
    ));
    let (url, bodies) = serve(vec![
        (200, json!(call)),
        (200, failed("server_error")),
        (200, unavailable),
        (200, failed("context_length_exceeded")),
        (200, json!(done)),
    ]);
    let key = env::temp_dir().join(format!("restloop-key-{}", std::process::id()));
    std::fs::write(&key, "tok1\n").unwrap();
    let key_file = key.clone();
    cfg.base_url = url;
    cfg.responses = true;
    cfg.reasoning_effort = Some("low".into());
    // The key comes from its file, and it is read for every request.
    cfg.key = Box::new(move || {
        std::fs::read_to_string(&key_file)
            .map_err(|e| format!("cannot read {}: {e}", key_file.display()))
    });
    let mut m = vec![
        json!({"role": "system", "content": "be brief"}),
        json!({"role": "user", "content": "four"}),
    ];
    run_loop(&cfg, &mut m).unwrap();
    let sent = bodies.lock().unwrap().clone();
    assert_eq!(sent.len(), 5);
    assert_eq!(sent[0]["auth"], "bearer tok1");
    assert_eq!(sent[0]["instructions"], "be brief");
    assert_eq!(
        sent[0]["input"],
        json!([{"role": "user", "content": "four"}])
    );
    assert_eq!(sent[0]["reasoning"], json!({"effort": "low"}));
    assert_eq!(
        (&sent[0]["store"], &sent[0]["stream"]),
        (&json!(false), &json!(true))
    );
    assert_eq!(sent[0]["tools"][0]["name"], "shell");
    assert_eq!(sent[0]["prompt_cache_key"], sent[4]["prompt_cache_key"]);
    // A server error is tried again with the same request.
    assert_eq!(sent[1]["input"], sent[2]["input"]);
    assert_eq!(sent[2]["input"], sent[3]["input"]);
    let input = sent[2]["input"].as_array().unwrap();
    let kinds: Vec<_> = input
        .iter()
        .map(|i| i["type"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(
        kinds,
        [
            "",
            "reasoning",
            "message",
            "reasoning",
            "function_call",
            "function_call_output"
        ]
    );
    assert_eq!(
        (&input[1]["id"], &input[1]["encrypted_content"]),
        (&json!("rs_1"), &json!("e1"))
    );
    assert_eq!(
        (&input[2]["id"], &input[2]["phase"]),
        (&Value::Null, &json!("commentary"))
    );
    assert_eq!(
        (&input[4]["id"], &input[4]["call_id"]),
        (&Value::Null, &json!("c1"))
    );
    assert!(input[5]["output"].as_str().unwrap().contains("hi"));
    // A request too long for the context gets its old command output cut, as with chat.
    assert_eq!(
        sent[4]["input"][5]["output"],
        "exit: 0\n[output removed to fit the context window]"
    );
    assert_eq!(
        (&m[2]["content"], &m[2]["tool_calls"][0]["id"]),
        (&json!("looking"), &json!("c1"))
    );
    assert_eq!(m[4]["content"], "no");

    // A used-up plan, an answer cut off by a content filter or an invalid request is
    // reported at once.
    let limit = json!({"error": {"code": "subscription_sharing_usage_limit_exceeded"}});
    let filtered = event(json!({"type": "response.incomplete",
        "response": {"incomplete_details": {"reason": "content_filter"}}}));
    let invalid = event(json!({"type": "error", "code": "invalid_request_error"}));
    for (reply, start) in [
        ((429, limit), "HTTP 429: "),
        ((200, json!(filtered)), "no reply: response.incomplete"),
        ((200, json!(invalid)), "no reply: error: "),
    ] {
        let (url, bodies) = serve(vec![reply]);
        cfg.base_url = url;
        let mut m = vec![json!({"role": "user", "content": "five"})];
        let e = run_loop(&cfg, &mut m).unwrap_err();
        assert!(e.starts_with(start), "{e}");
        assert_eq!(bodies.lock().unwrap().len(), 1);
    }

    // A key file that cannot be read stops the run before any request.
    std::fs::remove_file(&key).unwrap();
    let mut m = vec![json!({"role": "user", "content": "six"})];
    let e = run_loop(&cfg, &mut m).unwrap_err();
    assert!(e.starts_with("cannot read "), "{e}");

    // Back to chat completions, with a key of its own.
    cfg.responses = false;
    cfg.reasoning_effort = None;
    cfg.key = Box::new(|| Ok("k".into()));
    let (url, bodies) = serve(vec![(0, Value::Null)]);
    cfg.base_url = url;
    cfg.deadline = Some(elapsed() + 3);
    let mut m = vec![json!({"role": "user", "content": "seven"})];
    let start = Instant::now();
    let e = run_loop(&cfg, &mut m).unwrap_err();
    assert!(e.starts_with("deadline of"), "{e}");
    assert!(start.elapsed() < Duration::from_secs(10));
    assert_eq!(bodies.lock().unwrap().len(), 1);

    let (url, bodies) = serve(vec![(500, json!({"error": "server error"})); 5]);
    cfg.base_url = url;
    cfg.deadline = Some(elapsed() + 3);
    let mut m = vec![json!({"role": "user", "content": "eight"})];
    let start = Instant::now();
    let e = run_loop(&cfg, &mut m).unwrap_err();
    assert!(e.starts_with("deadline of"), "{e}");
    assert!(e.contains("HTTP 500"), "{e}");
    assert!(bodies.lock().unwrap().len() < 5);
    assert!(start.elapsed() < Duration::from_secs(10));

    let (url, bodies) = serve(vec![(400, json!({"error": "invalid"}))]);
    cfg.base_url = url;
    cfg.deadline = Some(elapsed() + 3);
    let e = run_loop(&cfg, &mut m).unwrap_err();
    assert_eq!(e, "HTTP 400: {\"error\":\"invalid\"}");
    let d = elapsed();
    cfg.deadline = Some(d);
    let e = run_loop(&cfg, &mut m).unwrap_err();
    assert_eq!(e, format!("deadline of {d}s reached"));
    assert_eq!(bodies.lock().unwrap().len(), 1);
    // A successful status whose body stalls is a request failure, not a bad JSON reply.
    let (url, bodies) = serve(vec![(1, Value::Null)]);
    cfg.base_url = url;
    cfg.deadline = Some(elapsed() + 3);
    let start = Instant::now();
    let e = run_loop(&cfg, &mut m).unwrap_err();
    assert!(e.starts_with("deadline of"), "{e}");
    assert!(e.contains("request failed"), "{e}");
    assert!(start.elapsed() < Duration::from_secs(10));
    assert_eq!(bodies.lock().unwrap().len(), 1);

    // A reply with no text and no tool call is a failed request, not an answer: it is
    // tried again, and the run ends with the next, complete reply.
    let say =
        |text: &str| json!({"choices": [{"message": {"role": "assistant", "content": text}}]});
    let (url, bodies) = serve(vec![(200, say("")), (200, say("after empty"))]);
    let last = env::temp_dir().join(format!("restloop-last-{}", std::process::id()));
    cfg.base_url = url;
    cfg.deadline = Some(elapsed() + 60);
    cfg.last_message = Some(last.to_string_lossy().into_owned());
    let mut m = vec![json!({"role": "user", "content": "nine"})];
    run_loop(&cfg, &mut m).unwrap();
    assert_eq!(bodies.lock().unwrap().len(), 2);
    assert_eq!(m[1]["content"], "after empty");
    let written = std::fs::read_to_string(&last).unwrap();
    assert_eq!(written.trim(), "after empty");
    cfg.last_message = None;
    let _ = std::fs::remove_file(&last);

    // A reply whose text is only the model's reasoning is an empty reply, too.
    let (url, bodies) = serve(vec![
        (200, say("<think>x</think>  ")),
        (200, say("after thinking")),
    ]);
    cfg.base_url = url;
    let mut m = vec![json!({"role": "user", "content": "ten"})];
    run_loop(&cfg, &mut m).unwrap();
    assert_eq!(bodies.lock().unwrap().len(), 2);
    assert_eq!(m[1]["content"], "after thinking");

    // The Responses API: a stream that ends without text and without a call is retried.
    let empty_stream = [
        event(json!({"type": "response.completed", "response": {"output": []}})),
        "data: [DONE]\n\n".to_string(),
    ]
    .concat();
    let (url, bodies) = serve(vec![(200, json!(empty_stream)), (200, json!(done))]);
    cfg.base_url = url;
    cfg.responses = true;
    let mut m = vec![json!({"role": "user", "content": "eleven"})];
    run_loop(&cfg, &mut m).unwrap();
    assert_eq!(bodies.lock().unwrap().len(), 2);
    assert_eq!(m[1]["content"], "no");
    cfg.responses = false;

    // An empty reply is tried again even when its metadata names a used-up quota:
    // the reply says nothing, so the next request may still work.
    let quota = json!({"choices": [{"message": {"role": "assistant", "content": "",
        "reasoning_content": "insufficient_quota"}}]});
    let (url, bodies) = serve(vec![(200, quota), (200, say("after quota"))]);
    cfg.base_url = url;
    let mut m = vec![json!({"role": "user", "content": "twelve"})];
    run_loop(&cfg, &mut m).unwrap();
    assert_eq!(bodies.lock().unwrap().len(), 2);
    assert_eq!(m[1]["content"], "after quota");

    // A final reply that quotes the tag is written to the last message file whole:
    // the verdict in front of it is the answer.
    let verdict = "VERDICT: PASS\nthe code calls rsplit_once(\"</think>\")";
    let reply = json!({"choices": [{"message": {"role": "assistant", "content": verdict}}]});
    let (url, bodies) = serve(vec![(200, reply)]);
    let last = env::temp_dir().join(format!("restloop-verdict-{}", std::process::id()));
    cfg.base_url = url;
    cfg.deadline = Some(elapsed() + 60);
    cfg.last_message = Some(last.to_string_lossy().into_owned());
    let mut m = vec![json!({"role": "user", "content": "thirteen"})];
    run_loop(&cfg, &mut m).unwrap();
    assert_eq!(bodies.lock().unwrap().len(), 1);
    assert_eq!(
        std::fs::read_to_string(&last).unwrap(),
        format!("{verdict}\n")
    );
    cfg.last_message = None;

    // Five empty replies: each one is tried again, and the fifth is the final reply.
    // The test is slow (about 30s): the retries wait 2s, 4s, 8s and 16s before the
    // next try, and the deadline has to be far enough ahead for all of them.
    let (url, bodies) = serve(vec![(200, say("")); 5]);
    cfg.base_url = url;
    cfg.deadline = Some(elapsed() + 120);
    let mut m = vec![json!({"role": "user", "content": "fourteen"})];
    run_loop(&cfg, &mut m).unwrap();
    assert_eq!(bodies.lock().unwrap().len(), 5);
    assert_eq!(m.last().unwrap()["role"], "assistant");
    assert_eq!(m.last().unwrap()["content"], "");

    // Empty replies that the deadline cuts short: the run stops with the deadline
    // error after the first retry instead of waiting for the fifth try.
    let (url, bodies) = serve(vec![(200, say("")); 5]);
    // The replies the deadline cut short are never sent: the guard wakes the server
    // when the test ends, so its thread does not stay waiting for them.
    let _wake = WakeOnEnd::new(&url);
    // The deadline counts whole seconds from the start of the run, so a deadline set
    // just before a second turns over would leave the retry a fraction of a second.
    // Wait for the turn, so the retry has a full second of the three.
    let millis = u64::from(START.get_or_init(Instant::now).elapsed().subsec_millis());
    if millis > 50 {
        thread::sleep(Duration::from_millis(1000 - millis));
    }
    cfg.base_url = url;
    cfg.deadline = Some(elapsed() + 3);
    let mut m = vec![json!({"role": "user", "content": "fifteen"})];
    let start = Instant::now();
    let e = run_loop(&cfg, &mut m).unwrap_err();
    assert!(e.starts_with("deadline of"), "{e}");
    assert!(e.contains("no reply"), "{e}");
    assert!(start.elapsed() < Duration::from_secs(10));
    assert!(bodies.lock().unwrap().len() <= 2);

    // The key, the path to it and the URL (which can hold user:token) must not
    // reach the model's commands. var_os keeps a value that is not valid UTF-8,
    // so restoring it afterwards does not drop it.
    let old_key = env::var_os("RESTLOOP_API_KEY");
    let old_key_file = env::var_os("RESTLOOP_API_KEY_FILE");
    let old_url = env::var_os("RESTLOOP_BASE_URL");
    let old_mixed = env::var_os(MIXED_CASE);
    unsafe {
        env::set_var("RESTLOOP_API_KEY", "test-key-not-for-commands");
        env::set_var("RESTLOOP_API_KEY_FILE", "/nonexistent/key-file");
        env::set_var("RESTLOOP_BASE_URL", "http://user:token@127.0.0.1:1/v1");
        env::set_var(MIXED_CASE, "mixed-case-restloop-name");
    }
    // Without Git Bash the commands run in PowerShell, where `env` is not a
    // command: ask for the environment there in the same NAME=VALUE form.
    // Only the RESTLOOP_ lines are printed: the whole environment can be longer
    // than the output limit, which would cut a leaked line out of the middle of
    // the output. grep exits 1 when it finds nothing, so `|| true` keeps the
    // command's exit code at 0.
    // Windows reads environment names without regard to case, so the grep is
    // case-insensitive there: a leaked Restloop_... line must be caught too.
    // Elsewhere names are exact and the mixed-case name is not restloop's.
    let env_cmd = if shell_path().is_some() {
        if cfg!(windows) {
            "env | grep -i '^RESTLOOP_' || true; echo marker-ok"
        } else {
            "env | grep '^RESTLOOP_' || true; echo marker-ok"
        }
    } else {
        "Get-ChildItem Env: | Where-Object { $_.Name -like 'RESTLOOP_*' } | ForEach-Object { \"$($_.Name)=$($_.Value)\" }; 'marker-ok'"
    };
    let out = shell(env_cmd, 10);
    // The command must have run: a shell that failed to start, or an `env` that
    // is not there, prints an error that would pass every check below.
    assert!(out.starts_with("exit: 0\n"), "{out}");
    assert!(out.contains("marker-ok"), "{out}");
    assert!(!out.lines().any(|l| l.starts_with("RESTLOOP_")), "{out}");
    assert!(!out.contains("test-key-not-for-commands"), "{out}");
    // Windows reads environment names without regard to case, so the spelling
    // Restloop_... names one of restloop's settings only there; anywhere else it
    // is a different variable that restloop never reads.
    if cfg!(windows) {
        assert!(!out.contains("mixed-case-restloop-name"), "{out}");
        assert!(
            !out.lines()
                .any(|l| l.to_ascii_uppercase().starts_with("RESTLOOP_")),
            "{out}"
        );
    }
    unsafe {
        match old_key {
            Some(k) => env::set_var("RESTLOOP_API_KEY", k),
            None => env::remove_var("RESTLOOP_API_KEY"),
        }
        match old_key_file {
            Some(k) => env::set_var("RESTLOOP_API_KEY_FILE", k),
            None => env::remove_var("RESTLOOP_API_KEY_FILE"),
        }
        match old_url {
            Some(u) => env::set_var("RESTLOOP_BASE_URL", u),
            None => env::remove_var("RESTLOOP_BASE_URL"),
        }
        match old_mixed {
            Some(m) => env::set_var(MIXED_CASE, m),
            None => env::remove_var(MIXED_CASE),
        }
    }
    let _ = std::fs::remove_file(&last);
}

// Windows matches environment names without regard to case, so the prefix that is
// stripped from a command's environment is matched the way each platform reads it:
// a key spelled Restloop_Api_Key would reach restloop on Windows and must go too.
#[test]
fn restloop_settings_are_matched_the_way_the_platform_reads_names() {
    assert!(restloop_var(OsStr::new("RESTLOOP_API_KEY")));
    assert!(restloop_var(OsStr::new("RESTLOOP_")));
    assert_eq!(
        restloop_var(OsStr::new("Restloop_Mixed_Case")),
        cfg!(windows)
    );
    assert_eq!(restloop_var(OsStr::new("restloop_api_key")), cfg!(windows));
    assert!(!restloop_var(OsStr::new("RESTLOOP")));
    // Any name with the prefix goes, known or not: a future setting must not leak.
    assert!(restloop_var(OsStr::new("RESTLOOP_ANYTHING")));
    assert!(!restloop_var(OsStr::new("MY_RESTLOOP_API_KEY")));
    assert!(!restloop_var(OsStr::new("XRESTLOOP_API_KEY")));
    assert!(!restloop_var(OsStr::new("")));
}

/// The core needs no shell: a tool that records the commands it is given and answers
/// with a fixed string is enough.
#[test]
fn the_core_runs_with_a_tool_of_its_own() {
    let call = json!({"choices": [{"message": {"role": "assistant", "content": "",
        "tool_calls": [{"id": "a", "type": "function", "function": {"name": "shell",
            "arguments": "{\"command\":\"echo hi\"}"}}]}}]});
    let done = json!({"choices": [{"message": {"role": "assistant", "content": "all done"}}]});
    let (url, bodies) = serve(vec![(200, call), (200, done)]);
    let mut cfg = test_config();
    cfg.base_url = url;
    let mut commands: Vec<String> = Vec::new();
    let mut messages = vec![json!({"role": "user", "content": "go"})];
    run(&cfg, &mut messages, &mut |cmd, _secs| {
        commands.push(cmd.to_string());
        "exit: 0\nfixed output".to_string()
    })
    .unwrap();
    // The tool got the command the model asked for, and what it said went back to it.
    assert_eq!(commands, ["echo hi"]);
    assert_eq!(messages[2]["content"], "exit: 0\nfixed output");
    assert_eq!(messages[3]["content"], "all done");
    assert_eq!(bodies.lock().unwrap().len(), 2);
}

/// A task whose time is up stops at once: no command runs, and the run says why.
#[test]
fn a_run_whose_deadline_has_passed_runs_no_command() {
    // The task started two seconds ago and had one second for itself: it is over.
    // The clock is asked before the server starts: a clock that cannot go back that
    // far leaves nothing to test, and then no server is left waiting for a request.
    let Some(past) = Instant::now().checked_sub(Duration::from_secs(2)) else {
        return; // the clock cannot go back that far: there is nothing to test.
    };
    let done = json!({"choices": [{"message": {"role": "assistant", "content": "all done"}}]});
    let (url, bodies, ended) = serve_until(vec![(200, done)]);
    // The server waits for a request that never comes: the guard wakes it when the
    // test ends, so a failed assertion cannot leave its thread waiting.
    let wake = WakeOnEnd::new(&url);
    let mut cfg = test_config();
    cfg.base_url = url.clone();
    cfg.start = past;
    cfg.deadline = Some(1);
    let mut commands: Vec<String> = Vec::new();
    let mut messages = vec![json!({"role": "user", "content": "go"})];
    let start = Instant::now();
    let e = run(&cfg, &mut messages, &mut |cmd, _secs| {
        commands.push(cmd.to_string());
        "exit: 0\nfixed output".to_string()
    })
    .unwrap_err();
    assert!(e.starts_with("deadline of"), "{e}");
    // The tool never ran, and the run stopped at once.
    assert!(commands.is_empty());
    assert!(start.elapsed() < Duration::from_secs(2));
    // The server got no request at all.
    assert_eq!(bodies.lock().unwrap().len(), 0);
    // Its thread is not left waiting for a request that never comes: the guard
    // wakes it, whether the test ends well or fails.
    drop(wake);
    assert!(
        ended.recv_timeout(Duration::from_secs(5)).is_ok(),
        "the server thread is still waiting for a request"
    );
}

/// A key that fails ends the run at once: it is nobody's model fault, so the
/// conversation is not shortened for it and no request is sent.
#[test]
fn a_key_that_fails_ends_the_run_at_once() {
    let done = json!({"choices": [{"message": {"role": "assistant", "content": "all done"}}]});
    let (url, bodies) = serve(vec![(200, done)]);
    // The server waits for a request that never comes: the guard wakes it when the
    // test ends, so a failed assertion cannot leave its thread waiting.
    let _wake = WakeOnEnd::new(&url);
    let mut cfg = test_config();
    cfg.base_url = url.clone();
    let calls = Arc::new(Mutex::new(0));
    let seen = calls.clone();
    cfg.key = Box::new(move || {
        *seen.lock().unwrap() += 1;
        Err("maximum context length exceeded".into())
    });
    let mut messages = vec![
        json!({"role": "user", "content": "go"}),
        json!({"role": "tool", "tool_call_id": "a", "content": "exit: 0\nlong output"}),
    ];
    let before = messages.clone();
    let e = run_loop(&cfg, &mut messages).unwrap_err();
    assert_eq!(e, "maximum context length exceeded");
    // The key was asked once, the server got no request, and the conversation is as it was.
    assert_eq!(*calls.lock().unwrap(), 1);
    assert_eq!(bodies.lock().unwrap().len(), 0);
    assert_eq!(messages, before);
}

/// Even a key that fails with the words of a full context is the key's fault alone:
/// nothing is shortened, no request is sent and the key is asked once.
#[test]
fn a_key_that_fails_like_a_full_context_is_not_recovered_from() {
    let done = json!({"choices": [{"message": {"role": "assistant", "content": "all done"}}]});
    let (url, bodies) = serve(vec![(200, done)]);
    // The server waits for a request that never comes: the guard wakes it when the
    // test ends, so a failed assertion cannot leave its thread waiting.
    let _wake = WakeOnEnd::new(&url);
    let mut cfg = test_config();
    cfg.base_url = url.clone();
    let calls = Arc::new(Mutex::new(0));
    let seen = calls.clone();
    cfg.key = Box::new(move || {
        *seen.lock().unwrap() += 1;
        Err("HTTP 400: maximum context length exceeded".into())
    });
    let mut messages = vec![
        json!({"role": "user", "content": "go"}),
        json!({"role": "tool", "tool_call_id": "a", "content": "exit: 0\nlong output"}),
    ];
    let before = messages.clone();
    let e = run_loop(&cfg, &mut messages).unwrap_err();
    assert_eq!(e, "HTTP 400: maximum context length exceeded");
    assert_eq!(*calls.lock().unwrap(), 1);
    assert_eq!(bodies.lock().unwrap().len(), 0);
    assert_eq!(messages, before);
}
