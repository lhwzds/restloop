---
title: 清空旧的推理内容
source: #f1809343c4dd
---

`trim_thinking()` 会清空除最后两条之外所有 assistant 消息的推理内容：`reasoning_content` 变为 `""`，`output_items` 中的 `reasoning` 项被删除（message 项保留其 phase），`content` 开头的 `<think>` 块被截掉。这个 key 保留为空值（`70f40924`）：DeepSeek 的思考模式会拒绝（HTTP 400）缺少它的工具轮次 assistant 消息。已经清空过的消息不算作有改动，因此重试一定会终止。
