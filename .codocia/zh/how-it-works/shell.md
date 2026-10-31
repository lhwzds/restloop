---
title: 执行一条命令
source: #248cf1b6c8d0
---

每条命令都在当前目录下一个全新的 shell 中执行：Unix 上用 `sh -c`；Windows 上，如果 `ProgramFiles` 或 `ProgramFiles(x86)` 下存在 `Git\bin\bash.exe` 就用它（模型写的是 bash；PowerShell 5.1 会加 BOM），否则用 `powershell -NoProfile -NonInteractive` 并设置 UTF-8 输出。stdin 为 null，并设置了 `GIT_TERMINAL_PROMPT=0`，所以不会有任何东西等待输入；`PYTHONUTF8=1` 让 Python 的 I/O 保持 UTF-8。

shell 退出后，仍占用其管道的进程最多还有 2 s，且不超过超时；之后整个进程组（Windows 上是 job object）会被 kill，而 `exit:` 报告的仍是 shell 自身的退出码。后台服务只要不再占用这些管道就能继续运行，例如把输出重定向（`server > log 2>&1 &`）。超时会 kill 整个进程组；在 Unix 上，已脱离进程组的子进程不会被 kill。在 Unix 上，两种 kill 都使用 shell 内建的 `kill -9 -<pgid>`（精简容器里经常没有 `/bin/kill`）；Windows 上用 `TerminateJobObject`。
