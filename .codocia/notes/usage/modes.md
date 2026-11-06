---
title: One task or a stdin session
order: 10
evidence:
  - src/main.rs:17-36 #65ee67684f47
  - src/main.rs:64-78 #36f8127fc871
  - src/shell.rs:390-394 #588147656b4e
---

All arguments are joined with spaces into one task: `restloop fix the build` and `restloop "fix the build"` are the same.

- **With a task**, RestLoop runs it and exits: status 0 when the model finished, 1 when the run failed (an API error after retries, or the step limit).
- **Without a task**, it prints `> ` on stderr and reads one task per line from stdin until end of input. Empty lines are skipped. All tasks share one conversation, so a later line can refer to earlier work. A failed task is reported on stderr and the loop continues.

Status 2 means the run never started: `RESTLOOP_MODEL` is unset, or `RESTLOOP_SESSION` names a file that is not a session. On Unix, SIGHUP, SIGINT and SIGTERM give 129, 130 and 143 ([[overview]]).
