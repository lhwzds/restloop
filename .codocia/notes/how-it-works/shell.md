---
title: Running a command
order: 60
evidence:
  - src/shell.rs:44-54 #8aff3a3d5051
  - src/shell.rs:101-206 #da8b12949a0c
  - src/shell.rs:279-332 #167b38ae9b31
  - src/main.rs:53-55 #4d85fbc6ef26
  - commit:92c09fd2
  - commit:3b27682c
---

`run()` takes the runner as a callback; the CLI passes the public `shell::shell()`, which runs each command in a fresh shell in the current directory: `sh -c` on Unix; on Windows, `Git\bin\bash.exe` under `ProgramFiles` or `ProgramFiles(x86)` if present (models write bash; PowerShell 5.1 adds a BOM), else `powershell -NoProfile -NonInteractive` with UTF-8 output. stdin is null and `GIT_TERMINAL_PROMPT=0`, so nothing waits for input; `PYTHONUTF8=1` keeps Python I/O in UTF-8.

After the shell exits, processes holding its pipes get up to 2 s more, never past the timeout; then the whole process group (a job object on Windows) is stopped, and `exit:` still reports the shell's code. On Unix a stop is SIGTERM, up to 1 s, then SIGKILL; Windows uses `TerminateJobObject`. A background service that redirects its output (`server > log 2>&1 &`) lives until the task ends: [[overview]]. Inherited variables: [[usage]].
