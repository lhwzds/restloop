---
title: restloop
---

# restloop

<sub>[English](../README.md) · **中文**</sub>

这份文档比 README 更深入一层，介绍 RestLoop：一个单文件的 Rust agent 循环，只给模型提供一个 `shell` 工具。内容对应 v0.3.0。与 v0.2.0 相比，它新增了 Responses API（`RESTLOOP_WIRE_API=responses`）、`RESTLOOP_API_KEY_FILE` 和 `RESTLOOP_REASONING_EFFORT`；额度用完后不再重试，模型请求受 `RESTLOOP_DEADLINE` 限制，响应 body 读不出来时会重试，CI 也会检查这份文档。建议按顺序阅读：RestLoop 是什么以及第一次运行；完整的命令行与环境变量接口；循环、HTTP 客户端和命令执行器如何工作以及为什么这样设计；最后是二进制如何构建和发布，以及如何修改代码。

## 目录

1. [概览](01-overview.md) — 刻意舍弃的部分 · 第一次运行
2. [用法与配置](02-usage.md) — 单个任务或 stdin 会话 · 环境变量 · stdout 和 stderr 各输出什么 · 最终回复文件 · Session 文件：恢复与分叉
3. [工作原理](03-how-it-works.md) — 对话里有什么 · 请求与重试 · Responses API · 上下文窗口满了之后的恢复 · 清空旧的推理内容 · 时间预算 · 执行一条命令 · 模型从命令中看到什么
4. [构建、发布与贡献](04-contributing.md) — 测试覆盖了什么 · 修改循环 · 如何发布一个版本 · npm 包

## 附录

- [代码结构](structure.md) — 从代码生成：功能区、依赖、讲解页

<sub>生成自 https://github.com/lhwzds/restloop @ `08ea2c7acc`</sub>
