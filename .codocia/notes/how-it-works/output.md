---
title: What the model sees from a command
order: 70
evidence:
  - src/shell.rs:171-176 #d9ccd68c1d09
  - src/shell.rs:184-189 #6f0efefea81f
  - src/shell.rs:14-16 #e0cf1886702a
  - src/shell.rs:136-139 #d87c22e4e9cf
  - src/shell.rs:199-219 #1fa562bc088d
  - src/lib.rs:143-145 #7e7e5bc18c93
---

A command's tool result is `exit: <status>` on the first line, then the combined output. Except: invalid arguments give `error: arguments must be JSON…`, a shell that cannot start gives `error: <reason>`. The status is the exit code, `killed` when a signal ended the shell, `exited, status unknown` when the child is gone without one, or `timeout after Ns`; a stopped command not seen to exit adds `(child did not exit after SIGKILL)`. Output over 10,000 characters (`MAX_OUTPUT`) keeps the first and last 5,000 with `... [N chars truncated] ...` between. Output past 1 MiB is dropped first. Invalid UTF-8 is replaced.
