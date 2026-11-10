---
title: 执行一条命令
source: #3bb897d7d6be
---

`run()` 以回调接收命令执行器；CLI 的执行器是公开的 `shell::shell()`，它让每条命令都在当前目录下一个全新的 shell 中执行：Unix 上用 `sh -c`；Windows 上，如果 `ProgramFiles` 或 `ProgramFiles(x86)` 下存在 `Git\bin\bash.exe` 就用它（模型写的是 bash；PowerShell 5.1 会加 BOM），否则用 `powershell -NoProfile -NonInteractive` 并设置 UTF-8 输出。stdin 为 null，并设置 `GIT_TERMINAL_PROMPT=0`；`PYTHONUTF8=1` 让 Python 的 I/O 保持 UTF-8。

shell 退出后，仍占用其管道的进程最多还有 2 s，且不超过超时；之后整个进程组（Windows 上是 job object）会被停止，而 `exit:` 报告的仍是 shell 的退出码。在 Unix 上，停止是先发 SIGTERM，最多等 1 s，再发 SIGKILL；Windows 上用 `TerminateJobObject`。重定向了输出的后台服务（`server > log 2>&1 &`）会一直运行到任务结束：[[overview]]。继承的变量见 [[usage]]。
