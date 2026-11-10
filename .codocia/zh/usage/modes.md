---
title: 单个任务或 stdin 会话
source: #0a71febf4095
---

所有参数用空格拼接成一个任务：`restloop fix the build` 和 `restloop "fix the build"` 效果相同。

- **带任务时**，RestLoop 执行完就退出：模型正常完成返回状态 0，运行失败（重试后仍有 API 错误、key 出错，或达到步数上限）返回 1。
- **不带任务时**，它在 stderr 上打印 `> `，然后从 stdin 每行读取一个任务，跳过空行。所有任务共用一个对话，所以后面的行可以引用前面做过的事。某个任务失败时会在 stderr 上报告，循环继续。

状态 2 表示根本没有开始运行：`RESTLOOP_MODEL` 未设置，`RESTLOOP_HEADERS` 不是有效的 UTF-8，或其中有一行不合法，或者 `RESTLOOP_SESSION` 指向的文件不是 session 文件。在 Unix 上，SIGHUP、SIGINT 和 SIGTERM 分别对应 129、130 和 143（[[overview]]）。
