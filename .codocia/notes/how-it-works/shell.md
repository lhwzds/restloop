---
title: Running a command
order: 60
evidence:
  - src/main.rs:516-606 #3bb7818743f2
  - commit:92c09fd2
  - commit:3b27682c
---

Each command runs in a fresh shell in the current directory: `sh -c` on Unix; on Windows, `Git\bin\bash.exe` under `ProgramFiles` or `ProgramFiles(x86)` if present (models write bash; PowerShell 5.1 adds a BOM), else `powershell -NoProfile -NonInteractive` with UTF-8 output. stdin is null and `GIT_TERMINAL_PROMPT=0` is set, so nothing waits for input; `PYTHONUTF8=1` keeps Python I/O in UTF-8.

After the shell exits, processes holding its pipes get up to 2 s more, never past the timeout; then the whole process group (a job object on Windows) is killed, and `exit:` still reports the shell's code. A background service survives if its output is redirected (`server > log 2>&1 &`). A timeout kills everything the command started. On Unix the kills use the shell builtin `kill -9 -<pgid>` (slim containers often lack `/bin/kill`); Windows uses `TerminateJobObject`.
