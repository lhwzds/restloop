---
title: 修改循环
source: #bd944b6295f0
---

历史中总结出的约定：

- **选项用环境变量，默认值保持旧行为。** CLI 在 `config()` 中把它读进 `Config` 的一个字段；用 `env_num` 读取数值，这样无效值会回退到默认值；并把变量加到 `main()` 的 `--help` 文本和 README 里。
- **保持面向模型的部分固定不变**（见 [[overview]]）。修改 `system_prompt()` 或工具 schema 应附上 benchmark 证据。
- **保持 prompt 前缀稳定。** 任何改写早先消息的操作都会破坏服务商的 prompt 缓存；RestLoop 只在服务商报告溢出时才这样做。基于阈值的裁剪在一次 A/B 测试显示没有收益后已被移除（`f09603f4`）。
- **核心不碰环境，保持两个依赖。** `src/lib.rs` 不读任何环境变量、不启动任何进程；这些属于 CLI（`src/main.rs`）和命令执行器（`src/shell.rs`）。小巧和可读性正是意义所在。
