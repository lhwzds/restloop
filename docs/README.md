---
title: restloop
---

# restloop

<sub>**English** · [中文](zh/README.md)</sub>

These docs go one level below the README for RestLoop, a single-file Rust agent loop that gives a model one `shell` tool. They describe v0.3.0. Compared with v0.2.0 it adds the Responses API (`RESTLOOP_WIRE_API=responses`), `RESTLOOP_API_KEY_FILE` and `RESTLOOP_REASONING_EFFORT`; a used-up quota is no longer retried, model requests are bounded by `RESTLOOP_DEADLINE`, an unreadable response body is retried, and CI checks these docs. Read them in order: what RestLoop is and the first run, the exact command-line and environment interface, how the loop, the HTTP client and the command runner work and why, and finally how binaries are built and released and how to change the code.

## Contents

1. [Overview](01-overview.md) — What it deliberately leaves out · A first result
2. [Usage and configuration](02-usage.md) — One task or a stdin session · Environment variables · What goes to stdout and stderr · The final-reply file · Session files: resume and fork
3. [How it works](03-how-it-works.md) — What the conversation holds · Requests and retries · The Responses API · Recovering from a full context window · Emptying old reasoning · The time budget · Running a command · What the model sees from a command
4. [Building, releasing and contributing](04-contributing.md) — What the tests cover · Changing the loop · How a release is made · The npm package

## Appendix

- [Code structure](structure.md) — generated from the code: areas, dependencies, pages

<sub>Generated from https://github.com/lhwzds/restloop @ `ee245a8f23`</sub>
