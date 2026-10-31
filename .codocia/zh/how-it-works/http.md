---
title: 请求与重试
source: #c73c2d684b37
---

`complete()` 向 `/chat/completions` POST `{"model", "messages", "tools"}`，设置了推理强度时再加上 `reasoning_effort`，没有采样参数、token 上限或流式输出。每次尝试都会重新读取 key（[[usage]]）；key 文件无法读取时以 `cannot read <path>` 失败。每次尝试可用 min(600 s, 剩余时间)。

网络错误（包括无法读取的 body）、HTTP 429、HTTP 5xx，以及没有 `choices[0].message` 的 HTTP 200（MiniMax 的某些错误，`333b7b64`）都会重试：总共五次尝试，中间分别等待 2、4、8、16 s，并打印 `restloop: <what>, retrying`。body 中出现 `insufficient_quota` 或 `subscription_sharing_usage_limit_exceeded` 时立即失败。其他任何状态码会让任务立即以 `<what>: <text>` 失败，`<what>` 是 `HTTP <status>`、`request failed` 或 `no reply`；第五次失败也是这样，但如果那时已经到了 deadline，则用下文的 deadline 格式。body 不是 JSON 的 200 会立即以 `bad response` 失败。
