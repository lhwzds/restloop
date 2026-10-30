---
title: restloop
source: #07592dd28b91
pages:
  - overview: 概览
  - usage: 用法与配置
  - how-it-works: 工作原理
  - contributing: 构建、发布与贡献
---

这份文档比 README 更深入一层，介绍 RestLoop：一个单文件的 Rust agent 循环，只给模型提供一个 `shell` 工具。内容对应 v0.2.0 之后的 `main`。与 v0.1.0 相比，v0.2.0 新增了上下文溢出恢复、`RESTLOOP_SESSION`、带 CI 的测试套件，以及对没有回复的 HTTP 200 响应的重试，并移除了 `RESTLOOP_TRIM_THINKING_AT`；`main` 又加入了 Responses API、key 文件和推理强度。建议按顺序阅读：RestLoop 是什么以及第一次运行；完整的命令行与环境变量接口；循环、HTTP 客户端和命令执行器如何工作以及为什么这样设计；最后是二进制如何构建和发布，以及如何修改代码。
