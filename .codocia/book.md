---
title: restloop
repo: ..
commit: f049519db49f6c1a789a8e4e68444e709b4845a8
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
  - README_CN.md: README.md #7cad35b5f982
---

These docs go one level below the README for RestLoop, a small Rust agent loop that gives a model one `shell` tool. They describe v0.4.0. Compared with v0.3.2 the core is a library: `src/lib.rs` runs the loop from a `Config` and a tool callback and can be used on its own, `src/shell.rs` is the default command runner and `src/main.rs` the CLI. The binary behaves exactly as before. Read them in order: what RestLoop is and the first run, the exact command-line and environment interface, how the loop, the HTTP client and the command runner work and why, and finally how binaries are built and released and how to change the code.
