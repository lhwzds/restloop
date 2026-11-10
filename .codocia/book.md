---
title: restloop
repo: ..
commit: e956437d9af9f969813901252699f3fe87c80ba6
remote: https://github.com/lhwzds/restloop
lang: en
max_pages: 8
max_page_words: 900
max_note_words: 180
pages:
  - overview: Overview
  - usage: Usage and configuration
  - how-it-works: How it works
  - contributing: Building, releasing and contributing
ignore:
  - Cargo.lock
  - LICENSE
  - .gitignore
langs:
  - zh
translations:
  - README_CN.md: README.md #cdfe4172530f
---

These docs go one level below the README for RestLoop, a small Rust agent loop that gives a model one `shell` tool. They describe v0.6.0, which adds `RESTLOOP_HEADERS` (extra request headers), no longer follows redirects, and ends the run on a key HTTP cannot carry. Since v0.5.0 the process groups a task's commands started are stopped when the task ends and on SIGHUP, SIGINT or SIGTERM, so a background server no longer outlives its task (see the overview). Since v0.4.0 the core is a library: `src/lib.rs` runs the loop from a `Config` and a tool callback and can be used on its own, `src/shell.rs` is the default command runner and `src/main.rs` the CLI. Read them in order: what RestLoop is and the first run, the exact command-line and environment interface, how the loop, the HTTP client and the command runner work and why, and finally how binaries are built and released and how to change the code.
