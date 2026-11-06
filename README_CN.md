# RestLoop

[English](README.md) | 中文

一个极简的 coding agent 循环：它替你持续推进任务，你可以去休息。
不计测试约 1260 行 Rust，其中约 520 行是 `src/lib.rs` 中的核心，其余是 CLI 和默认的命令
执行器，打包成约 1.5 MB 的二进制。

核心是 `src/lib.rs`：agent 循环和模型客户端。它不直接读取环境变量（其 HTTP 客户端 `ureq` 会遵循 `*_PROXY`），也不启动任何进程：
设置通过 `Config` 传入，命令交给你传入的执行器。它会写 stderr：每条命令和简短结果、
重试、上下文溢出提示，以及保存 session 或最终回复文件失败的错误；每条回复的文本写到
stdout。可以把它当作库使用，并提供你自己的命令执行器：

```rust
use restloop::serde_json::json;
let cfg = restloop::Config { model: "gpt-4o".into(), base_url: "https://api.openai.com/v1".into(),
    key: Box::new(|| Ok(std::env::var("OPENAI_API_KEY").unwrap_or_default())),
    responses: false, reasoning_effort: None, max_steps: 100, timeout: 120,
    deadline: None, start: std::time::Instant::now(), log_usage: false,
    last_message: None, session: None };
let cwd = std::env::current_dir().unwrap_or_default().display().to_string();
let mut messages = vec![
    json!({"role": "system", "content": restloop::system_prompt(&cfg, "sh", std::env::consts::OS, &cwd)}),
    json!({"role": "user", "content": "fix the failing test"})];
restloop::run(&cfg, &mut messages, &mut |cmd, secs| my_runner(cmd, secs))?;
```

RestLoop 是一个个人开源项目。它只给模型一个工具 `shell`，
并循环执行，直到模型不再调用这个工具。模型通过 shell 命令读、写、编辑文件，
做法与 mini-swe-agent 相同。

| | |
|---|---|
| 源码 | `src/lib.rs`（核心），`src/shell.rs`（默认的命令执行器，公开模块），`src/main.rs`（CLI） |
| 测试 | `src/tests.rs`、`tests/cleanup.rs`（Unix）；CI 在 macOS、Linux 和 Windows 上运行 |
| 依赖 | `ureq`（HTTPS）、`serde_json`、`libc`（Unix） |
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
   否则用 PowerShell。命令运行 120 s 后，它的进程组（Windows 上是 job）会被 kill
   （可用 `RESTLOOP_TIMEOUT` 修改，例如用于耗时较长的构建）；在 Unix 上，已脱离进程组的
   子进程不会被 kill。把退出码和输出
   （截断到 10,000 个字符）发回给模型。
3. 重复以上步骤，直到模型不调用工具直接回复，或达到 100 步
   （`RESTLOOP_MAX_STEPS`）。

一次运行结束时（stdin 模式下是每个任务结束时），以及处理到 SIGHUP、SIGINT 或 SIGTERM 时（仅 Unix；继承来的忽略设置保持不变，RestLoop 随后以
129、130 或 143 退出），它的命令启动的每个进程组都会先收到 SIGTERM，大约一秒后，如果进程组
还在，再收到 SIGKILL；任务的 deadline 会缩短这段等待。在 Windows 上则关闭 job object，这会 kill
其中的进程，没有优雅退出阶段。命令在后台启动的服务器，在同一次运行的各步之间会继续运行。

限制：
- 在 Unix 上，调用 setsid 或变成 daemon 的进程会逃脱（请用容器或 cgroup）；
- 在 Unix 上，同时最多存在 32 个进程组，再下一条命令会被拒绝；
- 每条命令超过 1 MiB 的输出会被丢弃；
- 无法读取 `/dev/fd` 时，只把编号不超过 1023 的文件描述符标记为 close-on-exec（标记失败会被忽略）；
- 同一进程内进程组 id 被复用，是一个已知的小缺口；
- 通过 npm 安装时，只发给启动器（`npm/bin/restloop.js`）的 SIGTERM 会杀掉 node，而 RestLoop 和它的命令会继续运行。

因 429、5xx、网络错误或没有回复的 HTTP 200 而失败的请求，
总共最多尝试 5 次，除非错误表明额度或套餐限额已经用完。
空回复（没有工具调用，除开头的 `<think>` 块外也没有文字）同样会这样重试，即使响应里提到了这类限额；
如果第五次回复仍为空，它就是最终回复。
Responses 事件流中途断开时会重试；流中报告了失败时，只有服务器错误、限流或套餐暂时不可用才会重试。
当对话超出模型的上下文窗口时，较早的一半命令输出会被缩减为第一行加一条说明
（并丢弃旧的推理内容），然后重新发送请求。

没有沙箱，也没有确认：命令以你的权限在当前目录执行。
如果这一点很重要，请在容器或一次性的 checkout 中运行。

RestLoop 自己的设置不再传入命令的环境：任何 `RESTLOOP_*` 变量都不会传给命令
（key、key 文件、URL 以及其他任何 `RESTLOOP_*` 名称；在 Windows 上匹配不区分大小写）。
其他变量命令都会继承，包括以其他名字保存的 key 副本。
以同一用户运行的命令仍然可以读到 RestLoop 的环境（例如 `/proc/$PPID/environ` 或 `ps -E`），
知道路径时也能读 key 文件；真正的解决办法是沙箱或换一个用户。

## 历史

RestLoop 直到 0.5 版本（含）都叫 RestFlow，是一个大得多的 agent CLI，
带 TUI、daemon 和可执行 skill；那些代码保留在这次重写之前的 git 历史中
（commit `6d3f49b5`）。新名字保留了其中的“Rest”：循环负责干活，你负责休息。

## 许可证

Apache-2.0
