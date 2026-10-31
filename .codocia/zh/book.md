---
title: restloop
source: #e31d7375c727
pages:
  - overview: 概览
  - usage: 用法与配置
  - how-it-works: 工作原理
  - contributing: 构建、发布与贡献
---

这份文档比 README 更深入一层，介绍 RestLoop：一个单文件的 Rust agent 循环，只给模型提供一个 `shell` 工具。内容对应 v0.3.0。与 v0.2.0 相比，它新增了 Responses API（`RESTLOOP_WIRE_API=responses`）、`RESTLOOP_API_KEY_FILE` 和 `RESTLOOP_REASONING_EFFORT`；额度用完后不再重试，模型请求受 `RESTLOOP_DEADLINE` 限制，响应 body 读不出来时会重试，CI 也会检查这份文档。建议按顺序阅读：RestLoop 是什么以及第一次运行；完整的命令行与环境变量接口；循环、HTTP 客户端和命令执行器如何工作以及为什么这样设计；最后是二进制如何构建和发布，以及如何修改代码。
