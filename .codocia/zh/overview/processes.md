---
title: 一次运行会留下什么
source: #db61b6415024
---

任务的命令启动的任何东西都不应比任务活得更久。任务结束时（stdin 模式下每个任务结束时都如此），以及收到 SIGHUP、SIGINT 或 SIGTERM 时（RestLoop 随后以 129、130 或 143 退出），命令启动的每个进程组都会先收到 SIGTERM，大约一秒后收到 SIGKILL；任务的 deadline 会缩短这段等待。在 Windows 上则关闭 job object，这会 kill 其中的进程，没有优雅退出阶段。后台服务器在同一任务的各步之间会继续运行。

限制：

- 调用 `setsid` 或变成 daemon 的进程会逃脱；请用容器或 cgroup。
- 在 Unix 上同时最多存在 32 个进程组；再下一条命令会得到错误，而不会运行。
- 每条命令超过 1 MiB 的输出会被丢弃。
- 继承来的文件描述符不会传给命令；无法读取 `/dev/fd` 时，只处理编号不超过 1023 的。
- 同一进程内进程组 id 被复用，是一个已知的小缺口。

这些属于 CLI 的设置：把 `shell::shell()` 当库调用的程序要自己调用 `shell::install()` 和 `shell::stop_all()`。
