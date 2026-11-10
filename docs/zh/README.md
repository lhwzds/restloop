---
title: restloop
---

# restloop

<sub>[English](../README.md) · **中文**</sub>

这份文档比 README 更深入一层，介绍 RestLoop：一个小巧的 Rust agent 循环，只给模型提供一个 `shell` 工具。内容对应 v0.6.0：它新增了 `RESTLOOP_HEADERS`（额外的请求头），不再跟随重定向，并且在 key 无法放进 HTTP 请求头时结束运行。从 v0.5.0 起，任务结束时以及收到 SIGHUP、SIGINT 或 SIGTERM 时，任务的命令启动的进程组都会被停止，所以后台服务器不再活过它的任务（见概览）。从 v0.4.0 起，核心是一个库：`src/lib.rs` 根据 `Config` 和一个工具回调运行循环，可以单独使用；`src/shell.rs` 是默认的命令执行器，`src/main.rs` 是 CLI。建议按顺序阅读：RestLoop 是什么以及第一次运行；完整的命令行与环境变量接口；循环、HTTP 客户端和命令执行器如何工作以及为什么这样设计；最后是二进制如何构建和发布，以及如何修改代码。

## 目录

1. [概览](01-overview.md) — 刻意舍弃的部分 · 一次运行会留下什么 · 这种清理的限制 · 第一次运行
2. [用法与配置](02-usage.md) — 单个任务或 stdin 会话 · 环境变量 · 命令会继承什么 · 额外的请求头 · stdout 和 stderr 各输出什么 · 最终回复文件 · Session 文件：恢复与分叉
3. [工作原理](03-how-it-works.md) — 对话里有什么 · 请求与重试 · Responses API · 上下文窗口满了之后的恢复 · 清空旧的推理内容 · 时间预算 · 执行一条命令 · 模型从命令中看到什么
4. [构建、发布与贡献](04-contributing.md) — 测试覆盖了什么 · 修改循环 · 如何发布一个版本 · npm 包

## 附录

- [代码结构](structure.md) — 从代码生成：功能区、依赖、讲解页

<sub>生成自 https://github.com/lhwzds/restloop @ `e956437d9a`</sub>
