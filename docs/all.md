---
title: restloop — All pages
---

# restloop

<sub>**English** · [中文](zh/all.md)</sub>

These docs go one level below the README for RestLoop, a single-file Rust agent loop that gives a model one `shell` tool. They describe v0.3.2. Compared with v0.3.1 commands no longer get any `RESTLOOP_*` variable in their environment, and a command's process group is killed without starting a shell. Read them in order: what RestLoop is and the first run, the exact command-line and environment interface, how the loop, the HTTP client and the command runner work and why, and finally how binaries are built and released and how to change the code.

<a id="overview"></a>

## Overview

RestLoop is a coding agent reduced to its loop: it sends a conversation and one tool, `shell`, to an OpenAI-compatible `/chat/completions` endpoint (or OpenAI's `/responses`), runs every command the model asks for, sends back the output, and stops when the model answers without a tool call. The whole program is `src/main.rs`; its only dependencies are `ureq` for HTTPS and `serde_json`.

It exists because its predecessor, RestFlow (up to 0.5), had grown into a TUI, a daemon, a runner and an executable-skills system. Commit `33511748` replaced all of that with one file and no storage, and `cad33d13` renamed the result RestLoop and restarted versioning at 0.1.0. Most later features (a configurable step limit, the time budget, usage logging) were added in response to benchmark runs, mostly Terminal-Bench, and each is an environment variable: the time budget and usage logging stay off unless set, and the step limit defaults to 100 steps.

<sub>Sources: [main.rs:1-13](../src/main.rs#L1-L13) · [README.md:33-35](../README.md#L33-L35) · [Cargo.toml:1-11](../Cargo.toml#L1-L11) · [commit 33511748](https://github.com/lhwzds/restloop/commit/33511748) · [commit cad33d13](https://github.com/lhwzds/restloop/commit/cad33d13)</sub>

### What it deliberately leaves out

- **One tool.** The model reads, writes and edits files through `shell`; there is no file-edit, search or patch tool. Its only parameter is `command`.
- **No model-facing knobs.** The system prompt from `system_prompt()` and the tool schema are fixed. Commit `3b27682c` records why the command timeout is not a tool parameter: in A/B runs a model-adjustable timeout roughly doubled the number of steps.
- **No configuration file and no database.** All options are environment variables read at the point of use; RestLoop itself writes only the optional session and last-message files (see [Usage and configuration](#usage)).
- **No sandbox and no confirmation.** Commands run with your permissions in the current directory. Isolation is the caller's job: a container or a throwaway checkout.

<sub>Sources: [main.rs:83-100](../src/main.rs#L83-L100) · [main.rs:124-129](../src/main.rs#L124-L129) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c)</sub>

### A first result

Install with Homebrew (`brew install lhwzds/tap/restloop`), npm (`npm install -g restloop`) or `cargo install --git https://github.com/lhwzds/restloop`. Only `RESTLOOP_MODEL` is required. The endpoint defaults to `https://api.openai.com/v1`, so for OpenAI a key and a model name are enough:

```bash
export RESTLOOP_API_KEY=sk-...
export RESTLOOP_MODEL=<model-id>
cd some-checkout
restloop "make the failing test pass"
```

For other providers set `RESTLOOP_BASE_URL` to the part before `/chat/completions`. Exit codes and every option are described in [Usage and configuration](#usage).

<sub>Sources: [README.md:19-26](../README.md#L19-L26) · [README.md:31-39](../README.md#L31-L39) · [main.rs:319-320](../src/main.rs#L319-L320)</sub>

<a id="usage"></a>

## Usage and configuration

RestLoop has no flags except `-h`/`--help`, which prints the list of environment variables. Everything else is the task text and the environment. This page lists the exact interface; [How it works](#how-it-works) explains what each option does to the loop.

<sub>Sources: [main.rs:16-28](../src/main.rs#L16-L28)</sub>

### One task or a stdin session

All arguments are joined with spaces into one task: `restloop fix the build` and `restloop "fix the build"` are the same.

- **With a task**, RestLoop runs it and exits: status 0 when the model finished, 1 when the run failed (an API error after retries, or the step limit).
- **Without a task**, it prints `> ` on stderr and reads one task per line from stdin until end of input. Empty lines are skipped. All tasks share one conversation, so a later line can refer to earlier work. A failed task is reported on stderr and the loop continues.

Status 2 means the run never started: `RESTLOOP_MODEL` is unset, or `RESTLOOP_SESSION` names a file that is not a session.

<sub>Sources: [main.rs:16-28](../src/main.rs#L16-L28) · [main.rs:30-63](../src/main.rs#L30-L63)</sub>

### Environment variables

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
| `RESTLOOP_DEADLINE` | unset | seconds for the whole task; see [How it works](#how-it-works) |
| `RESTLOOP_LOG_USAGE` | unset | `1` logs the token usage of each model reply on stderr |
| `RESTLOOP_LAST_MESSAGE` | unset | file that receives the final reply |
| `RESTLOOP_SESSION` | unset | conversation file to resume and save |

Numbers that do not parse as an unsigned integer silently fall back to the default (`env_num`). `RESTLOOP_LOG_USAGE` must be exactly `1`; empty strings count as unset for the key file and the effort.

<sub>Sources: [main.rs:11-14](../src/main.rs#L11-L14) · [main.rs:110-122](../src/main.rs#L110-L122) · [main.rs:130-138](../src/main.rs#L130-L138) · [main.rs:314-351](../src/main.rs#L314-L351) · [commit f09603f4](https://github.com/lhwzds/restloop/commit/f09603f4)</sub>

### What commands inherit

Restloop's own settings are no longer passed in commands' environment: every `RESTLOOP_*` name is removed before a command starts (`restloop_var`), so the key, the key file and the URL do not reach it; on Windows the match ignores case. Commands inherit everything else, including any copy of the key under another name. A command running as the same user can still reach restloop's environment (`/proc/$PPID/environ`, `ps -E`) or read the key file if it knows the path; a sandbox or another user is the real fix.

<sub>Sources: [main.rs:555-567](../src/main.rs#L555-L567) · [main.rs:604-609](../src/main.rs#L604-L609) · [commit 8d5e3a78](https://github.com/lhwzds/restloop/commit/8d5e3a78)</sub>

### What goes to stdout and stderr

stdout carries only the model's text: every non-empty reply is printed, trimmed, including the short comments a model writes alongside tool calls. There is no separator between steps, so a script that needs only the final answer should use `RESTLOOP_LAST_MESSAGE`.

stderr carries everything else: each command as a dimmed `$ <command>`, then its duration and the first 10 lines of its output, retry and context-overflow notices, and with `RESTLOOP_LOG_USAGE=1` one line per step that gets a reply, from that reply's usage (retried requests are not logged):

```
usage: step=3 prompt_tokens=8123 completion_tokens=412 cached_tokens=7936 elapsed=41s
```

Fields the provider does not report print as `null`.

<sub>Sources: [main.rs:149-163](../src/main.rs#L149-L163) · [main.rs:186-211](../src/main.rs#L186-L211) · [main.rs:142-145](../src/main.rs#L142-L145)</sub>

### The final-reply file

With `RESTLOOP_LAST_MESSAGE=<path>`, each task first deletes that file, then, when the model replies without a tool call, writes the reply text to it with a trailing newline. For models that inline their reasoning, a leading `<think>` block is cut off up to the first `</think>` (unclosed, nothing is left); any other reply is written whole, even if it quotes `</think>`. Surrounding whitespace is trimmed either way. The delete is best-effort (errors are ignored), so a run that ends early (step limit, API error, kill) usually leaves no file rather than the previous answer; for certainty, pass a fresh path on every run.

<sub>Sources: [main.rs:132-137](../src/main.rs#L132-L137) · [main.rs:176-184](../src/main.rs#L176-L184) · [main.rs:301-309](../src/main.rs#L301-L309) · [commit e8312d19](https://github.com/lhwzds/restloop/commit/e8312d19)</sub>

### Session files: resume and fork

`RESTLOOP_SESSION=<path>` persists the conversation as JSON:

```json
{"version": 1, "messages": [{"role": "system", "content": "..."}, ...]}
```

`messages` is exactly what the next request will send, in OpenAI chat format (with the Responses API, before conversion). `conversation()` loads it: a missing file starts a new conversation, created after the first step; an existing file needs `version` 1 and a non-empty `messages` array, or RestLoop exits with status 2. Nothing else is checked, so the file may be edited between runs: any valid chat messages are sent as they are. After editing an assistant message, remove its `output_items`, which would otherwise be sent instead.

Run again with the same path to resume; copy the file first to fork. The saved system prompt is reused unchanged. With `RESTLOOP_DEADLINE`, a resumed task gets `[You have Ns in total for this request; the time used in tool results counts from now.]` appended. The file is replaced after every step through a `<path>.<pid>.tmp` rename, so after a kill it holds the last complete step. Concurrent runs never corrupt it, but the last writer wins.

<sub>Sources: [main.rs:29-30](../src/main.rs#L29-L30) · [main.rs:64-81](../src/main.rs#L64-L81) · [main.rs:217-230](../src/main.rs#L217-L230) · [README.md:62-79](../README.md#L62-L79) · [commit ffd11c37](https://github.com/lhwzds/restloop/commit/ffd11c37) · +2</sub>

<a id="how-it-works"></a>

## How it works

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

### What the conversation holds

The system prompt from `system_prompt()` comes first: instructions plus OS, shell, working directory, command timeout and, with a deadline, the budget. Then come user tasks and, per model reply, an assistant message keeping only `content`, `tool_calls` and, when present, `reasoning_content` and the Responses API's `output_items`.

Each tool call gets one `tool` message with the matching `tool_call_id`. Arguments that are not JSON with a `command` string get an error quoting them, so the model can fix them.

<sub>Sources: [main.rs:83-100](../src/main.rs#L83-L100) · [main.rs:159-175](../src/main.rs#L159-L175) · [main.rs:186-211](../src/main.rs#L186-L211)</sub>

### Requests and retries

`complete()` posts `{"model", "messages", "tools"}` to `/chat/completions`, plus `reasoning_effort` if set, without sampling parameters, token limit or streaming. The key is reread per attempt ([Usage and configuration](#usage)); an unreadable key file fails with `cannot read <path>`.

Network errors (unreadable bodies included), HTTP 429, 5xx, a 200 without `choices[0].message` (some MiniMax errors, `333b7b64`) and an empty reply (no tool call, no text outside a leading `<think>`) are retried: five attempts, sleeping 2, 4, 8, 16 s, printing `restloop: <what>, retrying`; a fifth empty reply is the final one. A body with `insufficient_quota` or `subscription_sharing_usage_limit_exceeded` fails at once, unless the reply is empty. Other statuses, and the fifth failure, fail the task with `<what>: <text>` (`HTTP <status>`, `request failed` or `no reply`), or the deadline form below. A non-JSON 200 fails at once with `bad response`.

<sub>Sources: [main.rs:314-423](../src/main.rs#L314-L423) · [commit 333b7b64](https://github.com/lhwzds/restloop/commit/333b7b64)</sub>

### The Responses API

With `RESTLOOP_WIRE_API=responses`, `complete()` posts to `/responses`. The conversation stays in chat format, in memory and on disk; `responses_request()` converts it per request: a leading system message becomes `instructions`, a `tool` message a `function_call_output`, and an assistant message its `output_items` or, lacking them, its text and `function_call` items. It adds `store: false`, `stream: true`, encrypted reasoning in `include`, a `prompt_cache_key` hashed from the system prompt and, if set, `reasoning.effort`.

`responses_reply()` reads the event stream until `response.completed` and builds a chat reply: text (or refusal), tool calls, usage under chat names, and `output_items`, the output in `output_index` order minus message and call ids. These go back verbatim, so reasoning survives without server-side storage. A stream that breaks or ends early is retried; a reported failure (`response.failed`, `response.incomplete`, `error`) only for `server_error`, `rate_limit` or `unavailable`.

<sub>Sources: [main.rs:318-333](../src/main.rs#L318-L333) · [main.rs:356-361](../src/main.rs#L356-L361) · [main.rs:384-398](../src/main.rs#L384-L398) · [main.rs:437-550](../src/main.rs#L437-L550)</sub>

### Recovering from a full context window

Since `21f3c1dc`, `too_long()` recognises a context overflow: an HTTP 400 or 413, or a Responses stream failure (`no reply: `), whose text contains a provider phrase (`context_length_exceeded`, "maximum context length", "context window exceeds", "prompt is too long", "exceeds the available context size"). `04004c68` dropped looser words like "too long" after an overlong `tool_call_id` triggered history cuts.

On a match, `drop_old_output()` cuts the older half (rounded up) of the `tool` messages not yet shortened to their first line plus `[output removed to fit the context window]`, empties old reasoning with `trim_thinking()`, and the request is retried. Each retry halves the rest; when nothing changes, the error is returned. Commands and visible replies stay, so the model knows what it did.

<sub>Sources: [main.rs:139-148](../src/main.rs#L139-L148) · [main.rs:269-312](../src/main.rs#L269-L312) · [commit 21f3c1dc](https://github.com/lhwzds/restloop/commit/21f3c1dc) · [commit 04004c68](https://github.com/lhwzds/restloop/commit/04004c68)</sub>

### Emptying old reasoning

`trim_thinking()` empties the reasoning of every assistant message except the last two: `reasoning_content` becomes `""`, `reasoning` items are removed from `output_items` (message items keep their phase), and a leading `<think>` block is cut from `content`. Since `70f40924` the key is kept with an empty value rather than deleted, since DeepSeek's thinking mode rejects (HTTP 400) a tool-turn assistant message missing it. Already emptied messages do not count as changed, so retries terminate.

<sub>Sources: [main.rs:236-267](../src/main.rs#L236-L267) · [commit 70f40924](https://github.com/lhwzds/restloop/commit/70f40924) · [commit f09603f4](https://github.com/lhwzds/restloop/commit/f09603f4)</sub>

### The time budget

`RESTLOOP_DEADLINE` (`f9d3b4f0`: Terminal-Bench tasks ran out of time unaware) bounds the task:

1. A new conversation's system prompt ends with `You have Ns in total for this task; each tool result ends with the time used so far.`; a resumed session's new task, a similar sentence ([Usage and configuration](#usage)).
2. Each command's tool result ends with `[time used: Ns of Ds]`.
3. A command's timeout is the smaller of `RESTLOOP_TIMEOUT` and the time left (floored at 1 s).
4. A model request gets the time left after the key file is read (600 s at most); none starts after it: `restloop: deadline of <N>s reached`. A retry sleep reaching it stops with `deadline of <N>s reached (<what>: <text>)`; a failed fifth attempt only if the deadline has passed (`a4ba4994`).

Time counts from process start: in stdin mode one budget covers all tasks.

<sub>Sources: [main.rs:84-96](../src/main.rs#L84-L96) · [main.rs:99-109](../src/main.rs#L99-L109) · [main.rs:192-203](../src/main.rs#L192-L203) · [main.rs:348-356](../src/main.rs#L348-L356) · [main.rs:402-415](../src/main.rs#L402-L415) · +2</sub>

### Running a command

Each command runs in a fresh shell in the current directory: `sh -c` on Unix; on Windows, `Git\bin\bash.exe` under `ProgramFiles` or `ProgramFiles(x86)` if present (models write bash; PowerShell 5.1 adds a BOM), else `powershell -NoProfile -NonInteractive` with UTF-8 output. stdin is null and `GIT_TERMINAL_PROMPT=0` is set, so nothing waits for input; `PYTHONUTF8=1` keeps Python I/O in UTF-8.

After the shell exits, processes holding its pipes get up to 2 s more, never past the timeout; then the whole process group (a job object on Windows) is killed, and `exit:` still reports the shell's code. A background service survives once it stops holding those pipes, e.g. by redirecting output (`server > log 2>&1 &`). A timeout kills the process group; on Unix a child that left it survives. On Unix the kill signals the process group directly, starting no shell; Windows uses `TerminateJobObject`. Inherited variables: [Usage and configuration](#usage).

<sub>Sources: [main.rs:551-641](../src/main.rs#L551-L641) · [commit 92c09fd2](https://github.com/lhwzds/restloop/commit/92c09fd2) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c)</sub>

### What the model sees from a command

A command's tool result is `exit: <status>` on the first line, then the combined output. Two differ: invalid arguments give `error: arguments must be JSON…`, a shell that cannot start gives `error: <reason>`. The status is the exit code, `killed` when a signal ended the shell, or `timeout after Ns`. Output over 10,000 characters (`MAX_OUTPUT`) keeps the first and last 5,000 with `... [N chars truncated] ...` between. Invalid UTF-8 is replaced.

<sub>Sources: [main.rs:655-676](../src/main.rs#L655-L676)</sub>

<a id="contributing"></a>

## Building, releasing and contributing

`cargo build --release` produces `target/release/restloop`. The crate uses edition 2024 and `let` chains inside `if`, so it needs a toolchain where both are stable (Rust 1.88 or newer). The release profile optimises for size (`opt-level = "z"`, LTO, one codegen unit, `panic = "abort"`, stripped).

Tests live in `src/tests.rs` and run with `cargo test`; `.github/workflows/ci.yml` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the tests on macOS, Linux and Windows for every push to `main` and every pull request. Run the same three commands before opening one. A `docs` job on Linux checks out the full history (`fetch-depth: 0`, so cited commits resolve), installs Codocia 0.2 and runs `codocia check`, so stale docs or `README_CN.md` fail CI.

<sub>Sources: [Cargo.toml:1-18](../Cargo.toml#L1-L18) · [main.rs:634-641](../src/main.rs#L634-L641) · [ci.yml:1-34](../.github/workflows/ci.yml#L1-L34)</sub>

### What the tests cover

Unit tests pin: `too_long()` accepts recorded provider wordings and rejects near misses; `drop_old_output()` halves until it returns 0; `trim_thinking()` spares the last two replies, empties `reasoning_content` and drops only `reasoning` items of `output_items`; `truncate()` keeps head and tail. `shell()` tests: the exit line with both streams, a timeout kill, a background `sleep 30 &` not blocking.

One end-to-end test drives `run()` against `serve()`, a local canned-reply server: a tool call, an overflow 400 and its retry, a session resume, retried 200s without `choices` or with an empty or reasoning-only reply, five empty replies. Over the Responses API: the key file, output items sent back in order (only `message` and `function_call` items lose `id`), retried server and unavailable-plan failures, overflow recovery, failing at once on a used-up plan, content filter, invalid request or unreadable key file. Under a deadline: a server that hangs before or after its 200 headers, repeated 500s and empty replies stop early with `deadline of`; a 400 is reported; a passed deadline sends nothing. It alone touches `RESTLOOP_*`, clearing them and `*_PROXY` first, and checks commands get none.

<sub>Sources: [tests.rs:12-101](../src/tests.rs#L12-L101) · [tests.rs:145-200](../src/tests.rs#L145-L200) · [tests.rs:180-465](../src/tests.rs#L180-L465) · [commit 4c468f27](https://github.com/lhwzds/restloop/commit/4c468f27) · [commit 333b7b64](https://github.com/lhwzds/restloop/commit/333b7b64) · +1</sub>

### Changing the loop

Conventions the history suggests:

- **Options are environment variables with the old behaviour as default.** Read numbers with `env_num` so an invalid value falls back to the default, and add the variable to the `--help` text in `main()` and to the README.
- **Keep the model-facing surface fixed** (see [Overview](#overview)). Changes to `system_prompt()` or the tool schema should come with benchmark evidence.
- **Keep the prompt prefix stable.** Anything that rewrites earlier messages breaks provider prompt caching; RestLoop does it only when the provider reports an overflow. A threshold-based trim was removed after an A/B showed no benefit (`f09603f4`).
- **Keep the program in one file with two dependencies.** The program is `src/main.rs`; its tests live in `src/tests.rs`. Its size and readability are the point.

<sub>Sources: [main.rs:116-122](../src/main.rs#L116-L122) · [tests.rs:1-6](../src/tests.rs#L1-L6) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c) · [commit 740555b1](https://github.com/lhwzds/restloop/commit/740555b1) · [commit f09603f4](https://github.com/lhwzds/restloop/commit/f09603f4)</sub>

### How a release is made

Pushing a tag `vX.Y.Z` (prerelease tags do not match) runs `.github/workflows/release.yml`:

1. **build** compiles five targets: macOS arm64 and x86_64, Linux x86_64 and arm64 (static musl), Windows x86_64, named `restloop-<target>[.exe]`.
2. **release** writes `SHA256SUMS` plus one `.sha256` per binary and creates the GitHub release. It refuses to run if the release already exists: published binaries are never replaced, because the npm package pins their checksums. Re-run only failed jobs instead.
3. **homebrew** regenerates the `restloop` formula in `lhwzds/homebrew-tap` (needs the `HOMEBREW_TAP_TOKEN` secret) and commits only if it changed.
4. **npm**, also after **release**, stamps and publishes the npm package with trusted publishing; the npm package section below explains it.

Running the workflow by hand (`workflow_dispatch`) builds and checksums but publishes nothing, which makes it a dry run.

<sub>Sources: [release.yml:1-75](../.github/workflows/release.yml#L1-L75) · [release.yml:76-95](../.github/workflows/release.yml#L76-L95) · [release.yml:123-138](../.github/workflows/release.yml#L123-L138) · [release.yml:140-164](../.github/workflows/release.yml#L140-L164) · [commit b41c62a7](https://github.com/lhwzds/restloop/commit/b41c62a7) · +1</sub>

### The npm package

The npm package contains no binary. `npm/bin/restloop.js` looks up `"<platform> <arch>"` in `checksums.json`, and on first run downloads that asset from the `v<version>` GitHub release, verifies its SHA-256, and stores it next to itself (temporary file and rename). Later runs skip the download. Either way the launcher runs the binary, passing arguments and stdio through and returning its exit status.

A tag publishes the package from CI (`f76038cb`): after the GitHub release, the `npm` job runs `node npm/stamp.js X.Y.Z dist/SHA256SUMS`, which sets the package version and rewrites `checksums.json` for five platforms, then `npm publish --access public` from `npm/`. It uses npm trusted publishing (OIDC, `id-token: write`, npm 11.5.1 or later), so no npm token or secret is involved. Earlier releases were published by hand (`a21f7c21`).

The stamped `package.json` and `checksums.json` exist only on the runner and are never committed back, so the repository copies stay at the last manual stamp (0.3.0) and will lag the published version. At this commit `checksums.json` holds the five v0.3.0 entries: `darwin arm64`, `darwin x64`, `linux x64`, `linux arm64` (musl) and `win32 x64`.

<sub>Sources: [restloop.js:1-61](../npm/bin/restloop.js#L1-L61) · [stamp.js:1-31](../npm/stamp.js#L1-L31) · [checksums.json](../npm/checksums.json) · [release.yml:140-164](../.github/workflows/release.yml#L140-L164) · [commit a21f7c21](https://github.com/lhwzds/restloop/commit/a21f7c21) · +1</sub>

<a id="structure"></a>

## Code structure

Generated by Codocia from the code on every build: areas, each area's own description, import dependencies between areas, and the pages that explain them.

In scope: 12 files, 2,028 lines; excluded `Cargo.lock`, `LICENSE`, `.gitignore`

### Files

**Repository root**

- (root files) — 252 lines, 3 files. A minimal agent loop: one model, one shell tool.\
  Explained in: [Overview](#overview) (2), [Usage and configuration](#usage) (1), [Building, releasing and contributing](#contributing) (1)
- `.github/` — 198 lines, 2 files\
  Explained in: [Building, releasing and contributing](#contributing) (3)
- `src/` — 1,416 lines, 2 files\
  Hub files: `main.rs` `tests.rs`\
  Explained in: [Overview](#overview) (3), [Usage and configuration](#usage) (7), [How it works](#how-it-works) (9), [Building, releasing and contributing](#contributing) (3)

**`npm/`**

- (files) — 101 lines, 4 files. A minimal coding-agent loop: it keeps working on your task so you can rest.\
  Hub files: `stamp.js`\
  Explained in: [Building, releasing and contributing](#contributing) (1)
- `bin/` — 61 lines, 1 file\
  Hub files: `restloop.js`\
  Explained in: [Building, releasing and contributing](#contributing) (1)


### Dependencies

A → B means files in A import B; the number counts file-level imports. Showing the strongest 1 of 1 edges.

```mermaid
flowchart LR
  a2["npm/*"]
  a3["npm/bin/"]
  a3 -->|2| a2
```
