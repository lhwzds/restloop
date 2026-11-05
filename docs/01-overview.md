---
title: Overview
---

# Overview

<sub>**English** · [中文](zh/01-overview.md)</sub>

RestLoop is a coding agent reduced to its loop: it sends a conversation and one tool, `shell`, to an OpenAI-compatible `/chat/completions` endpoint (or OpenAI's `/responses`), runs every command the model asks for, sends back the output, and stops when the model answers without a tool call. The core, the loop and the model client, is `src/lib.rs` and can be used as a library; `src/shell.rs` is the default command runner and `src/main.rs` the CLI. Its only dependencies are `ureq` for HTTPS and `serde_json`.

It exists because its predecessor, RestFlow (up to 0.5), had grown into a TUI, a daemon, a runner and an executable-skills system. Commit `33511748` replaced all of that with one file and no storage, and `cad33d13` renamed the result RestLoop and restarted versioning at 0.1.0. Most later features (a configurable step limit, the time budget, usage logging) were added in response to benchmark runs, mostly Terminal-Bench, and each is an environment variable: the time budget and usage logging stay off unless set, and the step limit defaults to 100 steps.

<sub>Sources: [lib.rs:1-8](../src/lib.rs#L1-L8) · [main.rs:1-4](../src/main.rs#L1-L4) · [shell.rs:1](../src/shell.rs#L1) · [README.md:54-56](../README.md#L54-L56) · [Cargo.toml:1-11](../Cargo.toml#L1-L11) · +2</sub>

## What it deliberately leaves out

- **One tool.** The model reads, writes and edits files through `shell`; there is no file-edit, search or patch tool. Its only parameter is `command`.
- **No model-facing knobs.** The system prompt from `system_prompt()` and the tool schema are fixed. Commit `3b27682c` records why the command timeout is not a tool parameter: in A/B runs a model-adjustable timeout roughly doubled the number of steps.
- **No configuration file and no database.** All options are environment variables, which the CLI reads once at start (the key before every request) into the core's `Config`; RestLoop itself writes only the optional session and last-message files (see [Usage and configuration](02-usage.md)).
- **No sandbox and no confirmation.** Commands run with your permissions in the current directory. Isolation is the caller's job: a container or a throwaway checkout.

<sub>Sources: [lib.rs:58-62](../src/lib.rs#L58-L62) · [lib.rs:169-180](../src/lib.rs#L169-L180) · [main.rs:73-106](../src/main.rs#L73-L106) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c)</sub>

## A first result

Install with Homebrew (`brew install lhwzds/tap/restloop`), npm (`npm install -g restloop`) or `cargo install --git https://github.com/lhwzds/restloop`. Only `RESTLOOP_MODEL` is required. The endpoint defaults to `https://api.openai.com/v1`, so for OpenAI a key and a model name are enough:

```bash
export RESTLOOP_API_KEY=sk-...
export RESTLOOP_MODEL=<model-id>
cd some-checkout
restloop "make the failing test pass"
```

For other providers set `RESTLOOP_BASE_URL` to the part before `/chat/completions`. Exit codes and every option are described in [Usage and configuration](02-usage.md).

<sub>Sources: [README.md:40-47](../README.md#L40-L47) · [README.md:52-60](../README.md#L52-L60) · [main.rs:80-81](../src/main.rs#L80-L81)</sub>
