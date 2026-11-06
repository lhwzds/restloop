---
title: restloop
---

# restloop

<sub>**English** · [中文](zh/README.md)</sub>

These docs go one level below the README for RestLoop, a small Rust agent loop that gives a model one `shell` tool. They describe v0.5.0. Compared with v0.4.0 the process groups a task's commands started are stopped when the task ends and on SIGHUP, SIGINT or SIGTERM, so a background server no longer outlives its task (see the overview). Since v0.4.0 the core is a library: `src/lib.rs` runs the loop from a `Config` and a tool callback and can be used on its own, `src/shell.rs` is the default command runner and `src/main.rs` the CLI. Read them in order: what RestLoop is and the first run, the exact command-line and environment interface, how the loop, the HTTP client and the command runner work and why, and finally how binaries are built and released and how to change the code.

## Contents

1. [Overview](01-overview.md) — What it deliberately leaves out · What a run leaves behind · The limits of that cleanup · A first result
2. [Usage and configuration](02-usage.md) — One task or a stdin session · Environment variables · What commands inherit · What goes to stdout and stderr · The final-reply file · Session files: resume and fork
3. [How it works](03-how-it-works.md) — What the conversation holds · Requests and retries · The Responses API · Recovering from a full context window · Emptying old reasoning · The time budget · Running a command · What the model sees from a command
4. [Building, releasing and contributing](04-contributing.md) — What the tests cover · Changing the loop · How a release is made · The npm package

## Appendix

- [Code structure](structure.md) — generated from the code: areas, dependencies, pages

<sub>Generated from https://github.com/lhwzds/restloop @ `ee3afae6ab`</sub>
