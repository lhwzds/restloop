---
title: Changing the loop
order: 10
evidence:
  - src/main.rs:118-124 #579b9cedf39b
  - src/tests.rs:1-6 #3c2abef9f5c2
  - commit:3b27682c
  - commit:740555b1
  - commit:f09603f4
---

Conventions the history suggests:

- **Options are environment variables with the old behaviour as default.** Read numbers with `env_num` so an invalid value falls back to the default, and add the variable to the `--help` text in `main()` and to the README.
- **Keep the model-facing surface fixed** (see [[overview]]). Changes to `system_prompt()` or the tool schema should come with benchmark evidence.
- **Keep the prompt prefix stable.** Anything that rewrites earlier messages breaks provider prompt caching; RestLoop does it only when the provider reports an overflow. A threshold-based trim was removed after an A/B showed no benefit (`f09603f4`).
- **Keep the program in one file with two dependencies.** The program is `src/main.rs`; its tests live in `src/tests.rs`. Its size and readability are the point.
