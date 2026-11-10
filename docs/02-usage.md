---
title: Usage and configuration
---

# Usage and configuration

<sub>**English** · [中文](zh/02-usage.md)</sub>

RestLoop has no flags except `-h`/`--help`, which prints the list of environment variables. Everything else is the task text and the environment; [How it works](03-how-it-works.md) explains what each option does to the loop.

<sub>Sources: [main.rs:17-25](../src/main.rs#L17-L25)</sub>

## One task or a stdin session

All arguments are joined with spaces into one task: `restloop fix the build` equals `restloop "fix the build"`.

- **With a task**, RestLoop runs it and exits: status 0 when the model finished, 1 when the run failed (an API error after retries, a key error, or the step limit).
- **Without a task**, it prints `> ` on stderr and reads one task per line from stdin, skipping empty lines. All tasks share one conversation, so a later line can refer to earlier work. A failed task is reported on stderr and the loop continues.

Status 2 means the run never started: `RESTLOOP_MODEL` is unset, `RESTLOOP_HEADERS` has a bad line or is not valid UTF-8, or `RESTLOOP_SESSION` names a file that is not a session. On Unix, SIGHUP, SIGINT and SIGTERM give 129, 130 and 143 ([Overview](01-overview.md)).

<sub>Sources: [main.rs:17-36](../src/main.rs#L17-L36) · [main.rs:76-90](../src/main.rs#L76-L90) · [shell.rs:390-394](../src/shell.rs#L390-L394) · [main.rs:26-37](../src/main.rs#L26-L37)</sub>

## Environment variables

| Variable | Default | Effect |
|---|---|---|
| `RESTLOOP_MODEL` | required | `model` field of every request |
| `RESTLOOP_API_KEY` | empty | sent as `Authorization: Bearer <key>` |
| `RESTLOOP_API_KEY_FILE` | unset | file holding the key instead, reread and trimmed before every request, so a short-lived token can be refreshed |
| `RESTLOOP_BASE_URL` | `https://api.openai.com/v1` | `/chat/completions` (or `/responses`) is appended; a trailing `/` is ignored |
| `RESTLOOP_WIRE_API` | chat completions | `responses` uses the OpenAI Responses API; other values are ignored |
| `RESTLOOP_REASONING_EFFORT` | unset | e.g. `low`, sent as `reasoning_effort` (or `reasoning.effort`) |
| `RESTLOOP_HEADERS` | unset | extra request headers; see below |
| `RESTLOOP_TIMEOUT` | `120` | seconds a single command may run |
| `RESTLOOP_MAX_STEPS` | `100` | steps per task: one model reply and its commands (retries and overflow recovery stay within the step) |
| `RESTLOOP_DEADLINE` | unset | seconds for the whole task; see [How it works](03-how-it-works.md) |
| `RESTLOOP_LOG_USAGE` | unset | `1` logs the token usage of each model reply on stderr |
| `RESTLOOP_LAST_MESSAGE` | unset | file that receives the final reply |
| `RESTLOOP_SESSION` | unset | conversation file to resume and save |

Invalid numbers (not unsigned integers) silently fall back to the default (`env_num`). `RESTLOOP_LOG_USAGE` must be exactly `1`; empty strings count as unset for the key file and the effort.

<sub>Sources: [main.rs:12-15](../src/main.rs#L12-L15) · [main.rs:93-128](../src/main.rs#L93-L128) · [main.rs:169-177](../src/main.rs#L169-L177) · [commit f09603f4](https://github.com/lhwzds/restloop/commit/f09603f4)</sub>

## What commands inherit

Commands get none of RestLoop's settings: every `RESTLOOP_*` name is removed before a command starts (`restloop_var`), so the key, key file, URL and headers do not reach it; on Windows the match ignores case. Commands inherit everything else, including any copy of the key under another name. A command running as the same user can still reach restloop's environment (`/proc/$PPID/environ`, `ps -E`) or read the key file if it knows the path; a sandbox or another user is the real fix.

<sub>Sources: [shell.rs:29-42](../src/shell.rs#L29-L42) · [shell.rs:130-135](../src/shell.rs#L130-L135) · [commit 8d5e3a78](https://github.com/lhwzds/restloop/commit/8d5e3a78)</sub>

## Extra request headers

`RESTLOOP_HEADERS` holds extra request headers, one `Name: value` per line (blank lines skipped, CRLF fine), sent with every model request; a name given twice goes twice. OpenCode Go, for example, wants `x-opencode-session: <id>`. These names are refused, in any case: `Authorization`, `Proxy-Authorization`, `Content-Type`, `Content-Length`, `Host`, `Transfer-Encoding`, `Accept-Encoding`; any other valid name is sent. A bad line (no `:`, an invalid or refused header) or non-UTF-8 text exits with 2 first. The error gives the line number (`RESTLOOP_HEADERS: line 2 has no ':'`) and, for a refused name, the name, never the value; the UTF-8 error has no line number. Library callers set `Config.headers`, which `run()` refuses first (`restloop::bad_header()`).

<sub>Sources: [main.rs:26-37](../src/main.rs#L26-L37) · [main.rs:179-201](../src/main.rs#L179-L201) · [lib.rs:46-75](../src/lib.rs#L46-L75) · [lib.rs:82-92](../src/lib.rs#L82-L92)</sub>

## What goes to stdout and stderr

stdout carries only the model's text: every non-empty reply, trimmed, including comments written alongside tool calls. Steps have no separator; for the final answer only, use `RESTLOOP_LAST_MESSAGE`.

stderr carries everything else: each command as a dimmed `$ <command>`, then its duration and the first 10 lines of its output, retry and context-overflow notices, and with `RESTLOOP_LOG_USAGE=1` one line per step that gets a reply, from that reply's usage (retried requests are not logged):

```
usage: step=3 prompt_tokens=8123 completion_tokens=412 cached_tokens=7936 elapsed=41s
```

Fields the provider does not report print as `null`. They come from the core's `run()`, so a library caller gets them too; only the stdin prompt `> ` is the CLI's. The default runner adds `restloop: cannot stop a process group` when a SIGKILL fails on a group that is still there.

<sub>Sources: [shell.rs:270-277](../src/shell.rs#L270-L277) · [lib.rs:77-81](../src/lib.rs#L77-L81) · [lib.rs:115-129](../src/lib.rs#L115-L129) · [lib.rs:156-157](../src/lib.rs#L156-L157) · [lib.rs:164-168](../src/lib.rs#L164-L168) · +2</sub>

## The final-reply file

With `RESTLOOP_LAST_MESSAGE=<path>`, each task first deletes that file (unless RestLoop exited with status 2 first), then, when the model replies without a tool call, writes the reply text to it with a trailing newline. A leading `<think>` block (inline reasoning) is cut off up to the first `</think>` (unclosed, nothing is left); any other reply is written whole, even if it quotes `</think>`. Surrounding whitespace is trimmed either way. The delete is best-effort, so a run that ends early (step limit, API error, kill) usually leaves no file rather than the previous answer; to be sure, pass a fresh path each run.

<sub>Sources: [lib.rs:98-102](../src/lib.rs#L98-L102) · [lib.rs:142-150](../src/lib.rs#L142-L150) · [lib.rs:286-294](../src/lib.rs#L286-L294) · [commit e8312d19](https://github.com/lhwzds/restloop/commit/e8312d19) · [main.rs:26-37](../src/main.rs#L26-L37) · +1</sub>

## Session files: resume and fork

`RESTLOOP_SESSION=<path>` persists the conversation as JSON:

```json
{"version": 1, "messages": [{"role": "system", "content": "..."}, ...]}
```

`messages` is what the next request sends, in OpenAI chat format (before any Responses conversion). `conversation()` loads it: a missing file starts a new conversation, created after the first step; an existing file needs `version` 1 and a non-empty `messages` array, or RestLoop exits with status 2. Nothing else is checked, so the file may be edited between runs. After editing an assistant message, remove its `output_items`, which would otherwise be sent instead.

Run again with the same path to resume; copy the file first to fork. The saved system prompt is reused. With `RESTLOOP_DEADLINE`, a resumed task gets `[You have Ns in total for this request; the time used in tool results counts from now.]` appended. The file is replaced after every step through a `<path>.<pid>.tmp` rename, so after a kill it holds the last complete step; concurrent runs never corrupt it, the last writer wins.

<sub>Sources: [main.rs:54-64](../src/main.rs#L54-L64) · [main.rs:130-153](../src/main.rs#L130-L153) · [lib.rs:188-202](../src/lib.rs#L188-L202) · [README.md:84-102](../README.md#L84-L102) · [commit ffd11c37](https://github.com/lhwzds/restloop/commit/ffd11c37) · +1</sub>
