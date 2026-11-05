---
title: What the model sees from a command
order: 70
evidence:
  - src/shell.rs:72-75 #13ce2ea42652
  - src/shell.rs:123-138 #6948fdfb8cec
  - src/lib.rs:143-145 #7e7e5bc18c93
---

A command's tool result is `exit: <status>` on the first line, then the combined output. Except: invalid arguments give `error: arguments must be JSON…`, a shell that cannot start gives `error: <reason>`. The status is the exit code, `killed` when a signal ended the shell, or `timeout after Ns`. Output over 10,000 characters (`MAX_OUTPUT`) keeps the first and last 5,000 with `... [N chars truncated] ...` between. Invalid UTF-8 is replaced.
