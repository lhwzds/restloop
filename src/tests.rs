//! Tests: the helpers, the shell runner on this platform, and whole runs against a
//! small local server standing in for the model.

use super::*;
use std::io::{BufReader, Write};
use std::net::TcpListener;

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
    // Of the Responses API's output items only the reasoning is dropped.
    let items = json!([{"type": "reasoning"}, {"type": "message", "phase": "commentary"}]);
    let mut m = vec![json!({"role": "assistant", "content": "", "output_items": items})];
    assert_eq!(trim_thinking(&mut m, 0), 1);
    assert_eq!(m[0]["output_items"], json!([items[1]]));
    assert_eq!(trim_thinking(&mut m, 0), 0);
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
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let bodies = Arc::new(Mutex::new(Vec::new()));
    let seen = bodies.clone();
    thread::spawn(move || {
        for ((status, reply), stream) in replies.into_iter().zip(listener.incoming()) {
            let mut stream = stream.unwrap();
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
    });
    (url, bodies)
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
    // Settings from the shell running the tests must not reach this run: with
    // RESTLOOP_LAST_MESSAGE it would write to that file, with HTTP_PROXY and the like
    // its requests would miss the local server. Windows ignores the case of names.
    unsafe {
        for (name, _) in env::vars_os() {
            let upper = name.to_string_lossy().to_ascii_uppercase();
            if upper.starts_with("RESTLOOP_") || upper.ends_with("_PROXY") {
                env::remove_var(name);
            }
        }
        env::set_var("RESTLOOP_BASE_URL", &url);
        env::set_var("RESTLOOP_SESSION", &session);
    }

    let mut messages = conversation();
    messages.push(json!({"role": "user", "content": "one"}));
    run("m", &mut messages).unwrap();
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
    let mut resumed = conversation();
    assert_eq!(resumed, messages);
    resumed.push(json!({"role": "user", "content": "two"}));
    run("m", &mut resumed).unwrap();
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
    unsafe {
        env::set_var("RESTLOOP_BASE_URL", &url);
        env::remove_var("RESTLOOP_SESSION");
    }
    let mut m = vec![json!({"role": "user", "content": "three"})];
    run("m", &mut m).unwrap();
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
    unsafe {
        env::set_var("RESTLOOP_BASE_URL", &url);
        env::set_var("RESTLOOP_WIRE_API", "responses");
        env::set_var("RESTLOOP_API_KEY_FILE", &key);
        env::set_var("RESTLOOP_REASONING_EFFORT", "low");
    }
    let mut m = vec![
        json!({"role": "system", "content": "be brief"}),
        json!({"role": "user", "content": "four"}),
    ];
    run("m", &mut m).unwrap();
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
        unsafe { env::set_var("RESTLOOP_BASE_URL", &url) };
        let mut m = vec![json!({"role": "user", "content": "five"})];
        let e = run("m", &mut m).unwrap_err();
        assert!(e.starts_with(start), "{e}");
        assert_eq!(bodies.lock().unwrap().len(), 1);
    }

    // A key file that cannot be read stops the run before any request.
    std::fs::remove_file(&key).unwrap();
    let mut m = vec![json!({"role": "user", "content": "six"})];
    let e = run("m", &mut m).unwrap_err();
    assert!(e.starts_with("cannot read "), "{e}");

    unsafe {
        env::remove_var("RESTLOOP_WIRE_API");
        env::remove_var("RESTLOOP_API_KEY_FILE");
        env::remove_var("RESTLOOP_REASONING_EFFORT");
    }
    let (url, bodies) = serve(vec![(0, Value::Null)]);
    unsafe {
        env::set_var("RESTLOOP_BASE_URL", &url);
        env::set_var("RESTLOOP_DEADLINE", (elapsed() + 3).to_string());
    }
    let mut m = vec![json!({"role": "user", "content": "seven"})];
    let start = Instant::now();
    let e = run("m", &mut m).unwrap_err();
    assert!(e.starts_with("deadline of"), "{e}");
    assert!(start.elapsed() < Duration::from_secs(10));
    assert_eq!(bodies.lock().unwrap().len(), 1);

    let (url, bodies) = serve(vec![(500, json!({"error": "server error"})); 5]);
    unsafe {
        env::set_var("RESTLOOP_BASE_URL", &url);
        env::set_var("RESTLOOP_DEADLINE", (elapsed() + 3).to_string());
    }
    let mut m = vec![json!({"role": "user", "content": "eight"})];
    let start = Instant::now();
    let e = run("m", &mut m).unwrap_err();
    assert!(e.starts_with("deadline of"), "{e}");
    assert!(e.contains("HTTP 500"), "{e}");
    assert!(bodies.lock().unwrap().len() < 5);
    assert!(start.elapsed() < Duration::from_secs(10));

    let (url, bodies) = serve(vec![(400, json!({"error": "invalid"}))]);
    unsafe {
        env::set_var("RESTLOOP_BASE_URL", &url);
        env::set_var("RESTLOOP_DEADLINE", (elapsed() + 3).to_string());
    }
    let e = run("m", &mut m).unwrap_err();
    assert_eq!(e, "HTTP 400: {\"error\":\"invalid\"}");
    let d = elapsed();
    unsafe { env::set_var("RESTLOOP_DEADLINE", d.to_string()) };
    let e = run("m", &mut m).unwrap_err();
    assert_eq!(e, format!("deadline of {d}s reached"));
    assert_eq!(bodies.lock().unwrap().len(), 1);
    // A successful status whose body stalls is a request failure, not a bad JSON reply.
    let (url, bodies) = serve(vec![(1, Value::Null)]);
    unsafe {
        env::set_var("RESTLOOP_BASE_URL", &url);
        env::set_var("RESTLOOP_DEADLINE", (elapsed() + 3).to_string());
    }
    let start = Instant::now();
    let e = run("m", &mut m).unwrap_err();
    assert!(e.starts_with("deadline of"), "{e}");
    assert!(e.contains("request failed"), "{e}");
    assert!(start.elapsed() < Duration::from_secs(10));
    assert_eq!(bodies.lock().unwrap().len(), 1);
    unsafe { env::remove_var("RESTLOOP_DEADLINE") };
}
