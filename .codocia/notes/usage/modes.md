---
title: One task or a stdin session
order: 10
evidence:
  - src/main.rs:17-36 #28f684b93ba3
  - src/main.rs:76-90 #36f8127fc871
  - src/shell.rs:390-394 #588147656b4e
  - src/main.rs:26-37 #2164701a5216
---

All arguments are joined with spaces into one task: `restloop fix the build` equals `restloop "fix the build"`.

- **With a task**, RestLoop runs it and exits: status 0 when the model finished, 1 when the run failed (an API error after retries, a key error, or the step limit).
- **Without a task**, it prints `> ` on stderr and reads one task per line from stdin, skipping empty lines. All tasks share one conversation, so a later line can refer to earlier work. A failed task is reported on stderr and the loop continues.

Status 2 means the run never started: `RESTLOOP_MODEL` is unset, `RESTLOOP_HEADERS` has a bad line or is not valid UTF-8, or `RESTLOOP_SESSION` names a file that is not a session. On Unix, SIGHUP, SIGINT and SIGTERM give 129, 130 and 143 ([[overview]]).
