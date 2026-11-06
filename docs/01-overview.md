---
title: Overview
---

# Overview

<sub>**English** · [中文](zh/01-overview.md)</sub>

RestLoop is a coding agent reduced to its loop: it sends a conversation and one tool, `shell`, to an OpenAI-compatible `/chat/completions` endpoint (or OpenAI's `/responses`), runs every command the model asks for, sends back the output, and stops when the model answers without a tool call. The core, the loop and the model client, is `src/lib.rs` and can be used as a library; `src/shell.rs` is the default command runner and `src/main.rs` the CLI. Its only dependencies are `ureq` for HTTPS, `serde_json` and, on Unix, `libc`.

It exists because its predecessor, RestFlow (up to 0.5), had grown into a TUI, a daemon, a runner and an executable-skills system. Commit `33511748` replaced all of that with one file and no storage, and `cad33d13` renamed the result RestLoop and restarted versioning at 0.1.0. Most later features (a configurable step limit, the time budget, usage logging) were added in response to benchmark runs, mostly Terminal-Bench, and each is an environment variable: the time budget and usage logging stay off unless set, and the step limit defaults to 100 steps.

<sub>Sources: [lib.rs:1-9](../src/lib.rs#L1-L9) · [main.rs:1-4](../src/main.rs#L1-L4) · [shell.rs:1](../src/shell.rs#L1) · [README.md:54-56](../README.md#L54-L56) · [Cargo.toml:1-15](../Cargo.toml#L1-L15) · +2</sub>

## What it deliberately leaves out

- **One tool.** The model reads, writes and edits files through `shell`; there is no file-edit, search or patch tool. Its only parameter is `command`.
- **No model-facing knobs.** The system prompt from `system_prompt()` and the tool schema are fixed. Commit `3b27682c` records why the command timeout is not a tool parameter: in A/B runs a model-adjustable timeout roughly doubled the number of steps.
- **No configuration file and no database.** All options are environment variables, which the CLI reads once at start (the key before every request) into the core's `Config`; RestLoop itself writes only the optional session and last-message files (see [Usage and configuration](02-usage.md)).
- **No sandbox and no confirmation.** Commands run with your permissions in the current directory. Isolation is the caller's job: a container or a throwaway checkout.

<sub>Sources: [lib.rs:59-63](../src/lib.rs#L59-L63) · [lib.rs:170-181](../src/lib.rs#L170-L181) · [main.rs:81-114](../src/main.rs#L81-L114) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c)</sub>

## What a run leaves behind

Nothing a task's commands started should outlive it. When a task ends (in stdin mode, each one), and on a handled signal, every process group its commands started gets SIGTERM and, after about a second, SIGKILL if the group remains; the task's deadline cuts that wait short. On Unix the handled signals are SIGHUP, SIGINT and SIGTERM, unless inherited as ignored (`nohup`), and RestLoop then exits with 129, 130 or 143. On Windows the job object is closed, which kills its processes with no graceful phase. A background server keeps running between the steps of one task.

This is CLI setup: a library caller of `shell::shell()` calls `shell::install()` and `shell::stop_all()` itself.

<sub>Sources: [main.rs:30-34](../src/main.rs#L30-L34) · [main.rs:56-57](../src/main.rs#L56-L57) · [shell.rs:56-75](../src/shell.rs#L56-L75) · [shell.rs:279-311](../src/shell.rs#L279-L311) · [shell.rs:396-421](../src/shell.rs#L396-L421) · +1</sub>

## The limits of that cleanup

- On Unix, a process that calls `setsid` or daemonizes escapes; use a container or a cgroup.
- On Unix at most 32 groups may live at once; the next command gets an error instead of running.
- Output past 1 MiB per command is dropped.
- Inherited file descriptors are marked close-on-exec, and a failure to mark one is ignored; when `/dev/fd` cannot be read, only those up to 1023 are marked.
- A process group id reused within one process is a known small gap.
- With an npm install, a SIGTERM sent only to the launcher (`npm/bin/restloop.js`) kills node and leaves RestLoop and its commands running.

<sub>Sources: [shell.rs:229-232](../src/shell.rs#L229-L232) · [shell.rs:354-360](../src/shell.rs#L354-L360) · [shell.rs:423-437](../src/shell.rs#L423-L437) · [restloop.js:56-58](../npm/bin/restloop.js#L56-L58)</sub>

## A first result

Install with Homebrew (`brew install lhwzds/tap/restloop`), npm (`npm install -g restloop`) or `cargo install --git https://github.com/lhwzds/restloop`. Only `RESTLOOP_MODEL` is required. The endpoint defaults to `https://api.openai.com/v1`, so for OpenAI a key and a model name are enough:

```bash
export RESTLOOP_API_KEY=sk-...
export RESTLOOP_MODEL=<model-id>
cd some-checkout
restloop "make the failing test pass"
```

For other providers set `RESTLOOP_BASE_URL` to the part before `/chat/completions`. Exit codes and every option are described in [Usage and configuration](02-usage.md).

<sub>Sources: [README.md:40-47](../README.md#L40-L47) · [README.md:52-60](../README.md#L52-L60) · [main.rs:88-89](../src/main.rs#L88-L89)</sub>
