---
title: 请求与重试
source: #42ced409c80b
---

`complete()` 向 `/chat/completions` POST `{"model", "messages", "tools"}`，设置了推理强度时再加上 `reasoning_effort`，没有采样参数、token 上限或流式输出。每次尝试前都会读取 key（见 [[usage]]）；key 文件无法读取时，任务立即以 `cannot read <path>` 失败。每次尝试最多 600 s。

网络错误、HTTP 429、HTTP 5xx，以及没有 `choices[0].message` 的 HTTP 200（MiniMax 用这种方式报告某些错误，`333b7b64`）都会重试，总共五次尝试，中间分别等待 2、4、8、16 s，并打印 `restloop: <what>, retrying`。body 中出现 `insufficient_quota` 或 `subscription_sharing_usage_limit_exceeded`（额度或套餐用尽）时立即失败。其他任何状态码，或第五次失败，都会变成 `HTTP <status>: <body>`（或 `request failed: ...`、`no reply: ...`），任务失败（退出码见 [[usage]]）。body 不是 JSON 的 200 会立即以 `bad response` 失败。
