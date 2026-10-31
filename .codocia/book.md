---
title: restloop
repo: ..
commit: f76038cb90fe0edd5de52af7bae116ce6d9043e7
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
  - README_CN.md: README.md #3f6da43c4599
---

These docs go one level below the README for RestLoop, a single-file Rust agent loop that gives a model one `shell` tool. They describe v0.3.0. Compared with v0.2.0 it adds the Responses API (`RESTLOOP_WIRE_API=responses`), `RESTLOOP_API_KEY_FILE` and `RESTLOOP_REASONING_EFFORT`; a used-up quota is no longer retried, model requests are bounded by `RESTLOOP_DEADLINE`, an unreadable response body is retried, and CI checks these docs. Read them in order: what RestLoop is and the first run, the exact command-line and environment interface, how the loop, the HTTP client and the command runner work and why, and finally how binaries are built and released and how to change the code.
