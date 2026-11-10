---
title: 请求与重试
source: #5103169e35bd
---

`complete()` 向 `/chat/completions` POST `{"model", "messages", "tools"}`，设置了推理强度时再加上 `reasoning_effort`，没有采样参数、token 上限或流式输出。每次尝试都会重新读取 key；key 文件无法读取时以 `cannot read <path>` 失败，key 内部含换行时以 `the key cannot go in a header` 失败。不跟随重定向（3xx 即失败），所以额外的请求头（[[usage]]）永远不会发到另一台主机。

网络错误、HTTP 429、5xx、没有 `choices[0].message` 的 200（MiniMax 的错误，`333b7b64`），以及空回复（没有工具调用，除开头的 `<think>` 外也没有文字）都会重试：总共五次尝试，中间分别等待 2、4、8、16 s，并打印 `restloop: <what>, retrying`；第五次仍为空的回复即为最终回复。body 中出现 `insufficient_quota` 或 `subscription_sharing_usage_limit_exceeded` 时立即失败，除非回复为空。其他状态码和第五次失败都会让任务失败：`<what>: <text>`（`HTTP <status>`、`request failed` 或 `no reply`），或下文的 deadline 格式。不是 JSON 的 200 会立即以 `bad response` 失败。
