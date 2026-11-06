---
title: restloop
source: #407321c6146b
pages:
  - overview: 概览
  - usage: 用法与配置
  - how-it-works: 工作原理
  - contributing: 构建、发布与贡献
---

这份文档比 README 更深入一层，介绍 RestLoop：一个小巧的 Rust agent 循环，只给模型提供一个 `shell` 工具。内容对应 v0.4.0。与 v0.3.2 相比，核心变成了一个库：`src/lib.rs` 根据 `Config` 和一个工具回调运行循环，可以单独使用；`src/shell.rs` 是默认的命令执行器，`src/main.rs` 是 CLI。二进制的行为与之前完全相同。概览中介绍的进程清理比 v0.4.0 更新，尚未发布。建议按顺序阅读：RestLoop 是什么以及第一次运行；完整的命令行与环境变量接口；循环、HTTP 客户端和命令执行器如何工作以及为什么这样设计；最后是二进制如何构建和发布，以及如何修改代码。
