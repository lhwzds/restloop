---
title: Running a command
order: 60
evidence:
  - src/shell.rs:28-125 #5cff66f1a4d4
  - src/main.rs:48-50 #95fa55ec9b30
  - commit:92c09fd2
  - commit:3b27682c
---

`run()` takes the runner as a callback; the CLI's `shell::shell()` runs each command in a fresh shell in the current directory: `sh -c` on Unix; on Windows, `Git\bin\bash.exe` under `ProgramFiles` or `ProgramFiles(x86)` if present (models write bash; PowerShell 5.1 adds a BOM), else `powershell -NoProfile -NonInteractive` with UTF-8 output. stdin is null and `GIT_TERMINAL_PROMPT=0`, so nothing waits for input; `PYTHONUTF8=1` keeps Python I/O in UTF-8.

After the shell exits, processes holding its pipes get up to 2 s more, never past the timeout; then the whole process group (a job object on Windows) is killed, and `exit:` still reports the shell's code. A background service survives if it redirects its output (`server > log 2>&1 &`). On Unix a child that left the group survives a timeout. On Unix the kill signals the process group directly, starting no shell; Windows uses `TerminateJobObject`. Inherited variables: [[usage]].
