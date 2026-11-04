---
title: restloop
repo: ..
commit: 101efa4c44962f3857c0657930bd8ca8ea985a00
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
  - README_CN.md: README.md #24e875bd876b
---

These docs go one level below the README for RestLoop, a single-file Rust agent loop that gives a model one `shell` tool. They describe v0.3.2. Compared with v0.3.1 commands no longer get any `RESTLOOP_*` variable in their environment, and a command's process group is killed without starting a shell. Read them in order: what RestLoop is and the first run, the exact command-line and environment interface, how the loop, the HTTP client and the command runner work and why, and finally how binaries are built and released and how to change the code.
