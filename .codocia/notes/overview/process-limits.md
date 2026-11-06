---
title: The limits of that cleanup
order: 16
evidence:
  - src/shell.rs:229-232 #16d09c12fd1e
  - src/shell.rs:354-360 #1479bc50ad82
  - src/shell.rs:423-437 #a6ef095a625b
  - npm/bin/restloop.js:56-58 #70c96d88f017
---

- On Unix, a process that calls `setsid` or daemonizes escapes; use a container or a cgroup.
- On Unix at most 32 groups may live at once; the next command gets an error instead of running.
- Output past 1 MiB per command is dropped.
- Inherited file descriptors are marked close-on-exec, and a failure to mark one is ignored; when `/dev/fd` cannot be read, only those up to 1023 are marked.
- A process group id reused within one process is a known small gap.
- With an npm install, a SIGTERM sent only to the launcher (`npm/bin/restloop.js`) kills node and leaves RestLoop and its commands running.
