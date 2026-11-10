//! RestLoop's core: the agent loop and the model client.
//!
//! The core makes no direct environment reads (its HTTP client, `ureq`, honours
//! `*_PROXY`) and starts no process: the model, the settings and the key come in
//! through [`Config`], the commands come from the tool the caller passes to [`run`].
//! [`shell`] holds the default command runner.
//!
//! The core holds no settings of its own: the steps and the timeout come from the
//! caller, and the CLI's defaults live in `src/main.rs`.

use serde_json::{Value, json};
use std::io::BufRead;
use std::thread;
use std::time::{Duration, Instant};

pub use serde_json;

pub mod shell;

/// Everything the loop needs: the model, the settings and the clock.
pub struct Config {
    /// The model to talk to.
    pub model: String,
    /// The API's base URL.
    pub base_url: String,
    /// The key of a request: called before every request attempt, the retries
    /// included. An error ends the run without a retry.
    pub key: Box<dyn Fn() -> Result<String, String> + Send + Sync>,
    /// The Responses API instead of chat completions.
    pub responses: bool,
    /// The reasoning effort to ask for, if any.
    pub reasoning_effort: Option<String>,
    /// The steps a task may take.
    pub max_steps: u64,
    /// The seconds a command may run.
    pub timeout: u64,
    /// The seconds for the whole task, counted from `start`.
    pub deadline: Option<u64>,
    /// When the task started: the deadline and the time used count from here.
    pub start: Instant,
    /// Whether the token usage of each model reply goes to stderr.
    pub log_usage: bool,
    /// The file that receives the final reply.
    pub last_message: Option<String>,
    /// The file the conversation is saved in and continued from.
    pub session: Option<String>,
    /// Extra HTTP headers sent with every model request.
    pub headers: Vec<(String, String)>,
}

/// Why a header is refused, or None: a name or value HTTP cannot carry, or a name
/// RestLoop or its HTTP client sets itself. Never holds the value. A header it
/// passes can still be refused by the server.
pub fn bad_header(name: &str, value: &str) -> Option<String> {
    if ureq::http::HeaderName::from_bytes(name.as_bytes()).is_err()
        || ureq::http::HeaderValue::from_str(value).is_err()
    {
        return Some("is not a valid header".into());
    }
    if [
        "authorization",
        "proxy-authorization",
        "content-type",
        "content-length",
        "host",
        "transfer-encoding",
        "accept-encoding",
    ]
    .iter()
    .any(|h| name.eq_ignore_ascii_case(h))
    {
        return Some(format!("sets {name}, which restloop sets itself"));
    }
    None
}

/// The agent loop: send the conversation to the model, run the commands it asks for
/// and send back what they printed, until it answers without a tool call.
///
/// Each reply's text goes to stdout, and each command and a short result to stderr,
/// as the CLI does.
pub fn run(
    cfg: &Config,
    messages: &mut Vec<Value>,
    tool: &mut dyn FnMut(&str, u64) -> String,
) -> Result<(), String> {
    // A header the config cannot carry ends the run before anything else happens.
    for (n, (name, value)) in cfg.headers.iter().enumerate() {
        if let Some(reason) = bad_header(name, value) {
            return Err(format!("header {} of the config {reason}", n + 1));
        }
    }
    let tools = json!([{"type": "function", "function": {
        "name": "shell",
        "description": "Run a shell command and return its exit code and output.",
        "parameters": {"type": "object", "properties": {"command": {"type": "string"}}, "required": ["command"]}
    }}]);
    // The final reply goes to the file of RESTLOOP_LAST_MESSAGE. Remove an old one first
    // so a run that stops early cannot leave the previous run's answer behind.
    if let Some(path) = &cfg.last_message {
        let _ = std::fs::remove_file(path);
    }
    for step in 1..=cfg.max_steps {
        let reply = loop {
            let body = json!({"model": cfg.model, "messages": messages, "tools": tools});
            match complete(cfg, &body) {
                // Past the model's context window: shorten old command output and retry.
                // A key that fails is nobody's model fault: it ends the run at once.
                Err(Failure::Model(e)) if too_long(&e) && drop_old_output(messages) > 0 => {
                    eprintln!("restloop: context too long, dropped old command output")
                }
                r => break r.map_err(|f| f.message())?,
            }
        };
        if cfg.log_usage {
            let u = &reply["usage"];
            eprintln!(
                "usage: step={step} prompt_tokens={} completion_tokens={} cached_tokens={} elapsed={}s",
                u["prompt_tokens"],
                u["completion_tokens"],
                u["prompt_tokens_details"]["cached_tokens"],
                cfg.start.elapsed().as_secs()
            );
        }
        let msg = &reply["choices"][0]["message"];
        let text = msg["content"].as_str().unwrap_or("");
        if !text.trim().is_empty() {
            println!("{}", text.trim());
        }
        let calls = msg["tool_calls"].as_array().cloned().unwrap_or_default();
        let mut keep = json!({"role": "assistant", "content": text});
        // output_items: the Responses API's output, opaque reasoning included, sent back as it came.
        for field in ["reasoning_content", "output_items"] {
            if let Some(r) = msg.get(field).filter(|r| !r.is_null()) {
                keep[field] = r.clone();
            }
        }
        if !calls.is_empty() {
            keep["tool_calls"] = json!(calls);
        }
        messages.push(keep);
        if calls.is_empty() {
            save(cfg, messages);
            if let Some(path) = &cfg.last_message {
                let final_reply = answer(text).trim();
                if let Err(e) = std::fs::write(path, format!("{final_reply}\n")) {
                    eprintln!("restloop: cannot write {path}: {e}");
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
                    let left = cfg.deadline.map_or(u64::MAX, |d| {
                        d.saturating_sub(cfg.start.elapsed().as_secs()).max(1)
                    });
                    let before = cfg.start.elapsed();
                    let mut out = tool(cmd, cfg.timeout.min(left));
                    eprintln!(
                        "\x1b[2m[{:.1}s] {}\x1b[0m",
                        cfg.start.elapsed().saturating_sub(before).as_secs_f64(),
                        out.lines().take(10).collect::<Vec<_>>().join("\n")
                    );
                    if let Some(d) = cfg.deadline {
                        out.push_str(&format!(
                            "\n[time used: {}s of {d}s]",
                            cfg.start.elapsed().as_secs()
                        ));
                    }
                    out
                }
                None => {
                    format!("error: arguments must be JSON with a \"command\" string, got: {args}")
                }
            };
            messages.push(json!({"role": "tool", "tool_call_id": call["id"], "content": output}));
        }
        save(cfg, messages);
    }
    Err(format!("stopped after {} steps", cfg.max_steps))
}

/// Writes the conversation, exactly as the next request will send it, to the session
/// file. A temporary file and a rename mean a killed run leaves its last complete
/// step; the process id in its name keeps two runs on one file from writing into the
/// same one.
pub(crate) fn save(cfg: &Config, messages: &[Value]) {
    let Some(path) = cfg.session.as_deref() else {
        return;
    };
    let tmp = format!("{path}.{}.tmp", std::process::id());
    let doc = json!({"version": 1, "messages": messages});
    if let Err(e) = std::fs::write(&tmp, doc.to_string()).and_then(|_| std::fs::rename(&tmp, path))
    {
        eprintln!("restloop: cannot save {path}: {e}");
    }
}

/// The system prompt: what the model may do, where it is, and how much time it has.
/// The name of the shell the commands run in, the operating system and the working
/// directory are the caller's to say.
pub fn system_prompt(cfg: &Config, shell_name: &str, os: &str, cwd: &str) -> String {
    format!(
        "You are a coding agent. Use the shell tool to inspect files, edit them and run commands until the task is done, then reply with a short summary and no tool call.\nOS: {os}. Shell: {shell_name}. Working directory: {cwd}.\nCommands run non-interactively with no stdin and a {}s timeout; long output is truncated.{}",
        cfg.timeout,
        cfg.deadline.map_or(String::new(), |d| format!(
            "\nYou have {d}s in total for this task; each tool result ends with the time used so far."
        ))
    )
}

/// Removes the reasoning from all assistant messages but the last `keep`; returns how many changed.
/// `reasoning_content` is emptied, not removed: DeepSeek rejects a tool turn in which an
/// assistant message has lost the field.
pub(crate) fn trim_thinking(messages: &mut [Value], keep: usize) -> usize {
    let mut changed = 0;
    for m in messages
        .iter_mut()
        .rev()
        .filter(|m| m["role"] == "assistant")
        .skip(keep)
    {
        let had = m
            .get_mut("reasoning_content")
            .filter(|r| r.as_str() != Some(""))
            .map(|r| *r = json!(""));
        // Of the Responses API's output, only the reasoning goes: messages keep their phase.
        let items = m.get_mut("output_items").and_then(Value::as_array_mut);
        let had = had.or(items
            .filter(|o| o.iter().any(|i| i["type"] == "reasoning"))
            .map(|o| o.retain(|i| i["type"] != "reasoning")));
        let content = m["content"].as_str().unwrap_or("");
        let a = answer(content);
        let stripped = a != content;
        if stripped {
            m["content"] = json!(a.trim());
        }
        changed += usize::from(had.is_some() || stripped);
    }
    changed
}

/// Whether an API error says the conversation no longer fits the model's context window,
/// in the words providers use ("maximum context length is …" from OpenAI, DeepSeek and
/// vLLM; "context window exceeds limit" from MiniMax; "prompt is too long" from Anthropic;
/// "exceeds the available context size" from llama.cpp).
pub(crate) fn too_long(err: &str) -> bool {
    let e = err.to_lowercase();
    (e.starts_with("http 400") || e.starts_with("http 413") || e.starts_with("no reply: "))
        && [
            "context_length_exceeded",
            "maximum context length",
            "context window exceeds",
            "prompt is too long",
            "exceeds the available context size",
        ]
        .iter()
        .any(|k| e.contains(k))
}

/// Cuts the older half of the command outputs down to their first line and drops old
/// reasoning; returns how many messages changed (0: nothing left to shorten).
pub(crate) fn drop_old_output(messages: &mut [Value]) -> usize {
    const NOTE: &str = "[output removed to fit the context window]";
    let old: Vec<usize> = (0..messages.len())
        .filter(|&i| messages[i]["role"] == "tool")
        .filter(|&i| {
            let c = messages[i]["content"].as_str().unwrap_or("");
            !c.ends_with(NOTE)
        })
        .collect();
    let old = &old[..old.len().div_ceil(2)];
    for &i in old {
        let text = messages[i]["content"].as_str().unwrap_or("");
        let exit = text.lines().next().unwrap_or("");
        messages[i]["content"] = json!(format!("{exit}\n{NOTE}"));
    }
    old.len() + trim_thinking(messages, 2)
}

/// Some models put their reasoning in a leading `<think>` block inside the reply: the
/// answer is what follows the first `</think>` after it.
pub(crate) fn answer(text: &str) -> &str {
    let t = text.trim_start();
    if !t.starts_with("<think>") {
        return text;
    }
    t.split_once("</think>").map_or("", |(_, a)| a)
}

/// A reply with no tool call and no text is a failed request, not an answer.
pub(crate) fn empty(reply: &Value) -> bool {
    let msg = &reply["choices"][0]["message"];
    let text = msg["content"].as_str().unwrap_or("");
    msg["tool_calls"].as_array().is_none_or(|c| c.is_empty()) && answer(text).trim().is_empty()
}

/// Why a request failed: the key of the request or the model itself. A key that
/// fails is not the model's fault and never starts the recovery from a full context.
enum Failure {
    Key(String),
    Model(String),
}

impl Failure {
    /// The message the run ends with: the words of the failure itself, unchanged.
    fn message(self) -> String {
        match self {
            Failure::Key(e) | Failure::Model(e) => e,
        }
    }
}

/// One request to the model, with the retries a failing request gets.
fn complete(cfg: &Config, body: &Value) -> Result<Value, Failure> {
    let base = &cfg.base_url;
    let responses = cfg.responses;
    let (path, mut body) = match responses {
        true => ("responses", responses_request(body)),
        false => ("chat/completions", body.clone()),
    };
    if let Some(e) = &cfg.reasoning_effort {
        match responses {
            true => body["reasoning"] = json!({"effort": e}),
            false => body["reasoning_effort"] = json!(e),
        }
    }
    let url = format!("{}/{path}", base.trim_end_matches('/'));
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        // A redirect would carry the extra headers to another host.
        .max_redirects(0)
        .build()
        .into();
    for attempt in 0..5 {
        let mut empty_reply: Option<Value> = None;
        // The key is read for every request, so another program can replace a
        // short-lived token while a long task runs.
        let key = (cfg.key)().map_err(Failure::Key)?;
        // A key a header cannot carry ends the run as a key failure: the error says
        // why, never the key itself.
        if ureq::http::HeaderValue::from_str(&format!("Bearer {}", key.trim())).is_err() {
            return Err(Failure::Key(
                "the key cannot go in a header: it holds a line break or another character HTTP cannot carry"
                    .into(),
            ));
        }
        let limit = cfg.deadline;
        let left = limit.map_or(Duration::from_secs(600), |d| {
            Duration::from_secs(d)
                .saturating_sub(cfg.start.elapsed())
                .min(Duration::from_secs(600))
        });
        if left.is_zero() {
            return Err(Failure::Model(format!(
                "deadline of {}s reached",
                limit.unwrap()
            )));
        }
        let mut request = agent
            .post(&url)
            .config()
            .timeout_global(Some(left))
            .build()
            .header("Authorization", &format!("Bearer {}", key.trim()));
        // The headers of the caller go with every request: some models ask for
        // more than the key (a session id, a version).
        for (name, value) in &cfg.headers {
            request = request.header(name.as_str(), value.as_str());
        }
        let resp = request.send_json(&body);
        let (status, text) = match resp {
            Ok(mut r) if r.status() == 200 && responses => {
                match responses_reply(std::io::BufReader::new(r.body_mut().as_reader())) {
                    Ok(reply) if empty(&reply) => {
                        empty_reply = Some(reply);
                        (200, "empty reply".into())
                    }
                    Ok(reply) => return Ok(reply),
                    Err(e) => (200, e),
                }
            }
            Ok(mut r) => {
                let status = r.status().as_u16();
                match r.body_mut().read_to_string() {
                    Ok(text) => (status, text),
                    Err(e) => (0, e.to_string()),
                }
            }
            Err(e) => (0, e.to_string()),
        };
        if status == 200 && !responses {
            let reply: Value = serde_json::from_str(&text)
                .map_err(|e| Failure::Model(format!("bad response: {e}: {text}")))?;
            if reply["choices"][0]["message"].is_object() {
                if !empty(&reply) {
                    return Ok(reply);
                }
                empty_reply = Some(reply);
            }
            // MiniMax answers some errors with HTTP 200 and no choices; that is not a reply.
        }
        let what = match status {
            0 => "request failed".to_string(),
            200 => "no reply".to_string(),
            _ => format!("HTTP {status}"),
        };
        // Used-up credits or plan limits do not come back within the retries, nor does a
        // failure the stream reports for another reason than the server or a limit (a
        // refused or cut-off answer, a request too long for the context). An empty reply
        // is tried again all the same: it says nothing, so a limit named around it is
        // not the reason the request failed.
        let spent = empty_reply.is_none()
            && (text.contains("insufficient_quota")
                || text.contains("subscription_sharing_usage_limit_exceeded"));
        let failed = responses
            && status == 200
            && !(text.starts_with("the event stream") || text == "empty reply")
            && !["server_error", "rate_limit", "unavailable"]
                .iter()
                .any(|k| text.contains(k));
        if !(status == 0 || status == 200 || status == 429 || status >= 500) || spent || failed {
            return Err(Failure::Model(format!("{what}: {text}")));
        }
        // The last attempt has no retry to sleep for: only a deadline already passed counts.
        let sleep = if attempt == 4 { 0 } else { 2 << attempt };
        if let Some(d) = limit
            && cfg
                .start
                .elapsed()
                .saturating_add(Duration::from_secs(sleep))
                >= Duration::from_secs(d)
        {
            return Err(Failure::Model(format!(
                "deadline of {d}s reached ({what}: {text})"
            )));
        }
        if attempt == 4 {
            return empty_reply.map_or_else(|| Err(Failure::Model(format!("{what}: {text}"))), Ok);
        }
        eprintln!("restloop: {what}, retrying");
        thread::sleep(Duration::from_secs(sleep));
    }
    unreachable!()
}

/// A chat-completions request as a Responses API request. The first system message
/// becomes the instructions; an assistant message goes back as its `output_items` (the
/// model's output as it came, reasoning included) or, without them, as its text and calls.
pub(crate) fn responses_request(body: &Value) -> Value {
    let mut instructions = Value::Null;
    let mut input = Vec::new();
    for m in body["messages"].as_array().into_iter().flatten() {
        match m["role"].as_str().unwrap_or("") {
            "system" if instructions.is_null() && input.is_empty() => {
                instructions = m["content"].clone()
            }
            "assistant" if m["output_items"].is_array() => {
                input.extend(m["output_items"].as_array().into_iter().flatten().cloned())
            }
            "assistant" => {
                if m["content"].as_str().is_some_and(|t| !t.is_empty()) {
                    input.push(json!({"role": "assistant", "content": m["content"]}));
                }
                for c in m["tool_calls"].as_array().into_iter().flatten() {
                    input.push(json!({"type": "function_call", "call_id": c["id"],
                        "name": c["function"]["name"], "arguments": c["function"]["arguments"]}));
                }
            }
            "tool" => input.push(json!({"type": "function_call_output",
                "call_id": m["tool_call_id"], "output": m["content"]})),
            _ => input.push(json!({"role": m["role"], "content": m["content"]})),
        }
    }
    let tools: Vec<Value> = body["tools"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|t| {
            let mut f = t["function"].clone();
            f["type"] = json!("function");
            f
        })
        .collect();
    // One key per system prompt (it names the working directory) keeps a conversation's
    // requests on the same cache.
    let mut h = std::hash::DefaultHasher::new();
    std::hash::Hash::hash(&instructions.to_string(), &mut h);
    let mut r = json!({"model": body["model"], "input": input, "tools": tools, "store": false,
        "stream": true, "include": ["reasoning.encrypted_content"],
        "prompt_cache_key": format!("restloop-{:016x}", std::hash::Hasher::finish(&h))});
    if !instructions.is_null() {
        r["instructions"] = instructions;
    }
    r
}

/// Reads a Responses API event stream into a chat-completions reply.
pub(crate) fn responses_reply(stream: impl BufRead) -> Result<Value, String> {
    let mut items = Vec::new();
    for line in stream.lines() {
        let line = line.map_err(|e| format!("the event stream broke: {e}"))?;
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let Ok(event) = serde_json::from_str::<Value>(data.trim()) else {
            continue;
        };
        let kind = event["type"].as_str().unwrap_or("");
        match kind {
            "response.output_item.done" => items.push((
                event["output_index"].as_u64().unwrap_or(u64::MAX),
                event["item"].clone(),
            )),
            "response.completed" => {
                let r = &event["response"];
                // Items finish in any order; output_index (or the final output) has theirs.
                items.sort_by_key(|(i, _)| *i);
                let mut items: Vec<Value> = match r["output"].as_array() {
                    Some(o) if !o.is_empty() => o.clone(),
                    _ => items.into_iter().map(|(_, i)| i).collect(),
                };
                let (mut text, mut calls) = (String::new(), vec![]);
                for i in &mut items {
                    match i["type"].as_str().unwrap_or("") {
                        "message" => {
                            for c in i["content"].as_array().into_iter().flatten() {
                                text += c["text"].as_str().or(c["refusal"].as_str()).unwrap_or("");
                            }
                        }
                        "function_call" => calls
                            .push(json!({"id": i["call_id"], "type": "function",
                            "function": {"name": i["name"], "arguments": i["arguments"]}})),
                        _ => continue,
                    }
                    // Without `store`, only reasoning items may keep their ids (as Codex sends them).
                    i.as_object_mut().map(|o| o.remove("id"));
                }
                let mut msg = json!({"role": "assistant", "content": text, "output_items": items});
                if !calls.is_empty() {
                    msg["tool_calls"] = json!(calls);
                }
                let u = &r["usage"];
                return Ok(json!({"choices": [{"message": msg}], "usage": {
                    "prompt_tokens": u["input_tokens"], "completion_tokens": u["output_tokens"],
                    "prompt_tokens_details": {"cached_tokens": u["input_tokens_details"]["cached_tokens"]}}}));
            }
            "response.failed" | "response.incomplete" | "error" => {
                let r = &event["response"];
                let detail = [&r["error"], &r["incomplete_details"]]
                    .into_iter()
                    .find(|d| !d.is_null())
                    .unwrap_or(&event);
                return Err(format!("{kind}: {detail}"));
            }
            _ => {}
        }
    }
    Err("the event stream ended without a response".into())
}

#[cfg(test)]
mod tests;
