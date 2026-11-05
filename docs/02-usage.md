---
title: Usage and configuration
---

# Usage and configuration

<sub>**English** · [中文](zh/02-usage.md)</sub>

RestLoop has no flags except `-h`/`--help`, which prints the list of environment variables. Everything else is the task text and the environment. This page lists the exact interface; [How it works](03-how-it-works.md) explains what each option does to the loop.

<sub>Sources: [main.rs:17-25](../src/main.rs#L17-L25)</sub>

## One task or a stdin session

All arguments are joined with spaces into one task: `restloop fix the build` and `restloop "fix the build"` are the same.

- **With a task**, RestLoop runs it and exits: status 0 when the model finished, 1 when the run failed (an API error after retries, or the step limit).
- **Without a task**, it prints `> ` on stderr and reads one task per line from stdin until end of input. Empty lines are skipped. All tasks share one conversation, so a later line can refer to earlier work. A failed task is reported on stderr and the loop continues.

Status 2 means the run never started: `RESTLOOP_MODEL` is unset, or `RESTLOOP_SESSION` names a file that is not a session.

<sub>Sources: [main.rs:17-36](../src/main.rs#L17-L36) · [main.rs:56-70](../src/main.rs#L56-L70)</sub>

## Environment variables

| Variable | Default | Effect |
|---|---|---|
| `RESTLOOP_MODEL` | required | `model` field of every request |
| `RESTLOOP_API_KEY` | empty | sent as `Authorization: Bearer <key>` |
| `RESTLOOP_API_KEY_FILE` | unset | file holding the key instead, read and trimmed before every request, so another program can refresh a short-lived token |
| `RESTLOOP_BASE_URL` | `https://api.openai.com/v1` | `/chat/completions` (or `/responses`) is appended; a trailing `/` is ignored |
| `RESTLOOP_WIRE_API` | chat completions | `responses` uses the OpenAI Responses API; other values are ignored |
| `RESTLOOP_REASONING_EFFORT` | unset | e.g. `low`, sent as `reasoning_effort` (or `reasoning.effort`) |
| `RESTLOOP_TIMEOUT` | `120` | seconds a single command may run |
| `RESTLOOP_MAX_STEPS` | `100` | steps per task: one model reply and its commands (retries and overflow recovery stay within the step) |
| `RESTLOOP_DEADLINE` | unset | seconds for the whole task; see [How it works](03-how-it-works.md) |
| `RESTLOOP_LOG_USAGE` | unset | `1` logs the token usage of each model reply on stderr |
| `RESTLOOP_LAST_MESSAGE` | unset | file that receives the final reply |
| `RESTLOOP_SESSION` | unset | conversation file to resume and save |

Numbers that do not parse as an unsigned integer silently fall back to the default (`env_num`). `RESTLOOP_LOG_USAGE` must be exactly `1`; empty strings count as unset for the key file and the effort.

<sub>Sources: [main.rs:12-15](../src/main.rs#L12-L15) · [main.rs:73-106](../src/main.rs#L73-L106) · [main.rs:147-160](../src/main.rs#L147-L160) · [commit f09603f4](https://github.com/lhwzds/restloop/commit/f09603f4)</sub>

## What commands inherit

Restloop's own settings are no longer passed in commands' environment: every `RESTLOOP_*` name is removed before a command starts (`restloop_var`), so the key, the key file and the URL do not reach it; on Windows the match ignores case. Commands inherit everything else, including any copy of the key under another name. A command running as the same user can still reach restloop's environment (`/proc/$PPID/environ`, `ps -E`) or read the key file if it knows the path; a sandbox or another user is the real fix.

<sub>Sources: [shell.rs:13-26](../src/shell.rs#L13-L26) · [shell.rs:66-71](../src/shell.rs#L66-L71) · [commit 8d5e3a78](https://github.com/lhwzds/restloop/commit/8d5e3a78)</sub>

## What goes to stdout and stderr

stdout carries only the model's text: every non-empty reply is printed, trimmed, including the short comments a model writes alongside tool calls. There is no separator between steps, so a script that needs only the final answer should use `RESTLOOP_LAST_MESSAGE`.

stderr carries everything else: each command as a dimmed `$ <command>`, then its duration and the first 10 lines of its output, retry and context-overflow notices, and with `RESTLOOP_LOG_USAGE=1` one line per step that gets a reply, from that reply's usage (retried requests are not logged):

```
usage: step=3 prompt_tokens=8123 completion_tokens=412 cached_tokens=7936 elapsed=41s
```

Fields the provider does not report print as `null`. These lines come from the core's `run()`, not the CLI, so a program using it as a library gets them on its stderr too: commands and results, retries, the overflow notice, and session or last-message save errors. Only the stdin prompt `> ` is the CLI's.

<sub>Sources: [lib.rs:49-53](../src/lib.rs#L49-L53) · [lib.rs:81-95](../src/lib.rs#L81-L95) · [lib.rs:122-123](../src/lib.rs#L122-L123) · [lib.rs:130-134](../src/lib.rs#L130-L134) · [lib.rs:397-398](../src/lib.rs#L397-L398) · +1</sub>

## The final-reply file

With `RESTLOOP_LAST_MESSAGE=<path>`, each task first deletes that file, then, when the model replies without a tool call, writes the reply text to it with a trailing newline. For models that inline their reasoning, a leading `<think>` block is cut off up to the first `</think>` (unclosed, nothing is left); any other reply is written whole, even if it quotes `</think>`. Surrounding whitespace is trimmed either way. The delete is best-effort (errors are ignored), so a run that ends early (step limit, API error, kill) usually leaves no file rather than the previous answer; for certainty, pass a fresh path on every run.

<sub>Sources: [lib.rs:64-68](../src/lib.rs#L64-L68) · [lib.rs:108-116](../src/lib.rs#L108-L116) · [lib.rs:252-260](../src/lib.rs#L252-L260) · [commit e8312d19](https://github.com/lhwzds/restloop/commit/e8312d19)</sub>

## Session files: resume and fork

`RESTLOOP_SESSION=<path>` persists the conversation as JSON:

```json
{"version": 1, "messages": [{"role": "system", "content": "..."}, ...]}
```

`messages` is exactly what the next request will send, in OpenAI chat format (with the Responses API, before conversion). `conversation()` loads it: a missing file starts a new conversation, created after the first step; an existing file needs `version` 1 and a non-empty `messages` array, or RestLoop exits with status 2. Nothing else is checked, so the file may be edited between runs: any valid chat messages are sent as they are. After editing an assistant message, remove its `output_items`, which would otherwise be sent instead.

Run again with the same path to resume; copy the file first to fork. The saved system prompt is reused unchanged. With `RESTLOOP_DEADLINE`, a resumed task gets `[You have Ns in total for this request; the time used in tool results counts from now.]` appended. The file is replaced after every step through a `<path>.<pid>.tmp` rename, so after a kill it holds the last complete step. Concurrent runs never corrupt it, but the last writer wins.

<sub>Sources: [main.rs:37-47](../src/main.rs#L37-L47) · [main.rs:108-131](../src/main.rs#L108-L131) · [lib.rs:154-168](../src/lib.rs#L154-L168) · [README.md:83-100](../README.md#L83-L100) · [commit ffd11c37](https://github.com/lhwzds/restloop/commit/ffd11c37) · +1</sub>
