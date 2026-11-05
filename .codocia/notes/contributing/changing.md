---
title: Changing the loop
order: 10
evidence:
  - src/lib.rs:1-8 #3eeca20e9983
  - src/main.rs:157-160 #e9b807c7864d
  - src/tests.rs:1-2 #0cb9d8c7f1af
  - commit:3b27682c
  - commit:740555b1
  - commit:f09603f4
---

Conventions the history suggests:

- **Options are environment variables with the old behaviour as default.** The CLI reads them in `config()` into a `Config` field; read numbers with `env_num` so an invalid value falls back to the default, and add the variable to the `--help` text in `main()` and to the README.
- **Keep the model-facing surface fixed** (see [[overview]]). Changes to `system_prompt()` or the tool schema should come with benchmark evidence.
- **Keep the prompt prefix stable.** Anything that rewrites earlier messages breaks provider prompt caching; RestLoop does it only when the provider reports an overflow. A threshold-based trim was removed after an A/B showed no benefit (`f09603f4`).
- **Keep the core's settings in `Config`, with two dependencies.** `src/lib.rs` makes no direct environment reads (its HTTP client, `ureq`, honours `*_PROXY`) and starts no process; that belongs to the CLI (`src/main.rs`) and the default command runner, the public module `src/shell.rs`. Its size and readability are the point.
