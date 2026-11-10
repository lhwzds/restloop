---
title: restloop
source: #90a20238e2ba
pages:
  - overview: 概览
  - usage: 用法与配置
  - how-it-works: 工作原理
  - contributing: 构建、发布与贡献
---

这份文档比 README 更深入一层，介绍 RestLoop：一个小巧的 Rust agent 循环，只给模型提供一个 `shell` 工具。内容对应 v0.6.0：它新增了 `RESTLOOP_HEADERS`（额外的请求头），不再跟随重定向，并且在 key 无法放进 HTTP 请求头时结束运行。从 v0.5.0 起，任务结束时以及收到 SIGHUP、SIGINT 或 SIGTERM 时，任务的命令启动的进程组都会被停止，所以后台服务器不再活过它的任务（见概览）。从 v0.4.0 起，核心是一个库：`src/lib.rs` 根据 `Config` 和一个工具回调运行循环，可以单独使用；`src/shell.rs` 是默认的命令执行器，`src/main.rs` 是 CLI。建议按顺序阅读：RestLoop 是什么以及第一次运行；完整的命令行与环境变量接口；循环、HTTP 客户端和命令执行器如何工作以及为什么这样设计；最后是二进制如何构建和发布，以及如何修改代码。
