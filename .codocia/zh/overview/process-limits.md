---
title: 这种清理的限制
source: #243856142f1c
---

- 在 Unix 上，调用 `setsid` 或变成 daemon 的进程会逃脱；请用容器或 cgroup。
- 在 Unix 上同时最多存在 32 个进程组；再下一条命令会得到错误，而不会运行。
- 每条命令超过 1 MiB 的输出会被丢弃。
- 继承来的文件描述符会被标记为 close-on-exec，标记失败会被忽略；无法读取 `/dev/fd` 时，只标记编号不超过 1023 的。
- 同一进程内进程组 id 被复用，是一个已知的小缺口。
- 通过 npm 安装时，只发给启动器（`npm/bin/restloop.js`）的 SIGTERM 会杀掉 node，而 RestLoop 和它的命令会继续运行。
