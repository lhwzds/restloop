# RestLoop

[English](README.md) | 中文

一个极简的 coding agent 循环：它替你持续推进任务，你可以去休息。
约 650 行 Rust，打包成约 1.5 MB 的二进制。

RestLoop 是一个个人开源项目。它只给模型一个工具 `shell`，
并循环执行，直到模型不再调用这个工具。模型通过 shell 命令读、写、编辑文件，
做法与 mini-swe-agent 相同。

| | |
|---|---|
| 源码 | `src/main.rs`，约 650 行 |
| 测试 | `src/tests.rs`；CI 在 macOS、Linux 和 Windows 上运行 |
| 依赖 | `ureq`（HTTPS）、`serde_json` |
| 二进制（release） | macOS arm64 1.3 MB，Linux x86_64 1.9 MB（静态），Windows x86_64 1.5 MB |
| 存储 | 默认无：没有数据库，没有配置文件；只有设置 `RESTLOOP_SESSION` 或 `RESTLOOP_LAST_MESSAGE` 时才写文件 |

## 安装

```bash
brew install lhwzds/tap/restloop                       # macOS、Linux
npm install -g restloop                                # 首次运行时下载二进制
cargo install --git https://github.com/lhwzds/restloop
```

也可以从 [Releases](https://github.com/lhwzds/restloop/releases) 下载二进制
（macOS、Linux x86_64 和 arm64、Windows）；`SHA256SUMS` 列出了它们的校验和。

## 使用

RestLoop 可以对接任何 OpenAI 兼容的 `/chat/completions` 端点
（OpenAI、Anthropic 的兼容端点、OpenRouter、GLM、DeepSeek、MiniMax、Ollama 等），
设置 `RESTLOOP_WIRE_API=responses` 时也可以对接 OpenAI 的 `/responses` 端点。

```bash
export RESTLOOP_BASE_URL=https://api.openai.com/v1   # 默认值
export RESTLOOP_API_KEY=...
export RESTLOOP_MODEL=...
export RESTLOOP_API_KEY_FILE=key.txt                 # 可选：每次请求都从这个文件读取 key
export RESTLOOP_WIRE_API=responses                   # 可选：用 Responses API 代替 chat completions
export RESTLOOP_REASONING_EFFORT=low                 # 可选：作为请求的推理强度发送
export RESTLOOP_TIMEOUT=120                          # 每条命令的秒数，可选
export RESTLOOP_MAX_STEPS=100                        # 每个任务的步数（一次回复及其命令），可选
export RESTLOOP_LOG_USAGE=1                          # 在 stderr 上输出每条模型回复的 token 用量，可选
export RESTLOOP_DEADLINE=1800                        # 整个任务的秒数，可选；模型能看到已用时间
export RESTLOOP_LAST_MESSAGE=last.txt                # 接收最终回复的文件，可选
export RESTLOOP_SESSION=task.json                    # 可选：继续这个对话，每一步之后保存

restloop "fix the failing test in this repo"   # 执行一个任务后退出
restloop                                       # 从 stdin 读取任务，保留对话
```

assistant 的文本输出到 stdout；命令及其输出写入 stderr。

任务有时间限制时，把 `RESTLOOP_DEADLINE` 设为这个限制。在 Terminal-Bench
（30 个任务，MiniMax-M3.1-Flash）上，这让 RestLoop 解决的任务从 20 个增加到 24 个，
超时从 5 次减少到 1 次。模型请求同样会在 deadline 处被截断，之后 RestLoop 以
`restloop: deadline of <N>s reached` 结束。

`RESTLOOP_SESSION` 让对话跨运行保留。如果文件存在，RestLoop 会带着新任务
继续其中的对话；每一步之后都会把对话写回去（`{"version": 1, "messages": [...]}`，
与下一次请求要发送的内容完全一致；使用 Responses API 时，是转换之前的内容）。用同一个文件
再次运行即可恢复；先复制文件就是分叉。保存的系统 prompt 原样复用，因此服务商的 prompt
缓存仍能命中；恢复运行时，`RESTLOOP_DEADLINE` 会随新任务一起告知模型。
同一时间一个 session 文件只用于一个运行。可以在两次运行之间编辑该文件（裁剪、
改写、合并）；只要是 version 1 下合法的 chat 消息都能加载。

设置 `RESTLOOP_WIRE_API=responses` 时，对话仍以 chat 消息的形式保存在内存和文件中，
每次请求时再转换成 Responses 的输入（`store: false`，流式）。assistant 消息还会在
`output_items` 中原样保留模型的输出（包括加密的推理内容），发回给模型的正是这些（只去掉 message 和 function_call 的 id 字段）；
没有 `output_items` 的消息则以其文本和工具调用发回（所以编辑某条消息后，要删掉它们）。
`RESTLOOP_API_KEY_FILE` 适合短期 token，例如 Sign in with ChatGPT 的 token：
由另一个程序刷新该文件，RestLoop 每次请求都会读取它。RestLoop 自己不负责登录，也不刷新 token。

## 工作原理

1. 把对话和 `shell` 工具发送给模型。
2. 执行每条请求的命令：不提供 stdin，在当前目录下的全新 shell 中运行。
   macOS/Linux 上用 `sh`；Windows 上安装了 Git Bash 就用它，
   否则用 PowerShell。命令运行 120 s 后会连同它启动的所有进程一起被 kill
   （可用 `RESTLOOP_TIMEOUT` 修改，例如用于耗时较长的构建）。把退出码和输出
   （截断到 10,000 个字符）发回给模型。
3. 重复以上步骤，直到模型不调用工具直接回复，或达到 100 步
   （`RESTLOOP_MAX_STEPS`）。

因 429、5xx、网络错误或没有回复的 HTTP 200 而失败的请求，
总共最多尝试 5 次，除非错误表明额度或套餐限额已经用完。
Responses 事件流中途断开时会重试；流中报告了失败时，只有服务器错误、限流或套餐暂时不可用才会重试。
当对话超出模型的上下文窗口时，较早的一半命令输出会被缩减为第一行加一条说明
（并丢弃旧的推理内容），然后重新发送请求。

没有沙箱，也没有确认：命令以你的权限在当前目录执行。
如果这一点很重要，请在容器或一次性的 checkout 中运行。

## 历史

RestLoop 直到 0.5 版本（含）都叫 RestFlow，是一个大得多的 agent CLI，
带 TUI、daemon 和可执行 skill；那些代码保留在这次重写之前的 git 历史中
（commit `6d3f49b5`）。新名字保留了其中的“Rest”：循环负责干活，你负责休息。

## 许可证

Apache-2.0
