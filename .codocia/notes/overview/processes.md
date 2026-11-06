---
title: What a run leaves behind
order: 15
evidence:
  - src/main.rs:30-34 #ac3120f2da5d
  - src/main.rs:56-57 #7e5890370f05
  - src/shell.rs:56-75 #f54362e09559
  - src/shell.rs:279-311 #aa9f7f608361
  - src/shell.rs:396-421 #8d8530480dcd
  - src/shell.rs:555-562 #e3588b076672
---

Nothing a task's commands started should outlive it. When a task ends (in stdin mode, each one), and on a handled signal, every process group its commands started gets SIGTERM and, after about a second, SIGKILL if the group remains; the task's deadline cuts that wait short. On Unix the handled signals are SIGHUP, SIGINT and SIGTERM, unless inherited as ignored (`nohup`), and RestLoop then exits with 129, 130 or 143. On Windows the job object is closed, which kills its processes with no graceful phase. A background server keeps running between the steps of one task.

This is CLI setup: a library caller of `shell::shell()` calls `shell::install()` and `shell::stop_all()` itself.
