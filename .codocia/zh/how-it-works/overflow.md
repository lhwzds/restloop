---
title: 上下文窗口满了之后的恢复
source: #288322a6601b
---

自 `21f3c1dc` 起，`too_long()` 负责识别上下文溢出：HTTP 400 或 413，或 Responses 事件流报告的失败（`no reply: `），且文本包含服务商使用的某个短语（`context_length_exceeded`、"maximum context length"、"context window exceeds"、"prompt is too long"、"exceeds the available context size"）。`04004c68` 去掉了 "too long" 这类更宽泛的词，因为一个无关的 400（`tool_call_id` 过长）触发过历史裁剪。

匹配成功后，`drop_old_output()` 把尚未缩短的 `tool` 消息中较早的一半（向上取整）缩成它们的第一行加上 `[output removed to fit the context window]`，用 `trim_thinking()` 清空旧的推理内容，然后重试请求。每次重试都会把剩下的再减半；没有可改动的内容时，返回该错误。命令和可见的回复都会保留，所以模型仍然知道自己做过什么。
