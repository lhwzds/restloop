---
title: How it works
---

# How it works

<sub>**English** · [中文](zh/03-how-it-works.md)</sub>

Everything happens in `run()`: one iteration (a step) is one model reply plus the commands it requested.

```mermaid
flowchart TD
  A[messages + shell tool] --> B["complete(): POST /chat/completions or /responses"]
  B -->|"context overflow"| C["drop_old_output()"] --> B
  B --> D[append assistant message, print its text]
  D -->|no tool_calls| E[save session, write last message, return Ok]
  D -->|tool_calls| F["shell() each command in order"]
  F --> G[append one tool message per call; save session]
  G -->|step < RESTLOOP_MAX_STEPS| B
  G -->|limit reached| H["Err: stopped after N steps"]
```

<sub>Sources: [main.rs:124-129](../src/main.rs#L124-L129) · [main.rs:138-148](../src/main.rs#L138-L148) · [main.rs:213-215](../src/main.rs#L213-L215)</sub>

## What the conversation holds

The system prompt from `system_prompt()` comes first: instructions plus OS, shell, working directory, command timeout and, with a deadline, the budget. Then come user tasks and, per model reply, an assistant message keeping only `content`, `tool_calls` and, when present, `reasoning_content` and the Responses API's `output_items`.

Each tool call gets one `tool` message with the matching `tool_call_id`. Arguments that are not JSON with a `command` string get an error quoting them, so the model can fix them.

<sub>Sources: [main.rs:83-100](../src/main.rs#L83-L100) · [main.rs:159-175](../src/main.rs#L159-L175) · [main.rs:186-211](../src/main.rs#L186-L211)</sub>

## Requests and retries

`complete()` posts `{"model", "messages", "tools"}` to `/chat/completions`, plus `reasoning_effort` if set, without sampling parameters, token limit or streaming. The key is reread per attempt ([Usage and configuration](02-usage.md)); an unreadable key file fails with `cannot read <path>`.

Network errors (unreadable bodies included), HTTP 429, 5xx, a 200 without `choices[0].message` (some MiniMax errors, `333b7b64`) and an empty reply (no tool call, no text outside a leading `<think>`) are retried: five attempts, sleeping 2, 4, 8, 16 s, printing `restloop: <what>, retrying`; a fifth empty reply is the final one. A body with `insufficient_quota` or `subscription_sharing_usage_limit_exceeded` fails at once, unless the reply is empty. Other statuses, and the fifth failure, fail the task with `<what>: <text>` (`HTTP <status>`, `request failed` or `no reply`), or the deadline form below. A non-JSON 200 fails at once with `bad response`.

<sub>Sources: [main.rs:314-423](../src/main.rs#L314-L423) · [commit 333b7b64](https://github.com/lhwzds/restloop/commit/333b7b64)</sub>

## The Responses API

With `RESTLOOP_WIRE_API=responses`, `complete()` posts to `/responses`. The conversation stays in chat format, in memory and on disk; `responses_request()` converts it per request: a leading system message becomes `instructions`, a `tool` message a `function_call_output`, and an assistant message its `output_items` or, lacking them, its text and `function_call` items. It adds `store: false`, `stream: true`, encrypted reasoning in `include`, a `prompt_cache_key` hashed from the system prompt and, if set, `reasoning.effort`.

`responses_reply()` reads the event stream until `response.completed` and builds a chat reply: text (or refusal), tool calls, usage under chat names, and `output_items`, the output in `output_index` order minus message and call ids. These go back verbatim, so reasoning survives without server-side storage. A stream that breaks or ends early is retried; a reported failure (`response.failed`, `response.incomplete`, `error`) only for `server_error`, `rate_limit` or `unavailable`.

<sub>Sources: [main.rs:318-333](../src/main.rs#L318-L333) · [main.rs:356-361](../src/main.rs#L356-L361) · [main.rs:384-398](../src/main.rs#L384-L398) · [main.rs:437-550](../src/main.rs#L437-L550)</sub>

## Recovering from a full context window

Since `21f3c1dc`, `too_long()` recognises a context overflow: an HTTP 400 or 413, or a Responses stream failure (`no reply: `), whose text contains a provider phrase (`context_length_exceeded`, "maximum context length", "context window exceeds", "prompt is too long", "exceeds the available context size"). `04004c68` dropped looser words like "too long" after an overlong `tool_call_id` triggered history cuts.

On a match, `drop_old_output()` cuts the older half (rounded up) of the `tool` messages not yet shortened to their first line plus `[output removed to fit the context window]`, empties old reasoning with `trim_thinking()`, and the request is retried. Each retry halves the rest; when nothing changes, the error is returned. Commands and visible replies stay, so the model knows what it did.

<sub>Sources: [main.rs:139-148](../src/main.rs#L139-L148) · [main.rs:269-312](../src/main.rs#L269-L312) · [commit 21f3c1dc](https://github.com/lhwzds/restloop/commit/21f3c1dc) · [commit 04004c68](https://github.com/lhwzds/restloop/commit/04004c68)</sub>

## Emptying old reasoning

`trim_thinking()` empties the reasoning of every assistant message except the last two: `reasoning_content` becomes `""`, `reasoning` items are removed from `output_items` (message items keep their phase), and a leading `<think>` block is cut from `content`. Since `70f40924` the key is kept with an empty value rather than deleted, since DeepSeek's thinking mode rejects (HTTP 400) a tool-turn assistant message missing it. Already emptied messages do not count as changed, so retries terminate.

<sub>Sources: [main.rs:236-267](../src/main.rs#L236-L267) · [commit 70f40924](https://github.com/lhwzds/restloop/commit/70f40924) · [commit f09603f4](https://github.com/lhwzds/restloop/commit/f09603f4)</sub>

## The time budget

`RESTLOOP_DEADLINE` (`f9d3b4f0`: Terminal-Bench tasks ran out of time unaware) bounds the task:

1. A new conversation's system prompt ends with `You have Ns in total for this task; each tool result ends with the time used so far.`; a resumed session's new task, a similar sentence ([Usage and configuration](02-usage.md)).
2. Each command's tool result ends with `[time used: Ns of Ds]`.
3. A command's timeout is the smaller of `RESTLOOP_TIMEOUT` and the time left (floored at 1 s).
4. A model request gets the time left after the key file is read (600 s at most); none starts after it: `restloop: deadline of <N>s reached`. A retry sleep reaching it stops with `deadline of <N>s reached (<what>: <text>)`; a failed fifth attempt only if the deadline has passed (`a4ba4994`).

Time counts from process start: in stdin mode one budget covers all tasks.

<sub>Sources: [main.rs:84-96](../src/main.rs#L84-L96) · [main.rs:99-109](../src/main.rs#L99-L109) · [main.rs:192-203](../src/main.rs#L192-L203) · [main.rs:348-356](../src/main.rs#L348-L356) · [main.rs:402-415](../src/main.rs#L402-L415) · +2</sub>

## Running a command

Each command runs in a fresh shell in the current directory: `sh -c` on Unix; on Windows, `Git\bin\bash.exe` under `ProgramFiles` or `ProgramFiles(x86)` if present (models write bash; PowerShell 5.1 adds a BOM), else `powershell -NoProfile -NonInteractive` with UTF-8 output. stdin is null and `GIT_TERMINAL_PROMPT=0` is set, so nothing waits for input; `PYTHONUTF8=1` keeps Python I/O in UTF-8.

After the shell exits, processes holding its pipes get up to 2 s more, never past the timeout; then the whole process group (a job object on Windows) is killed, and `exit:` still reports the shell's code. A background service survives once it stops holding those pipes, e.g. by redirecting output (`server > log 2>&1 &`). A timeout kills the process group; on Unix a child that left it survives. On Unix the kill signals the process group directly, starting no shell; Windows uses `TerminateJobObject`. Inherited variables: [Usage and configuration](02-usage.md).

<sub>Sources: [main.rs:551-641](../src/main.rs#L551-L641) · [commit 92c09fd2](https://github.com/lhwzds/restloop/commit/92c09fd2) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c)</sub>

## What the model sees from a command

A command's tool result is `exit: <status>` on the first line, then the combined output. Two differ: invalid arguments give `error: arguments must be JSON…`, a shell that cannot start gives `error: <reason>`. The status is the exit code, `killed` when a signal ended the shell, or `timeout after Ns`. Output over 10,000 characters (`MAX_OUTPUT`) keeps the first and last 5,000 with `... [N chars truncated] ...` between. Invalid UTF-8 is replaced.

<sub>Sources: [main.rs:655-676](../src/main.rs#L655-L676)</sub>
