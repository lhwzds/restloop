---
title: 概览
---

# 概览

<sub>[English](../01-overview.md) · **中文**</sub>

RestLoop 是一个只剩下循环本身的 coding agent：它把对话和唯一的工具 `shell` 发给一个 OpenAI 兼容的 `/chat/completions` 端点（或 OpenAI 的 `/responses`），执行模型请求的每条命令，把输出发回去，直到模型不再调用工具、直接给出回答时停止。核心（循环和模型客户端）是 `src/lib.rs`，可以当作库使用；`src/shell.rs` 是默认的命令执行器，`src/main.rs` 是 CLI。仅有的依赖是用于 HTTPS 的 `ureq`、`serde_json`，以及 Unix 上的 `libc`。

它的前身 RestFlow（到 0.5 为止）已经膨胀成一个 TUI、一个 daemon、一个 runner 加一套可执行 skill 系统，RestLoop 正是为此而生。commit `33511748` 把这些全部替换成一个文件、不做任何存储，`cad33d13` 将其改名为 RestLoop，版本号从 0.1.0 重新开始。之后的大多数功能（可配置的步数上限、时间预算、用量日志）都是针对 benchmark 运行（主要是 Terminal-Bench）加上的，每一项都是一个环境变量：时间预算和用量日志不设置就不启用，步数上限默认为 100 步。

<sub>来源: [lib.rs:1-9](../../src/lib.rs#L1-L9) · [main.rs:1-4](../../src/main.rs#L1-L4) · [shell.rs:1](../../src/shell.rs#L1) · [README.md:54-56](../../README.md#L54-L56) · [Cargo.toml:1-15](../../Cargo.toml#L1-L15) · +2</sub>

## 刻意舍弃的部分

- **只有一个工具。** 模型通过 `shell` 读、写、编辑文件；没有文件编辑、搜索或打补丁工具。它唯一的参数是 `command`。
- **不给模型可调的开关。** 来自 `system_prompt()` 的系统 prompt 和工具 schema 都是固定的。commit `3b27682c` 记录了命令超时不作为工具参数的原因：A/B 测试中，让模型自己调超时会让步数大约翻倍。
- **没有配置文件，也没有数据库。** 所有选项都是环境变量，由 CLI 在启动时一次读入核心的 `Config`（key 则在每次请求前读取）；RestLoop 自己只会写可选的 session 文件和最终回复文件（见 [用法与配置](02-usage.md)）。
- **没有沙箱，也没有确认。** 命令以你的权限在当前目录执行。隔离由调用方负责：用容器或一次性的 checkout。

<sub>来源: [lib.rs:59-63](../../src/lib.rs#L59-L63) · [lib.rs:170-181](../../src/lib.rs#L170-L181) · [main.rs:81-114](../../src/main.rs#L81-L114) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c)</sub>

## 一次运行会留下什么

任务的命令启动的任何东西都不应比任务活得更久。任务结束时（stdin 模式下每个任务结束时都如此），以及收到 SIGHUP、SIGINT 或 SIGTERM 时（RestLoop 随后以 129、130 或 143 退出），命令启动的每个进程组都会先收到 SIGTERM，大约一秒后收到 SIGKILL；任务的 deadline 会缩短这段等待。在 Windows 上则关闭 job object，这会 kill 其中的进程，没有优雅退出阶段。后台服务器在同一任务的各步之间会继续运行。

限制：

- 调用 `setsid` 或变成 daemon 的进程会逃脱；请用容器或 cgroup。
- 在 Unix 上同时最多存在 32 个进程组；再下一条命令会得到错误，而不会运行。
- 每条命令超过 1 MiB 的输出会被丢弃。
- 继承来的文件描述符不会传给命令；无法读取 `/dev/fd` 时，只处理编号不超过 1023 的。
- 同一进程内进程组 id 被复用，是一个已知的小缺口。

这些属于 CLI 的设置：把 `shell::shell()` 当库调用的程序要自己调用 `shell::install()` 和 `shell::stop_all()`。

<sub>来源: [main.rs:30-34](../../src/main.rs#L30-L34) · [main.rs:56-57](../../src/main.rs#L56-L57) · [shell.rs:56-75](../../src/shell.rs#L56-L75) · [shell.rs:229-232](../../src/shell.rs#L229-L232) · [shell.rs:354-360](../../src/shell.rs#L354-L360) · +2</sub>

## 第一次运行

可以用 Homebrew（`brew install lhwzds/tap/restloop`）、npm（`npm install -g restloop`）或 `cargo install --git https://github.com/lhwzds/restloop` 安装。必填的只有 `RESTLOOP_MODEL`。端点默认是 `https://api.openai.com/v1`，所以用 OpenAI 时只需一个 key 和一个模型名：

```bash
export RESTLOOP_API_KEY=sk-...
export RESTLOOP_MODEL=<model-id>
cd some-checkout
restloop "make the failing test pass"
```

用其他服务商时，把 `RESTLOOP_BASE_URL` 设为 `/chat/completions` 之前的那部分。退出码和所有选项见 [用法与配置](02-usage.md)。

<sub>来源: [README.md:40-47](../../README.md#L40-L47) · [README.md:52-60](../../README.md#L52-L60) · [main.rs:88-89](../../src/main.rs#L88-L89)</sub>
