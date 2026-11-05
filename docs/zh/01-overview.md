---
title: 概览
---

# 概览

<sub>[English](../01-overview.md) · **中文**</sub>

RestLoop 是一个只剩下循环本身的 coding agent：它把对话和唯一的工具 `shell` 发给一个 OpenAI 兼容的 `/chat/completions` 端点（或 OpenAI 的 `/responses`），执行模型请求的每条命令，把输出发回去，直到模型不再调用工具、直接给出回答时停止。核心（循环和模型客户端）是 `src/lib.rs`，可以当作库使用；`src/shell.rs` 是默认的命令执行器，`src/main.rs` 是 CLI。仅有的依赖是用于 HTTPS 的 `ureq` 和 `serde_json`。

它的前身 RestFlow（到 0.5 为止）已经膨胀成一个 TUI、一个 daemon、一个 runner 加一套可执行 skill 系统，RestLoop 正是为此而生。commit `33511748` 把这些全部替换成一个文件、不做任何存储，`cad33d13` 将其改名为 RestLoop，版本号从 0.1.0 重新开始。之后的大多数功能（可配置的步数上限、时间预算、用量日志）都是针对 benchmark 运行（主要是 Terminal-Bench）加上的，每一项都是一个环境变量：时间预算和用量日志不设置就不启用，步数上限默认为 100 步。

<sub>来源: [lib.rs:1-9](../../src/lib.rs#L1-L9) · [main.rs:1-4](../../src/main.rs#L1-L4) · [shell.rs:1](../../src/shell.rs#L1) · [README.md:54-56](../../README.md#L54-L56) · [Cargo.toml:1-11](../../Cargo.toml#L1-L11) · +2</sub>

## 刻意舍弃的部分

- **只有一个工具。** 模型通过 `shell` 读、写、编辑文件；没有文件编辑、搜索或打补丁工具。它唯一的参数是 `command`。
- **不给模型可调的开关。** 来自 `system_prompt()` 的系统 prompt 和工具 schema 都是固定的。commit `3b27682c` 记录了命令超时不作为工具参数的原因：A/B 测试中，让模型自己调超时会让步数大约翻倍。
- **没有配置文件，也没有数据库。** 所有选项都是环境变量，由 CLI 在启动时一次读入核心的 `Config`（key 则在每次请求前读取）；RestLoop 自己只会写可选的 session 文件和最终回复文件（见 [用法与配置](02-usage.md)）。
- **没有沙箱，也没有确认。** 命令以你的权限在当前目录执行。隔离由调用方负责：用容器或一次性的 checkout。

<sub>来源: [lib.rs:59-63](../../src/lib.rs#L59-L63) · [lib.rs:170-181](../../src/lib.rs#L170-L181) · [main.rs:73-106](../../src/main.rs#L73-L106) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c)</sub>

## 第一次运行

可以用 Homebrew（`brew install lhwzds/tap/restloop`）、npm（`npm install -g restloop`）或 `cargo install --git https://github.com/lhwzds/restloop` 安装。必填的只有 `RESTLOOP_MODEL`。端点默认是 `https://api.openai.com/v1`，所以用 OpenAI 时只需一个 key 和一个模型名：

```bash
export RESTLOOP_API_KEY=sk-...
export RESTLOOP_MODEL=<model-id>
cd some-checkout
restloop "make the failing test pass"
```

用其他服务商时，把 `RESTLOOP_BASE_URL` 设为 `/chat/completions` 之前的那部分。退出码和所有选项见 [用法与配置](02-usage.md)。

<sub>来源: [README.md:40-47](../../README.md#L40-L47) · [README.md:52-60](../../README.md#L52-L60) · [main.rs:80-81](../../src/main.rs#L80-L81)</sub>
