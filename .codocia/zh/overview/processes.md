---
title: 一次运行会留下什么
source: #df28effec29b
---

任务的命令启动的任何东西都不应比任务活得更久。任务结束时（stdin 模式下每个任务结束时都如此），以及处理到信号时，命令启动的每个进程组都会先收到 SIGTERM，大约一秒后，如果进程组还在，再收到 SIGKILL；任务的 deadline 会缩短这段等待。在 Unix 上，处理的信号是 SIGHUP、SIGINT 和 SIGTERM，除非继承时就被忽略（`nohup`），RestLoop 随后以 129、130 或 143 退出。在 Windows 上则关闭 job object，这会 kill 其中的进程，没有优雅退出阶段。后台服务器在同一任务的各步之间会继续运行。

这些属于 CLI 的设置：把 `shell::shell()` 当库调用的程序要自己调用 `shell::install()` 和 `shell::stop_all()`。
