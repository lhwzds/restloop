---
title: What a run leaves behind
order: 15
evidence:
  - src/main.rs:30-34 #ac3120f2da5d
  - src/main.rs:56-57 #7e5890370f05
  - src/shell.rs:56-75 #f54362e09559
  - src/shell.rs:229-232 #16d09c12fd1e
  - src/shell.rs:354-360 #1479bc50ad82
  - src/shell.rs:396-437 #91dd64f4ae84
  - src/shell.rs:555-562 #e3588b076672
---

Nothing a task's commands started should outlive it. When a task ends (in stdin mode, each one), and on SIGHUP, SIGINT or SIGTERM (RestLoop then exits with 129, 130 or 143), every process group its commands started gets SIGTERM and, after about a second, SIGKILL; the task's deadline cuts that wait short. On Windows the job object is closed, which kills its processes with no graceful phase. A background server keeps running between the steps of one task.

The limits:

- A process that calls `setsid` or daemonizes escapes; use a container or a cgroup.
- On Unix at most 32 groups may live at once; the next command gets an error instead of running.
- Output past 1 MiB per command is dropped.
- Inherited file descriptors are closed for commands; when `/dev/fd` cannot be read, only those up to 1023.
- A process group id reused within one process is a known small gap.

This is CLI setup: a library caller of `shell::shell()` calls `shell::install()` and `shell::stop_all()` itself.
