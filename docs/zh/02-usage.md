---
title: 用法与配置
---

# 用法与配置

<sub>[English](../02-usage.md) · **中文**</sub>

RestLoop 除了 `-h`/`--help`（打印环境变量列表）之外没有其他命令行选项。其余的一切就是任务文本和环境变量；[工作原理](03-how-it-works.md) 解释每个选项对循环有什么影响。

<sub>来源: [main.rs:17-25](../../src/main.rs#L17-L25)</sub>

## 单个任务或 stdin 会话

所有参数用空格拼接成一个任务：`restloop fix the build` 和 `restloop "fix the build"` 效果相同。

- **带任务时**，RestLoop 执行完就退出：模型正常完成返回状态 0，运行失败（重试后仍有 API 错误、key 出错，或达到步数上限）返回 1。
- **不带任务时**，它在 stderr 上打印 `> `，然后从 stdin 每行读取一个任务，跳过空行。所有任务共用一个对话，所以后面的行可以引用前面做过的事。某个任务失败时会在 stderr 上报告，循环继续。

状态 2 表示根本没有开始运行：`RESTLOOP_MODEL` 未设置，`RESTLOOP_HEADERS` 不是有效的 UTF-8，或其中有一行不合法，或者 `RESTLOOP_SESSION` 指向的文件不是 session 文件。在 Unix 上，SIGHUP、SIGINT 和 SIGTERM 分别对应 129、130 和 143（[概览](01-overview.md)）。

<sub>来源: [main.rs:17-36](../../src/main.rs#L17-L36) · [main.rs:76-90](../../src/main.rs#L76-L90) · [shell.rs:390-394](../../src/shell.rs#L390-L394) · [main.rs:26-37](../../src/main.rs#L26-L37)</sub>

## 环境变量

| 变量 | 默认值 | 作用 |
|---|---|---|
| `RESTLOOP_MODEL` | 必填 | 每个请求的 `model` 字段 |
| `RESTLOOP_API_KEY` | 空 | 以 `Authorization: Bearer <key>` 发送 |
| `RESTLOOP_API_KEY_FILE` | 未设置 | 改从这个文件读取 key，每次请求前都重新读取并去掉首尾空白，因此短期 token 可以被刷新 |
| `RESTLOOP_BASE_URL` | `https://api.openai.com/v1` | 后面拼上 `/chat/completions`（或 `/responses`）；末尾的 `/` 会被忽略 |
| `RESTLOOP_WIRE_API` | chat completions | 设为 `responses` 时使用 OpenAI Responses API；其他值会被忽略 |
| `RESTLOOP_REASONING_EFFORT` | 未设置 | 例如 `low`，作为 `reasoning_effort`（或 `reasoning.effort`）发送 |
| `RESTLOOP_HEADERS` | 未设置 | 额外的请求头；见下文 |
| `RESTLOOP_TIMEOUT` | `120` | 单条命令可运行的秒数 |
| `RESTLOOP_MAX_STEPS` | `100` | 每个任务的步数：一次模型回复及其命令（重试和溢出恢复都算在同一步内） |
| `RESTLOOP_DEADLINE` | 未设置 | 整个任务的秒数；见 [工作原理](03-how-it-works.md) |
| `RESTLOOP_LOG_USAGE` | 未设置 | 设为 `1` 时在 stderr 上记录每条模型回复的 token 用量 |
| `RESTLOOP_LAST_MESSAGE` | 未设置 | 接收最终回复的文件 |
| `RESTLOOP_SESSION` | 未设置 | 要恢复并保存的对话文件 |

无效的数值（不是无符号整数）会静默回退到默认值（`env_num`）。`RESTLOOP_LOG_USAGE` 必须恰好是 `1`；对 key 文件和推理强度来说，空字符串等同于未设置。

<sub>来源: [main.rs:12-15](../../src/main.rs#L12-L15) · [main.rs:93-128](../../src/main.rs#L93-L128) · [main.rs:169-177](../../src/main.rs#L169-L177) · [commit f09603f4](https://github.com/lhwzds/restloop/commit/f09603f4)</sub>

## 命令会继承什么

RestLoop 自己的设置不会传入命令：命令启动前会去掉所有 `RESTLOOP_*` 名称（`restloop_var`），所以 key、key 文件、URL 和请求头都不会传给命令；在 Windows 上匹配不区分大小写。其他变量命令都会继承，包括以其他名字保存的 key 副本。以同一用户运行的命令仍然可以读到 restloop 的环境（`/proc/$PPID/environ`、`ps -E`），知道路径时也能读 key 文件；真正的解决办法是沙箱或换一个用户。

<sub>来源: [shell.rs:29-42](../../src/shell.rs#L29-L42) · [shell.rs:130-135](../../src/shell.rs#L130-L135) · [commit 8d5e3a78](https://github.com/lhwzds/restloop/commit/8d5e3a78)</sub>

## 额外的请求头

`RESTLOOP_HEADERS` 存放额外的请求头，每行一个 `Name: value`（跳过空行，CRLF 也可以），随每个模型请求一起发送；同一个名字写两次就发两次。例如 OpenCode Go 需要 `x-opencode-session: <id>`。以下名字不论大小写都会被拒绝：`Authorization`、`Proxy-Authorization`、`Content-Type`、`Content-Length`、`Host`、`Transfer-Encoding`、`Accept-Encoding`；其他合法的名字都会发送。某一行不合法（没有 `:`、请求头无效或被拒绝）或文本不是 UTF-8 时，程序在一开始就以 2 退出。错误信息给出行号（`RESTLOOP_HEADERS: line 2 has no ':'`），名字被拒绝时还给出该名字，但从不包含值；UTF-8 错误没有行号。作为库调用时设置 `Config.headers`，由 `run()` 先拒绝（`restloop::bad_header()`）。

<sub>来源: [main.rs:26-37](../../src/main.rs#L26-L37) · [main.rs:179-201](../../src/main.rs#L179-L201) · [lib.rs:46-75](../../src/lib.rs#L46-L75) · [lib.rs:82-92](../../src/lib.rs#L82-L92)</sub>

## stdout 和 stderr 各输出什么

stdout 只输出模型的文本：每条非空回复都会去掉首尾空白后打印，包括模型在调用工具时顺带写的说明。步骤之间没有分隔符；只需要最终答案时，用 `RESTLOOP_LAST_MESSAGE`。

其余内容都走 stderr：每条命令以暗色的 `$ <command>` 显示，接着是耗时和输出的前 10 行；还有重试和上下文溢出的提示；设置 `RESTLOOP_LOG_USAGE=1` 时，每一步拿到回复后输出一行，取自该回复的 usage（重试的请求不记录）：

```
usage: step=3 prompt_tokens=8123 completion_tokens=412 cached_tokens=7936 elapsed=41s
```

服务商没有返回的字段打印为 `null`。这些输出来自核心的 `run()`，所以作为库调用的程序也会得到它们；只有 stdin 提示符 `> ` 属于 CLI。默认的命令执行器在对仍存在的进程组发送 SIGKILL 失败时，会输出 `restloop: cannot stop a process group`。

<sub>来源: [shell.rs:270-277](../../src/shell.rs#L270-L277) · [lib.rs:77-81](../../src/lib.rs#L77-L81) · [lib.rs:115-129](../../src/lib.rs#L115-L129) · [lib.rs:156-157](../../src/lib.rs#L156-L157) · [lib.rs:164-168](../../src/lib.rs#L164-L168) · +2</sub>

## 最终回复文件

设置 `RESTLOOP_LAST_MESSAGE=<path>` 后，每个任务开始时先删除该文件（除非 RestLoop 已先以状态 2 退出），然后当模型不调用工具直接回复时，把回复文本加上末尾换行写入该文件。开头的 `<think>` 块（内联的推理内容）会被截掉，截到第一个 `</think>` 为止（没有闭合时什么都不剩）；其他回复即使引用了 `</think>` 也会完整写入。两种情况都会去掉首尾空白。删除只是尽力而为，所以提前结束的运行（步数上限、API 错误、被 kill）通常不会留下文件，而不是留下上一次的答案；要确定的话，每次运行都传一个新路径。

<sub>来源: [lib.rs:98-102](../../src/lib.rs#L98-L102) · [lib.rs:142-150](../../src/lib.rs#L142-L150) · [lib.rs:286-294](../../src/lib.rs#L286-L294) · [commit e8312d19](https://github.com/lhwzds/restloop/commit/e8312d19) · [main.rs:26-37](../../src/main.rs#L26-L37) · +1</sub>

## Session 文件：恢复与分叉

`RESTLOOP_SESSION=<path>` 会把对话以 JSON 形式持久化：

```json
{"version": 1, "messages": [{"role": "system", "content": "..."}, ...]}
```

`messages` 就是下一次请求要发送的内容，采用 OpenAI chat 格式（在任何 Responses 转换之前）。`conversation()` 负责加载：文件不存在时开始新对话，并在第一步之后创建文件；文件已存在时，要求 `version` 为 1 且 `messages` 数组非空，否则 RestLoop 以状态 2 退出。除此之外不做任何检查，所以可以在两次运行之间编辑文件。编辑某条 assistant 消息后，要删掉它的 `output_items`，否则发出去的仍是它们。

用同一路径再次运行即可恢复；先复制文件就是分叉。保存的系统 prompt 原样复用。设置了 `RESTLOOP_DEADLINE` 时，恢复后的任务末尾会追加 `[You have Ns in total for this request; the time used in tool results counts from now.]`。每一步之后都会通过 `<path>.<pid>.tmp` 重命名来替换文件，所以进程被 kill 后，文件里保存的是最后一个完整的步骤；并发运行不会损坏文件，最后写入的胜出。

<sub>来源: [main.rs:54-64](../../src/main.rs#L54-L64) · [main.rs:130-153](../../src/main.rs#L130-L153) · [lib.rs:188-202](../../src/lib.rs#L188-L202) · [README.md:84-102](../../README.md#L84-L102) · [commit ffd11c37](https://github.com/lhwzds/restloop/commit/ffd11c37) · +1</sub>
