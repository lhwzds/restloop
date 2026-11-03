---
title: restloop
source: #64505319e799
pages:
  - overview: 概览
  - usage: 用法与配置
  - how-it-works: 工作原理
  - contributing: 构建、发布与贡献
---

这份文档比 README 更深入一层，介绍 RestLoop：一个单文件的 Rust agent 循环，只给模型提供一个 `shell` 工具。内容对应 v0.3.1。与 v0.3.0 相比，模型的空回复会重试，而不是直接结束运行；把回复写入 `RESTLOOP_LAST_MESSAGE` 或清理旧推理时，只去掉开头的 `<think>` 块（标准输出打印完整回复）。建议按顺序阅读：RestLoop 是什么以及第一次运行；完整的命令行与环境变量接口；循环、HTTP 客户端和命令执行器如何工作以及为什么这样设计；最后是二进制如何构建和发布，以及如何修改代码。
