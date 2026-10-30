---
title: 用法与配置
---

# 用法与配置

<sub>[English](../02-usage.md) · **中文**</sub>

RestLoop 除了 `-h`/`--help`（打印环境变量列表）之外没有其他命令行选项。其余的一切就是任务文本和环境变量。本页列出完整的接口；[工作原理](03-how-it-works.md) 解释每个选项对循环有什么影响。

<sub>来源: [main.rs:15-27](../../src/main.rs#L15-L27)</sub>

## 单个任务或 stdin 会话

所有参数用空格拼接成一个任务：`restloop fix the build` 和 `restloop "fix the build"` 效果相同。

- **带任务时**，RestLoop 执行完就退出：模型正常完成返回状态 0，运行失败（重试后仍有 API 错误，或达到步数上限）返回 1。
- **不带任务时**，它在 stderr 上打印 `> `，然后从 stdin 每行读取一个任务，直到输入结束。空行会被跳过。所有任务共用一个对话，所以后面的行可以引用前面做过的事。某个任务失败时会在 stderr 上报告，循环继续。

状态 2 表示根本没有开始运行：`RESTLOOP_MODEL` 未设置，或者 `RESTLOOP_SESSION` 指向的文件不是 session 文件。

<sub>来源: [main.rs:15-27](../../src/main.rs#L15-L27) · [main.rs:29-62](../../src/main.rs#L29-L62)</sub>

## 环境变量

| 变量 | 默认值 | 作用 |
|---|---|---|
| `RESTLOOP_MODEL` | 必填 | 每个请求的 `model` 字段 |
| `RESTLOOP_API_KEY` | 空 | 以 `Authorization: Bearer <key>` 发送 |
| `RESTLOOP_API_KEY_FILE` | 未设置 | 改从这个文件读取 key，每次请求前都重新读取并去掉首尾空白，因此可以由另一个程序刷新短期 token |
| `RESTLOOP_BASE_URL` | `https://api.openai.com/v1` | 后面拼上 `/chat/completions`（或 `/responses`）；末尾的 `/` 会被忽略 |
| `RESTLOOP_WIRE_API` | chat completions | 设为 `responses` 时使用 OpenAI Responses API；其他值会被忽略 |
| `RESTLOOP_REASONING_EFFORT` | 未设置 | 例如 `low`，作为 `reasoning_effort`（或 `reasoning.effort`）发送 |
| `RESTLOOP_TIMEOUT` | `120` | 单条命令可运行的秒数 |
| `RESTLOOP_MAX_STEPS` | `100` | 每个任务的步数：一次模型回复及其命令（重试和溢出恢复都在同一步内重新请求） |
| `RESTLOOP_DEADLINE` | 未设置 | 整个任务的秒数；见 [工作原理](03-how-it-works.md) |
| `RESTLOOP_LOG_USAGE` | 未设置 | 设为 `1` 时在 stderr 上记录每条模型回复的 token 用量 |
| `RESTLOOP_LAST_MESSAGE` | 未设置 | 接收最终回复的文件 |
| `RESTLOOP_SESSION` | 未设置 | 要恢复并保存的对话文件 |

数值无法解析为无符号整数时，会静默回退到默认值（`env_num`）。`RESTLOOP_LOG_USAGE` 必须恰好是 `1`；对 key 文件和推理强度来说，空字符串等同于未设置。

<sub>来源: [main.rs:10-13](../../src/main.rs#L10-L13) · [main.rs:101-124](../../src/main.rs#L101-L124) · [main.rs:132-140](../../src/main.rs#L132-L140) · [main.rs:313-346](../../src/main.rs#L313-L346) · [commit f09603f4](https://github.com/lhwzds/restloop/commit/f09603f4)</sub>

## stdout 和 stderr 各输出什么

stdout 只输出模型的文本：每条非空回复都会去掉首尾空白后打印，包括模型在调用工具时顺带写的简短说明。步骤之间没有分隔符，所以只需要最终答案的脚本应该用 `RESTLOOP_LAST_MESSAGE`。

其余内容都走 stderr：每条命令以暗色的 `$ <command>` 显示，接着是耗时和输出的前 10 行；还有重试和上下文溢出的提示；设置 `RESTLOOP_LOG_USAGE=1` 时，每一步拿到回复后输出一行，取自该回复的 usage（重试的请求不单独记录）：

```
usage: step=3 prompt_tokens=8123 completion_tokens=412 cached_tokens=7936 elapsed=41s
```

服务商没有返回的字段打印为 `null`。

<sub>来源: [main.rs:151-165](../../src/main.rs#L151-L165) · [main.rs:189-214](../../src/main.rs#L189-L214) · [main.rs:144-147](../../src/main.rs#L144-L147)</sub>

## 最终回复文件

设置 `RESTLOOP_LAST_MESSAGE=<path>` 后，每个任务开始时先删除该文件；当模型不调用工具直接回复时，把回复文本加上末尾换行写入该文件。对于把推理内容内联在回复里的模型，最后一个 `</think>` 及其之前的内容都会被截掉。删除只是尽力而为（错误会被忽略），所以提前结束的运行（步数上限、API 错误、被 kill）通常不会留下文件，而不是留下上一次的答案；需要确定性时，每次运行都传一个新路径。

<sub>来源: [main.rs:134-139](../../src/main.rs#L134-L139) · [main.rs:178-187](../../src/main.rs#L178-L187) · [commit e8312d19](https://github.com/lhwzds/restloop/commit/e8312d19)</sub>

## Session 文件：恢复与分叉

`RESTLOOP_SESSION=<path>` 会把对话以 JSON 形式持久化：

```json
{"version": 1, "messages": [{"role": "system", "content": "..."}, ...]}
```

`messages` 就是下一次请求要发送的内容，采用 OpenAI chat 格式（使用 Responses API 时，是转换之前的内容）。`conversation()` 负责加载：文件不存在时开始新对话，并在第一步之后创建文件；文件已存在时，要求 `version` 为 1 且 `messages` 数组非空，否则 RestLoop 以状态 2 退出。除此之外不做任何检查，所以可以在两次运行之间编辑文件：合法的 chat 消息都会原样发送。编辑某条 assistant 消息后，要删掉它的 `output_items`，否则发出去的仍是它们。

用同一路径再次运行即可恢复；先复制文件就是分叉。保存的系统 prompt 原样复用。设置了 `RESTLOOP_DEADLINE` 时，恢复后的任务末尾会追加 `[You have Ns in total for this request; the time used in tool results counts from now.]`。每一步之后都会通过 `<path>.<pid>.tmp` 重命名来替换文件，所以进程被 kill 后，文件里保存的是最后一个完整的步骤。并发运行不会损坏文件，但最后写入的会覆盖前面的。

<sub>来源: [main.rs:28-29](../../src/main.rs#L28-L29) · [main.rs:63-80](../../src/main.rs#L63-L80) · [main.rs:220-233](../../src/main.rs#L220-L233) · [README.md:61-78](../../README.md#L61-L78) · [commit ffd11c37](https://github.com/lhwzds/restloop/commit/ffd11c37) · +2</sub>
