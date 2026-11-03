---
title: restloop
repo: ..
commit: 89acba0e3d811891d59b69600d9a24c294305ffb
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
  - README_CN.md: README.md #4eaa75ff9765
---

These docs go one level below the README for RestLoop, a single-file Rust agent loop that gives a model one `shell` tool. They describe v0.3.1. Compared with v0.3.0 an empty model reply is retried instead of ending the run, and only a leading `<think>` block is removed when a reply is written to `RESTLOOP_LAST_MESSAGE` or its old reasoning is trimmed (standard output prints the whole reply). Read them in order: what RestLoop is and the first run, the exact command-line and environment interface, how the loop, the HTTP client and the command runner work and why, and finally how binaries are built and released and how to change the code.
