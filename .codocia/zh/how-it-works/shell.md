---
title: 执行一条命令
source: #8f1fd101364b
---

`run()` 以回调接收命令执行器；CLI 传入公开的 `shell::shell()`，它让每条命令都在当前目录下一个全新的 shell 中执行：Unix 上用 `sh -c`；Windows 上，如果 `ProgramFiles` 或 `ProgramFiles(x86)` 下存在 `Git\bin\bash.exe` 就用它（模型写的是 bash；PowerShell 5.1 会加 BOM），否则用 `powershell -NoProfile -NonInteractive` 并设置 UTF-8 输出。stdin 为 null，并设置了 `GIT_TERMINAL_PROMPT=0`，所以不会有任何东西等待输入；`PYTHONUTF8=1` 让 Python 的 I/O 保持 UTF-8。

shell 退出后，仍占用其管道的进程最多还有 2 s，且不超过超时；之后整个进程组（Windows 上是 job object）会被 kill，而 `exit:` 报告的仍是 shell 自身的退出码。后台服务只要重定向自己的输出（`server > log 2>&1 &`）就能继续运行。在 Unix 上，已脱离进程组的子进程不会因超时被 kill。在 Unix 上，kill 直接向进程组发信号；Windows 上用 `TerminateJobObject`。继承哪些变量见 [[usage]]。
