---
title: restloop
repo: ..
commit: d116b90d4ff21867689e11e61dbf9f96b2d1e3cd
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
  - README_CN.md: README.md #89c6e76dc481
---

These docs go one level below the README for RestLoop, a single-file Rust agent loop that gives a model one `shell` tool. They describe `main` after v0.2.0. Compared with v0.1.0, v0.2.0 added context-overflow recovery, `RESTLOOP_SESSION`, a test suite with CI and retries of HTTP 200 responses without a reply, and dropped `RESTLOOP_TRIM_THINKING_AT`; `main` adds the Responses API, a key file and the reasoning effort. Read them in order: what RestLoop is and the first run, the exact command-line and environment interface, how the loop, the HTTP client and the command runner work and why, and finally how binaries are built and released and how to change the code.
