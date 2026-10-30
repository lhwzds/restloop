---
title: 
source: #55c4bffbb0e1
---

RestLoop 是一个只剩下循环本身的 coding agent：它把对话和唯一的工具 `shell` 发给一个 OpenAI 兼容的 `/chat/completions` 端点（或 OpenAI 的 `/responses`），执行模型请求的每条命令，把输出发回去，直到模型不再调用工具、直接给出回答时停止。整个程序就是 `src/main.rs`；仅有的依赖是用于 HTTPS 的 `ureq` 和 `serde_json`。

它的前身 RestFlow（到 0.5 为止）已经膨胀成一个 TUI、一个 daemon、一个 runner 加一套可执行 skill 系统，RestLoop 正是为此而生。commit `33511748` 把这些全部替换成一个文件、不做任何存储，`cad33d13` 将其改名为 RestLoop，版本号从 0.1.0 重新开始。之后的大多数功能（可配置的步数上限、时间预算、用量日志）都是针对 benchmark 运行（主要是 Terminal-Bench）加上的，每一项都是一个环境变量：时间预算和用量日志不设置就不启用，步数上限默认为 100 步。
