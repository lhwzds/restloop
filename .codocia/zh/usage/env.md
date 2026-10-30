---
title: 环境变量
source: #61f49a2c40b4
---

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
| `RESTLOOP_DEADLINE` | 未设置 | 整个任务的秒数；见 [[how-it-works]] |
| `RESTLOOP_LOG_USAGE` | 未设置 | 设为 `1` 时在 stderr 上记录每条模型回复的 token 用量 |
| `RESTLOOP_LAST_MESSAGE` | 未设置 | 接收最终回复的文件 |
| `RESTLOOP_SESSION` | 未设置 | 要恢复并保存的对话文件 |

数值无法解析为无符号整数时，会静默回退到默认值（`env_num`）。`RESTLOOP_LOG_USAGE` 必须恰好是 `1`；对 key 文件和推理强度来说，空字符串等同于未设置。
